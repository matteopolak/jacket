//! Git access for `jacket dep`. Every command takes it through the [`Git`] trait, so unit tests
//! never start a process or touch the network. [`RealGit`] runs the `git` on `PATH` through
//! [`crate::git::Repo`], so SSH keys and credential helpers work as they do for any submodule.
use std::collections::BTreeMap;
use std::path::Path;

use super::DepError;
use super::spec::{RepoUrl, Source};
use super::types::{CommitId, RefName, Revision, SubPath, Target};
use crate::git::{
    self, Failure, Reach, Refs, Removal, Repo, cause, config_entries, staged_entries,
};

/// A tag of a remote repository and the commit it points at (annotated tags peeled).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteTag {
    pub name: RefName,
    pub commit: CommitId,
}

/// A gitlink under the project folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Submodule {
    /// Relative to the project folder.
    pub path: SubPath,
    pub commit: CommitId,
    /// The `url` in `.gitmodules`, when it has one.
    pub url: Option<Source>,
}

type Result<T> = std::result::Result<T, DepError>;

pub trait Git {
    /// Stops with next steps when `git` is not installed or `dir` is not in a repository. Run
    /// before anything changes.
    fn preflight(&self, dir: &Path) -> Result<()>;

    /// Every tag of the remote at `url`.
    fn remote_tags(&self, url: &RepoUrl) -> Result<Vec<RemoteTag>>;

    /// The commit the remote's default branch points at.
    fn remote_head(&self, url: &RepoUrl) -> Result<CommitId>;

    /// The gitlinks at or under `dir`, with their `.gitmodules` URLs.
    fn submodules(&self, dir: &Path) -> Result<Vec<Submodule>>;

    /// Adds the submodule, checks out `commit` and stages the gitlink. A failure after the
    /// submodule exists undoes it, and says so when it cannot.
    fn add(&self, dir: &Path, url: &RepoUrl, path: &SubPath, commit: &Target) -> Result<()>;

    /// Moves an existing submodule to `commit`, fetching what is missing, and stages it.
    fn move_to(&self, dir: &Path, path: &SubPath, commit: &Target) -> Result<()>;

    /// Checks out the pinned commits of `paths`, initialising them. Their own submodules are
    /// left to [`Git::nested`].
    fn sync(&self, dir: &Path, paths: &[SubPath]) -> Result<()>;

    /// Checks out the submodules inside the submodule at `path`, except those at the paths in
    /// `skip` (relative to it), which stay empty.
    fn nested(&self, dir: &Path, path: &SubPath, skip: &[SubPath]) -> Result<()>;

    /// Deinits the submodule and removes its folder, gitlink, `.gitmodules` entry and clone.
    fn remove(&self, dir: &Path, path: &SubPath) -> Result<()>;

    /// The commit `rev` names inside the submodule, if the clone has it.
    fn local_commit(&self, dir: &Path, path: &SubPath, rev: &Revision) -> Result<Option<CommitId>>;

    /// The tags of the submodule's clone that point at `commit`.
    fn tags_at(&self, dir: &Path, path: &SubPath, commit: &CommitId) -> Result<Vec<RefName>>;

    /// Whether the submodule is checked out.
    fn initialised(&self, dir: &Path, path: &SubPath) -> Result<bool>;

    /// Brings the submodule's tags and objects up to date with its remote.
    fn fetch(&self, dir: &Path, path: &SubPath) -> Result<()>;

    /// Moves a submodule's folder, keeping the index and `.gitmodules` in step.
    fn rename(&self, dir: &Path, from: &SubPath, to: &SubPath) -> Result<()>;

    /// The submodule's checked-out commit.
    fn head(&self, dir: &Path, path: &SubPath) -> Result<CommitId>;

    /// Removes a tracked folder (a vendored copy) from the repository and the disk. Fails when
    /// it has uncommitted changes.
    fn remove_tree(&self, dir: &Path, path: &SubPath) -> Result<()>;
}

