use std::{
    collections::HashSet,
    path::{Path, PathBuf, absolute},
};

/// Resolves `path` into a flat list of audio file paths.
/// If `path` is an `.m3u` file, its contents are expanded recursively.
/// Cycles are detected via canonicalized paths and silently skipped.
pub async fn expand(path: PathBuf) -> Vec<PathBuf> {
    tokio::task::spawn_blocking(move || {
        let mut visited = HashSet::new();
        expand_inner(&path, &mut visited)
    })
    .await
    .unwrap_or_default()
}

fn expand_inner(path: &Path, visited: &mut HashSet<PathBuf>) -> Vec<PathBuf> {
    let canonical = match std::fs::canonicalize(path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Cannot resolve {:?}: {e}", path);
            return vec![];
        }
    };

    if !visited.insert(canonical) {
        eprintln!("Cycle detected, skipping {:?}", path);
        return vec![];
    }

    if !is_playlist(path) {
        return vec![path.to_path_buf()];
    }

    let contents = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Cannot read playlist {:?}: {e}", path);
            return vec![];
        }
    };

    let base = path.parent().unwrap_or(Path::new("."));
    let mut result = Vec::new();

    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let entry = if Path::new(line).is_absolute() {
            PathBuf::from(line)
        } else {
            base.join(line)
        };
        result.extend(expand_inner(&entry, visited));
    }

    result
}

fn is_playlist(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("m3u"))
        .unwrap_or(false)
}
