//! The only code in jacket that starts `git`. A [`Repo`] is git bound to a folder (or to none,
//! for commands that ask a remote), with one method per operation taking the types of
//! [`crate::dep::types`]; no caller builds an argument list. It also owns the environment git
//! runs in and turns a failure into a [`GitError`]: git's own words, or the install note when
//! git is missing.
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::dep::spec::RepoUrl;
use crate::dep::types::{CommitId, Revision, SubPath, Target};

/// A failure to show as `error:` with `note:` lines.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitError {
    pub message: String,
    pub notes: Vec<String>,
}

impl GitError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            notes: Vec::new(),
        }
    }

    #[must_use]
    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }
}

impl std::fmt::Display for GitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

type Result<T> = std::result::Result<T, GitError>;

/// Why a git command gave no answer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Failure {
    /// `git` is not on `PATH`.
    Missing,
    /// It could not be started or printed something unreadable.
    Unusable(String),
    /// It ran and exited with an error; `said` is its stderr.
    Failed { said: String },
}

/// Which refs of a remote to list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refs {
    Tags,
    Head,
}

/// What `rm` takes away.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Removal {
    /// A gitlink and its folder.
    Submodule,
    /// A tracked folder and everything in it.
    Tree,
    /// A tracked file.
    File,
}

/// How far `submodule update` goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reach {
    /// The named submodules.
    Named,
    /// Their own submodules too.
    Nested,
}

/// The variables that point git at a repository other than the folder it runs in. A git hook
/// sets them for its own repository.
const LOCATING: [&str; 9] = [
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_COMMON_DIR",
    "GIT_NAMESPACE",
    "GIT_PREFIX",
    "GIT_QUARANTINE_PATH",
];

/// Git, bound to a folder.
pub struct Repo<'a> {
    dir: Option<&'a Path>,
}

impl<'a> Repo<'a> {
    pub fn at(dir: &'a Path) -> Self {
        Self {
            dir: Some(dir),
        }
    }

    pub fn anywhere() -> Self {
        Self {
            dir: None,
        }
    }

    /// Runs `git args` and returns its stdout. Git never prompts (a missing credential is a
    /// failure to report, not a hang), never uses a repository the environment names, and
    /// speaks English so its messages can be told apart.
    fn run(&self, args: &[&str]) -> std::result::Result<String, Failure> {
        self.run_with(&[], args)
    }

    /// As [`Repo::run`], with `-c key=value` settings for this command only.
    fn run_with(&self, config: &[&str], args: &[&str]) -> std::result::Result<String, Failure> {
        self.spawn(config, args, None)
    }

    /// As [`Repo::run_with`], for a command that talks to a remote: when the user named no `ssh`
    /// program, SSH runs in batch mode, so a missing key fails instead of asking.
    fn run_net(&self, config: &[&str], args: &[&str]) -> std::result::Result<String, Failure> {
        let ssh = self.ssh_command();
        self.spawn(config, args, ssh.as_deref())
    }

    /// The `ssh` command for a remote: none when `GIT_SSH_COMMAND` or `GIT_SSH` is set (git
    /// reads those itself), else this repository's `core.sshCommand` (or `ssh`) in batch mode.
    fn ssh_command(&self) -> Option<String> {
        if jacket_config::env::GIT_SSH_COMMAND.get().is_some()
            || jacket_config::env::GIT_SSH.get().is_some()
        {
            return None;
        }

        let own = self
            .spawn(&[], &["config", "--get", "core.sshCommand"], None)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        Some(format!(
            "{} -o BatchMode=yes",
            own.as_deref().unwrap_or("ssh")
        ))
    }

