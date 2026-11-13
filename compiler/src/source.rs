//! Source files and places in them.
//!
//! A place is written twice in an error (D16): `file:line:col` for tools, with the column in UTF-8 bytes, and
//! `line N column M` for people, with the column in grapheme clusters. So a file keeps its text and where each line
//! starts, and turns a byte offset into both.

use unicode_segmentation::UnicodeSegmentation;

/// Which file a span is in, as an index into [`Sources`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct FileId(pub u32);

/// A run of bytes in one file, `start` included and `end` not.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Span {
    pub file: FileId,
    pub start: u32,
    pub end: u32,
}

impl Span {
    #[must_use]
    pub fn new(file: FileId, start: usize, end: usize) -> Span {
        Span { file, start: offset(start), end: offset(end) }
    }

    /// The span from the start of `self` to the end of `other`.
    #[must_use]
    pub fn to(self, other: Span) -> Span {
        Span { file: self.file, start: self.start.min(other.start), end: self.end.max(other.end) }
    }

    #[must_use]
    pub fn is_empty(self) -> bool {
        self.start == self.end
    }

    #[must_use]
    pub fn range(self) -> std::ops::Range<usize> {
        self.start as usize..self.end as usize
    }
}

/// Byte offsets fit in 32 bits, since a source file over 4 GB isn't something anyone writes; a larger one is cut to
/// the last offset, which only makes its places wrong past that point, never crashes.
fn offset(at: usize) -> u32 {
    u32::try_from(at).unwrap_or(u32::MAX)
}

