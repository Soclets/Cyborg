//! The one error format, for building and running alike. (Standard 11.)

use unicode_segmentation::UnicodeSegmentation;

/// A stretch of the source, in UTF-8 bytes from the start of the file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn to(self, other: Span) -> Span {
        Span {
            start: self.start,
            end: other.end.max(self.start),
        }
    }
}

/// The first word of a message. (Standard 11.2.)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Refused,
    CheckFailed,
    /// Not one of 11.2's kinds. Rule 4 of `IMPLEMENTATION.md` asks for a `NotDesignedYet` error,
    /// and the design session has been told it's printed this way.
    NotDesignedYet,
}

impl Kind {
    fn word(self) -> &'static str {
        match self {
            Kind::Refused => "refused",
            Kind::CheckFailed => "check failed",
            Kind::NotDesignedYet => "NotDesignedYet",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Because {
    pub span: Span,
    pub note: String,
}

#[derive(Clone, Debug)]
pub struct Report {
    pub kind: Kind,
    pub span: Span,
    pub what: String,
    pub fix: Option<String>,
    /// Whether the fix goes somewhere other than over this report's span, like a missing
    /// `import std;`. Such a fix can't be applied in memory, so it finds no cascades. (D207)
    pub fix_elsewhere: bool,
    pub why: String,
    pub when: Option<String>,
    pub because: Vec<Because>,
}

/// Everything that can stop the oracle goes through this, so nothing panics on a bad program.
pub type Outcome<T> = Result<T, Box<Report>>;

impl Report {
    pub fn refused(span: Span, what: impl Into<String>, why: impl Into<String>) -> Box<Self> {
        Box::new(Self {
            kind: Kind::Refused,
            span,
            what: what.into(),
            fix: None,
            fix_elsewhere: false,
            why: why.into(),
            when: None,
            because: Vec::new(),
        })
    }

    pub fn check_failed(span: Span, what: impl Into<String>, why: impl Into<String>) -> Box<Self> {
        Box::new(Self {
            kind: Kind::CheckFailed,
            ..*Self::refused(span, what, why)
        })
    }

    /// `section` names the part of standard 12 that lists it, or is empty when the standard is
    /// silent. Those silent ones have gone to the design session. (Rule 4.)
    pub fn not_designed(span: Span, what: impl Into<String>, section: &str) -> Box<Self> {
        let why = if section.is_empty() {
            "not decided yet, and the standard doesn't say".to_string()
        } else {
            format!("not decided yet (standard {section})")
        };
        Box::new(Self {
            kind: Kind::NotDesignedYet,
            ..*Self::refused(span, what, why)
        })
    }
}

/// Adding to a boxed report, written as a chain at the place that raises it.
pub trait ReportExtra {
    #[must_use]
    fn fix(self, code: impl Into<String>) -> Self;
    /// A fix written somewhere other than over the report's span. (D207)
    #[must_use]
    fn fix_elsewhere(self, code: impl Into<String>) -> Self;
    #[must_use]
    fn when(self, values: impl Into<String>) -> Self;
    #[must_use]
    fn because(self, span: Span, note: impl Into<String>) -> Self;
}

impl ReportExtra for Box<Report> {
    fn fix(mut self, code: impl Into<String>) -> Self {
        self.fix = Some(code.into());
        self
    }

    fn fix_elsewhere(mut self, code: impl Into<String>) -> Self {
        self.fix = Some(code.into());
        self.fix_elsewhere = true;
        self
    }

    fn when(mut self, values: impl Into<String>) -> Self {
        self.when = Some(values.into());
        self
    }

    fn because(mut self, span: Span, note: impl Into<String>) -> Self {
        self.because.push(Because {
            span,
            note: note.into(),
        });
        self
    }
}

pub struct Source {
    pub name: String,
    pub text: String,
}

/// One line's facts about a place in the source.
struct Place<'a> {
    line_number: usize,
    byte_column: usize,
    grapheme_column: usize,
    line: &'a str,
    /// Where the `^` run starts in the line, in bytes.
    line_offset: usize,
    /// Where the `^` run ends in the line, in bytes.
    line_end: usize,
}

impl Source {
    fn place(&self, span: Span) -> Place<'_> {
        let start = span.start.min(self.text.len());
        let before = self.text.get(..start).unwrap_or("");
        let line_start = before.rfind('\n').map_or(0, |at| at + 1);
        let line_number = before.matches('\n').count() + 1;
        let rest = self.text.get(line_start..).unwrap_or("");
        let line = rest.split('\n').next().unwrap_or("").trim_end_matches('\r');
        let line_offset = start - line_start;
        let prefix = line.get(..line_offset.min(line.len())).unwrap_or("");
        // trace: D16
        let grapheme_column = prefix.graphemes(true).count() + 1;
        let line_end = span
            .end
            .saturating_sub(line_start)
            .clamp(line_offset, line.len());
        Place {
            line_number,
            byte_column: line_offset + 1,
            grapheme_column,
            line,
            line_offset,
            line_end,
        }
    }

    fn write_place(out: &mut String, place: &Place<'_>, width: usize) {
        use std::fmt::Write;
        let _ = writeln!(
            out,
            "{:>width$} | {}",
            place.line_number,
            place.line,
            width = width
        );
        let prefix = place.line.get(..place.line_offset).unwrap_or("");
        // Tabs stay tabs, so the run still lines up under the code.
        let indent: String = prefix
            .graphemes(true)
            .map(|grapheme| if grapheme == "\t" { '\t' } else { ' ' })
            .collect();
        let marked = place
            .line
            .get(place.line_offset..place.line_end)
            .unwrap_or("");
        let run = marked.graphemes(true).count().max(1);
        let _ = writeln!(
            out,
            "{:width$} | {}{}",
            "",
            indent,
            "^".repeat(run),
            width = width
        );
    }

    fn heading(&self, place: &Place<'_>) -> String {
        format!(
            "{}:{}:{}, line {} column {}",
            self.name,
            place.line_number,
            place.byte_column,
            place.line_number,
            place.grapheme_column
        )
    }

    /// trace: D16
    pub fn render(&self, report: &Report) -> String {
        use std::fmt::Write;
        let main_place = self.place(report.span);
        let because_places: Vec<Place<'_>> = report
            .because
            .iter()
            .map(|because| self.place(because.span))
            .collect();
        // The gutter fits the largest line number, and is never under two wide, as 11.1 and 11.5
        // both show.
        let largest = because_places
            .iter()
            .map(|place| place.line_number)
            .chain(std::iter::once(main_place.line_number))
            .max()
            .unwrap_or(1);
        let width = largest.to_string().len().max(2);
        let mut out = String::new();
        let _ = writeln!(out, "{}: {}", report.kind.word(), self.heading(&main_place));
        Self::write_place(&mut out, &main_place, width);
        if let Some(fix) = &report.fix {
            let _ = writeln!(out, " fix:  `{fix}`");
        } else {
            let _ = writeln!(out, " what: {}", report.what);
        }
        let _ = writeln!(out, " why:  {}", report.why);
        if let Some(when) = &report.when {
            let _ = writeln!(out, " when: {when}");
        }
        for (because, place) in report.because.iter().zip(&because_places) {
            let _ = writeln!(out, " because: {}", self.heading(place));
            Self::write_place(&mut out, place, width);
            let _ = writeln!(out, "     {}", because.note);
        }
        out
    }
}