    /// The one place a `git` process starts.
    fn spawn(
        &self,
        config: &[&str],
        args: &[&str],
        ssh: Option<&str>,
    ) -> std::result::Result<String, Failure> {
        if let Some(dir) = self.dir
            && !dir.is_dir()
        {
            return Err(Failure::Failed {
                said: format!("{} is not a folder", dir.display()),
            });
        }

        let mut command = Command::new("git");

        if let Some(dir) = self.dir {
            command.current_dir(dir);
        }

        for setting in config {
            command.args(["-c", setting]);
        }

        for var in LOCATING {
            command.env_remove(var);
        }

        if let Some(ssh) = ssh {
            command.env("GIT_SSH_COMMAND", ssh);
        }

        let output = command
            .args(args)
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("LC_ALL", "C")
            .stdin(Stdio::null())
            .output()
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => Failure::Missing,
                _ => Failure::Unusable(e.to_string()),
            })?;

        if !output.status.success() {
            return Err(Failure::Failed {
                said: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            });
        }

        String::from_utf8(output.stdout)
            .map_err(|_| Failure::Unusable("git printed text that is not UTF-8".into()))
    }

    fn ask(&self, args: &[&str]) -> Result<String> {
        self.run(args).map_err(|f| failed(&f, args))
    }

    fn tell(&self, args: &[&str]) -> Result<()> {
        self.ask(args).map(drop)
    }

    /// Whether `git` runs at all.
    pub fn version(&self) -> Result<()> {
        self.tell(&["--version"])
    }

    /// The top of the work tree holding this folder; `Err` carries git's reason.
    pub fn top(&self) -> std::result::Result<PathBuf, Failure> {
        self.run(&["rev-parse", "--show-toplevel"])
            .map(|s| PathBuf::from(s.trim()))
    }

    pub fn toplevel(&self) -> Result<PathBuf> {
        self.top().map_err(|f| failed(&f, &["rev-parse"]))
    }

    /// The path of this folder inside its repository, with a trailing `/` (empty at the top).
    pub fn prefix(&self) -> Result<String> {
        self.ask(&["rev-parse", "--show-prefix"])
            .map(|s| s.trim().to_string())
    }

    /// A path inside the repository's git folder, made absolute.
    pub fn git_path(&self, name: &str) -> Result<PathBuf> {
        let path = self.ask(&["rev-parse", "--git-path", name])?;
        let path = PathBuf::from(path.trim());

        Ok(match self.dir {
            Some(dir) if path.is_relative() => dir.join(path),
            _ => path,
        })
    }

    pub fn head(&self) -> Result<CommitId> {
        let text = self.ask(&["rev-parse", "HEAD"])?;
        CommitId::parse(&text).map_err(GitError::new)
    }

    /// The commit `rev` names in this clone, when it has it.
    pub fn commit_of(&self, rev: &Revision) -> Option<CommitId> {
        let spelled = format!("{}^{{commit}}", rev.as_str());
        let text = self
            .run(&["rev-parse", "--verify", "--quiet", &spelled])
            .ok()?;

        CommitId::parse(&text).ok()
    }

    /// The tags of this clone that point at `commit`, one per line.
    pub fn tags_at(&self, commit: &CommitId) -> Result<String> {
        self.ask(&["tag", "--points-at", commit.as_str()])
    }

    /// Whether `path` has uncommitted changes.
    pub fn changed(&self, path: &SubPath) -> Result<bool> {
        let text = self.ask(&["status", "--porcelain", "--", path.as_str()])?;
        Ok(!text.trim().is_empty())
    }

    /// The files under `targets` with staged, unstaged or untracked changes (`status -z`).
    pub fn dirty(&self, targets: &[&Path]) -> Result<Vec<String>> {
        let mut args = vec![
            "status",
            "--porcelain=v1",
            "-z",
            "--untracked-files=all",
            "--",
        ];
        let targets: Vec<String> = targets.iter().map(|t| t.display().to_string()).collect();

        args.extend(targets.iter().map(String::as_str));
        let text = self.ask(&args)?;
        // Each entry is `XY path`, NUL-terminated; a rename adds the old path as a second field.
        let mut dirty = Vec::new();
        let mut fields = text.split('\0').filter(|f| !f.is_empty());

        while let Some(entry) = fields.next() {
            let Some(path) = entry.get(3..) else {
                continue;
            };

            if entry.starts_with(['R', 'C']) {
                fields.next();
            }

            dirty.push(path.to_string());
        }

        Ok(dirty)
    }

    /// The index, NUL-separated (`ls-files -s -z`).
    pub fn staged(&self) -> Result<String> {
        self.ask(&["ls-files", "-s", "-z"])
    }

    /// The settings of `.gitmodules`, NUL-separated (`config -z`).
    pub fn gitmodules(&self) -> Result<String> {
        self.ask(&["config", "-z", "-f", ".gitmodules", "--list"])
    }

    pub fn stage(&self, path: &SubPath) -> Result<()> {
        self.tell(&["add", "--", path.as_str()])
    }

    pub fn mv(&self, from: &SubPath, to: &SubPath) -> Result<()> {
        self.tell(&["mv", "--", from.as_str(), to.as_str()])
    }

    pub fn rm(&self, path: &SubPath, what: Removal) -> Result<()> {
        match what {
            Removal::Tree => self.tell(&["rm", "-r", "-q", "--", path.as_str()]),
            Removal::Submodule | Removal::File => self.tell(&["rm", "-f", "--", path.as_str()]),
        }
    }

    pub fn deinit(&self, path: &SubPath) -> Result<()> {
        self.tell(&["submodule", "deinit", "-f", "--", path.as_str()])
    }

    /// Clones `url` into `path` as a submodule. A local path is the user's own choice, so git
    /// may clone it; nested submodules keep git's default.
    pub fn submodule_add(&self, url: &RepoUrl, path: &SubPath) -> Result<()> {
        let args = ["submodule", "add", "--", url.as_str(), path.as_str()];
        let config: &[&str] = if url.is_local() {
            &["protocol.file.allow=always"]
        } else {
            &[]
        };

        self.run_net(config, &args)
            .map(drop)
            .map_err(|f| reach(&f, url))
    }

    /// Checks out the recorded commits of `paths`, initialising them.
    pub fn submodule_update(&self, paths: &[SubPath], reach_of: Reach) -> Result<()> {
        let mut args = vec!["submodule", "update", "--init"];

        if reach_of == Reach::Nested {
            args.push("--recursive");
        }

        args.push("--");
        args.extend(paths.iter().map(SubPath::as_str));
        self.run_net(&[], &args).map(drop).map_err(|f| {
            let url = cloned_from(&f).unwrap_or_else(|| "a dependency's repository".into());
            reach(&f, &url)
        })
    }

    /// Brings tags and objects up to date with `origin`.
    pub fn fetch_tags(&self) -> Result<()> {
        let args = ["fetch", "--tags", "--force", "origin"];

        self.run_net(&[], &args)
            .map(drop)
            .map_err(|f| reach(&f, &self.origin()))
    }

    /// Fetches one commit by name from `origin`.
    pub fn fetch_commit(&self, commit: &Target) -> Result<()> {
        let args = ["fetch", "origin", commit.as_str()];

        self.run_net(&[], &args)
            .map(drop)
            .map_err(|f| reach(&f, &self.origin()))
    }

    pub fn checkout_detached(&self, commit: &Target) -> Result<()> {
        self.tell(&["checkout", "--detach", commit.as_str()])
    }

    /// The refs of a remote (`ls-remote`).
    pub fn ls_remote(&self, url: &RepoUrl, refs: Refs) -> Result<String> {
        let mut args = vec!["ls-remote"];

        if refs == Refs::Tags {
            args.push("--tags");
        }

        args.extend(["--", url.as_str()]);

        if refs == Refs::Head {
            args.push("HEAD");
        }

        self.run_net(&[], &args).map_err(|f| reach(&f, url))
    }

    /// What `origin` is called, for messages.
    fn origin(&self) -> String {
        self.run(&["config", "--get", "remote.origin.url"])
            .map(|s| s.trim().to_string())
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "its remote".into())
    }
}

