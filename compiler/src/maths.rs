//! `std::math`'s correctly rounded functions, from CORE-MATH's C, compiled by `OKAY\n` itself for the target being
//! built, through the clang inside Lyftar, and only when a program calls them (D161, D256, D308).

use std::path::Path;

use crate::cache;
use crate::checked::MathsFunction;
use crate::target::{Mode, Target};

/// A function's C, and the headers `std` ships for it (D308).
fn source(function: MathsFunction) -> &'static str {
    match function {
        MathsFunction::Sin => include_str!("../core-math/src/binary64/sin/sin.c"),
        MathsFunction::Cos => include_str!("../core-math/src/binary64/cos/cos.c"),
        MathsFunction::Tan => include_str!("../core-math/src/binary64/tan/tan.c"),
        // Square root is one instruction, and never compiled from C (D405).
        MathsFunction::Sqrt => "",
    }
}

const HEADERS: [(&str, &str); 2] = [
    ("fenv.h", include_str!("../core-math/include/fenv.h")),
    ("inttypes.h", include_str!("../core-math/include/inttypes.h")),
];

/// The functions a program's units call, by their symbols in its IR.
#[must_use]
pub fn called(irs: &[String]) -> Vec<MathsFunction> {
    MathsFunction::ALL
        .into_iter()
        .filter(|function| irs.iter().any(|ir| ir.contains(&format!("@{}(", function.symbol()))))
        .collect()
}

/// One function's object for a target, from the cache or compiled now. The C doesn't depend on the build mode, so
/// every mode shares it.
///
/// # Errors
/// If clang can't compile it, or there's nowhere to write it while compiling.
pub fn object(function: MathsFunction, target: Target, folder: Option<&Path>) -> Result<Vec<u8>, String> {
    let mut hasher = cache::Hasher::new();
    hasher.text("core-math");
    hasher.text(source(function));
    for (name, text) in HEADERS {
        hasher.text(name);
        hasher.text(text);
    }
    let key = cache::key(target, Mode::FinalSafe, &hasher.finish());
    if let Some(object) = folder.and_then(|folder| cache::get(folder, &key)) {
        return Ok(object);
    }
    let work = std::env::temp_dir().join(format!("okay-core-math-{}-{key}", std::process::id()));
    let made = compile(function, target, &work);
    let _ = std::fs::remove_dir_all(&work);
    let object = made?;
    if let Some(folder) = folder {
        cache::put(folder, &key, &object);
    }
    Ok(object)
}

fn compile(function: MathsFunction, target: Target, work: &Path) -> Result<Vec<u8>, String> {
    let include = work.join("include");
    std::fs::create_dir_all(&include).map_err(|error| format!("couldn't make {}: {error}", include.display()))?;
    for (name, text) in HEADERS {
        std::fs::write(include.join(name), text).map_err(|error| format!("couldn't write {name}: {error}"))?;
    }
    let file = work.join(format!("{}.c", function.symbol()));
    std::fs::write(&file, source(function)).map_err(|error| format!("couldn't write {}: {error}", file.display()))?;
    let output = work.join(format!("{}.o", function.symbol()));
    // Lyftar refuses a path that isn't UTF-8 by panicking, so it's checked here first.
    if [&include, &file, &output].iter().any(|path| path.to_str().is_none()) {
        return Err(format!("the folder {} isn't UTF-8, which clang needs", work.display()));
    }
    let folders = [include.as_path()];
    let options = lyftar::compiling::CCompileOptions { include_folders: &folders, ..Default::default() };
    lyftar::compiling::compile_c(target.link_target(), &file, &output, &options)
        .map_err(|error| format!("compiling CORE-MATH's {}: {}", function.symbol(), error.message()))?;
    std::fs::read(&output).map_err(|error| format!("couldn't read {}: {error}", output.display()))
}
