//! Local bare git repositories and a project to add them to, so `jacket dep` runs against real
//! git without touching the network.
#![allow(dead_code)]
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::process::Command;
use test_support::TempDir;

pub mod forge;

pub fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(dir)
        .args(["-c", "commit.gpgsign=false", "-c", "tag.gpgsign=false"])
        .args(args)
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@example.com")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@example.com")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .output()
        .expect("git runs");
    assert!(
        output.status.success(),
        "git {args:?} in {}: {}",
        dir.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

/// A repository with a bare copy to depend on. Commits go to the working copy and are pushed.
pub struct Origin {
    pub bare: PathBuf,
    pub work: PathBuf,
}

impl Origin {
    pub fn new(root: &Path, name: &str, files: &[(&str, &str)]) -> Self {
        let work = root.join("work").join(name);
        let bare = root.join("origins").join(format!("{name}.git"));
        std::fs::create_dir_all(&work).unwrap();
        std::fs::create_dir_all(bare.parent().unwrap()).unwrap();
        git(&work, &["init", "-q", "-b", "main"]);
        let origin = Self {
            bare,
            work,
        };
        origin.write(files);
        origin.commit("first");
        git(
            &origin.work,
            &["clone", "-q", "--bare", ".", origin.bare.to_str().unwrap()],
        );
        git(
            &origin.work,
            &["remote", "add", "origin", origin.bare.to_str().unwrap()],
        );
        git(&origin.work, &["fetch", "-q", "origin"]);
        git(&origin.work, &["branch", "-q", "-u", "origin/main"]);
        origin
    }

    pub fn write(&self, files: &[(&str, &str)]) {
        for (path, text) in files {
            let full = self.work.join(path);
            std::fs::create_dir_all(full.parent().unwrap()).unwrap();
            std::fs::write(full, text).unwrap();
        }
    }

    /// Commits everything and pushes; returns the commit.
    pub fn commit(&self, message: &str) -> String {
        git(&self.work, &["add", "-A"]);
        git(
            &self.work,
            &["commit", "-q", "--allow-empty", "-m", message],
        );
        if git(&self.work, &["remote"]).contains("origin") {
            git(&self.work, &["push", "-q", "origin", "HEAD:main"]);
        }
        git(&self.work, &["rev-parse", "HEAD"])
    }

    /// Adds `files`, commits, and tags the commit.
    pub fn release(&self, tag: &str, files: &[(&str, &str)]) -> String {
        self.write(files);
        let commit = self.commit(tag);
        self.tag(tag);
        commit
    }

    pub fn tag(&self, tag: &str) {
        git(&self.work, &["tag", "-f", tag]);
        git(
            &self.work,
            &["push", "-q", "-f", "origin", &format!("refs/tags/{tag}")],
        );
    }

    pub fn url(&self) -> String {
        self.bare.to_string_lossy().into_owned()
    }
}

/// A git repository with a `jacket.toml`, to run `jacket dep` in.
pub struct Project {
    pub root: TempDir,
    pub dir: PathBuf,
    /// `(url, local path)`: git is told to fetch `url` from the path.
    rewrites: RefCell<Vec<(String, String)>>,
}

pub struct Run {
    pub code: i32,
    pub out: String,
    pub err: String,
}

impl Project {
    pub fn new(toml: &str) -> Self {
        let root = TempDir::new("dep");
        let dir = root.join("project");
        std::fs::create_dir_all(&dir).unwrap();
        git(&dir, &["init", "-q", "-b", "main"]);
        std::fs::write(dir.join("jacket.toml"), toml).unwrap();
        git(&dir, &["add", "-A"]);
        git(&dir, &["commit", "-q", "-m", "init"]);
        Self {
            root,
            dir,
            rewrites: RefCell::default(),
        }
    }

    pub fn jacket(&self, args: &[&str]) -> Run {
        self.jacket_env(args, &[])
    }

    /// Makes git fetch `url` (as `https://github.com/someone/lib`) from a local repository.
    pub fn rewrite(&self, url: &str, local: &str) {
        self.rewrites.borrow_mut().push((url.into(), local.into()));
    }

    pub fn jacket_env(&self, args: &[&str], env: &[(&str, &str)]) -> Run {
        let rewrites = self.rewrites.borrow();
        let mut all: Vec<(String, String)> =
            vec![("GIT_CONFIG_COUNT".into(), (1 + rewrites.len()).to_string())];
        for (i, (url, local)) in rewrites.iter().enumerate() {
            all.push((
                format!("GIT_CONFIG_KEY_{}", i + 1),
                format!("url.{local}.insteadOf"),
            ));
            all.push((format!("GIT_CONFIG_VALUE_{}", i + 1), url.clone()));
        }
        let mut pairs: Vec<(&str, &str)> =
            all.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        pairs.extend_from_slice(env);
        run_in(&self.dir, &self.root, args, &pairs)
    }

    pub fn toml(&self) -> String {
        std::fs::read_to_string(self.dir.join("jacket.toml")).unwrap()
    }

    pub fn commit_count(&self) -> usize {
        git(&self.dir, &["rev-list", "--count", "HEAD"])
            .parse()
            .unwrap()
    }
}

pub fn run_in(dir: &Path, root: &Path, args: &[&str], env: &[(&str, &str)]) -> Run {
    let mut command = Command::new(env!("CARGO_BIN_EXE_jacket"));
    command
        .current_dir(dir)
        .args(args)
        .env("HOME", root.join("home"))
        .env("XDG_CACHE_HOME", root.join("cache"))
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@example.com")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@example.com")
        // Local bare repositories stand in for remotes, including nested ones.
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "protocol.file.allow")
        .env("GIT_CONFIG_VALUE_0", "always");
    for (k, v) in env {
        command.env(k, v);
    }
    let output = command.output().expect("jacket runs");
    Run {
        code: output.status.code().unwrap_or(-1),
        out: String::from_utf8_lossy(&output.stdout).into_owned(),
        err: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}
