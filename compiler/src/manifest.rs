//! The manifest, `CyJet.siml` (standard 9.1): one line per fact.

use crate::diagnostic::{Diagnostic, Kind};
use crate::source::{FileId, SourceFile, Span};

const LINE_WHY: &str =
    "a manifest has one line per fact, starting with what it is: `name`, `ver`, `dialect` or `need` (D228, D229, D309)";
const VERSION_WHY: &str = "a version is a labelled date, biggest first, as in `2026y-11m-14d`, with `.2` for a second release that day (D231)";
const NOT_DECIDED_PROJECTS: &str = "not decided yet (standard 12.9)";

/// The packages a project needs, by name, with where each `need` line is.
#[derive(Debug, Default)]
pub struct Manifest {
    pub needs: Vec<(String, Span)>,
    /// There's no manifest at all, which is said once, so nothing missing from it is said again (D183).
    pub missing: bool,
}

impl Manifest {
    #[must_use]
    pub fn needs(&self, package: &str) -> bool {
        self.needs.iter().any(|(name, _)| name == package)
    }
}

/// A word on a line and where it is.
struct Word<'text> {
    text: &'text str,
    start: usize,
    end: usize,
}

/// Splits a line into words, stopping at a `//` comment.
fn words(line: &str, line_start: usize) -> Vec<Word<'_>> {
    let mut found = Vec::new();
    let mut rest = line.char_indices().peekable();
    while let Some(&(index, character)) = rest.peek() {
        if character.is_whitespace() {
            rest.next();
            continue;
        }
        if line[index..].starts_with("//") {
            break;
        }
        let mut end = index;
        if character == '"' {
            rest.next();
            end = index + 1;
            for (inner, inner_character) in rest.by_ref() {
                end = inner + inner_character.len_utf8();
                if inner_character == '"' {
                    break;
                }
            }
        } else {
            while let Some(&(inner, inner_character)) = rest.peek() {
                if inner_character.is_whitespace() {
                    break;
                }
                end = inner + inner_character.len_utf8();
                rest.next();
            }
        }
        found.push(Word { text: &line[index..end], start: line_start + index, end: line_start + end });
    }
    found
}

/// Reads a manifest, adding its errors to `errors`.
#[must_use]
pub fn read(file_id: FileId, file: &SourceFile, errors: &mut Vec<Diagnostic>) -> Manifest {
    let mut manifest = Manifest::default();
    let span = |start: usize, end: usize| Span::new(file_id, start, end);
    let mut has_dialect = false;
    for index in 0..file.line_count() {
        let line = file.line_text(index);
        let start = file.line_start(index);
        let found = words(line, start);
        let Some(first) = found.first() else { continue };
        let last_end = found.last().map_or(first.end, |word| word.end);
        let whole = span(first.start, last_end);
        // How many words each kind of line takes after its first, and what to do with them.
        let takes = match first.text {
            "name" => {
                if let Some(value) = found.get(1) {
                    check_quotes(value, span(value.start, value.end), errors);
                }
                2
            }
            "ver" => {
                if let Some(value) = found.get(1) {
                    check_version(value, span(value.start, value.end), errors);
                }
                match found.get(2) {
                    Some(flag) if flag.text == "same" || flag.text == "breaks" => {}
                    Some(_) | None => errors.push(Diagnostic::refused(
                        whole,
                        "a release says whether it `breaks` anything or stays the `same`",
                        "a release's version is its date plus whether it breaks (D231)",
                    )),
                }
                3
            }
            // trace: D310
            "okay" => {
                // It's said once, as the old spelling of the `dialect` line.
                has_dialect = true;
                let version = found.get(1).map_or(String::new(), |version| format!(" {}", version.text));
                errors.push(Diagnostic::refused_fix(
                    whole,
                    format!("dialect original{version}"),
                    "`dialect` says which compiler builds the project and which version of it, so plain Cyborg says `dialect original` and its version (D309, D310)",
                ));
                continue;
            }
            // trace: D309, D310
            "dialect" => {
                has_dialect = true;
                if let Some(name) = found.get(1)
                    && name.text != "original"
                {
                    errors.push(Diagnostic::new(
                        Kind::NotBuiltYet,
                        span(name.start, name.end),
                        format!("building with the Dialect `{}`, which CyJet loads from its registry", name.text),
                        "decided, but OKAY\\n doesn't handle it yet (D286)",
                    ));
                    continue;
                }
                if let Some(value) = found.get(2) {
                    check_version(value, span(value.start, value.end), errors);
                }
                3
            }
            "need" => {
                let Some(package) = found.get(1) else {
                    errors.push(Diagnostic::refused(whole, "a `need` line names a package", LINE_WHY));
                    continue;
                };
                let package_span = span(package.start, package.end);
                let Some(version) = found.get(2) else {
                    errors.push(Diagnostic::refused(
                        whole,
                        format!("`need {}` says no version", package.text),
                        "each package a program uses has a `need` line with its version (D228, D231)",
                    ));
                    // It still says the program needs the package, so its imports aren't refused too.
                    manifest.needs.push((package.text.to_string(), package_span));
                    continue;
                };
                if !matches!(package.text, "std" | "os" | "csup" | "concur") {
                    errors.push(Diagnostic::not_designed(
                        package_span,
                        format!("how the package `{}` is found", package.text),
                        NOT_DECIDED_PROJECTS,
                    ));
                }
                check_version(version, span(version.start, version.end), errors);
                manifest.needs.push((package.text.to_string(), package_span));
                if let Some(with) = found.get(3)
                    && with.text == "with"
                {
                    errors.push(Diagnostic::not_designed(
                        span(with.start, last_end),
                        "a package's features, which `with` adds",
                        NOT_DECIDED_PROJECTS,
                    ));
                    continue;
                }
                3
            }
            "rule" => {
                errors.push(Diagnostic::not_designed(
                    whole,
                    "a `rule` line, since rule names and the words `rule`, `refuse`, `allow` and `in` are placeholders",
                    NOT_DECIDED_PROJECTS,
                ));
                continue;
            }
            other => {
                errors.push(Diagnostic::refused(
                    span(first.start, first.end),
                    format!("`{other}` isn't something a manifest says"),
                    LINE_WHY,
                ));
                continue;
            }
        };
        if let Some(extra) = found.get(takes) {
            errors.push(Diagnostic::refused(
                span(extra.start, last_end),
                format!("a `{}` line ends before this", first.text),
                LINE_WHY,
            ));
        }
    }
    // trace: D309
    if !has_dialect {
        let end = file.text.len();
        errors.push(Diagnostic::refused_fix(
            span(end, end),
            "dialect original",
            "every manifest has a `dialect` line, and plain Cyborg says `dialect original` and the version of `okay` that builds it (D309, D310)",
        ));
    }
    manifest
}

