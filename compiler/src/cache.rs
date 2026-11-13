//! The object cache (D168): each unit's object file, kept between builds under a hash of everything it depends on, so
//! a unit that hasn't changed isn't optimized or compiled again, in this project or any other.
//!
//! It lives in the system's cache folder, `~/Library/Caches/okay` on macOS and `~/.cache/okay` on Linux, or wherever
//! `OKAY_CACHE` says, so nothing is added to a project's folder. Without one, every build is a fresh build.

use std::path::{Path, PathBuf};

use crate::target::{Mode, Target};

/// Where cached objects go, if anywhere.
#[must_use]
pub fn folder() -> Option<PathBuf> {
    if let Some(chosen) = std::env::var_os("OKAY_CACHE") {
        return Some(PathBuf::from(chosen));
    }
    let home = std::env::var_os("HOME").map(PathBuf::from);
    if cfg!(target_os = "macos") {
        return home.map(|home| home.join("Library").join("Caches").join("okay"));
    }
    if let Some(cache) = std::env::var_os("XDG_CACHE_HOME") {
        return Some(PathBuf::from(cache).join("okay"));
    }
    home.map(|home| home.join(".cache").join("okay"))
}

/// Which compiler this is: its own file's size and time, so a rebuilt compiler, which may make different code from
/// the same IR, never reuses an older one's objects.
fn compiler() -> String {
    let found = std::env::current_exe().and_then(std::fs::metadata);
    match found {
        Ok(metadata) => {
            let time = metadata
                .modified()
                .ok()
                .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                .map_or(0, |since| since.as_nanos());
            format!("{} {time} {}", metadata.len(), env!("CARGO_PKG_VERSION"))
        }
        Err(_) => env!("CARGO_PKG_VERSION").to_string(),
    }
}

/// The name a unit's object is kept under: a hash of the compiler, the target, the mode and the unit's IR.
#[must_use]
pub fn key(target: Target, mode: Mode, ir: &str) -> String {
    let mut hasher = lyftar::blake3::Blake3Hasher::new();
    hasher.update(compiler().as_bytes());
    hasher.update(b"\0");
    hasher.update(target.triple().as_bytes());
    hasher.update(b"\0");
    hasher.update(format!("{mode:?}").as_bytes());
    hasher.update(b"\0");
    hasher.update(ir.as_bytes());
    hasher.finalize().iter().fold(String::new(), |mut text, byte| {
        use std::fmt::Write;
        let _ = write!(text, "{byte:02x}");
        text
    })
}

fn path(folder: &Path, key: &str) -> PathBuf {
    folder.join("objects").join(format!("{key}.o"))
}

/// A cached object, if there is one. Its time is set to now, so trimming keeps what's used.
#[must_use]
pub fn get(folder: &Path, key: &str) -> Option<Vec<u8>> {
    let path = path(folder, key);
    let object = std::fs::read(&path).ok()?;
    if let Ok(file) = std::fs::File::options().append(true).open(&path) {
        let _ = file.set_modified(std::time::SystemTime::now());
    }
    Some(object)
}

/// The most objects kept, and how many are left after trimming, so the cache can't grow without end.
const MOST_KEPT: usize = 4096;
const KEPT_AFTER_TRIMMING: usize = 3072;

/// Removes the objects used longest ago once there are too many.
fn trim(objects: &Path) {
    let Ok(entries) = std::fs::read_dir(objects) else { return };
    let mut found: Vec<(std::time::SystemTime, PathBuf)> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let time = entry.metadata().ok()?.modified().ok()?;
            Some((time, entry.path()))
        })
        .collect();
    if found.len() <= MOST_KEPT {
        return;
    }
    found.sort();
    for (_, path) in &found[..found.len() - KEPT_AFTER_TRIMMING] {
        let _ = std::fs::remove_file(path);
    }
}

/// Keeps an object for later builds. It's written apart first and then renamed, so two builds at once never see half
/// of one, and a cache that can't be written only makes builds slower.
pub fn put(folder: &Path, key: &str, object: &[u8]) {
    let objects = folder.join("objects");
    if std::fs::create_dir_all(&objects).is_err() {
        return;
    }
    let thread =
        format!("{:?}", std::thread::current().id()).replace(|character: char| !character.is_alphanumeric(), "");
    let partial = objects.join(format!("{key}.{}.{thread}.partial", std::process::id()));
    if std::fs::write(&partial, object).is_ok() && std::fs::rename(&partial, path(folder, key)).is_err() {
        let _ = std::fs::remove_file(&partial);
    }
    trim(&objects);
}
