//! Which root reports each file when several roots compile it.
use jaic::fxhash::{HashMap, HashSet};
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::{Condvar, Mutex};

/// Which root reports each file. A file several roots compile is reported from the first of
/// them (by position in the sorted root list), so a root need not lint a file an earlier one
/// compiled; it only has to know which files those were. Roots run concurrently, so each
/// publishes the files it compiled as soon as it has, and then waits for every earlier root to
/// have done the same.
pub(super) struct Claims {
    state: Mutex<ClaimState>,
    published: Condvar,
}

struct ClaimState {
    /// The lowest published root that compiled each file.
    owner: HashMap<PathBuf, usize>,
    done: Vec<bool>,
    /// Roots `0..prefix` have all published.
    prefix: usize,
}

impl Claims {
    pub(super) fn new(roots: usize) -> Claims {
        Claims {
            state: Mutex::new(ClaimState {
                owner: HashMap::default(),
                done: vec![false; roots],
                prefix: 0,
            }),
            published: Condvar::new(),
        }
    }

    /// Record the files root `index` compiled. Publishing twice does nothing.
    pub(super) fn publish(&self, index: usize, files: impl IntoIterator<Item = PathBuf>) {
        let mut state = self.state.lock().expect("claims lock");
        if state.done[index] {
            return;
        }
        for file in files {
            let slot = state.owner.entry(file).or_insert(index);
            *slot = (*slot).min(index);
        }
        state.done[index] = true;
        while state.prefix < state.done.len() && state.done[state.prefix] {
            state.prefix += 1;
        }
        self.published.notify_all();
    }

    /// Wait until every root before `index` has published, then say which of `files` root
    /// `index` reports.
    pub(super) fn owned(&self, index: usize, files: &BTreeSet<PathBuf>) -> HashSet<PathBuf> {
        let mut state = self.state.lock().expect("claims lock");
        while state.prefix < index {
            state = self.published.wait(state).expect("claims lock");
        }
        files
            .iter()
            .filter(|file| state.owner.get(*file) == Some(&index))
            .cloned()
            .collect()
    }
}

/// Publishes nothing on behalf of a root that stops before publishing, so the roots after it
/// are not left waiting.
pub(super) struct Release<'a> {
    pub(super) claims: &'a Claims,
    pub(super) index: usize,
}

impl Drop for Release<'_> {
    fn drop(&mut self) {
        self.claims.publish(self.index, std::iter::empty());
    }
}
