//! Incremental checking (D168, rule 10): each function's checked body is kept between builds under a hash of what
//! checking it can depend on, so an edit only checks again the functions it changed, and the callers of a function
//! whose interface changed.
//!
//! A function's key hashes its own text, and every declaration it can reach through the words in it: the interface of
//! each function it names, which is its header and the `labels`, `defaults`, `requires` and `ensures` that start its
//! body; the types, units and failures those name; and each operator on the types it reaches. Functions, structs and unions are numbered by the order they're declared in, so a
//! kept body refers to them by name, and gets their numbers again when it's read back, so adding or moving a function
//! only checks again what names it. Its spans are kept relative to where it starts, and its messages are rendered again where it
//! is now, so lines added above it don't matter. A function whose check said anything, or that points outside itself,
//! isn't kept.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::checked;
use crate::source::{Sources, Span, relocation};
use crate::syntax::{self, ItemKind, StatementKind};
use crate::target::{Mode, Target};

/// How many functions were checked and how many were kept from before.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub checked: usize,
    pub reused: usize,
}

/// The checked functions kept for one program, target and mode.
pub struct Store {
    path: Option<PathBuf>,
    /// Each function's checked body, by its key, with the line it started on.
    found: HashMap<String, (usize, Vec<u8>)>,
    kept: Vec<(String, (usize, Vec<u8>))>,
    sources: Option<Rc<Sources>>,
    /// The names of this build's functions, structs and unions, which kept bodies refer to.
    pub names: Option<Rc<relocation::Names>>,
    pub stats: Stats,
}

impl Store {
    /// The store for the program in `file`, read from the cache folder if it has one.
    #[must_use]
    pub fn open(folder: Option<&Path>, file: &Path, target: Target, mode: Mode) -> Store {
        let path = folder.map(|folder| {
            let named = std::fs::canonicalize(file).unwrap_or_else(|_| file.to_path_buf());
            let key = crate::cache::key(target, mode, &format!("check {}", named.display()));
            folder.join("checks").join(format!("{key}.bin"))
        });
        let found = path
            .as_ref()
            .and_then(|path| std::fs::read(path).ok())
            .and_then(|bytes| postcard::from_bytes::<Vec<(String, (usize, Vec<u8>))>>(&bytes).ok())
            .map(|entries| entries.into_iter().collect())
            .unwrap_or_default();
        Store { path, found, kept: Vec::new(), sources: None, names: None, stats: Stats::default() }
    }

    /// Writes what this build kept, replacing the last build's, so the store only ever holds the program as it is.
    pub fn save(&self) {
        let Some(path) = &self.path else { return };
        let Some(folder) = path.parent() else { return };
        if std::fs::create_dir_all(folder).is_err() {
            return;
        }
        let Ok(bytes) = postcard::to_stdvec(&self.kept) else { return };
        let partial = path.with_extension(format!("{}.partial", std::process::id()));
        if std::fs::write(&partial, bytes).is_ok() && std::fs::rename(&partial, path).is_err() {
            let _ = std::fs::remove_file(&partial);
        }
    }

    fn sources(&mut self, sources: &Sources) -> Rc<Sources> {
        self.sources.get_or_insert_with(|| Rc::new(sources.clone())).clone()
    }

    /// A function kept from before under `key`, moved to where it is now.
    pub fn reuse(&mut self, key: &str, sources: &Sources, place: Span) -> Option<checked::Function> {
        let line = sources.get(place.file).place(place.start as usize).line;
        let shared = self.sources(sources);
        let (stored_line, bytes) = self.found.get(key)?;
        // Messages only show lines, so a function still starting on the same line keeps its messages as they were.
        let shared = (*stored_line != line).then_some(shared);
        let relocation = relocation::Relocation {
            start: place.start,
            end: place.end,
            outside: false,
            sources: shared,
            names: self.names.clone(),
        };
        let (function, _) = relocation::with(relocation, || postcard::from_bytes::<checked::Function>(bytes).ok());
        let function = function?;
        self.kept.push((key.to_string(), (line, bytes.clone())));
        self.stats.reused += 1;
        Some(function)
    }

