//! What the editor shows on each `[dependencies]` entry: where it is pinned and which releases
//! are newer. Looking that up asks git and the forges, so the server never does it while a
//! request waits: a [`Source`] answers from what it has and fetches the rest in the background,
//! then says it has news ([`Source::take_arrivals`]) so the hints are asked for again.
use crate::Position;
use crate::model::InlayHint;
use jacket_config::{Dependency, Resolver};
use std::path::PathBuf;

/// The dependency to look up.
#[derive(Clone, Debug, PartialEq)]
pub struct Request {
    /// The folder of the `jacket.toml`.
    pub dir: PathBuf,
    pub name: String,
    pub entry: Dependency,
    pub resolver: Resolver,
}

/// What the lookup found, each part in the words of a hint.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Standing {
    /// The tag the checked-out commit is, else the start of its hash.
    pub pinned: Option<String>,
    /// The newest release the entry's range and the release-age policy accept, when it is newer
    /// than the pin.
    pub newer: Option<String>,
    /// A newer release still inside the release-age window: its tag and how long it waits
    /// (`3 days`).
    pub waiting: Option<(String, String)>,
    /// Why nothing could be found (git or `curl` missing, the remote not answering).
    pub problem: Option<String>,
}

impl Standing {
    pub(crate) fn hints(&self, position: Position) -> Vec<InlayHint> {
        let hint = |label: String, tooltip: &str| InlayHint {
            position,
            label,
            kind: None,
            padding_left: true,
            padding_right: false,
            tooltip: Some(tooltip.to_string()),
        };
        let mut hints = Vec::new();

        if let Some(problem) = &self.problem {
            hints.push(hint("can't check for updates".into(), problem));
        }

        if let Some(pinned) = &self.pinned {
            hints.push(hint(
                format!("pinned {pinned}"),
                "What the submodule is checked out at.",
            ));
        }

        if let Some(newer) = &self.newer {
            hints.push(hint(
                format!("{newer} available"),
                "The newest release the version range and `min-release-age` accept. `jacket dep update` moves to it.",
            ));
        }

        if let Some((tag, wait)) = &self.waiting {
            hints.push(hint(
                format!("{tag} in {wait}"),
                "A newer release that is inside the release-age window; it is accepted when the wait is over.",
            ));
        }

        hints
    }
}

/// Where standings come from.
pub trait Source {
    /// The standing of `request`, if it is known now. Never waits: when it is not known, the
    /// fetch starts in the background and this answers `None` until it is done.
    fn lookup(&self, request: &Request) -> Option<Standing>;

    /// A fetch is still running.
    fn waiting(&self) -> bool;

    /// Whether a fetch finished since the last call.
    fn take_arrivals(&self) -> bool;
}
