//! Which not-yet-imported module to suggest first, for the "import it" help of an unknown name
//! and for the language server's auto-import completion.
//!
//! The order is derived, not listed: modules the project already imports come first (most
//! imports first), then the stdlib modules that more other stdlib modules import (the ones
//! most code reaches for), and `Extensions/` modules last. Ties go by name.
use crate::fxhash::HashMap;

/// How often modules are imported, for ordering suggestions.
#[derive(Clone, Debug, Default)]
pub struct ModuleRanking {
    /// Imports per module in the project's own files.
    project: HashMap<String, u32>,
    /// How many other stdlib modules import each stdlib module.
    importers: HashMap<String, u32>,
}

/// A position in the suggestions: lower is suggested first.
pub type Rank = u64;

const FIELD: u32 = 20;
const MAX_FIELD: u64 = (1 << FIELD) - 1;

impl ModuleRanking {
    pub fn new(project: HashMap<String, u32>, importers: HashMap<String, u32>) -> Self {
        Self {
            project,
            importers,
        }
    }

    /// The same ranking with the project's import counts replaced.
    #[must_use]
    pub fn with_project(mut self, project: HashMap<String, u32>) -> Self {
        self.project = project;
        self
    }

    /// The module's rank: project imports, then stdlib importers, then `Extensions/` last. Ranks
    /// start at 10, so callers can place their own groups before them.
    pub fn rank(&self, module: &str) -> Rank {
        let count = |counts: &HashMap<String, u32>| {
            MAX_FIELD - u64::from(counts.get(module).copied().unwrap_or(0)).min(MAX_FIELD)
        };
        let extension = u64::from(module.contains('/'));
        10 + (extension << (2 * FIELD)) + (count(&self.project) << FIELD) + count(&self.importers)
    }
}

/// For each module, how many of the other `modules` import it. `modules` pairs a module's name
/// with the paths its files `#import`.
pub fn count_importers<'a, I, P>(modules: I) -> HashMap<String, u32>
where
    I: IntoIterator<Item = (&'a str, P)>,
    P: IntoIterator<Item = &'a str>,
{
    let mut counts: HashMap<String, u32> = HashMap::default();
    for (module, imports) in modules {
        let mut seen: Vec<&str> = Vec::new();
        for path in imports {
            if path != module && !seen.contains(&path) {
                seen.push(path);
                *counts.entry(path.to_string()).or_insert(0) += 1;
            }
        }
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    fn counts(pairs: &[(&str, u32)]) -> HashMap<String, u32> {
        pairs.iter().map(|(m, n)| ((*m).to_string(), *n)).collect()
    }

    #[test]
    fn project_imports_beat_importers_and_extensions_come_last() {
        let ranking = ModuleRanking::new(
            counts(&[("Hash", 3)]),
            counts(&[("Basic", 40), ("Math", 9), ("Extensions/Pool", 99)]),
        );
        let mut modules = vec!["Extensions/Pool", "Math", "Hash", "Unseen", "Basic"];
        modules.sort_by_key(|m| (ranking.rank(m), *m));
        assert_eq!(
            modules,
            ["Hash", "Basic", "Math", "Unseen", "Extensions/Pool"]
        );
    }

    #[test]
    fn importers_are_counted_once_per_module_and_not_for_itself() {
        let importers =
            count_importers([("A", vec!["B", "B", "A"]), ("C", vec!["B"]), ("B", vec![])]);
        assert_eq!(importers.get("B"), Some(&2));
        assert_eq!(importers.get("A"), None);
    }
}
