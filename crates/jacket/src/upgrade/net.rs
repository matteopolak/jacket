//! The two things `upgrade` needs from outside: the GitHub releases API plus the files a
//! release holds, and a way to unpack an archive. Both are behind [`Source`] so tests run
//! against a directory instead of the network.
//!
//! The real source starts `curl` and `tar`, as `install.sh` does: both ship with macOS, nearly
//! every Linux and Windows 10 and later (`curl.exe`, and a `tar.exe` that also reads zip), and
//! `curl` already honours `HTTPS_PROXY`, `NO_PROXY` and the system certificate store, so no
//! HTTP or TLS stack is linked into `jacket`.
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

pub const REPO: &str = "matteopolak/jacket";

pub trait Source {
    /// The releases API's JSON for the latest release, or for `tag`.
    fn release_json(&self, tag: Option<&str>) -> Result<String, String>;

    /// Saves the file at `url` as `dest`.
    fn download(&self, url: &str, dest: &Path) -> Result<(), String>;

    /// Unpacks `archive` into the existing folder `dest`.
    fn extract(&self, archive: &Path, dest: &Path) -> Result<(), String>;
}

/// GitHub, through `curl` and `tar`.
pub struct Github {
    /// `GITHUB_TOKEN`, sent to the API only, to lift its anonymous rate limit.
    token: Option<String>,
}

impl Github {
    pub fn from_env() -> Github {
        let token = jacket_config::env::GITHUB_TOKEN
            .get()
            .map(|t| t.trim().to_owned())
            .filter(|t| !t.is_empty());
        Github {
            token,
        }
    }

    fn curl(&self, args: &[&str], url: &str, with_token: bool) -> Result<Vec<u8>, String> {
        if !url.starts_with("https://") {
            return Err(format!("refusing to fetch {url}: only https URLs are used"));
        }
        let token = self.token.as_deref().filter(|_| with_token);
        if token.is_some_and(|t| t.contains(['"', '\\', '\n', '\r'])) {
            return Err("GITHUB_TOKEN contains characters a token never has".into());
        }
        let mut command = Command::new("curl");
        command
            .args([
                "--proto",
                "=https",
                "--tlsv1.2",
                "--fail",
                "--silent",
                "--show-error",
            ])
            .args(["--location", "--retry", "3"])
            .args([
                "--user-agent",
                concat!("jacket-upgrade/", env!("CARGO_PKG_VERSION")),
            ])
            .args(args);
        if token.is_some() {
            // Read from stdin, so the token is never on a command line other users can list.
            command.args(["--config", "-"]).stdin(Stdio::piped());
        } else {
            command.stdin(Stdio::null());
        }
        let mut child = command
            .arg(url)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                format!(
                    "could not run `curl`: {e}\nhelp: `jacket upgrade` needs curl and tar on PATH"
                )
            })?;
        if let (Some(token), Some(mut stdin)) = (token, child.stdin.take()) {
            let _ = writeln!(stdin, "header = \"Authorization: Bearer {token}\"");
        }
        let output = child
            .wait_with_output()
            .map_err(|e| format!("curl failed: {e}"))?;
        if output.status.success() {
            return Ok(output.stdout);
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        let mut message = format!("could not fetch {url}: {}", stderr.trim());
        if stderr.contains("403") || stderr.contains("429") {
            message
                .push_str("\nhelp: GitHub's rate limit may be reached; set GITHUB_TOKEN and retry");
        }
        Err(message)
    }
}

impl Source for Github {
    fn release_json(&self, tag: Option<&str>) -> Result<String, String> {
        let url = match tag {
            Some(tag) => format!("https://api.github.com/repos/{REPO}/releases/tags/{tag}"),
            None => format!("https://api.github.com/repos/{REPO}/releases/latest"),
        };
        let body = self.curl(
            &["--header", "Accept: application/vnd.github+json"],
            &url,
            true,
        )?;
        String::from_utf8(body).map_err(|_| "GitHub's answer was not text".to_owned())
    }

    fn download(&self, url: &str, dest: &Path) -> Result<(), String> {
        let dest_arg = dest.to_string_lossy();
        self.curl(&["--output", &dest_arg], url, false).map(|_| ())
    }

    fn extract(&self, archive: &Path, dest: &Path) -> Result<(), String> {
        // Windows: System32's tar.exe is bsdtar and reads zip; a `tar` earlier on PATH (Git's)
        // may not.
        let program = if cfg!(windows) {
            let root = jacket_config::env::SYSTEM_ROOT
                .os()
                .unwrap_or_else(|| "C:\\Windows".into());
            Path::new(&root).join("System32").join("tar.exe")
        } else {
            "tar".into()
        };
        let output = Command::new(&program)
            .arg("-xf")
            .arg(archive)
            .arg("-C")
            .arg(dest)
            .output()
            .map_err(|e| {
                format!(
                    "could not run `tar`: {e}\nhelp: `jacket upgrade` needs curl and tar on PATH"
                )
            })?;
        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "could not unpack {}: {}",
                archive.display(),
                String::from_utf8_lossy(&output.stderr).trim()
            ))
        }
    }
}
