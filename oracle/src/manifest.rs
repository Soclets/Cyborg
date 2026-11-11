//! The manifest, `CyJet.siml`: one line per fact, starting with what it is, with no sections,
//! closers, `=` or `;`. (D183, D228, D229, D230, D231)
//!
//! ```text
//! name weather
//! ver 2026y-11m-14d same
//! okay 2026y-11m-01d
//!
//! need std 2026y-11m-01d
//! ```

use crate::report::{Outcome, Report, ReportExtra, Span};

/// The packages a manifest needs, by name. The oracle has only one `std` and one `csup`, so a
/// version is checked for its form and nothing else.
pub struct Manifest {
    pub needs: Vec<String>,
}

/// The packages the oracle has. Any other is found some way that isn't designed.
const KNOWN_PACKAGES: &[&str] = &["std", "csup", "os"];

const WHY_LINES: &str =
    "a manifest has one line per fact, starting with what it is: `name`, `ver`, `okay` or `need` (D228, D229)";
const WHY_VERSION: &str =
    "a version is a labelled date, biggest first, as in `2026y-11m-14d`, with `.2` for a second release that day (D231)";

/// One word of a line, with where it is in the manifest.
struct Word {
    text: String,
    span: Span,
}

/// Reads a manifest, giving what it needs and every error in it. (D207)
/// trace: D183, D228, D229, D230, D231
pub fn read(text: &str) -> (Manifest, Vec<Report>) {
    let mut needs: Vec<String> = Vec::new();
    let mut errors: Vec<Report> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    let mut start = 0;
    for line in text.split('\n') {
        let line_start = start;
        start += line.len() + 1;
        // A broken line still says what fact it is and which package it needs, so the errors
        // after it aren't knock-ons, as a skipped `let` still makes its variable. (D215)
        let mut plain = line.split("//").next().unwrap_or("").split_whitespace();
        let (fact, named) = (plain.next(), plain.next());
        let words = match words_of(line, line_start) {
            Ok(words) => words,
            Err(report) => {
                errors.push(*report);
                if let Some(fact) = fact.filter(|fact| matches!(*fact, "name" | "ver" | "okay")) {
                    seen.push(fact.to_string());
                }
                if let (Some("need"), Some(package)) = (fact, named) {
                    needs.push(package.to_string());
                }
                continue;
            }
        };
        if words.is_empty() {
            continue;
        }
        match read_fact(&words, &mut seen) {
            Ok(Some(package)) => needs.push(package),
            Ok(None) => {}
            Err(report) => {
                errors.push(*report);
                if let (Some("need"), Some(package)) = (fact, named) {
                    needs.push(package.to_string());
                }
            }
        }
    }
    // `name`, `ver` and `okay` are in every example, but nothing says one can't be left out.
    let end = Span::new(text.len(), text.len());
    for fact in ["name", "ver", "okay"] {
        if !seen.iter().any(|found| found == fact) {
            errors.push(*Report::not_designed(
                end,
                format!("whether a manifest can leave out its `{fact}` line"),
                "12.9",
            ));
        }
    }
    (Manifest { needs }, errors)
}

/// A line's words, with a `//` comment and the spaces between words left out. A quoted value
/// is one word, and is quoted only when it needs to be. (D230)
fn words_of(line: &str, line_start: usize) -> Outcome<Vec<Word>> {
    let mut words = Vec::new();
    let mut characters = line.char_indices().peekable();
    while let Some((at, character)) = characters.next() {
        if character.is_whitespace() {
            continue;
        }
        if line.get(at..).is_some_and(|rest| rest.starts_with("//")) {
            break;
        }
        if character == '"' {
            let mut value = String::new();
            let mut closed = None;
            for (inside_at, inside) in characters.by_ref() {
                if inside == '"' {
                    closed = Some(inside_at);
                    break;
                }
                if inside == '\\' {
                    return Err(Report::not_designed(
                        Span::new(line_start + inside_at, line_start + inside_at + 1),
                        "escapes inside a manifest's quoted value",
                        "12.9",
                    ));
                }
                value.push(inside);
            }
            let Some(end) = closed else {
                return Err(Report::refused(
                    Span::new(line_start + at, line_start + line.trim_end().len()),
                    "this quoted value never closes",
                    WHY_LINES,
                ));
            };
            let span = Span::new(line_start + at, line_start + end + 1);
            // trace: D228
            if !needs_quotes(&value) {
                return Err(Report::refused(
                    span,
                    "this value doesn't need quotes",
                    "a manifest's value is quoted only when it needs to be (D228)",
                )
                .fix(value));
            }
            words.push(Word { text: value, span });
            continue;
        }
        let mut end = at + character.len_utf8();
        while let Some(&(next_at, next)) = characters.peek() {
            if next.is_whitespace() || next == '"' {
                break;
            }
            if line
                .get(next_at..)
                .is_some_and(|rest| rest.starts_with("//"))
            {
                break;
            }
            end = next_at + next.len_utf8();
            characters.next();
        }
        words.push(Word {
            text: line.get(at..end).unwrap_or("").to_string(),
            span: Span::new(line_start + at, line_start + end),
        });
    }
    Ok(words)
}

