//! Git access for `jacket dep`, behind a trait so unit tests never start a process or touch the
//! network. [`RealGit`] runs the `git` on `PATH`, so SSH keys and credential helpers work as
//! they do for any submodule.
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// A tag of a remote repository and the commit it points at (annotated tags peeled).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteTag {
    pub name: String,
    pub commit: String,
}

/// A gitlink under the project folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Submodule {
    /// Relative to the project folder.
    pub path: String,
    pub commit: String,
    /// The `url` in `.gitmodules`, when it has one.
    pub url: Option<String>,
}

pub trait Git {
    /// The folder of the repository holding `dir`.
    fn toplevel(&self, dir: &Path) -> Result<PathBuf, String>;

    /// Every tag of the remote at `url` (`git ls-remote --tags`).
    fn remote_tags(&self, url: &str) -> Result<Vec<RemoteTag>, String>;

    /// The commit the remote's default branch points at.
    fn remote_head(&self, url: &str) -> Result<String, String>;

    /// The gitlinks at or under `dir`, with their `.gitmodules` URLs.
    fn submodules(&self, dir: &Path) -> Result<Vec<Submodule>, String>;

    /// Adds the submodule, checks out `commit` and stages the gitlink.
    fn add(&self, dir: &Path, url: &str, path: &str, commit: &str) -> Result<(), String>;

    /// Moves an existing submodule to `commit`, fetching what is missing, and stages it.
    fn move_to(&self, dir: &Path, path: &str, commit: &str) -> Result<(), String>;

    /// Checks out the pinned commits of `paths` (and their own submodules), initialising them.
    fn sync(&self, dir: &Path, paths: &[String]) -> Result<(), String>;

    /// Deinits the submodule and removes its folder, gitlink and `.gitmodules` entry.
    fn remove(&self, dir: &Path, path: &str) -> Result<(), String>;

    /// The commit `rev` names inside the submodule, if the clone has it.
    fn local_commit(&self, dir: &Path, path: &str, rev: &str) -> Result<Option<String>, String>;

    /// The tags of the submodule's clone that point at `commit`.
    fn tags_at(&self, dir: &Path, path: &str, commit: &str) -> Result<Vec<String>, String>;

    /// Whether the submodule is checked out.
    fn initialised(&self, dir: &Path, path: &str) -> bool;

    /// Brings the submodule's tags and objects up to date with its remote.
    fn fetch(&self, dir: &Path, path: &str) -> Result<(), String>;

    /// Moves a submodule's folder (`git mv`), keeping `.gitmodules` in step.
    fn rename(&self, dir: &Path, from: &str, to: &str) -> Result<(), String>;

    /// The submodule's checked-out commit.
    fn head(&self, dir: &Path, path: &str) -> Result<String, String>;

    /// Removes a tracked folder (a vendored copy) from the repository and the disk. Fails when
    /// it has uncommitted changes.
    fn remove_tree(&self, dir: &Path, path: &str) -> Result<(), String>;
}

pub struct RealGit;

fn run(dir: Option<&Path>, args: &[&str]) -> Result<String, String> {
    let mut command = Command::new("git");
    if let Some(dir) = dir {
        command.current_dir(dir);
    }
    let output = command
        .args(args)
        .stdin(Stdio::inherit())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("could not run `git`: {e}"))?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }
    let said = String::from_utf8_lossy(&output.stderr);
    Err(format!(
        "`git {}` failed: {}",
        args.join(" "),
        said.trim().lines().last().unwrap_or("no output")
    ))
}

impl Git for RealGit {
    fn toplevel(&self, dir: &Path) -> Result<PathBuf, String> {
        run(Some(dir), &["rev-parse", "--show-toplevel"]).map(|s| PathBuf::from(s.trim()))
    }

    fn remote_tags(&self, url: &str) -> Result<Vec<RemoteTag>, String> {
        let text = run(None, &["ls-remote", "--tags", "--", url])?;
        Ok(parse_ls_remote(&text))
    }

