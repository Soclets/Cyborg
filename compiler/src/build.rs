//! One build: read the manifest and the program, check them, and make the program.

use std::path::Path;

use crate::diagnostic::{Diagnostic, render_all};
use crate::source::{FileId, SourceFile, Sources, Span};
use crate::target::{Mode, Target};
use crate::{check, codegen, lexer, manifest, parser, spacing};

/// musl's MIT notice, which every Linux program `okay` links carries.
const MUSL_NOTICE: &str = include_str!("../licenses/LICENSE-musl.txt");

pub enum Outcome {
    Built,
    /// The program, or its manifest, was refused, and this is every error, ready to print.
    Refused(String),
    /// The compiler itself failed (rule 6).
    Failed(String),
}

/// Builds `file` into the program at `output`.
#[must_use]
pub fn build(file: &Path, target: Target, mode: Mode, output: &Path) -> Outcome {
    let mut sources = Sources::default();
    let text = match std::fs::read_to_string(file) {
        Ok(text) => text,
        Err(error) => {
            return Outcome::Refused(format!("okay: can't read {}: {error}\n", file.display()));
        }
    };
    let manifest_path = file.with_file_name("CyJet.siml");
    let program_id;
    let mut errors: Vec<Diagnostic> = Vec::new();
    let manifest = if let Ok(manifest_text) = std::fs::read_to_string(&manifest_path) {
        let shown = manifest_path.strip_prefix("").unwrap_or(&manifest_path).display().to_string();
        let manifest_id = sources.add(SourceFile::new(shown, manifest_text));
        program_id = sources.add(SourceFile::new(file.display().to_string(), text.clone()));
        manifest::read(manifest_id, sources.get(manifest_id), &mut errors)
    } else {
        program_id = sources.add(SourceFile::new(file.display().to_string(), text.clone()));
        errors.push(Diagnostic::refused(
            Span::new(program_id, 0, 1),
            "there's no `CyJet.siml` next to this program",
            "every program has a manifest, hello world included (D183, D229)",
        ));
        manifest::Manifest::default()
    };
    let (program, mut program_errors) = check_program(&mut sources, program_id, &text, &manifest, target, mode);
    cascades(&mut program_errors, &mut sources, program_id, &text, &manifest, target, mode);
    errors.extend(program_errors);
    if !errors.is_empty() {
        return Outcome::Refused(render_all(&errors, &sources));
    }
    let Some(program) = program else {
        return Outcome::Failed("the checker gave no program and no error".to_string());
    };
    // Lyftar panics on a build call that's wrong, which is a bug in this compiler, so it's reported as one (rule 6).
    let generated =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| codegen::object_file(&program, target, mode)));
    let object = match generated {
        Ok(Ok(object)) => object,
        Ok(Err(message)) => return Outcome::Failed(message),
        Err(panic) => {
            let message = panic
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| panic.downcast_ref::<&str>().map(|text| (*text).to_string()))
                .unwrap_or_else(|| "a panic while making machine code".to_string());
            return Outcome::Failed(message);
        }
    };
    let object_path = output.with_extension("o");
    if let Err(error) = std::fs::write(&object_path, object) {
        return Outcome::Failed(format!("can't write {}: {error}", object_path.display()));
    }
    let script_path = output.with_extension("ld");
    let script = if target.has_os() {
        None
    } else {
        if let Err(error) = std::fs::write(&script_path, codegen::BOARD_MEMORY) {
            return Outcome::Failed(format!("can't write {}: {error}", script_path.display()));
        }
        Some(script_path.as_path())
    };
    let linked = lyftar::link(target.link_target(), &[&object_path], output, script);
    let _ = std::fs::remove_file(&object_path);
    if script.is_some() {
        let _ = std::fs::remove_file(&script_path);
    }
    match linked {
        Ok(_) if matches!(target, Target::LinuxAarch64 | Target::LinuxX86_64) => {
            // A Linux program has musl inside it, so it carries musl's notice.
            let notice = output.with_file_name(format!(
                "{}.LICENSE-musl.txt",
                output.file_name().map_or_else(String::new, |name| name.to_string_lossy().into_owned())
            ));
            match std::fs::write(&notice, MUSL_NOTICE) {
                Ok(()) => Outcome::Built,
                Err(error) => Outcome::Failed(format!("can't write {}: {error}", notice.display())),
            }
        }
        Ok(_) => Outcome::Built,
        Err(error) => Outcome::Failed(format!("linking: {}", error.message())),
    }
}