pub struct RealGit;

impl Git for RealGit {
    fn preflight(&self, dir: &Path) -> Result<()> {
        Repo::anywhere().version()?;

        match Repo::at(dir).top() {
            Ok(_) => Ok(()),
            Err(Failure::Failed {
                said,
            }) if said.contains("not a git repository") => Err(DepError::new(format!(
                "{} is not inside a git repository",
                dir.display()
            ))
            .note("run `git init` first; dependencies are git submodules")),
            Err(Failure::Failed {
                said,
            }) => {
                let lines = cause(&said);
                let mut error = DepError::new(format!(
                    "git cannot use the repository at {}: {}",
                    dir.display(),
                    lines.last().copied().unwrap_or("no output")
                ));

                for line in lines.iter().rev().skip(1).rev() {
                    error = error.note((*line).to_string());
                }

                Err(error)
            }
            Err(other) => Err(git::failed(&other, &["rev-parse"]).into()),
        }
    }

    fn remote_tags(&self, url: &RepoUrl) -> Result<Vec<RemoteTag>> {
        let text = Repo::anywhere().ls_remote(url, Refs::Tags)?;
        parse_ls_remote(&text).map_err(|e| DepError::new(format!("{url}: {e}")))
    }

    fn remote_head(&self, url: &RepoUrl) -> Result<CommitId> {
        let text = Repo::anywhere().ls_remote(url, Refs::Head)?;
        parse_head(&text).ok_or_else(|| DepError::new(format!("{url} has no default branch")))
    }

    fn submodules(&self, dir: &Path) -> Result<Vec<Submodule>> {
        let repo = Repo::at(dir);
        let staged = repo.staged()?;
        let top = repo.toplevel()?;
        let prefix = repo.prefix()?;
        let config = if top.join(".gitmodules").is_file() {
            Repo::at(&top).gitmodules()?
        } else {
            String::new()
        };

        Ok(parse_gitlinks(&staged, &config, &prefix))
    }

    fn add(&self, dir: &Path, url: &RepoUrl, path: &SubPath, commit: &Target) -> Result<()> {
        let repo = Repo::at(dir);
        let folder = path.under(dir);
        let clone = repo.git_path(&format!("modules/{path}"))?;
        let (had_folder, had_clone) = (folder.exists(), clone.exists());

        if let Err(error) = repo.submodule_add(url, path) {
            // A clone that stopped half way is not left to block the next try.
            let mut error = DepError::from(error);
            let registered = match self.submodules(dir) {
                Ok(known) => known.iter().any(|s| &s.path == path),
                Err(failed) => {
                    error = error.note(format!(
                        "could not look for what is left: {}",
                        failed.message
                    ));
                    false
                }
            };

            if registered {
                if let Err(failed) = self.remove(dir, path) {
                    error = error.note(format!("could not undo the add: {}", failed.message));
                }
            } else {
                for (had, leftover) in [(had_folder, &folder), (had_clone, &clone)] {
                    if !had
                        && let Err(e) = std::fs::remove_dir_all(leftover)
                        && e.kind() != std::io::ErrorKind::NotFound
                    {
                        error = error.note(format!("could not remove {}: {e}", leftover.display()));
                    }
                }
            }

            return Err(error);
        }

        let finish = || -> Result<()> {
            self.move_to(dir, path, commit)?;
            repo.submodule_update(std::slice::from_ref(path), Reach::Named)?;
            Ok(())
        };

        finish().map_err(|error| match self.remove(dir, path) {
            Ok(()) => error,
            Err(failed) => error
                .note(format!("could not undo the add: {}", failed.message))
                .note(format!(
                    "remove what is left with `git submodule deinit -f -- {path}` and `git rm -f -- {path}`"
                )),
        })
    }