    /// Keeps a function just checked, unless it points outside itself.
    pub fn keep(&mut self, key: &str, sources: &Sources, place: Span, function: &checked::Function) {
        let line = sources.get(place.file).place(place.start as usize).line;
        let relocation = relocation::Relocation {
            start: place.start,
            end: place.end,
            outside: false,
            sources: None,
            names: self.names.clone(),
        };
        let (bytes, finished) = relocation::with(relocation, || postcard::to_stdvec(function).ok());
        if let Some(bytes) = bytes
            && !finished.outside
        {
            self.kept.push((key.to_string(), (line, bytes)));
        }
    }
}

/// The keys of a file's functions, by their place in the file's items.
pub struct Keys {
    pub keys: HashMap<usize, String>,
}

/// The word a call uses for a function: its name's last part, without `%` marks, as in `warm_up` for
/// `reading.%%warm_up`.
fn word_of(name: &str) -> &str {
    let last = name.rsplit(['.', ':']).next().unwrap_or(name);
    last.trim_start_matches('%')
}

/// Every word in a text, the way names are written.
fn words(text: &str) -> HashSet<&str> {
    text.split(|character: char| !(character.is_alphanumeric() || character == '_'))
        .filter(|word| !word.is_empty())
        .collect()
}

/// One declaration a body can depend on: the words that name it, the text its users depend on, and the words in that
/// text, which name what it depends on in turn.
struct Declaration {
    provides: Vec<String>,
    text: String,
    mentions: Vec<String>,
}