/// What git said, without the `hint:` lines that follow the cause.
pub fn cause(said: &str) -> Vec<&str> {
    said.lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with("hint:"))
        .collect()
}

/// A failed command as an error: git's own words, or the install note when git is missing.
pub fn failed(failure: &Failure, args: &[&str]) -> GitError {
    let command = args.first().copied().unwrap_or("");

    match failure {
        Failure::Missing => missing(),
        Failure::Unusable(why) => GitError::new(format!("could not run `git {command}`: {why}")),
        Failure::Failed {
            said,
        } => {
            let lines = cause(said);
            let mut error = GitError::new(format!(
                "`git {command}` failed: {}",
                lines.last().copied().unwrap_or("no output")
            ));

            if lines.len() > 1 {
                error = error.note(format!("git said:\n{}", lines.join("\n")));
            }

            error
        }
    }
}

/// A failed command that talks to `url`: says which URL and what to do.
pub fn reach(failure: &Failure, url: &dyn std::fmt::Display) -> GitError {
    let said = match failure {
        Failure::Missing => return missing(),
        Failure::Unusable(why) => return GitError::new(format!("could not read {url}: {why}")),
        Failure::Failed {
            said,
        } => said,
    };
    let lines = cause(said);
    let last = lines.last().copied().unwrap_or("no output");
    let mut error = if is_denied(said) {
        GitError::new(format!("git was not allowed to read {url}: {last}")).note(
            "check the address, then that git can sign in to that host: an SSH key loaded in your agent, or a credential helper",
        )
    } else {
        GitError::new(format!("could not reach {url}: {last}"))
            .note("check the address and your network connection, then try again")
    };

    if lines.len() > 1 {
        error = error.note(format!("git said:\n{}", lines.join("\n")));
    }

    error
}

/// The address in git's `clone of '<url>' into submodule path` message.
pub fn cloned_from(failure: &Failure) -> Option<String> {
    let Failure::Failed {
        said,
    } = failure
    else {
        return None;
    };
    let after = said.split("clone of '").nth(1)?;

    Some(after.split('\'').next()?.to_string())
}

