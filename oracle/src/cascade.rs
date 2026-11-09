//! Every error, with the knock-on ones found by applying fixes. (D207)
//!
//! The first error's fix is applied to the source in memory and the program checked again. A
//! later error that's gone was caused by it, and says so with a `because:`. The ones still there
//! are independent, and the same is done for the next. Because cascades are found this way, and
//! not guessed, the oracle and the compiler agree on them.

use crate::report::{Because, Report, Span};

/// One applied fix: the text from `start` to `end`, in the source as it was then, was replaced
/// by text `delta` bytes longer.
struct Edit {
    start: usize,
    end: usize,
    delta: isize,
}

/// Marks each error that applying an earlier one's fix makes go away. `build` gives every error
/// in a source. An error with no fix, or one written somewhere else, can't be applied, so the
/// errors after it stay independent. (D207)
/// trace: D207
pub fn mark_cascades(
    text: &str,
    mut errors: Vec<Report>,
    build: impl Fn(&str) -> Vec<Report>,
) -> Vec<Report> {
    let mut cause: Vec<Option<usize>> = vec![None; errors.len()];
    let mut current = text.to_string();
    let mut edits: Vec<Edit> = Vec::new();
    for index in 0..errors.len() {
        if cause.get(index).copied().flatten().is_some() {
            continue;
        }
        let Some(report) = errors.get(index) else {
            continue;
        };
        let Some(fix) = report.fix.as_ref().filter(|_| !report.fix_elsewhere) else {
            continue;
        };
        let Some(span) = map_span(report.span, &edits) else {
            continue;
        };
        let Some((next, edit)) = apply(&current, span, fix) else {
            continue;
        };
        edits.push(edit);
        let after = build(&next);
        for later in index + 1..errors.len() {
            let (Some(None), Some(original)) = (cause.get(later), errors.get(later)) else {
                continue;
            };
            let still_there = map_span(original.span, &edits)
                .is_some_and(|mapped| after.iter().any(|found| same(found, original, mapped)));
            if !still_there {
                if let Some(slot) = cause.get_mut(later) {
                    *slot = Some(index);
                }
            }
        }
        current = next;
    }
    let notes: Vec<Option<Because>> = cause
        .iter()
        .map(|found| {
            let earlier = errors.get((*found)?)?;
            Some(Because {
                span: earlier.span,
                note: format!(
                    "with `{}` here, this error goes away",
                    earlier.fix.clone().unwrap_or_default()
                ),
            })
        })
        .collect();
    for (report, note) in errors.iter_mut().zip(notes) {
        if let Some(note) = note {
            report.because.push(note);
        }
    }
    errors
}

/// The same error, found again at the place it moved to.
fn same(found: &Report, original: &Report, mapped: Span) -> bool {
    found.kind == original.kind
        && found.span == mapped
        && found.what == original.what
        && found.fix == original.fix
        && found.why == original.why
}

/// Replaces `span` with `fix`, recording only the part that changes, so errors next to the fix
/// still map to where they moved.
fn apply(current: &str, span: Span, fix: &str) -> Option<(String, Edit)> {
    let old = current.get(span.start..span.end)?;
    let prefix = old
        .char_indices()
        .zip(fix.chars())
        .take_while(|((_, a), b)| a == b)
        .last()
        .map_or(0, |((at, a), _)| at + a.len_utf8());
    let old_rest = old.get(prefix..)?;
    let fix_rest = fix.get(prefix..)?;
    let suffix = old_rest
        .chars()
        .rev()
        .zip(fix_rest.chars().rev())
        .take_while(|(a, b)| a == b)
        .map(|(a, _)| a.len_utf8())
        .sum::<usize>();
    let start = span.start + prefix;
    let end = span.end - suffix;
    let middle = fix_rest.get(..fix_rest.len() - suffix)?;
    let next = format!("{}{middle}{}", current.get(..start)?, current.get(end..)?);
    let delta = isize::try_from(middle.len()).ok()? - isize::try_from(end - start).ok()?;
    Some((next, Edit { start, end, delta }))
}

/// Where a span of the original source is after the fixes so far, or nothing if a fix changed
/// part of it.
fn map_span(span: Span, edits: &[Edit]) -> Option<Span> {
    let mut start = span.start;
    let mut end = span.end;
    for edit in edits {
        start = if start < edit.start {
            start
        } else if start >= edit.end {
            start.checked_add_signed(edit.delta)?
        } else {
            return None;
        };
        end = if end <= edit.start {
            end
        } else if end >= edit.end {
            end.checked_add_signed(edit.delta)?
        } else {
            return None;
        };
    }
    Some(Span::new(start, end))
}