/// Works out every plain function's key. Every declaration a body can reach through the words in its text counts:
/// the functions it names, through `use` short names too, the types, units and failures they name, and so on, with
/// an operator reached through the types in its header. Only what has no name, like an `import`, counts for every
/// function.
#[must_use]
pub fn keys(file: &syntax::File, text: &str, other_files: &[&str]) -> Keys {
    let slice = |span: Span| text.get(span.range()).unwrap_or("");
    let owned_words = |text: &str| words(text).into_iter().map(str::to_string).collect::<Vec<_>>();
    let mut everyone = crate::cache::Hasher::new();
    for other in other_files {
        everyone.text(other);
    }
    let mut declarations: Vec<Declaration> = Vec::new();
    let mut plain: Vec<usize> = Vec::new();
    for (index, item) in file.items.iter().enumerate() {
        let full = slice(item.span).to_string();
        let declaration = match &item.kind {
            ItemKind::Function(function) if is_plain(function) => {
                plain.push(index);
                let name = function.name.as_ref().map(syntax::Path::text).unwrap_or_default();
                let interface = interface(function, &slice);
                Declaration {
                    provides: vec![word_of(&name).to_string()],
                    mentions: owned_words(&interface),
                    text: interface,
                }
            }
            ItemKind::Function(function) if function.operator.is_some() => {
                // An operator applies wherever its types meet, so the types in its header lead to it.
                let header = slice(function.header_span).to_string();
                let parameters: Vec<&str> =
                    function.parameters.list.iter().map(|parameter| parameter.name.text.as_str()).collect();
                let provides =
                    owned_words(&header).into_iter().filter(|word| !parameters.contains(&word.as_str())).collect();
                Declaration { provides, mentions: owned_words(&header), text: interface(function, &slice) }
            }
            ItemKind::Function(function) => {
                let name = function.name.as_ref().map(syntax::Path::text).unwrap_or_default();
                Declaration { provides: vec![word_of(&name).to_string()], mentions: owned_words(&full), text: full }
            }
            ItemKind::Use(short) => Declaration {
                provides: vec![short.short.text.clone()],
                mentions: owned_words(&short.path.text()),
                text: full,
            },
            ItemKind::Struct(declared) => {
                Declaration { provides: owned_words(&declared.name.text()), mentions: owned_words(&full), text: full }
            }
            ItemKind::Union(declared) => {
                Declaration { provides: owned_words(&declared.name.text()), mentions: owned_words(&full), text: full }
            }
            ItemKind::Family(family) => {
                let mut provides = owned_words(&family.name.text());
                for entry in &family.entries {
                    provides.extend(owned_words(&entry.name.text()));
                }
                Declaration { provides, mentions: owned_words(&full), text: full }
            }
            ItemKind::TypeName { name, .. } => {
                Declaration { provides: owned_words(&name.text()), mentions: owned_words(&full), text: full }
            }
            ItemKind::Const(constant) => {
                Declaration { provides: vec![constant.name.text.clone()], mentions: owned_words(&full), text: full }
            }
            ItemKind::Interface(declared) | ItemKind::TypeSet(declared) => {
                Declaration { provides: owned_words(&declared.name.text()), mentions: owned_words(&full), text: full }
            }
            ItemKind::Satisfies { subject, interface } => {
                let mut provides = owned_words(&subject.text());
                provides.extend(owned_words(&interface.text()));
                Declaration { provides, mentions: owned_words(&full), text: full }
            }
            ItemKind::Import(_) | ItemKind::Stray(_) => {
                everyone.text(&full);
                continue;
            }
        };
        declarations.push(declaration);
    }
    let everyone = everyone.finish();
    let mut by_word: HashMap<&str, Vec<usize>> = HashMap::new();
    for (number, declaration) in declarations.iter().enumerate() {
        for word in &declaration.provides {
            by_word.entry(word.as_str()).or_default().push(number);
        }
    }
    let mut keys = HashMap::new();
    for index in plain {
        let Some(item) = file.items.get(index) else { continue };
        let own = slice(item.span);
        let mut reached: Vec<usize> = Vec::new();
        let mut seen_words: HashSet<&str> = HashSet::new();
        let mut seen: HashSet<usize> = HashSet::new();
        let mut waiting: Vec<&str> = words(own).into_iter().collect();
        while let Some(word) = waiting.pop() {
            if !seen_words.insert(word) {
                continue;
            }
            for number in by_word.get(word).into_iter().flatten() {
                if seen.insert(*number) {
                    reached.push(*number);
                    if let Some(declaration) = declarations.get(*number) {
                        waiting.extend(declaration.mentions.iter().map(String::as_str));
                    }
                }
            }
        }
        // By their text, not where they are, so moving a declaration changes nothing.
        let mut texts: Vec<&str> =
            reached.iter().filter_map(|number| declarations.get(*number)).map(|each| each.text.as_str()).collect();
        texts.sort_unstable();
        let mut hasher = crate::cache::Hasher::new();
        hasher.text(&everyone);
        hasher.text(own);
        for text in texts {
            hasher.text(text);
        }
        keys.insert(index, hasher.finish());
    }
    Keys { keys }
}

/// A plain function: not an operator, which applies without its name being written, and not baked, whose body runs
/// while checking others.
fn is_plain(function: &syntax::Function) -> bool {
    function.operator.is_none() && function.keyword.text() != "fn.pure.baked"
}

/// What callers depend on: the header, and the lines that start the body and describe calls (D114).
fn interface<'text>(function: &syntax::Function, slice: &impl Fn(Span) -> &'text str) -> String {
    let mut interface = slice(function.header_span).to_string();
    for statement in &function.body {
        match statement.kind {
            StatementKind::Labels(_)
            | StatementKind::Defaults(_)
            | StatementKind::Requires { .. }
            | StatementKind::Ensures { .. } => {
                interface.push('\n');
                interface.push_str(slice(statement.span));
            }
            _ => break,
        }
    }
    interface
}