    fn move_to(&self, dir: &Path, path: &SubPath, commit: &Target) -> Result<()> {
        let folder = path.under(dir);
        let sub = Repo::at(&folder);
        let rev = Revision::from(commit);

        if self.local_commit(dir, path, &rev)?.is_none() {
            self.fetch(dir, path)?;
        }

        if self.local_commit(dir, path, &rev)?.is_none() {
            sub.fetch_commit(commit)?;
        }

        sub.checkout_detached(commit)?;
        Ok(Repo::at(dir).stage(path)?)
    }

    fn sync(&self, dir: &Path, paths: &[SubPath]) -> Result<()> {
        Ok(Repo::at(dir).submodule_update(paths, Reach::Named)?)
    }

    fn nested(&self, dir: &Path, path: &SubPath, skip: &[SubPath]) -> Result<()> {
        if !self.initialised(dir, path)? {
            return Ok(());
        }

        let sub = path.under(dir);
        let repo = Repo::at(&sub);
        let all: Vec<SubPath> = self.submodules(&sub)?.into_iter().map(|s| s.path).collect();

        // A copy the project now shares is not kept.
        for inner in all.iter().filter(|p| skip.contains(p)) {
            if self.initialised(&sub, inner)? {
                repo.deinit(inner)?;
            }
        }

        let wanted: Vec<SubPath> = all.into_iter().filter(|p| !skip.contains(p)).collect();

        if wanted.is_empty() {
            return Ok(());
        }

        Ok(repo.submodule_update(&wanted, Reach::Nested)?)
    }

    fn remove(&self, dir: &Path, path: &SubPath) -> Result<()> {
        let repo = Repo::at(dir);
        let was_initialised = self.initialised(dir, path)?;

        repo.deinit(path)?;

        if let Err(error) = repo.rm(path, Removal::Submodule) {
            let mut error = DepError::from(error);

            // Leave the checkout as it was found.
            if was_initialised
                && let Err(failed) = repo.submodule_update(std::slice::from_ref(path), Reach::Named)
            {
                error = error.note(format!("could not restore {path}: {failed}"));
            }

            return Err(error);
        }

        // `git rm` leaves the clone in the superproject's `modules` folder.
        let top = repo.toplevel()?;
        // The last submodule leaves an empty `.gitmodules`; it goes too, as it came with the first.
        let file = top.join(".gitmodules");

        if std::fs::metadata(&file).is_ok_and(|m| m.len() == 0) {
            let name = SubPath::parse(".gitmodules").map_err(DepError::new)?;

            Repo::at(&top).rm(&name, Removal::File)?;
        }

        let modules = Repo::at(&top).git_path("modules")?;
        let clone = Repo::at(&top).git_path(&format!("modules/{path}"))?;

        if clone.starts_with(&modules) && clone != modules {
            match std::fs::remove_dir_all(&clone) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => {
                    return Err(DepError::new(format!(
                        "could not remove the clone at {}: {e}",
                        clone.display()
                    )));
                }
            }
        }

        Ok(())
    }

    fn local_commit(&self, dir: &Path, path: &SubPath, rev: &Revision) -> Result<Option<CommitId>> {
        if !self.initialised(dir, path)? {
            return Ok(None);
        }

        Ok(Repo::at(&path.under(dir)).commit_of(rev))
    }

    fn tags_at(&self, dir: &Path, path: &SubPath, commit: &CommitId) -> Result<Vec<RefName>> {
        let text = Repo::at(&path.under(dir)).tags_at(commit)?;

        text.lines()
            .map(|line| RefName::parse(line).map_err(DepError::new))
            .collect()
    }

    fn initialised(&self, dir: &Path, path: &SubPath) -> Result<bool> {
        path.under(dir)
            .join(".git")
            .try_exists()
            .map_err(|e| DepError::new(format!("cannot look at {path}: {}", jaic::io_reason(&e))))
    }

    fn fetch(&self, dir: &Path, path: &SubPath) -> Result<()> {
        Ok(Repo::at(&path.under(dir)).fetch_tags()?)
    }

    fn head(&self, dir: &Path, path: &SubPath) -> Result<CommitId> {
        Ok(Repo::at(&path.under(dir)).head()?)
    }

    fn rename(&self, dir: &Path, from: &SubPath, to: &SubPath) -> Result<()> {
        Ok(Repo::at(dir).mv(from, to)?)
    }

    fn remove_tree(&self, dir: &Path, path: &SubPath) -> Result<()> {
        let repo = Repo::at(dir);

        if repo.changed(path)? {
            return Err(DepError::new(format!(
                "{path} has uncommitted changes; commit or discard them first"
            )));
        }

        Ok(repo.rm(path, Removal::Tree)?)
    }
}