    fn remote_head(&self, url: &str) -> Result<String, String> {
        let text = run(None, &["ls-remote", "--", url, "HEAD"])?;
        text.split_whitespace()
            .next()
            .map(str::to_string)
            .ok_or_else(|| format!("{url} has no default branch"))
    }

    fn submodules(&self, dir: &Path) -> Result<Vec<Submodule>, String> {
        let staged = run(Some(dir), &["ls-files", "-s"])?;
        let top = self.toplevel(dir)?;
        let prefix = run(Some(dir), &["rev-parse", "--show-prefix"])?;
        let prefix = prefix.trim();
        let urls = std::fs::metadata(top.join(".gitmodules"))
            .ok()
            .map(|_| run(Some(&top), &["config", "-f", ".gitmodules", "--list"]))
            .transpose()?
            .unwrap_or_default();
        Ok(parse_gitlinks(&staged, &urls, prefix))
    }

    fn add(&self, dir: &Path, url: &str, path: &str, commit: &str) -> Result<(), String> {
        // A local path is the user's own choice, so git may clone it as a submodule; nested
        // submodules keep git's default.
        let mut add = vec!["submodule", "add", "--", url, path];
        if is_local(url) {
            add.splice(0..0, ["-c", "protocol.file.allow=always"]);
        }
        run(Some(dir), &add)?;
        self.move_to(dir, path, commit)?;
        run(
            Some(dir),
            &["submodule", "update", "--init", "--recursive", "--", path],
        )
        .map(drop)
    }

    fn move_to(&self, dir: &Path, path: &str, commit: &str) -> Result<(), String> {
        let sub = dir.join(path);
        if self.local_commit(dir, path, commit)?.is_none() {
            self.fetch(dir, path)?;
        }
        if self.local_commit(dir, path, commit)?.is_none() {
            run(Some(&sub), &["fetch", "origin", commit])?;
        }
        run(Some(&sub), &["checkout", "--detach", commit])?;
        run(Some(dir), &["add", "--", path]).map(drop)
    }

    fn sync(&self, dir: &Path, paths: &[String]) -> Result<(), String> {
        let mut args = vec!["submodule", "update", "--init", "--recursive", "--"];
        args.extend(paths.iter().map(String::as_str));
        run(Some(dir), &args).map(drop)
    }

    fn remove(&self, dir: &Path, path: &str) -> Result<(), String> {
        run(Some(dir), &["submodule", "deinit", "-f", "--", path])?;
        run(Some(dir), &["rm", "-f", "--", path])?;
        let top = self.toplevel(dir)?;
        let name = run(
            Some(&top),
            &["rev-parse", "--git-path", &format!("modules/{path}")],
        )
        .ok()
        .map(|p| top.join(p.trim()));
        if let Some(name) = name {
            let _ = std::fs::remove_dir_all(name);
        }
        Ok(())
    }

    fn local_commit(&self, dir: &Path, path: &str, rev: &str) -> Result<Option<String>, String> {
        let sub = dir.join(path);
        if !self.initialised(dir, path) {
            return Ok(None);
        }
        let spelled = format!("{rev}^{{commit}}");
        Ok(
            run(Some(&sub), &["rev-parse", "--verify", "--quiet", &spelled])
                .ok()
                .map(|s| s.trim().to_string()),
        )
    }

    fn tags_at(&self, dir: &Path, path: &str, commit: &str) -> Result<Vec<String>, String> {
        let text = run(Some(&dir.join(path)), &["tag", "--points-at", commit])?;
        Ok(text.lines().map(str::to_string).collect())
    }

    fn initialised(&self, dir: &Path, path: &str) -> bool {
        dir.join(path).join(".git").exists()
    }

    fn fetch(&self, dir: &Path, path: &str) -> Result<(), String> {
        run(
            Some(&dir.join(path)),
            &["fetch", "--tags", "--force", "origin"],
        )
        .map(drop)
    }

    fn head(&self, dir: &Path, path: &str) -> Result<String, String> {
        run(Some(&dir.join(path)), &["rev-parse", "HEAD"]).map(|s| s.trim().to_string())
    }

