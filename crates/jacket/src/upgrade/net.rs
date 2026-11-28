//! The two things `upgrade` needs from outside: the GitHub releases API plus the files a
//! release holds, and a way to unpack an archive. Both are behind [`Source`] so tests run
//! against a directory instead of the network.
//!
//! The real source starts `curl` and `tar` through [`crate::net`], as `install.sh` does.
use std::path::Path;

use crate::net::Curl;

pub(super) const REPO: &str = "matteopolak/jacket";

pub(crate) trait Source {
    /// The releases API's JSON for the latest release, or for `tag`.
    fn release_json(&self, tag: Option<&str>) -> Result<String, String>;

    /// Saves the file at `url` as `dest`.
    fn download(&self, url: &str, dest: &Path) -> Result<(), String>;

    /// Unpacks `archive` into the existing folder `dest`.
    fn extract(&self, archive: &Path, dest: &Path) -> Result<(), String>;
}

/// `curl`, and `tar` unless only checking (Windows uses its own `tar.exe`), before any download
/// or prompt: an upgrade that cannot unpack must not start.
pub(super) fn require_tools(check_only: bool) -> Result<(), String> {
    use jaic_toolchain::package::Need;
    crate::prerequisite::require(&Need::Curl, "curl", "jacket upgrade")?;

    if !check_only && !cfg!(windows) {
        crate::prerequisite::require(&Need::Tar, "tar", "jacket upgrade")?;
    }

    Ok(())
}

/// GitHub, through `curl` and `tar`.
pub(super) struct Github {
    /// `Bearer` and `GITHUB_TOKEN`, sent to the API only, to lift its anonymous rate limit.
    token: Option<String>,
}

impl Github {
    pub(super) fn from_env() -> Github {
        let token = jacket_config::env::GITHUB_TOKEN
            .get()
            .map(|t| t.trim().to_owned())
            .filter(|t| !t.is_empty())
            .map(|t| format!("Bearer {t}"));

        Github {
            token,
        }
    }

    /// `url` through `curl`, retried; the token goes only to the API.
    fn curl<'a>(&'a self, url: &'a str, with_token: bool) -> Curl<'a> {
        let curl = Curl::get(url, "jacket upgrade").retry(3);

        match self.token.as_deref().filter(|_| with_token) {
            Some(token) => curl.secret_header("Authorization", token),
            None => curl,
        }
    }
}

/// The error of a failed request, with the way past GitHub's rate limit when it was that.
fn rate_limited(message: String) -> String {
    if message.contains("403") || message.contains("429") {
        format!("{message}\nhelp: GitHub's rate limit may be reached; set GITHUB_TOKEN and retry")
    } else {
        message
    }
}

impl Source for Github {
    fn release_json(&self, tag: Option<&str>) -> Result<String, String> {
        let url = match tag {
            Some(tag) => format!("https://api.github.com/repos/{REPO}/releases/tags/{tag}"),
            None => format!("https://api.github.com/repos/{REPO}/releases/latest"),
        };
        let body = self
            .curl(&url, true)
            .header("Accept: application/vnd.github+json")
            .run()
            .map_err(rate_limited)?;

        String::from_utf8(body).map_err(|_| "GitHub's answer was not text".to_owned())
    }

    fn download(&self, url: &str, dest: &Path) -> Result<(), String> {
        self.curl(url, false)
            .output(dest)
            .run()
            .map(|_| ())
            .map_err(rate_limited)
    }

    fn extract(&self, archive: &Path, dest: &Path) -> Result<(), String> {
        crate::net::untar(archive, dest, "jacket upgrade")
    }
}
