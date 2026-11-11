//! One build: read the manifest and the program, check them, and make the program.

use std::path::Path;

use crate::diagnostic::{Diagnostic, render_all};
use crate::source::{SourceFile, Sources, Span};
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
    let mut program_errors = Vec::new();
    let lexed = lexer::lex(program_id, &text, &mut program_errors);
    let (parsed, parse_errors) = parser::parse(program_id, &text, &lexed.tokens);
    // After a parse error, the parser's view of what each token does can't be trusted, so line breaks aren't judged.
    let parsed_cleanly = parse_errors.is_empty();
    program_errors.extend(parse_errors);
    program_errors.extend(spacing::check(program_id, &text, &lexed.tokens, &parsed.roles, parsed_cleanly));
    let (program, check_errors) = check::check(program_id, &sources, &manifest, &parsed.file, target, mode);
    program_errors.extend(check_errors);
    program_errors.sort_by_key(|error| error.span.start);
    errors.extend(program_errors);
    if !errors.is_empty() {
        return Outcome::Refused(render_all(&errors, &sources));
    }
    let Some(program) = program else {
        return Outcome::Failed("the checker gave no program and no error".to_string());
    };
    let object = match codegen::object_file(&program, target, mode) {
        Ok(object) => object,
        Err(message) => return Outcome::Failed(message),
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
