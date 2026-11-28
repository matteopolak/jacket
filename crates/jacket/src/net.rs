//! The network and archives, for `jacket upgrade`, `jacket toolchain add` and `jacket dep`: one
//! way to start `curl` ([`Curl`]) and one to start `tar` ([`untar`]).
//!
//! Both ship with macOS, nearly every Linux and Windows 10 and later (`curl.exe`, and a `tar.exe`
//! that also reads zip and xz), and `curl` honours `HTTPS_PROXY`, `NO_PROXY` and the system
//! certificate store, so no HTTP or TLS stack is linked into `jacket`. The commands keep their
//! own traits over these (`upgrade::net::Source`, `dep::forge::Forge`) so tests run offline.
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use jaic_toolchain::package::Need;

use crate::prerequisite::spawn_error;

/// The `User-Agent` of every request.
const USER_AGENT: &str = concat!("jacket/", env!("CARGO_PKG_VERSION"));

/// One request run by `curl`. HTTPS only unless [`Curl::allow_http`]; an HTTP error status is a
/// failure unless [`Curl::include`] asks for the status and headers instead.
///
/// ```ignore
/// let body = Curl::get("https://api.github.com/repos/o/r", "jacket upgrade")
///     .header("Accept: application/vnd.github+json")
///     .secret_header("Authorization", &format!("Bearer {token}"))
///     .retry(3)
///     .run()?;
/// ```
pub struct Curl<'a> {
    url: &'a str,
    /// The command that needs it, for the error when `curl` is missing.
    command: &'a str,
    https_only: bool,
    include: bool,
    follow: bool,
    retries: u32,
    max_time: Option<u32>,
    headers: Vec<String>,
    /// A header name and value, given through standard input so no other user can list it.
    secret: Option<(&'a str, &'a str)>,
    /// A JSON body, which makes it a POST.
    json: Option<&'a str>,
    output: Option<&'a Path>,
}

impl<'a> Curl<'a> {
    /// A GET of `url`, following redirects, for `command`.
    pub fn get(url: &'a str, command: &'a str) -> Self {
        Self {
            url,
            command,
            https_only: true,
            include: false,
            follow: true,
            retries: 0,
            max_time: None,
            headers: Vec::new(),
            secret: None,
            json: None,
            output: None,
        }
    }

    /// Allows `http://` (a local test server named through the environment).
    #[must_use]
    pub fn allow_http(mut self) -> Self {
        self.https_only = false;
        self
    }

    /// Returns the status line and headers before the body, whatever the status, and does not
    /// follow redirects: the caller reads the answer itself ([`parse_response`]).
    #[must_use]
    pub fn include(mut self) -> Self {
        self.include = true;
        self.follow = false;
        self
    }

    #[must_use]
    pub fn retry(mut self, times: u32) -> Self {
        self.retries = times;
        self
    }

    #[must_use]
    pub fn max_time(mut self, seconds: u32) -> Self {
        self.max_time = Some(seconds);
        self
    }

    #[must_use]
    pub fn header(mut self, header: impl Into<String>) -> Self {
        self.headers.push(header.into());
        self
    }

    /// A header with a secret value (a token).
    #[must_use]
    pub fn secret_header(mut self, name: &'a str, value: &'a str) -> Self {
        self.secret = Some((name, value));
        self
    }

    /// POSTs `body` as JSON.
    #[must_use]
    pub fn post_json(mut self, body: &'a str) -> Self {
        self.json = Some(body);
        self
    }

    /// Saves the body as `path` instead of returning it.
    #[must_use]
    pub fn output(mut self, path: &'a Path) -> Self {
        self.output = Some(path);
        self
    }