    fn rename(&self, dir: &Path, from: &str, to: &str) -> Result<(), String> {
        run(Some(dir), &["mv", "--", from, to]).map(drop)
    }

    fn remove_tree(&self, dir: &Path, path: &str) -> Result<(), String> {
        let changes = run(Some(dir), &["status", "--porcelain", "--", path])?;
        if !changes.trim().is_empty() {
            return Err(format!(
                "{path} has uncommitted changes; commit or discard them first"
            ));
        }
        run(Some(dir), &["rm", "-r", "-q", "--", path]).map(drop)
    }
}

/// Whether `url` names a folder on this machine.
pub fn is_local(url: &str) -> bool {
    url.starts_with('/')
        || url.starts_with("./")
        || url.starts_with("../")
        || url.starts_with("file://")
        || (url.len() > 2 && url.as_bytes()[1] == b':' && url.as_bytes()[0].is_ascii_alphabetic())
}

/// The tags in `git ls-remote --tags` output: a peeled line (`^{}`) wins over the tag object's.
pub fn parse_ls_remote(text: &str) -> Vec<RemoteTag> {
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
        match tags.iter_mut().find(|t| t.name == name) {
            Some(existing) if peeled => existing.commit = commit.to_string(),
            Some(_) => {}
            None => tags.push(RemoteTag {
                name: name.to_string(),
                commit: commit.to_string(),
            }),
        }
    }
    tags
}

/// The gitlinks of `git ls-files -s` output under `prefix`, with the URLs of
/// `git config -f .gitmodules --list` output.
pub fn parse_gitlinks(staged: &str, config: &str, prefix: &str) -> Vec<Submodule> {
    let mut urls = std::collections::BTreeMap::new();
    let mut paths = std::collections::BTreeMap::new();
    for line in config.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if let Some(name) = key
            .strip_prefix("submodule.")
            .and_then(|k| k.strip_suffix(".path"))
        {
            paths.insert(name.to_string(), value.to_string());
        } else if let Some(name) = key
            .strip_prefix("submodule.")
            .and_then(|k| k.strip_suffix(".url"))
        {
            urls.insert(name.to_string(), value.to_string());
        }
    }
    let by_path: std::collections::BTreeMap<String, String> = paths
        .into_iter()
        .filter_map(|(name, path)| urls.get(&name).map(|url| (path, url.clone())))
        .collect();
    let mut out = Vec::new();
    for line in staged.lines() {
        let Some((meta, path)) = line.split_once('\t') else {
            continue;
        };
        let mut fields = meta.split_whitespace();
        if fields.next() != Some("160000") {
            continue;
        }
        let commit = fields.next().unwrap_or_default().to_string();
        out.push(Submodule {
            url: by_path.get(&format!("{prefix}{path}")).cloned(),
            path: path.to_string(),
            commit,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peeled_lines_replace_tag_objects() {
        let text =
            "aaa\trefs/tags/v1\nbbb\trefs/tags/v1^{}\nccc\trefs/tags/v2\nddd\trefs/heads/main\n";
        assert_eq!(
            parse_ls_remote(text),
            [
                RemoteTag {
                    name: "v1".into(),
                    commit: "bbb".into()
                },
                RemoteTag {
                    name: "v2".into(),
                    commit: "ccc".into()
                },
            ]
        );
    }

    #[test]
    fn gitlinks_carry_their_urls_and_skip_files() {
        let staged = "100644 aaa 0\tjacket.toml\n160000 bbb 0\tmodules/jaison\n";
        let config = "submodule.modules/jaison.path=modules/jaison\nsubmodule.modules/jaison.url=https://github.com/x/jaison\n";
        assert_eq!(
            parse_gitlinks(staged, config, ""),
            [Submodule {
                path: "modules/jaison".into(),
                commit: "bbb".into(),
                url: Some("https://github.com/x/jaison".into()),
            }]
        );
        let nested = parse_gitlinks(staged, &config.replace("modules/", "app/modules/"), "app/");
        assert_eq!(
            nested[0].url.as_deref(),
            Some("https://github.com/x/jaison")
        );
    }
}