/// A value needs quotes when it couldn't be read as one word without them.
fn needs_quotes(value: &str) -> bool {
    value.is_empty() || value.contains(char::is_whitespace) || value.contains("//")
}

/// Checks one fact, and gives the package a `need` line names.
fn read_fact(words: &[Word], seen: &mut Vec<String>) -> Outcome<Option<String>> {
    let Some(first) = words.first() else {
        return Ok(None);
    };
    let line_span = first
        .span
        .to(words.last().map_or(first.span, |last| last.span));
    let fact = first.text.as_str();
    match fact {
        "name" | "ver" | "okay" => {
            if seen.iter().any(|found| found == fact) {
                return Err(Report::not_designed(
                    line_span,
                    format!("a second `{fact}` line"),
                    "12.9",
                ));
            }
            seen.push(fact.to_string());
        }
        "need" => {}
        // trace: D265
        "rule" => {
            return Err(Report::not_designed(
                line_span,
                "a `rule` line, since rule names and the words `rule`, `refuse`, `allow` and `in` are placeholders",
                "12.9",
            ));
        }
        // A file doesn't say which package it belongs to, like Go's `package main`. (D228)
        // trace: D228
        _ => {
            return Err(Report::refused(
                first.span,
                format!("`{fact}` isn't something a manifest says"),
                WHY_LINES,
            ));
        }
    }
    let value = |index: usize, what: &str| -> Outcome<&Word> {
        words.get(index).ok_or_else(|| {
            Report::refused(
                line_span,
                format!("this `{fact}` line has no {what}"),
                WHY_LINES,
            )
        })
    };
    let refuse_extra = |from: usize| -> Outcome<()> {
        match words.get(from) {
            Some(extra) => Err(Report::refused(
                extra
                    .span
                    .to(words.last().map_or(extra.span, |last| last.span)),
                format!("a `{fact}` line ends before this"),
                WHY_LINES,
            )),
            None => Ok(()),
        }
    };
    match fact {
        "name" => {
            value(1, "name")?;
            refuse_extra(2)?;
            Ok(None)
        }
        "okay" => {
            check_version(value(1, "compiler version")?)?;
            refuse_extra(2)?;
            Ok(None)
        }
        "ver" => {
            check_version(value(1, "version")?)?;
            // trace: D231
            let flag = words.get(2).ok_or_else(|| {
                Report::refused(
                    line_span,
                    "a release says whether it `breaks` anything or stays the `same`",
                    "a release's version is its date plus whether it breaks (D231)",
                )
            })?;
            if flag.text != "breaks" && flag.text != "same" {
                return Err(Report::refused(
                    flag.span,
                    format!("`{}` isn't `breaks` or `same`", flag.text),
                    "a release's version is its date plus whether it breaks (D231)",
                ));
            }
            refuse_extra(3)?;
            Ok(None)
        }
        _ => {
            let package = value(1, "package")?;
            read_need(words, package, line_span)?;
            refuse_extra(3)?;
            if !KNOWN_PACKAGES.contains(&package.text.as_str()) {
                return Err(Report::not_designed(
                    package.span,
                    format!("how the package `{}` is found", package.text),
                    "12.9",
                ));
            }
            Ok(Some(package.text.clone()))
        }
    }
}