    /// The arguments before the URL, and what goes to `curl`'s standard input.
    fn arguments(&self) -> Result<(Vec<String>, Option<String>), String> {
        let url = self.url;

        if self.https_only && !url.starts_with("https://") {
            return Err(format!("refusing to fetch {url}: only https URLs are used"));
        }

        let mut args: Vec<String> = ["--silent", "--show-error"].map(String::from).to_vec();

        if self.https_only {
            args.extend(["--proto", "=https", "--tlsv1.2"].map(String::from));
        }

        if self.include {
            args.push("--include".into());
        } else {
            args.push("--fail".into());
        }

        if self.follow {
            args.push("--location".into());
        }

        if self.retries > 0 {
            args.extend(["--retry".into(), self.retries.to_string()]);
        }

        if let Some(seconds) = self.max_time {
            args.extend(["--max-time".into(), seconds.to_string()]);
        }

        args.extend(["--user-agent".into(), USER_AGENT.into()]);

        for header in &self.headers {
            args.extend(["--header".into(), header.clone()]);
        }

        if let Some(body) = self.json {
            args.extend([
                "--header".into(),
                "Content-Type: application/json".into(),
                "--data-binary".into(),
                body.into(),
            ]);
        }

        if let Some(path) = self.output {
            args.extend(["--output".into(), path.to_string_lossy().into_owned()]);
        }

        let config = match self.secret {
            Some((name, value)) => {
                if value.contains(['"', '\\', '\n', '\r']) {
                    return Err(format!(
                        "the {name} header has characters a token never has"
                    ));
                }

                args.extend(["--config".into(), "-".into()]);
                Some(format!("header = \"{name}: {value}\"\n"))
            }
            None => None,
        };
        Ok((args, config))
    }

    /// Runs the request; the body (or with [`Curl::include`] the whole answer), empty when
    /// saved to a file. A failure says what `curl` printed.
    pub fn run(&self) -> Result<Vec<u8>, String> {
        let (args, config) = self.arguments()?;
        let mut child = Command::new("curl")
            .args(&args)
            .arg(self.url)
            .stdin(if config.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| spawn_error(&e, &Need::Curl, "curl", self.command))?;

        if let (Some(config), Some(mut stdin)) = (config, child.stdin.take()) {
            stdin
                .write_all(config.as_bytes())
                .map_err(|e| format!("could not give curl its settings: {e}"))?;
        }

        let output = child
            .wait_with_output()
            .map_err(|e| format!("curl failed: {e}"))?;

        if output.status.success() {
            Ok(output.stdout)
        } else {
            Err(format!(
                "could not fetch {}: {}",
                self.url,
                String::from_utf8_lossy(&output.stderr).trim()
            ))
        }
    }
}

/// An HTTP answer: the status, the headers (names in lower case) and the body.
pub type Response = (u16, Vec<(String, String)>, String);

/// The status, headers and body of a [`Curl::include`] answer, after any interim `1xx` heads.
pub fn parse_response(text: &str) -> Option<Response> {
    let mut rest = text;

    loop {
        let (head, body) = rest
            .split_once("\r\n\r\n")
            .or_else(|| rest.split_once("\n\n"))?;
        let mut lines = head.lines();
        let status: u16 = lines.next()?.split_whitespace().nth(1)?.parse().ok()?;

        if (100..200).contains(&status) {
            rest = body;
            continue;
        }

        let headers = lines
            .filter_map(|l| l.split_once(':'))
            .map(|(k, v)| (k.trim().to_ascii_lowercase(), v.trim().to_string()))
            .collect();

        return Some((status, headers, body.to_string()));
    }
}

/// The `tar` to run. On Windows it is System32's `tar.exe`, which is bsdtar and reads xz and zip;
/// a `tar` earlier on PATH (Git's) may not. Its folder comes from `SystemRoot`, which Windows
/// always sets: without it this is an error, not a guessed `C:\Windows`.
fn tar_program() -> Result<PathBuf, String> {
    if !cfg!(windows) {
        return Ok("tar".into());
    }

    system_tar(
        jacket_config::env::SYSTEM_ROOT
            .os()
            .filter(|root| !root.is_empty()),
    )
}