// ---- reading git's output ---------------------------------------------------------------------

/// The tags in `git ls-remote --tags` output: a peeled line (`^{}`) wins over the tag object's.
/// Lines that are not a tag are skipped; a tag whose name or commit git should not have written
/// is an error.
pub fn parse_ls_remote(text: &str) -> std::result::Result<Vec<RemoteTag>, String> {
    let mut tags: Vec<RemoteTag> = Vec::new();

    for line in text.lines() {
        let Some((commit, name)) = line.split_once(char::is_whitespace) else {
            continue;
        };
        let Some(name) = name.trim().strip_prefix("refs/tags/") else {
            continue;
        };
        let (name, peeled) = match name.strip_suffix("^{}") {
            Some(base) => (base, true),
            None => (name, false),
        };
        let name = RefName::parse(name)?;
        let commit = CommitId::parse(commit)?;

        match tags.iter_mut().find(|t| t.name == name) {
            Some(existing) if peeled => existing.commit = commit,
            Some(_) => {}
            None => tags.push(RemoteTag {
                name,
                commit,
            }),
        }
    }

    Ok(tags)
}

/// The commit of the first line of `git ls-remote <url> HEAD` output.
pub fn parse_head(text: &str) -> Option<CommitId> {
    CommitId::parse(text.split_whitespace().next()?).ok()
}

