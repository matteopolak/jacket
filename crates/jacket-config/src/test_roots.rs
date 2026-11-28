//! Which folders `[test] paths` makes test roots. `jacket test` walks the tree for them
//! (`expand`); an editor asks about one folder (`is_root`). Both answer from [`names_root`].
use std::path::Path;

use crate::{Config, PathPattern, relative_text};

/// Whether `pattern` names `dir`, a folder of the project at `project`: a plain path is that
/// folder, a pattern matches it exactly (`fuzz/**` does not match `fuzz` itself).
pub fn names_root(pattern: &PathPattern, project: &Path, dir: &Path) -> bool {
    if pattern.is_literal() {
        return project.join(pattern.literal_prefix()) == dir;
    }

    dir.starts_with(project) && pattern.is_exact_match(&relative_text(dir, project))
}

/// Whether the folder `relative` (below the project) is left out of the search for test roots:
/// the patterns match a file in it.
pub fn ignored(ignore: &[PathPattern], relative: &str) -> bool {
    let inside = format!("{relative}/x");
    ignore.iter().any(|i| i.is_match(&inside))
}

/// The patterns whose folders a walk for tests, benchmarks or lints leaves out: `[format]
/// ignore` and `[lint] exclude`.
pub fn ignore_patterns(config: &Config) -> Vec<PathPattern> {
    config
        .format
        .ignore
        .iter()
        .filter_map(|p| p.parse().ok())
        .chain(config.lint.exclude.iter().cloned())
        .collect()
}

/// Whether `dir` is a test root of the project at `project` under `[test] paths`: one of the
/// folders they name (reached through no hidden, `tests` or ignored folder), or the project
/// folder itself when they name none.
pub fn is_root(config: &Config, project: &Path, dir: &Path) -> bool {
    let paths = &config.test.paths.0;

    if paths.is_empty() {
        return dir == project;
    }

    let ignore = ignore_patterns(config);
    let Ok(below) = dir.strip_prefix(project) else {
        return false;
    };
    let walked = below.iter().all(|part| {
        let part = part.to_string_lossy();
        !part.starts_with('.') && part != "tests"
    });
    let kept = dir
        .ancestors()
        .take_while(|up| *up != project)
        .all(|up| !ignored(&ignore, &relative_text(up, project)));

    paths.iter().any(|text| {
        PathPattern::new(text).is_ok_and(|pattern| {
            names_root(&pattern, project, dir) && (pattern.is_literal() || (walked && kept))
        })
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn config(text: &str) -> Config {
        toml::from_str(text).unwrap()
    }

    #[test]
    fn no_paths_make_the_project_the_root() {
        let config = Config::default();

        assert!(is_root(&config, Path::new("/p"), Path::new("/p")));
        assert!(!is_root(&config, Path::new("/p"), Path::new("/p/sub")));
    }

    #[test]
    fn paths_name_folders_and_patterns() {
        let config = config("[test]\npaths = [\"a\", \"lib/*\"]\n");
        let root = |dir: &str| is_root(&config, Path::new("/p"), &PathBuf::from(dir));

        assert!(root("/p/a"));
        assert!(root("/p/lib/x"));
        assert!(!root("/p"));
        assert!(!root("/p/lib/x/y"));
        assert!(!root("/p/b"));
    }

    #[test]
    fn patterns_skip_hidden_and_ignored_folders() {
        let config = config("[test]\npaths = [\"**\"]\n\n[lint]\nexclude = [\"fuzz/**\"]\n");
        let root = |dir: &str| is_root(&config, Path::new("/p"), &PathBuf::from(dir));

        assert!(root("/p/a"));
        assert!(!root("/p/.hidden/a"));
        assert!(!root("/p/fuzz/a"));
    }
}
