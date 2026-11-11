//! Errors, in the one layout building and running share (standard 11).

use std::fmt::Write as _;

use unicode_segmentation::UnicodeSegmentation;

use crate::source::{Sources, Span};

/// The first word of an error (standard 11.2), plus `NotDesignedYet` for what isn't decided (rule 4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Refused,
    Unproven,
    CheckFailed,
    Warning,
    NotDesignedYet,
    /// Decided, but not built in this compiler yet (D286).
    NotBuiltYet,
}

impl Kind {
    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            Kind::Refused => "refused",
            Kind::Unproven => "unproven",
            Kind::CheckFailed => "check failed",
            Kind::Warning => "warning",
            Kind::NotDesignedYet => "NotDesignedYet",
            Kind::NotBuiltYet => "NotBuiltYet",
        }
    }
}

/// One edit to a file's text: what replaces a span. A fix carries one, so cascades can be found by applying it in
/// memory and checking again (standard 11.7).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edit {
    pub span: Span,
    pub replacement: String,
}

/// The single certain fix (standard 11.5): the code shown, and the edit that makes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fix {
    pub shown: String,
    pub edits: Vec<Edit>,
}

/// An earlier step that led here (standard 11.6).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Because {
    pub span: Span,
    pub note: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub kind: Kind,
    pub span: Span,
    /// What's wrong, unless `fix` replaces it.
    pub what: Option<String>,
    pub fix: Option<Fix>,
    pub why: String,
    pub when: Option<String>,
    pub because: Vec<Because>,
}

impl Diagnostic {
    #[must_use]
    pub fn new(kind: Kind, span: Span, what: impl Into<String>, why: impl Into<String>) -> Diagnostic {
        Diagnostic { kind, span, what: Some(what.into()), fix: None, why: why.into(), when: None, because: Vec::new() }
    }

    #[must_use]
    pub fn refused(span: Span, what: impl Into<String>, why: impl Into<String>) -> Diagnostic {
        Diagnostic::new(Kind::Refused, span, what, why)
    }

    /// A refusal whose one fix replaces `what:`, shown as code, and edits `span` to `shown`.
    #[must_use]
    pub fn refused_fix(span: Span, shown: impl Into<String>, why: impl Into<String>) -> Diagnostic {
        let shown = shown.into();
        Diagnostic {
            kind: Kind::Refused,
            span,
            what: None,
            fix: Some(Fix { edits: vec![Edit { span, replacement: shown.clone() }], shown }),
            why: why.into(),
            when: None,
            because: Vec::new(),
        }
    }

    /// Something the standard leaves undecided (rule 4).
    #[must_use]
    pub fn not_designed(span: Span, what: impl Into<String>, why: impl Into<String>) -> Diagnostic {
        Diagnostic::new(Kind::NotDesignedYet, span, what, why)
    }

    /// Replaces the fix's edits, for a fix whose shown code isn't the whole edit.
    #[must_use]
    pub fn with_edits(mut self, edits: Vec<Edit>) -> Diagnostic {
        if let Some(fix) = &mut self.fix {
            fix.edits = edits;
        }
        self
    }

    #[must_use]
    pub fn with_when(mut self, when: impl Into<String>) -> Diagnostic {
        self.when = Some(when.into());
        self
    }

    #[must_use]
    pub fn with_because(mut self, span: Span, note: impl Into<String>) -> Diagnostic {
        self.because.push(Because { span, note: note.into() });
        self
    }

    /// The error as it's printed, ending in a newline.
    #[must_use]
    pub fn render(&self, sources: &Sources) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "{}: {}", self.kind.word(), place_line(sources, self.span));
        snippet(&mut out, sources, self.span);
        if let Some(fix) = &self.fix {
            let _ = writeln!(out, " fix:  `{}`", fix.shown);
        } else if let Some(what) = &self.what {
            let _ = writeln!(out, " what: {what}");
        }
        let _ = writeln!(out, " why:  {}", self.why);
        if let Some(when) = &self.when {
            let _ = writeln!(out, " when: {when}");
        }
        for because in &self.because {
            let _ = writeln!(out, " because: {}", place_line(sources, because.span));
            snippet(&mut out, sources, because.span);
            let _ = writeln!(out, "     {}", because.note);
        }
        out
    }
}

/// A `because:` part on its own, as a failure's crash message adds it once it knows where the failure was raised.
#[must_use]
pub fn render_because(sources: &Sources, span: Span, note: &str) -> String {
    let mut out = String::new();
    let _ = writeln!(out, " because: {}", place_line(sources, span));
    snippet(&mut out, sources, span);
    let _ = writeln!(out, "     {note}");
    out
}

/// `file:line:col, line N column M`.
fn place_line(sources: &Sources, span: Span) -> String {
    let file = sources.get(span.file);
    let place = file.place(span.start as usize);
    format!("{}:{}:{}, line {} column {}", file.name, place.line, place.byte_column, place.line, place.grapheme_column)
}

/// The line a span starts on, with a `^` under each grapheme it covers on that line.
fn snippet(out: &mut String, sources: &Sources, span: Span) {
    let file = sources.get(span.file);
    let place = file.place(span.start as usize);
    let index = place.line - 1;
    let text = file.line_text(index);
    let number = place.line.to_string();
    let width = number.len().max(2);
    let _ = writeln!(out, "{number:>width$} | {text}");
    let line_start = file.line_start(index);
    let start = (span.start as usize).saturating_sub(line_start).min(text.len());
    let end = (span.end as usize).saturating_sub(line_start).clamp(start, text.len());
    let before = text.get(..start).map_or(0, |part| part.graphemes(true).count());
    let covered = text.get(start..end).map_or(0, |part| part.graphemes(true).count()).max(1);
    let _ = writeln!(out, "{:width$} | {}{}", "", " ".repeat(before), "^".repeat(covered));
}

/// Every error of a build, printed with a blank line between them.
#[must_use]
pub fn render_all(diagnostics: &[Diagnostic], sources: &Sources) -> String {
    let mut out = String::new();
    for (index, diagnostic) in diagnostics.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        out.push_str(&diagnostic.render(sources));
    }
    out
}