/// A `need` line's version, and any features after it.
/// trace: D228, D231
fn read_need(words: &[Word], package: &Word, line_span: Span) -> Outcome<()> {
    let date = words.get(2).ok_or_else(|| {
        Report::refused(
            line_span,
            format!("`need {}` says no version", package.text),
            "each package a program uses has a `need` line with its version (D228, D231)",
        )
    })?;
    check_version(date)?;
    if let Some(with) = words.get(3).filter(|word| word.text == "with") {
        return Err(Report::not_designed(
            with.span
                .to(words.last().map_or(with.span, |last| last.span)),
            "a package's features, which `with` adds",
            "12.9",
        ));
    }
    Ok(())
}

/// `2026y-11m-14d`, or `2026y-11m-14d.2` for a second release that day: four digits of year,
/// two of month and two of day, so it sorts as plain text. (D231)
/// trace: D231
fn check_version(word: &Word) -> Outcome<()> {
    let (date, counter) = match word.text.split_once('.') {
        Some((date, counter)) => (date, Some(counter)),
        None => (word.text.as_str(), None),
    };
    let Some((year, month, day)) = labelled_date(date) else {
        let refused = Report::refused(
            word.span,
            format!("`{}` isn't a version", word.text),
            WHY_VERSION,
        );
        return Err(match unlabelled_date(date) {
            Some(fixed) => refused.fix(match counter {
                Some(counter) => format!("{fixed}.{counter}"),
                None => fixed,
            }),
            None => refused,
        });
    };
    if !is_a_real_day(year, month, day) {
        return Err(Report::refused(
            word.span,
            format!("`{date}` isn't a day on the calendar"),
            WHY_VERSION,
        ));
    }
    if let Some(counter) = counter {
        let number: Option<u32> = counter
            .chars()
            .all(|digit| digit.is_ascii_digit())
            .then(|| counter.parse().ok())
            .flatten();
        if number.is_none_or(|number| number < 2) || counter.starts_with('0') {
            return Err(Report::refused(
                word.span,
                format!("`.{counter}` isn't a counter for a second release or later"),
                "the first release on a day has no counter, and a second adds `.2` (D231)",
            ));
        }
    }
    Ok(())
}

/// The year, month and day of `2026y-11m-14d`.
fn labelled_date(text: &str) -> Option<(u32, u32, u32)> {
    let mut parts = text.split('-');
    let year = number_with(parts.next()?, 'y', 4)?;
    let month = number_with(parts.next()?, 'm', 2)?;
    let day = number_with(parts.next()?, 'd', 2)?;
    parts.next().is_none().then_some((year, month, day))
}

fn number_with(part: &str, label: char, digits: usize) -> Option<u32> {
    let number = part.strip_suffix(label)?;
    (number.len() == digits && number.chars().all(|digit| digit.is_ascii_digit()))
        .then(|| number.parse().ok())
        .flatten()
}

/// The labelled form of a date written another way, when there's exactly one: an unlabelled
/// `2026-11-14`, or the labels in another order, like `14d-11m-2026y`.
fn unlabelled_date(text: &str) -> Option<String> {
    let parts: Vec<&str> = text.split('-').collect();
    let [first, second, third] = parts.as_slice() else {
        return None;
    };
    let digits = |part: &str| !part.is_empty() && part.chars().all(|digit| digit.is_ascii_digit());
    if digits(first) && first.len() == 4 && digits(second) && digits(third) {
        let month: u32 = second.parse().ok()?;
        let day: u32 = third.parse().ok()?;
        let year: u32 = first.parse().ok()?;
        return is_a_real_day(year, month, day)
            .then(|| format!("{year:04}y-{month:02}m-{day:02}d"));
    }
    let mut year = None;
    let mut month = None;
    let mut day = None;
    for part in &parts {
        if let Some(found) = number_with(part, 'y', 4) {
            year = Some(found);
        } else if let Some(found) = part.strip_suffix('m').filter(|found| digits(found)) {
            month = found.parse::<u32>().ok();
        } else if let Some(found) = part.strip_suffix('d').filter(|found| digits(found)) {
            day = found.parse::<u32>().ok();
        }
    }
    let (year, month, day) = (year?, month?, day?);
    is_a_real_day(year, month, day).then(|| format!("{year:04}y-{month:02}m-{day:02}d"))
}

fn is_a_real_day(year: u32, month: u32, day: u32) -> bool {
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=days).contains(&day)
}
