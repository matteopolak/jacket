//! Where one dependency stands, for editors: what its submodule is at, the newest release its
//! range and the release-age policy accept, and a newer one still inside the policy's window.
//! Nothing here changes the project; it asks what `jacket dep update` would.
use std::path::Path;

use jacket_config::{Dependency, Resolver};

use super::commands::{Gate, Verdict, path_of, short, source_of};
use super::policy;
use super::resolve::{self, Asked, Reading};
use super::version::Tagged;
use super::{Ctx, DepError, Result, forge, git};

/// What is known of a dependency.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Standing {
    /// The tag the checked-out commit is, else the start of its hash.
    pub pinned: Option<String>,
    /// The newest release that is newer than the pin and that the entry's range and the
    /// policy accept.
    pub newer: Option<String>,
    /// A newer release the policy does not accept yet, with the seconds it still waits.
    pub waiting: Option<(String, i64)>,
}

/// The standing of dependency `name` of the project in `dir`. Asks the remote for its tags and
/// the forge for release times, so it can take a while.
pub fn standing(
    ctx: &mut Ctx,
    dir: &Path,
    name: &str,
    entry: &Dependency,
    resolver: &Resolver,
) -> Result<Standing> {
    let path = path_of(name, entry)?;
    let known = ctx.git.submodules(dir)?;
    let sub = known.iter().find(|s| s.path == path);
    let reading = Reading::of(entry).map_err(|e| DepError::new(format!("{name}: {e}")))?;
    let at_pin = match sub {
        Some(sub) if ctx.git.initialised(dir, &path)? => {
            ctx.git.tags_at(dir, &path, &sub.commit)?
        }
        _ => Vec::new(),
    };
    let tag = resolve::resolved(&reading, &at_pin);
    let current = tag.and_then(|tag| reading.parse(tag.as_str()));
    let mut out = Standing {
        pinned: match (tag, sub) {
            (Some(tag), _) => Some(tag.to_string()),
            (None, Some(sub)) => Some(short(&sub.commit).to_string()),
            (None, None) => None,
        },
        ..Standing::default()
    };
    let url = source_of(entry)?.resolved(dir).url();
    let tags = ctx.git.remote_tags(&url)?;
    let range = match Asked::of(entry).map_err(DepError::new)? {
        Asked::Range(requirement) => Some(requirement),
        _ => None,
    };
    let newer: Vec<Tagged> = resolve::matching(
        &reading,
        &tags,
        range.as_ref().map(super::version::Requirement::range),
    )
    .into_iter()
    .filter(|t| current.as_ref().is_none_or(|current| t.version > *current))
    .collect();
    let gate = Gate::from_resolver(resolver, name, entry, &url)?;

    for candidate in newer {
        match gate.check(ctx, &candidate.tag, &candidate.commit) {
            Verdict::Allowed => {
                out.newer = Some(candidate.tag);
                break;
            }
            Verdict::TooNew {
                age,
                required,
            } => {
                out.waiting.get_or_insert((candidate.tag, required - age));
            }
            // No release time to compare: `jacket dep update` stops here too.
            Verdict::Unknown(_) => break,
        }
    }

    Ok(out)
}

/// [`standing`] with the real git, the forges and the release-time cache: for a background
/// thread of an editor. The error is the text `jacket dep` would print.
pub fn lookup(
    dir: &Path,
    name: &str,
    entry: &Dependency,
    resolver: &Resolver,
) -> std::result::Result<Standing, String> {
    let mut cache = forge::Cache::open();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64);
    let forges = forge::Http::from_env();

    forge::Forge::preflight(&forges).map_err(describe)?;
    let mut ctx = Ctx {
        git: &git::RealGit,
        forge: &forges,
        cache: &mut cache,
        now,
        out: &mut std::io::sink(),
        err: &mut std::io::sink(),
    };
    let found = standing(&mut ctx, dir, name, entry, resolver).map_err(describe);

    cache.save();
    found
}

fn describe(error: DepError) -> String {
    let mut text = error.message;

    for note in error.notes {
        text.push_str("; ");
        text.push_str(note.lines().next().unwrap_or(""));
    }

    text
}