/// Whether git's message is a refusal rather than an unreachable host.
pub fn is_denied(said: &str) -> bool {
    let said = said.to_ascii_lowercase();

    [
        "authentication failed",
        "could not read username",
        "could not read password",
        "permission denied (",
        "terminal prompts disabled",
        "repository not found",
        "requested url returned error: 401",
        "requested url returned error: 403",
        "host key verification failed",
    ]
    .iter()
    .any(|m| said.contains(m))
}

/// The error for a machine with no `git`, with the command that installs it.
pub fn missing() -> GitError {
    let text =
        crate::prerequisite::missing(&jaic_toolchain::package::Need::Git, "git", "jacket dep");

    match text.split_once("\nnote: ") {
        Some((message, note)) => GitError::new(message).note(note),
        None => GitError::new(text),
    }
}

/// The entries of `git config -z` output: `key`, a newline and the value, NUL-terminated.
pub fn config_entries(text: &str) -> Vec<(&str, &str)> {
    text.split('\0')
        .filter(|e| !e.is_empty())
        .map(|e| e.split_once('\n').unwrap_or((e, "")))
        .collect()
}

/// The entries of `git ls-files -s -z` output as (mode, id, stage, path), NUL-terminated.
pub fn staged_entries(text: &str) -> Vec<(&str, &str, &str, &str)> {
    text.split('\0')
        .filter(|e| !e.is_empty())
        .filter_map(|entry| {
            let (meta, path) = entry.split_once('\t')?;
            let mut fields = meta.split_whitespace();

            Some((fields.next()?, fields.next()?, fields.next()?, path))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn said(text: &str) -> Failure {
        Failure::Failed {
            said: text.into(),
        }
    }

    #[test]
    fn refusals_are_told_from_unreachable_hosts() {
        assert!(is_denied("fatal: Authentication failed for 'https://x/y'"));
        assert!(is_denied("git@host: Permission denied (publickey)."));
        assert!(is_denied(
            "fatal: could not read Username for 'https://x': terminal prompts disabled"
        ));
        assert!(is_denied(
            "fatal: unable to access 'https://x/': The requested URL returned error: 403"
        ));
        assert!(!is_denied(
            "fatal: unable to access 'https://x/': Could not resolve host: x"
        ));
        // A file's permissions or a path named "access denied" are not a refused login.
        assert!(!is_denied("error: cannot open x: Permission denied"));
        assert!(!is_denied("fatal: could not open access denied/config"));
        // A number in a path or id is not an HTTP status.
        assert!(!is_denied(
            "fatal: repository 'https://x/a401b/c.git' is gone"
        ));
        assert!(!is_denied(
            "fatal: unable to access 'https://x:4030/': timed out"
        ));
    }

    #[test]
    fn a_failed_remote_command_names_the_url_and_a_next_step() {
        let url = "https://example.invalid/a/b";
        let unreachable = reach(
            &said("fatal: unable to access: Could not resolve host"),
            &url,
        );

        assert!(unreachable.message.contains(url));
        assert!(unreachable.notes[0].contains("network"));
        let denied = reach(&said("git@host: Permission denied (publickey)."), &url);

        assert!(denied.message.starts_with("git was not allowed to read"));
        assert!(denied.notes[0].contains("SSH key"));
    }

    #[test]
    fn git_s_hints_are_not_its_cause() {
        let text = "fatal: the thing is wrong\nhint: try this\nhint: or that";

        assert_eq!(cause(text), ["fatal: the thing is wrong"]);
        let error = failed(&said(text), &["fetch"]);

        assert_eq!(
            error.message,
            "`git fetch` failed: fatal: the thing is wrong"
        );
        assert!(error.notes.is_empty());
        let long = failed(&said("error: first\nfatal: last"), &["fetch"]);

        assert!(long.message.ends_with("fatal: last"));
        assert!(long.notes[0].contains("error: first"));
    }

    #[test]
    fn a_failed_submodule_clone_names_its_address() {
        let failure = said("fatal: clone of 'https://h/x/y' into submodule path '/p/m' failed");

        assert_eq!(cloned_from(&failure).as_deref(), Some("https://h/x/y"));
        assert_eq!(cloned_from(&Failure::Missing), None);
    }

    #[test]
    fn config_and_index_output_is_read_by_nul() {
        let config = "submodule.a b.path\nmodules/é\0submodule.a b.url\nhttps://h/x\0";

        assert_eq!(
            config_entries(config),
            [
                ("submodule.a b.path", "modules/é"),
                ("submodule.a b.url", "https://h/x")
            ]
        );
        let staged = "160000 abc 0\tmodules/\u{e9}\u{3b1}\x00100644 def 2\ta\tb\0";

        assert_eq!(
            staged_entries(staged),
            [
                ("160000", "abc", "0", "modules/\u{e9}\u{3b1}"),
                ("100644", "def", "2", "a\tb")
            ]
        );
    }
}