/// Checks the program in `file` against its manifest, giving the checked program or every error, rendered.
///
/// # Errors
/// Every error, ready to print, if the program or its manifest was refused.
pub fn checked(file: &Path, target: Target, mode: Mode) -> Result<crate::checked::Program, String> {
    let text = std::fs::read_to_string(file).map_err(|error| format!("can't read {}: {error}", file.display()))?;
    let manifest_text = std::fs::read_to_string(file.with_file_name("CyJet.siml")).unwrap_or_default();
    let mut sources = Sources::default();
    let manifest_id = sources.add(SourceFile::new("CyJet.siml".to_string(), manifest_text));
    let program_id = sources.add(SourceFile::new(file.display().to_string(), text.clone()));
    let mut errors = Vec::new();
    let manifest = manifest::read(manifest_id, sources.get(manifest_id), &mut errors);
    let (program, program_errors) = check_program(&mut sources, program_id, &text, &manifest, target, mode);
    errors.extend(program_errors);
    match program {
        Some(program) if errors.is_empty() => Ok(program),
        _ => Err(render_all(&errors, &sources)),
    }
}

/// Checks a program's text, giving the checked program if nothing was refused, and every error in order.
fn check_program(
    sources: &mut Sources,
    program_id: FileId,
    text: &str,
    manifest: &manifest::Manifest,
    target: Target,
    mode: Mode,
) -> (Option<crate::checked::Program>, Vec<Diagnostic>) {
    if let Some(file) = sources.get_mut(program_id) {
        let name = file.name.clone();
        *file = SourceFile::new(name, text.to_string());
    }
    let mut errors = Vec::new();
    let lexed = lexer::lex(program_id, text, &mut errors);
    let (parsed, parse_errors) = parser::parse(program_id, text, &lexed.tokens);
    // After a parse error, the parser's view of what each token does can't be trusted, so line breaks aren't judged.
    let parsed_cleanly = parse_errors.is_empty();
    errors.extend(parse_errors);
    errors.extend(spacing::check(program_id, text, &lexed.tokens, &parsed.roles, parsed_cleanly));
    let (program, check_errors) = check::check(program_id, sources, manifest, &parsed.file, target, mode);
    errors.extend(check_errors);
    errors.sort_by_key(|error| error.span.start);
    (program, errors)
}

/// Finds which errors an earlier one caused (D207, standard 11.7): each error with a fix has it applied in memory,
/// the program is checked again, and a later error that goes away points back at the fix with `because:`.
fn cascades(
    errors: &mut [Diagnostic],
    sources: &mut Sources,
    program_id: FileId,
    text: &str,
    manifest: &manifest::Manifest,
    target: Target,
    mode: Mode,
) {
    for index in 0..errors.len() {
        let Some(fix) = errors[index].fix.clone() else { continue };
        if index + 1 >= errors.len() || fix.edits.iter().any(|edit| edit.span.file != program_id) {
            continue;
        }
        let Some((fixed_text, shift)) = apply(text, &fix.edits) else { continue };
        let (_, after) = check_program(sources, program_id, &fixed_text, manifest, target, mode);
        for later in index + 1..errors.len() {
            let moved = shift(errors[later].span);
            let still_there = after.iter().any(|other| same_error(other, &errors[later], moved));
            if !still_there {
                let note = format!("with `{}` here, this error goes away", fix.shown);
                errors[later].because.push(crate::diagnostic::Because { span: errors[index].span, note });
            }
        }
    }
    // The program's text is the original again, for printing.
    if let Some(file) = sources.get_mut(program_id) {
        let name = file.name.clone();
        *file = SourceFile::new(name, text.to_string());
    }
}

/// Applies edits to a text, giving the new text and how a span after them moves.
fn apply(text: &str, edits: &[crate::diagnostic::Edit]) -> Option<(String, impl Fn(Span) -> Span)> {
    let mut sorted: Vec<&crate::diagnostic::Edit> = edits.iter().collect();
    sorted.sort_by_key(|edit| edit.span.start);
    let mut result = String::with_capacity(text.len());
    let mut at = 0usize;
    let mut moves: Vec<(u32, i64)> = Vec::new();
    for edit in &sorted {
        let start = edit.span.start as usize;
        let end = edit.span.end as usize;
        if start < at || end > text.len() {
            return None;
        }
        result.push_str(text.get(at..start)?);
        result.push_str(&edit.replacement);
        at = end;
        let change = i64::try_from(edit.replacement.len()).ok()? - i64::try_from(end - start).ok()?;
        moves.push((edit.span.end, change));
    }
    result.push_str(text.get(at..)?);
    // A span after every edit moves by their changes, and one that overlaps an edit keeps its length from its end.
    let shift = move |span: Span| {
        let delta_at =
            |position: u32| -> i64 { moves.iter().filter(|(end, _)| position >= *end).map(|(_, change)| change).sum() };
        let moved = |position: u32, delta: i64| u32::try_from(i64::from(position) + delta).unwrap_or(position);
        let end = moved(span.end, delta_at(span.end));
        let start = end.saturating_sub(span.end - span.start);
        Span { file: span.file, start, end }
    };
    Some((result, shift))
}

/// Whether an error after a fix is the same as one before it, where it has moved to.
fn same_error(after: &Diagnostic, before: &Diagnostic, moved: Span) -> bool {
    after.kind == before.kind
        && after.span == moved
        && after.what == before.what
        && after.why == before.why
        && after.fix.as_ref().map(|fix| &fix.shown) == before.fix.as_ref().map(|fix| &fix.shown)
}
