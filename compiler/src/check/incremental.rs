//! Incremental checking (D168, rule 10): each function's checked body is kept between builds under a hash of what
//! checking it can depend on, so an edit only checks again the functions it changed, and the callers of a function
//! whose interface changed.
//!
//! A function's key hashes its own text; everything every function can see, like types, units, operators and baked
//! functions; and the interface of each function it names: its header and the `labels`, `defaults`, `requires` and
//! `ensures` that start its body. Functions, structs and unions are numbered by the order they're declared in, so a
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

/// Works out every plain function's key. A function whose name a body uses is a dependency, through `use` short
/// names too; anything that can't be told apart that way, like an operator or a baked function, counts for every
/// function.
#[must_use]
pub fn keys(file: &syntax::File, text: &str, other_files: &[&str]) -> Keys {
    let slice = |span: Span| text.get(span.range()).unwrap_or("");
    let mut everyone = crate::cache::Hasher::new();
    for other in other_files {
        everyone.text(other);
    }
    // Each plain function's interface, by the word calls use for it.
    let mut interfaces: HashMap<String, Vec<String>> = HashMap::new();
    let mut plain: Vec<usize> = Vec::new();
    let mut aliases: Vec<(String, String)> = Vec::new();
    for (index, item) in file.items.iter().enumerate() {
        match &item.kind {
            ItemKind::Function(function) if is_plain(function) => {
                let name = function.name.as_ref().map(syntax::Path::text).unwrap_or_default();
                let interface = interface(function, &slice);
                interfaces.entry(word_of(&name).to_string()).or_default().push(interface);
                plain.push(index);
            }
            ItemKind::Use(short) => {
                aliases.push((short.short.text.clone(), word_of(&short.path.text()).to_string()));
                everyone.text(slice(item.span));
            }
            _ => everyone.text(slice(item.span)),
        }
    }
    let everyone = everyone.finish();
    let mut keys = HashMap::new();
    for index in plain {
        let Some(item) = file.items.get(index) else { continue };
        let own = slice(item.span);
        let mut used: Vec<&str> = words(own).into_iter().collect();
        for (short, target) in &aliases {
            if used.contains(&short.as_str()) {
                used.push(target);
            }
        }
        used.sort_unstable();
        used.dedup();
        let mut hasher = crate::cache::Hasher::new();
        hasher.text(&everyone);
        hasher.text(own);
        for word in used {
            if let Some(found) = interfaces.get(word) {
                hasher.text(word);
                for each in found {
                    hasher.text(each);
                }
            }
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