/// The wait of a [`Standing::waiting`] release as a hint says it (`3d 5h`).
pub fn wait_text(seconds: i64) -> String {
    policy::span(seconds)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dep::forge::{Fetched, Forge, ForgeError, Kind, Release, Repo};
    use crate::dep::git::{Git, RemoteTag, Submodule};
    use crate::dep::spec::RepoUrl;
    use crate::dep::types::{CommitId, RefName, Revision, SubPath, Target};

    const NOW: i64 = 10_000_000;
    const DAY: i64 = 86_400;

    struct Remote(Vec<(&'static str, i64)>);

    fn commit(tag: &str) -> CommitId {
        CommitId::named(tag)
    }

    impl Git for Remote {
        fn preflight(&self, _: &Path) -> super::super::Result<()> {
            Ok(())
        }

        fn remote_tags(&self, _: &RepoUrl) -> super::super::Result<Vec<RemoteTag>> {
            Ok(self
                .0
                .iter()
                .map(|(tag, _)| RemoteTag {
                    name: RefName::parse(tag).unwrap(),
                    commit: commit(tag),
                })
                .collect())
        }

        fn remote_head(&self, _: &RepoUrl) -> super::super::Result<CommitId> {
            unreachable!()
        }

        fn submodules(&self, _: &Path) -> super::super::Result<Vec<Submodule>> {
            Ok(vec![Submodule {
                path: SubPath::parse("modules/jaison").unwrap(),
                commit: commit("v1.2.0"),
                url: None,
            }])
        }

        fn add(&self, _: &Path, _: &RepoUrl, _: &SubPath, _: &Target) -> super::super::Result<()> {
            unreachable!()
        }

        fn move_to(&self, _: &Path, _: &SubPath, _: &Target) -> super::super::Result<()> {
            unreachable!()
        }

        fn sync(&self, _: &Path, _: &[SubPath]) -> super::super::Result<()> {
            unreachable!()
        }

        fn nested(&self, _: &Path, _: &SubPath, _: &[SubPath]) -> super::super::Result<()> {
            unreachable!()
        }

        fn remove(&self, _: &Path, _: &SubPath) -> super::super::Result<()> {
            unreachable!()
        }

        fn local_commit(
            &self,
            _: &Path,
            _: &SubPath,
            _: &Revision,
        ) -> super::super::Result<Option<CommitId>> {
            unreachable!()
        }

        fn tags_at(
            &self,
            _: &Path,
            _: &SubPath,
            _: &CommitId,
        ) -> super::super::Result<Vec<RefName>> {
            Ok(vec![RefName::parse("v1.2.0").unwrap()])
        }

        fn initialised(&self, _: &Path, _: &SubPath) -> super::super::Result<bool> {
            Ok(true)
        }

        fn fetch(&self, _: &Path, _: &SubPath) -> super::super::Result<()> {
            unreachable!()
        }

        fn rename(&self, _: &Path, _: &SubPath, _: &SubPath) -> super::super::Result<()> {
            unreachable!()
        }

        fn head(&self, _: &Path, _: &SubPath) -> super::super::Result<CommitId> {
            unreachable!()
        }

        fn remove_tree(&self, _: &Path, _: &SubPath) -> super::super::Result<()> {
            unreachable!()
        }
    }

    impl Forge for Remote {
        fn releases(
            &self,
            _: Kind,
            _: &Repo,
            _: Option<&str>,
        ) -> std::result::Result<Fetched, ForgeError> {
            Ok(Fetched::Fresh {
                etag: None,
                releases: self
                    .0
                    .iter()
                    .map(|(tag, age)| Release {
                        tag: (*tag).into(),
                        published: NOW - age,
                    })
                    .collect(),
            })
        }
    }

    fn stand(tags: Vec<(&'static str, i64)>, version: &str, min_age: &str) -> Standing {
        let remote = Remote(tags);
        let mut cache = forge::Cache::in_memory();
        let mut ctx = Ctx {
            git: &remote,
            forge: &remote,
            cache: &mut cache,
            now: NOW,
            out: &mut std::io::sink(),
            err: &mut std::io::sink(),
        };
        let entry = Dependency {
            git: "someone/jaison".into(),
            version: Some(version.into()),
            ..Dependency::default()
        };
        let resolver = Resolver {
            min_release_age: Some(min_age.into()),
            release_age_jitter: Some("0".into()),
            ..Resolver::default()
        };

        standing(&mut ctx, Path::new("/p"), "jaison", &entry, &resolver).unwrap()
    }

    #[test]
    fn the_pin_is_its_tag_and_the_newest_accepted_release_is_newer() {
        let found = stand(
            vec![
                ("v1.2.0", 90 * DAY),
                ("v1.3.0", 30 * DAY),
                ("v1.4.0", 20 * DAY),
            ],
            "^1.2",
            "2w",
        );

        assert_eq!(found.pinned.as_deref(), Some("v1.2.0"));
        assert_eq!(found.newer.as_deref(), Some("v1.4.0"));
        assert_eq!(found.waiting, None);
    }

    #[test]
    fn a_release_inside_the_window_waits_and_the_one_before_it_is_offered() {
        let found = stand(
            vec![
                ("v1.2.0", 90 * DAY),
                ("v1.3.0", 30 * DAY),
                ("v1.5.0", 3 * DAY),
            ],
            "^1.2",
            "2w",
        );

        assert_eq!(found.newer.as_deref(), Some("v1.3.0"));
        assert_eq!(found.waiting, Some(("v1.5.0".to_string(), 11 * DAY)));
    }

    #[test]
    fn nothing_newer_in_range_is_nothing() {
        let found = stand(
            vec![("v1.2.0", 90 * DAY), ("v2.0.0", 90 * DAY)],
            "^1.2",
            "2w",
        );

        assert_eq!((found.newer, found.waiting), (None, None));
    }

    #[test]
    fn a_release_only_in_the_window_is_waiting_with_nothing_available() {
        let found = stand(vec![("v1.2.0", 90 * DAY), ("v1.3.0", DAY)], "^1.2", "2w");

        assert_eq!(found.newer, None);
        assert_eq!(found.waiting, Some(("v1.3.0".to_string(), 13 * DAY)));
    }
}