fn system_tar(root: Option<std::ffi::OsString>) -> Result<PathBuf, String> {
    let root = root.ok_or_else(|| {
        format!(
            "{} is not set, so jacket cannot find Windows' own tar.exe\nnote: set it to the Windows folder, as Windows does for every program",
            jacket_config::env::SYSTEM_ROOT.name
        )
    })?;
    Ok(Path::new(&root).join("System32").join("tar.exe"))
}

/// Unpacks `archive` (`.tar`, `.tar.gz`, `.tar.xz`, and `.zip` on Windows) into the existing
/// folder `dest`, for `command`.
pub fn untar(archive: &Path, dest: &Path, command: &str) -> Result<(), String> {
    let output = Command::new(tar_program()?)
        .arg("-xf")
        .arg(archive)
        .arg("-C")
        .arg(dest)
        .output()
        .map_err(|e| spawn_error(&e, &Need::Tar, "tar", command))?;

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

#[cfg(test)]
mod tests {
    use super::*;

    fn args(curl: &Curl) -> Vec<String> {
        curl.arguments().unwrap().0
    }

    #[test]
    fn a_request_is_https_only_unless_allowed() {
        let error = Curl::get("http://example.com/a", "jacket test")
            .run()
            .unwrap_err();

        assert_eq!(
            error,
            "refusing to fetch http://example.com/a: only https URLs are used"
        );
        let plain = args(&Curl::get("http://127.0.0.1:1/a", "jacket test").allow_http());

        assert!(!plain.contains(&"--proto".to_string()), "{plain:?}");
        let secure = args(&Curl::get("https://example.com/a", "jacket test"));

        assert!(
            secure.windows(2).any(|w| w == ["--proto", "=https"]),
            "{secure:?}"
        );
        assert!(secure.contains(&"--fail".to_string()));
        assert!(secure.contains(&"--location".to_string()));
    }

    #[test]
    fn a_secret_goes_through_standard_input_never_the_command_line() {
        let curl = Curl::get("https://example.com/a", "jacket test")
            .secret_header("Authorization", "Bearer abc");
        let (args, config) = curl.arguments().unwrap();

        assert!(args.iter().all(|arg| !arg.contains("abc")), "{args:?}");
        assert!(args.windows(2).any(|w| w == ["--config", "-"]));
        assert_eq!(
            config.as_deref(),
            Some("header = \"Authorization: Bearer abc\"\n")
        );
        let bad = Curl::get("https://example.com/a", "jacket test")
            .secret_header("Authorization", "a\"b");

        assert!(bad.arguments().is_err());
    }

    #[test]
    fn the_options_become_curl_flags() {
        let path = Path::new("/tmp/out");
        let curl = Curl::get("https://example.com/a", "jacket test")
            .include()
            .retry(3)
            .max_time(30)
            .header("Accept: application/json")
            .post_json("{}")
            .output(path);
        let args = args(&curl);

        for pair in [
            ["--retry", "3"],
            ["--max-time", "30"],
            ["--header", "Accept: application/json"],
            ["--data-binary", "{}"],
            ["--output", "/tmp/out"],
            ["--user-agent", USER_AGENT],
        ] {
            assert!(args.windows(2).any(|w| w == pair), "{pair:?} in {args:?}");
        }

        assert!(args.contains(&"--include".to_string()));
        assert!(!args.contains(&"--fail".to_string()));
        assert!(!args.contains(&"--location".to_string()));
    }

    #[test]
    fn curl_output_loses_its_interim_headers() {
        let text = "HTTP/1.1 100 Continue\r\n\r\nHTTP/2 304\r\nETag: \"x\"\r\n\r\n";
        let (status, headers, body) = parse_response(text).unwrap();

        assert_eq!(status, 304);
        assert_eq!(headers, [("etag".to_string(), "\"x\"".to_string())]);
        assert!(body.is_empty());
    }

    #[test]
    fn the_windows_tar_comes_from_system_root_never_a_guess() {
        let error = system_tar(None).unwrap_err();

        assert!(error.starts_with("SystemRoot is not set"), "{error}");
        assert_eq!(
            system_tar(Some("/win".into())).unwrap(),
            Path::new("/win").join("System32").join("tar.exe")
        );
    }
}