/// A value is quoted only when it needs to be (D228): when it has a space, a quote or a `//` in it.
fn check_quotes(value: &Word<'_>, span: Span, errors: &mut Vec<Diagnostic>) {
    if let Some(inner) = value.text.strip_prefix('"').and_then(|text| text.strip_suffix('"'))
        && !inner.is_empty()
        && !inner.contains(char::is_whitespace)
        && !inner.contains('"')
        && !inner.contains("//")
    {
        errors.push(Diagnostic::refused_fix(
            span,
            inner,
            "a manifest's value is quoted only when it needs to be (D228)",
        ));
    }
}

/// A labelled date, `2026y-11m-14d`, maybe with a counter from `.2` up (D231).
fn check_version(word: &Word<'_>, span: Span, errors: &mut Vec<Diagnostic>) {
    let (date, counter) = match word.text.split_once('.') {
        Some((date, counter)) => (date, Some(counter)),
        None => (word.text, None),
    };
    let parts: Vec<&str> = date.split('-').collect();
    let [first, second, third] = parts.as_slice() else {
        errors.push(Diagnostic::refused(span, format!("`{}` isn't a version", word.text), VERSION_WHY));
        return;
    };
    let digits = |text: &str| !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit());
    if let (Some(year), Some(month), Some(day)) = (labelled(first, 'y'), labelled(second, 'm'), labelled(third, 'd')) {
        let (Ok(year), Ok(month), Ok(day)) = (year.parse::<u32>(), month.parse::<u32>(), day.parse::<u32>()) else {
            errors.push(Diagnostic::refused(span, format!("`{date}` isn't a day on the calendar"), VERSION_WHY));
            return;
        };
        if !is_calendar_day(year, month, day) {
            errors.push(Diagnostic::refused(span, format!("`{date}` isn't a day on the calendar"), VERSION_WHY));
            return;
        }
        if let Some(counter) = counter
            && counter.parse::<u32>().is_ok_and(|number| number < 2)
        {
            errors.push(Diagnostic::refused(
                span,
                format!("`.{counter}` isn't a counter for a second release or later"),
                "the first release on a day has no counter, and a second adds `.2` (D231)",
            ));
        }
        return;
    }
    if let (Some(day), Some(month), Some(year)) = (labelled(first, 'd'), labelled(second, 'm'), labelled(third, 'y')) {
        errors.push(Diagnostic::refused_fix(span, format!("{year}y-{month}m-{day}d"), VERSION_WHY));
        return;
    }
    if digits(first) && digits(second) && digits(third) && first.len() == 4 {
        errors.push(Diagnostic::refused_fix(span, format!("{first}y-{second}m-{third}d"), VERSION_WHY));
        return;
    }
    errors.push(Diagnostic::refused(span, format!("`{}` isn't a version", word.text), VERSION_WHY));
}

fn labelled(text: &str, label: char) -> Option<&str> {
    text.strip_suffix(label).filter(|number| !number.is_empty() && number.bytes().all(|byte| byte.is_ascii_digit()))
}

fn is_calendar_day(year: u32, month: u32, day: u32) -> bool {
    let leap = (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400);
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=days).contains(&day)
}
