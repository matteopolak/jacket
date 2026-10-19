//! `jacket upgrade`: install the latest release over the current installation. See
//! `docs/tools/jacket.md`.
//!
//! The steps, each of which can fail without touching the install: find the release, pick this
//! platform's archive, download it and `SHA256SUMS`, check the hash, unpack beside the install,
//! check the unpacked tree, then swap ([`swap`]).
mod checksum;
mod managed;
mod net;
mod swap;
mod version;

use net::Source;
use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use version::Version;

/// The flags of `jacket upgrade`.
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// A specific release (`0.22.0` or `v0.22.0`) instead of the latest.
    pub version: Option<String>,
    /// Report whether an update exists; install nothing.
    pub check: bool,
    /// Do not ask for confirmation.
    pub yes: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    UpToDate,
    Available { to: String },
    Declined,
    Upgraded { to: String },
}

/// What a run works on, so tests can point it at a temporary folder and a fake release.
pub struct Target<'a> {
    /// The running `jacket`, canonical: its folder is the install.
    pub exe: &'a Path,
    /// The running version.
    pub current: &'a str,
    /// This platform's archive, as the release names it.
    pub asset: &'a str,
    pub source: &'a dyn Source,
    pub mode: swap::Mode,
    /// Asked before installing; the argument says what will happen.
    pub confirm: &'a dyn Fn(&str) -> bool,
}

pub fn run(options: &Options) -> ExitCode {
    let exe = match std::env::current_exe().and_then(|exe| exe.canonicalize()) {
        Ok(exe) => exe,
        Err(error) => return fail(format!("cannot find the running jacket: {error}")),
    };
    let Some(asset) = asset_name(std::env::consts::OS, std::env::consts::ARCH) else {
        return fail(format!(
            "there is no prebuilt toolchain for {} on {}\nhelp: build from source instead: https://github.com/{}#install",
            std::env::consts::OS,
            std::env::consts::ARCH,
            net::REPO
        ));
    };
    let source = net::Github::from_env();
    let interactive = std::io::stdin().is_terminal();
    let confirm = |question: &str| options.yes || !interactive || ask(question);
    let target = Target {
        exe: &exe,
        current: env!("CARGO_PKG_VERSION"),
        asset: &asset,
        source: &source,
        mode: swap::Mode::native(),
        confirm: &confirm,
    };
    match upgrade(&target, options) {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => fail(error),
    }
}

fn fail(message: impl std::fmt::Display) -> ExitCode {
    eprintln!("jacket: error: {message}");
    ExitCode::from(2)
}

fn ask(question: &str) -> bool {
    eprint!("{question} [y/N] ");
    let _ = std::io::stderr().flush();
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).is_ok()
        && matches!(line.trim().to_ascii_lowercase().as_str(), "y" | "yes")
}

/// The archive name for an OS and architecture as `std::env::consts` spells them; `None` when
/// releases have no build for it. Mirrors `tests/installers/vectors.json`.
pub fn asset_name(os: &str, arch: &str) -> Option<String> {
    let arch = match arch {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        _ => return None,
    };
    match os {
        "macos" => Some(format!("jacket-macos-{arch}.tar.gz")),
        "linux" => Some(format!("jacket-linux-{arch}.tar.gz")),
        "windows" => Some(format!("jacket-windows-{arch}.zip")),
        _ => None,
    }
}

/// The folder an archive unpacks to: the asset name without its extension.
fn archive_root(asset: &str) -> &str {
    asset
        .strip_suffix(".tar.gz")
        .or_else(|| asset.strip_suffix(".zip"))
        .unwrap_or(asset)
}

struct Release {
    tag: String,
    assets: Vec<(String, String)>,
}