/// The gitlinks of `git ls-files -s -z` output under `prefix`, with the URLs of
/// `git config -z -f .gitmodules --list` output. Conflicted entries (stages 1 to 3) and paths
/// that cannot be a folder of the project are left out.
pub fn parse_gitlinks(staged: &str, config: &str, prefix: &str) -> Vec<Submodule> {
    let mut urls = BTreeMap::new();
    let mut paths = BTreeMap::new();

    for (key, value) in config_entries(config) {
        let Some(rest) = key.strip_prefix("submodule.") else {
            continue;
        };

        if let Some(name) = rest.strip_suffix(".path") {
            paths.insert(name.to_string(), value.to_string());
        } else if let Some(name) = rest.strip_suffix(".url") {
            urls.insert(name.to_string(), value.to_string());
        }
    }

    let by_path: BTreeMap<String, String> = paths
        .into_iter()
        .filter_map(|(name, path)| urls.get(&name).map(|url| (path, url.clone())))
        .collect();
    let mut out = Vec::new();

    for (mode, id, stage, path) in staged_entries(staged) {
        if mode != "160000" || stage != "0" {
            continue;
        }

        let (Ok(commit), Ok(sub)) = (CommitId::parse(id), SubPath::parse(path)) else {
            continue;
        };

        out.push(Submodule {
            url: by_path
                .get(&format!("{prefix}{path}"))
                .and_then(|url| Source::parse(url).ok()),
            path: sub,
            commit,
        });
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u64) -> CommitId {
        CommitId::numbered(n)
    }

    #[test]
    fn peeled_lines_replace_tag_objects() {
        let text = format!(
            "{}\trefs/tags/v1\n{}\trefs/tags/v1^{{}}\n{}\trefs/tags/v2\n{}\trefs/heads/main\n",
            id(1),
            id(2),
            id(3),
            id(4)
        );

        assert_eq!(
            parse_ls_remote(&text).unwrap(),
            [
                RemoteTag {
                    name: RefName::parse("v1").unwrap(),
                    commit: id(2)
                },
                RemoteTag {
                    name: RefName::parse("v2").unwrap(),
                    commit: id(3)
                },
            ]
        );
    }

    #[test]
    fn a_tag_git_would_not_write_is_an_error_and_other_refs_are_skipped() {
        let a = id(1);

        assert!(parse_ls_remote(&format!("{a}\trefs/tags/a..b\n")).is_err());
        assert!(parse_ls_remote("short\trefs/tags/v1\n").is_err());
        let tags = parse_ls_remote(&format!(
            "warning: noise\n{a}\trefs/heads/x\n{a}\trefs/tags/ok\n"
        ));

        assert_eq!(tags.unwrap().len(), 1);
    }

    #[test]
    fn an_empty_remote_has_no_tags_and_no_head() {
        assert_eq!(parse_ls_remote("").unwrap(), []);
        assert_eq!(parse_head(""), None);
        assert_eq!(parse_head("nothing here"), None);
    }

    #[test]
    fn the_head_is_the_first_id_in_either_hash_size() {
        let sha1 = id(7);

        assert_eq!(parse_head(&format!("{sha1}\tHEAD\n")), Some(sha1));
        let sha256 = CommitId::parse(&"ab".repeat(32)).unwrap();

        assert_eq!(
            parse_head(&format!("{sha256}\tHEAD\n")),
            Some(sha256.clone())
        );
        let text = format!("{sha256}\trefs/tags/v1\n");

        assert_eq!(parse_ls_remote(&text).unwrap()[0].commit, sha256);
    }

    #[test]
    fn gitlinks_carry_their_urls_and_skip_files() {
        let staged = format!(
            "100644 {} 0\tjacket.toml\0160000 {} 0\tmodules/jaison\0",
            id(1),
            id(2)
        );
        let config = "submodule.modules/jaison.path\nmodules/jaison\0submodule.modules/jaison.url\nhttps://github.com/x/jaison\0";
        let url = Source::parse("https://github.com/x/jaison").unwrap();

        assert_eq!(
            parse_gitlinks(&staged, config, ""),
            [Submodule {
                path: SubPath::parse("modules/jaison").unwrap(),
                commit: id(2),
                url: Some(url.clone()),
            }]
        );
        let nested = parse_gitlinks(&staged, &config.replace("modules/", "app/modules/"), "app/");

        assert_eq!(nested[0].url.as_ref(), Some(&url));
    }

    #[test]
    fn paths_with_non_ascii_characters_find_their_urls() {
        let staged = format!("160000 {} 0\tmodules/\u{e9}t\u{e9}\0", id(1));
        let config = "submodule.modules/\u{e9}t\u{e9}.path\nmodules/\u{e9}t\u{e9}\0submodule.modules/\u{e9}t\u{e9}.url\nhttps://h/x\0";
        let found = parse_gitlinks(&staged, config, "");

        assert_eq!(found[0].path.as_str(), "modules/\u{e9}t\u{e9}");
        assert!(found[0].url.is_some());
    }

    #[test]
    fn conflicted_and_unusable_entries_are_skipped() {
        let staged = format!(
            "160000 {a} 1\tmodules/a\0160000 {a} 2\tmodules/a\0160000 {a} 3\tmodules/a\0160000 {a} 0\t../outside\0160000 {a} 0\tmodules/b\0",
            a = id(1)
        );
        let found = parse_gitlinks(&staged, "", "");

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].path.as_str(), "modules/b");
    }

    #[test]
    fn sha256_gitlinks_are_read() {
        let long = "cd".repeat(32);
        let staged = format!("160000 {long} 0\tmodules/a\0");

        assert_eq!(parse_gitlinks(&staged, "", "")[0].commit.as_str(), long);
    }
}