// A span is stored relative to the function being kept, so the function can be used again after the lines above it
// change (see `relocation`).
impl serde::Serialize for Span {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let (start, end) = relocation::store(*self);
        serde::Serialize::serialize(&(self.file, start, end), serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Span {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Span, D::Error> {
        let (file, start, end): (FileId, u32, u32) = serde::Deserialize::deserialize(deserializer)?;
        Ok(relocation::load(file, start, end))
    }
}

/// While a checked function is stored for later builds, or read back, its spans are written relative to where it
/// starts, and messages that show places are rendered again from the file as it is now (D168).
pub mod relocation {
    use std::cell::RefCell;
    use std::sync::Arc;

    use super::{FileId, Sources, Span};

    /// Where the function being stored or read starts and ends, and the files as they are now.
    pub struct Relocation {
        pub start: u32,
        pub end: u32,
        /// Whether a span reached outside the function, which a stored function can't have, since nothing says where
        /// the place it points at went.
        pub outside: bool,
        /// The files as they are now, when messages have to be rendered again because the lines above moved.
        pub sources: Option<Arc<Sources>>,
        /// The names of the functions, structs and unions a body refers to, which it's kept by instead of their
        /// numbers, since those come from the order of declarations.
        pub names: Option<Arc<Names>>,
    }

    /// What a function, struct or union is called, by its number, and the number for each name.
    #[derive(Default)]
    pub struct Names {
        pub tables: [Vec<Option<String>>; 3],
        numbers: [std::collections::HashMap<String, usize>; 3],
    }

    /// Which kind of thing a reference names.
    #[derive(Clone, Copy)]
    pub enum Named {
        Function = 0,
        Struct = 1,
        Union = 2,
    }

    impl Names {
        #[must_use]
        pub fn new(tables: [Vec<Option<String>>; 3]) -> Names {
            let numbers = tables.clone().map(|table| {
                table.into_iter().enumerate().filter_map(|(number, name)| Some((name?, number))).collect()
            });
            Names { tables, numbers }
        }
    }

    /// The name kept for a number. A number with no name can't be kept, which counts as pointing outside.
    pub fn name_of(kind: Named, number: usize) -> Option<String> {
        CURRENT.with(|current| {
            let mut current = current.borrow_mut();
            let relocation = current.as_mut()?;
            let found =
                relocation.names.as_ref().and_then(|names| names.tables[kind as usize].get(number).cloned().flatten());
            if found.is_none() {
                relocation.outside = true;
            }
            found
        })
    }

    /// The number a kept name has now.
    pub fn number_of(kind: Named, name: &str) -> Option<usize> {
        CURRENT.with(|current| {
            let current = current.borrow();
            current.as_ref()?.names.as_ref()?.numbers[kind as usize].get(name).copied()
        })
    }

    thread_local! {
        static CURRENT: RefCell<Option<Relocation>> = const { RefCell::new(None) };
    }

    /// Runs `body` with spans relative to `relocation`, giving back what it found.
    pub fn with<R>(relocation: Relocation, body: impl FnOnce() -> R) -> (R, Relocation) {
        let outer = CURRENT.with(|current| current.borrow_mut().replace(relocation));
        let result = body();
        let finished = CURRENT.with(|current| std::mem::replace(&mut *current.borrow_mut(), outer));
        match finished {
            Some(relocation) => (result, relocation),
            None => (result, Relocation { start: 0, end: 0, outside: true, sources: None, names: None }),
        }
    }

    pub(super) fn store(span: Span) -> (u32, u32) {
        CURRENT.with(|current| match current.borrow_mut().as_mut() {
            Some(relocation) => {
                if span.start < relocation.start || span.end > relocation.end {
                    relocation.outside = true;
                }
                (span.start.wrapping_sub(relocation.start), span.end.wrapping_sub(relocation.start))
            }
            None => (span.start, span.end),
        })
    }

    pub(super) fn load(file: FileId, start: u32, end: u32) -> Span {
        CURRENT.with(|current| match current.borrow().as_ref() {
            Some(relocation) => {
                Span { file, start: start.wrapping_add(relocation.start), end: end.wrapping_add(relocation.start) }
            }
            None => Span { file, start, end },
        })
    }

    /// Renders with the files as they are now, while reading a function back.
    pub fn render<R>(body: impl FnOnce(&Sources) -> R) -> Option<R> {
        CURRENT.with(|current| current.borrow().as_ref().and_then(|relocation| relocation.sources.as_deref()).map(body))
    }
}

/// One source file: the name errors show, and its text.
#[derive(Clone, Debug)]
pub struct SourceFile {
    pub name: String,
    pub text: String,
    line_starts: Vec<usize>,
}

/// Where a byte offset is, both ways.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Place {
    /// Counting from 1.
    pub line: usize,
    /// Counting from 1, in UTF-8 bytes.
    pub byte_column: usize,
    /// Counting from 1, in grapheme clusters.
    pub grapheme_column: usize,
}

impl SourceFile {
    #[must_use]
    pub fn new(name: String, text: String) -> SourceFile {
        let mut line_starts = vec![0];
        for (index, byte) in text.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push(index + 1);
            }
        }
        SourceFile { name, text, line_starts }
    }

    /// The line an offset is on, counting from 0.
    fn line_index(&self, at: usize) -> usize {
        match self.line_starts.binary_search(&at) {
            Ok(index) => index,
            Err(index) => index.saturating_sub(1),
        }
    }

    /// The text of a line, counting from 0, without its newline.
    #[must_use]
    pub fn line_text(&self, index: usize) -> &str {
        let start = self.line_starts.get(index).copied().unwrap_or(self.text.len());
        let end = self.line_starts.get(index + 1).map_or(self.text.len(), |next| next - 1);
        let line = self.text.get(start..end.max(start)).unwrap_or("");
        line.strip_suffix('\r').unwrap_or(line)
    }

    /// Where a byte offset is.
    #[must_use]
    pub fn place(&self, at: usize) -> Place {
        let at = at.min(self.text.len());
        let index = self.line_index(at);
        let start = self.line_starts.get(index).copied().unwrap_or(0);
        let before = self.text.get(start..at).unwrap_or("");
        Place { line: index + 1, byte_column: at - start + 1, grapheme_column: before.graphemes(true).count() + 1 }
    }

    /// The start of a line, counting from 0, as a byte offset.
    #[must_use]
    pub fn line_start(&self, index: usize) -> usize {
        self.line_starts.get(index).copied().unwrap_or(self.text.len())
    }

    #[must_use]
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    /// The text a span covers.
    #[must_use]
    pub fn slice(&self, span: Span) -> &str {
        self.text.get(span.range()).unwrap_or("")
    }
}

/// Every file a build reads, so a span can name any of them.
#[derive(Clone, Debug, Default)]
pub struct Sources {
    files: Vec<SourceFile>,
}

impl Sources {
    pub fn add(&mut self, file: SourceFile) -> FileId {
        let id = FileId(u32::try_from(self.files.len()).unwrap_or(u32::MAX));
        self.files.push(file);
        id
    }

    /// The file a span names. Every `FileId` comes from [`Sources::add`], so a missing one is a compiler bug, and
    /// shows as an empty file rather than a crash (rule 6).
    #[must_use]
    pub fn get(&self, id: FileId) -> &SourceFile {
        static EMPTY: std::sync::OnceLock<SourceFile> = std::sync::OnceLock::new();
        self.files
            .get(id.0 as usize)
            .unwrap_or_else(|| EMPTY.get_or_init(|| SourceFile::new(String::new(), String::new())))
    }

    /// The text of every file but one.
    pub fn others(&self, but: FileId) -> impl Iterator<Item = &str> {
        self.files
            .iter()
            .enumerate()
            .filter(move |(index, _)| *index != but.0 as usize)
            .map(|(_, file)| file.text.as_str())
    }

    pub fn get_mut(&mut self, id: FileId) -> Option<&mut SourceFile> {
        self.files.get_mut(id.0 as usize)
    }
}
