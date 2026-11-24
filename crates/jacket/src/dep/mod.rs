//! `jacket dep`: git dependencies pinned in `jacket.toml`. See `docs/tools/dependencies.md`.
//!
//! The submodule's commit (its gitlink) is the lock. Every command takes its git access through
//! [`git::Git`], so unit tests use a fake and never start a process.
mod commands;
pub mod git;
pub mod manifest;
pub mod spec;

use clap::Subcommand;
use jacket_config::{Config, Disk, FILE_NAME};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Subcommand, Debug)]
pub enum Action {
    /// Add a git dependency as a submodule and an entry in `[dependencies]`
    Add {
        /// `user/repo` (GitHub), `host/path` or a URL, with an optional `@tag` or `@rev`
        spec: String,
        /// The module's name and folder under `modules/` (default: the repository's name)
        #[arg(long)]
        name: Option<String>,
        /// The folder of the repository holding `module.jai`
        #[arg(long, value_name = "PATH")]
        module: Option<String>,
    },
    /// Remove a dependency: its submodule, folder and entry
    Remove { name: String },
    /// Show each dependency's pin and whether it is checked out
    List,
    /// Move dependencies to what their entries now allow
    Update {
        /// One dependency (default: all)
        name: Option<String>,
        /// Pin this tag
        #[arg(long, value_name = "TAG")]
        to: Option<String>,
    },
    /// Check out every pinned commit, as after a fresh clone
    Sync,
    /// Verify `jacket.toml` and the gitlinks agree and no tag has moved (for CI)
    Check {
        /// Skip the comparison with the remotes' tags
        #[arg(long)]
        offline: bool,
    },
    /// Adopt the submodules already under the module folders
    Init {
        /// Write a new `[resolver] seed` even if there is one
        #[arg(long)]
        reseed: bool,
    },
}

/// A failure, as `error:` with optional `note:` lines.
#[derive(Debug, PartialEq, Eq)]
pub struct DepError {
    pub message: String,
    pub notes: Vec<String>,
}

impl DepError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            notes: Vec::new(),
        }
    }

    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }
}

impl From<String> for DepError {
    fn from(message: String) -> Self {
        Self::new(message)
    }
}

pub type Result<T> = std::result::Result<T, DepError>;

/// The project a command runs in: the nearest `jacket.toml`, as text and as settings.
pub struct Project {
    pub dir: PathBuf,
    pub file: PathBuf,
    pub text: String,
    pub config: Config,
}

impl Project {
    pub fn open(cwd: &Path) -> Result<Self> {
        let Some(location) = jacket_config::locate(&Disk, cwd) else {
            return Err(
                DepError::new(format!("no {FILE_NAME} at or above {}", cwd.display()))
                    .note("run `jacket init` to create a project"),
            );
        };
        let file = location.file();
        let text = std::fs::read_to_string(&file)
            .map_err(|e| DepError::new(format!("could not read {}: {e}", file.display())))?;
        let (config, _) = jacket_config::parse_noting(&text)
            .map_err(|e| DepError::new(format!("{}: {e}", file.display())))?;
        Ok(Self {
            dir: location.dir,
            file,
            text,
            config,
        })
    }

    pub fn save(&mut self, text: String) -> Result<()> {
        std::fs::write(&self.file, &text)
            .map_err(|e| DepError::new(format!("could not write {}: {e}", self.file.display())))?;
        self.config = jacket_config::parse_noting(&text).map_err(DepError::new)?.0;
        self.text = text;
        Ok(())
    }
}

/// What a command reads and writes besides the project.
pub struct Ctx<'a> {
    pub git: &'a dyn git::Git,
    pub out: &'a mut dyn Write,
    pub err: &'a mut dyn Write,
}

pub fn run(action: Action, cwd: &Path) -> ExitCode {
    let (stdout, stderr) = (&mut std::io::stdout(), &mut std::io::stderr());
    let mut ctx = Ctx {
        git: &git::RealGit,
        out: stdout,
        err: stderr,
    };
    match commands::dispatch(action, cwd, &mut ctx) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(error) => {
            let _ = writeln!(ctx.err, "error: {}", error.message);
            for note in &error.notes {
                let mut lines = note.lines();
                let _ = writeln!(ctx.err, "note: {}", lines.next().unwrap_or(""));
                for rest in lines {
                    let _ = writeln!(ctx.err, "      {rest}");
                }
            }
            ExitCode::from(2)
        }
    }
}

/// A build with a dependency whose submodule is not checked out cannot find its module: says
/// which, and how to fix it. Nothing is fetched here.
pub fn require_checked_out(config: &Config, dir: &Path) -> std::result::Result<(), String> {
    let missing: Vec<String> = config
        .dependencies
        .iter()
        .map(|(name, entry)| commands::path_of(name, entry))
        .filter(|path| !dir.join(path).join(".git").exists())
        .collect();
    if missing.is_empty() {
        return Ok(());
    }
    Err(format!(
        "{} not checked out\nhelp: run `jacket dep sync`",
        missing.join(", ")
    ))
}