impl Release {
    fn parse(json: &str) -> Result<Release, String> {
        let value: serde_json::Value = serde_json::from_str(json)
            .map_err(|e| format!("GitHub's answer was not a release: {e}"))?;
        let tag = value
            .get("tag_name")
            .and_then(|t| t.as_str())
            .ok_or("GitHub's answer has no tag_name; is there a release yet?")?
            .to_owned();
        let assets = value
            .get("assets")
            .and_then(|a| a.as_array())
            .map(|assets| {
                assets
                    .iter()
                    .filter_map(|a| {
                        let name = a.get("name")?.as_str()?;
                        let url = a.get("browser_download_url")?.as_str()?;
                        Some((name.to_owned(), url.to_owned()))
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(Release {
            tag,
            assets,
        })
    }

    fn asset(&self, name: &str) -> Option<&str> {
        self.assets
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, url)| url.as_str())
    }
}

/// An install is a folder with the compiler and the standard library beside `jacket`.
fn install_dir(exe: &Path) -> Result<&Path, String> {
    let dir = exe.parent().ok_or("the running jacket has no folder")?;
    let jaic = format!("jaic{}", std::env::consts::EXE_SUFFIX);
    if dir.join(jaic).is_file() && dir.join("stdlib").is_dir() {
        Ok(dir)
    } else {
        Err(format!(
            "{} is not an installed toolchain (no jaic and stdlib beside jacket), so there is nothing to replace\nhelp: install a release with install.sh or install.ps1 (docs/tools/releases.md), then run `jacket upgrade`",
            dir.display()
        ))
    }
}

pub fn upgrade(target: &Target, options: &Options) -> Result<Outcome, String> {
    let install = install_dir(target.exe)?;
    let managed = managed::managed_by(target.exe);
    if let Some(managed) = managed
        && !options.check
    {
        return Err(managed.advice().to_owned());
    }
    swap::clean_stale(install);

    let wanted = options.version.as_deref().map(Version::tag).transpose()?;
    let release = Release::parse(&target.source.release_json(wanted.as_deref())?)?;
    let latest = Version::parse(&release.tag)?;
    let current = Version::parse(target.current)?;
    let version = release.tag.trim_start_matches('v').to_owned();

    let up_to_date = latest == current || (wanted.is_none() && latest < current);
    if up_to_date {
        println!("jacket {} is already up to date", target.current);
        return Ok(Outcome::UpToDate);
    }
    if options.check {
        println!("update available: jacket {} -> {version}", target.current);
        match managed {
            Some(managed) => println!("{}", managed.advice()),
            None => println!("run `jacket upgrade` to install it"),
        }
        return Ok(Outcome::Available {
            to: version,
        });
    }

    let archive_url = release.asset(target.asset).ok_or_else(|| {
        format!(
            "release {} has no {}, so it has no build for this platform\nhelp: build from source instead: https://github.com/{}#install",
            release.tag,
            target.asset,
            net::REPO
        )
    })?;
    let sums_url = release.asset("SHA256SUMS").ok_or_else(|| {
        format!(
            "release {} has no SHA256SUMS, so the download cannot be verified",
            release.tag
        )
    })?;

    let question = format!(
        "Replace jacket {} in {} with {version}?",
        target.current,
        install.display()
    );
    if !(target.confirm)(&question) {
        println!("not upgrading");
        return Ok(Outcome::Declined);
    }

    let staging = swap::sibling(install, swap::NEW_SUFFIX);
    let result = install_release(
        target,
        install,
        &staging,
        &release.tag,
        (archive_url, sums_url),
    );
    let _ = std::fs::remove_dir_all(&staging);
    result?;
    println!(
        "upgraded jacket {} -> {version} in {}",
        target.current,
        install.display()
    );
    Ok(Outcome::Upgraded {
        to: version,
    })
}

/// Download, verify, unpack and swap. Everything but the final swap happens in `staging`, so an
/// error anywhere leaves `install` as it was.
fn install_release(
    target: &Target,
    install: &Path,
    staging: &Path,
    tag: &str,
    (archive_url, sums_url): (&str, &str),
) -> Result<(), String> {
    std::fs::create_dir_all(staging).map_err(|e| {
        format!(
            "cannot write next to the install ({}): {e}\nhelp: the install folder's parent must be writable; for a system-wide install, run this as its owner",
            staging.display()
        )
    })?;
    let archive = staging.join(target.asset);
    let sums = staging.join("SHA256SUMS");
    println!("downloading {} ({tag})", target.asset);
    target.source.download(sums_url, &sums)?;
    target.source.download(archive_url, &archive)?;
    let sums = std::fs::read_to_string(&sums)
        .map_err(|e| format!("could not read the downloaded SHA256SUMS: {e}"))?;
    checksum::verify(&archive, target.asset, &sums)?;

    let unpacked = staging.join("unpacked");
    std::fs::create_dir_all(&unpacked)
        .map_err(|e| format!("could not create {}: {e}", unpacked.display()))?;
    target.source.extract(&archive, &unpacked)?;
    let fresh = unpacked.join(archive_root(target.asset));
    check_tree(&fresh, target.asset)?;
    swap::swap(install, &fresh, target.mode)
}

/// The unpacked tree must look like an install before it replaces one.
fn check_tree(fresh: &Path, asset: &str) -> Result<(), String> {
    let exe = std::env::consts::EXE_SUFFIX;
    let missing: Vec<PathBuf> = [
        format!("jaic{exe}"),
        format!("jacket{exe}"),
        "stdlib".to_owned(),
    ]
    .iter()
    .map(|name| fresh.join(name))
    .filter(|path| !path.exists())
    .collect();
    if missing.is_empty() {
        Ok(())
    } else {
        let names: Vec<_> = missing
            .iter()
            .filter_map(|p| p.file_name()?.to_str())
            .collect();
        Err(format!(
            "{asset} does not hold a complete toolchain (missing {})",
            names.join(", ")
        ))
    }
}

#[cfg(test)]
mod tests;
