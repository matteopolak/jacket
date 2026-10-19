//! `upgrade` end to end against a fake release in a temporary folder: no network, no `curl`,
//! no `tar`.
use super::checksum::sha256_hex;
use super::net::Source;
use super::swap::{self, Mode};
use super::*;
use serde_json::json;
use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, Ordering};

const ASSET: &str = "jacket-linux-x64.tar.gz";
const EXE: &str = std::env::consts::EXE_SUFFIX;

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Scratch {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("jacket-upgrade-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Scratch(dir)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn write(path: &Path, text: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

fn read(path: &Path) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

/// An install holding the files of `label`.
fn make_install(dir: &Path, label: &str) -> PathBuf {
    write(&dir.join(format!("jaic{EXE}")), label);
    write(&dir.join(format!("jacket{EXE}")), label);
    write(&dir.join("stdlib/prelude/module.jai"), label);
    write(&dir.join("native-libs/lib.a"), label);
    dir.join(format!("jacket{EXE}"))
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let dest = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &dest);
        } else {
            std::fs::copy(entry.path(), dest).unwrap();
        }
    }
}

/// A release folder: `assets/<name>` are the downloads, `trees/<archive>` is what extracting
/// that archive produces.
struct Fake {
    dir: PathBuf,
    latest: String,
    asked: RefCell<Vec<Option<String>>>,
}

impl Fake {
    /// A release `tag` whose archive unpacks to a toolchain labelled `label`.
    fn new(dir: &Path, tag: &str, label: &str) -> Fake {
        let fake = Fake {
            dir: dir.join("release"),
            latest: tag.to_owned(),
            asked: RefCell::default(),
        };
        let archive = format!("archive-of-{label}");
        write(&fake.dir.join("assets").join(ASSET), &archive);
        let sums = format!("{}  {ASSET}\n", sha256_hex(archive.as_bytes()));
        write(&fake.dir.join("assets/SHA256SUMS"), &sums);
        make_install(
            &fake.dir.join("trees").join(ASSET).join("jacket-linux-x64"),
            label,
        );
        fake
    }

    fn json(&self) -> String {
        let assets: Vec<_> = ["SHA256SUMS", ASSET]
            .iter()
            .filter(|name| self.dir.join("assets").join(name).exists())
            .map(|name| json!({ "name": name, "browser_download_url": format!("https://example.test/{name}") }))
            .collect();
        json!({ "tag_name": self.latest, "assets": assets }).to_string()
    }
}

impl Source for Fake {
    fn release_json(&self, tag: Option<&str>) -> Result<String, String> {
        self.asked.borrow_mut().push(tag.map(str::to_owned));
        Ok(self.json())
    }

    fn download(&self, url: &str, dest: &Path) -> Result<(), String> {
        let name = url.rsplit('/').next().unwrap();
        std::fs::copy(self.dir.join("assets").join(name), dest)
            .map(|_| ())
            .map_err(|e| format!("fake download of {name}: {e}"))
    }

    fn extract(&self, archive: &Path, dest: &Path) -> Result<(), String> {
        let name = archive.file_name().unwrap();
        copy_tree(&self.dir.join("trees").join(name), dest);
        Ok(())
    }
}

fn run(
    exe: &Path,
    current: &str,
    fake: &Fake,
    mode: Mode,
    options: &Options,
    answer: bool,
) -> Result<Outcome, String> {
    let confirm = move |_: &str| answer;
    let target = Target {
        exe,
        current,
        asset: ASSET,
        source: fake,
        mode,
        confirm: &confirm,
    };
    upgrade(&target, options)
}

fn yes() -> Options {
    Options {
        yes: true,
        ..Options::default()
    }
}

#[test]
fn the_same_version_is_already_up_to_date() {
    let s = Scratch::new();
    let exe = make_install(&s.0.join("install"), "old");
    let fake = Fake::new(&s.0, "v0.21.0", "new");
    let outcome = run(&exe, "0.21.0", &fake, Mode::native(), &yes(), true).unwrap();
    assert_eq!(outcome, Outcome::UpToDate);
    assert_eq!(read(&exe).as_deref(), Some("old"));
}

#[test]
fn an_older_latest_release_is_not_a_downgrade() {
    let s = Scratch::new();
    let exe = make_install(&s.0.join("install"), "old");
    let fake = Fake::new(&s.0, "v0.20.0", "new");
    let outcome = run(&exe, "0.21.0", &fake, Mode::native(), &yes(), true).unwrap();
    assert_eq!(outcome, Outcome::UpToDate);
    assert_eq!(read(&exe).as_deref(), Some("old"));
}

#[test]
fn check_reports_and_changes_nothing() {
    let s = Scratch::new();
    let exe = make_install(&s.0.join("install"), "old");
    let fake = Fake::new(&s.0, "v0.22.0", "new");
    let options = Options {
        check: true,
        ..Options::default()
    };
    let outcome = run(&exe, "0.21.0", &fake, Mode::native(), &options, true).unwrap();
    assert_eq!(
        outcome,
        Outcome::Available {
            to: "0.22.0".into()
        }
    );
    assert_eq!(read(&exe).as_deref(), Some("old"));
    assert!(!swap::sibling(exe.parent().unwrap(), swap::NEW_SUFFIX).exists());
}

#[test]
fn the_whole_install_is_replaced() {
    for mode in [Mode::Folder, Mode::Entries] {
        let s = Scratch::new();
        let install = s.0.join("install");
        let exe = make_install(&install, "old");
        write(&install.join("only-in-old"), "old");
        let fake = Fake::new(&s.0, "v0.22.0", "new");
        let outcome = run(&exe, "0.21.0", &fake, mode, &yes(), true).unwrap();
        assert_eq!(
            outcome,
            Outcome::Upgraded {
                to: "0.22.0".into()
            }
        );
        for file in [
            format!("jaic{EXE}"),
            format!("jacket{EXE}"),
            "stdlib/prelude/module.jai".into(),
            "native-libs/lib.a".into(),
        ] {
            assert_eq!(
                read(&install.join(&file)).as_deref(),
                Some("new"),
                "{file} ({mode:?})"
            );
        }
        assert_eq!(read(&install.join("only-in-old")), None, "{mode:?}");
        let parent = install.parent().unwrap();
        let left: Vec<_> = std::fs::read_dir(parent)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert!(
            !left.iter().any(|n| n.contains(".jacket-")),
            "leftovers {left:?}"
        );
    }
}

#[test]
fn a_checksum_mismatch_leaves_the_install_untouched() {
    let s = Scratch::new();
    let install = s.0.join("install");
    let exe = make_install(&install, "old");
    let fake = Fake::new(&s.0, "v0.22.0", "new");
    // The archive no longer matches the published hash.
    write(&fake.dir.join("assets").join(ASSET), "tampered");
    let error = run(&exe, "0.21.0", &fake, Mode::native(), &yes(), true).unwrap_err();
    assert!(error.contains("checksum mismatch"), "{error}");
    assert_eq!(read(&exe).as_deref(), Some("old"));
    assert_eq!(
        read(&install.join("stdlib/prelude/module.jai")).as_deref(),
        Some("old")
    );
    assert!(
        !swap::sibling(&install, swap::NEW_SUFFIX).exists(),
        "staging is cleaned up"
    );
}

#[test]
fn a_release_without_this_platforms_archive_is_refused() {
    let s = Scratch::new();
    let exe = make_install(&s.0.join("install"), "old");
    let fake = Fake::new(&s.0, "v0.22.0", "new");
    std::fs::remove_file(fake.dir.join("assets").join(ASSET)).unwrap();
    let error = run(&exe, "0.21.0", &fake, Mode::native(), &yes(), true).unwrap_err();
    assert!(
        error.contains(ASSET) && error.contains("no build"),
        "{error}"
    );
    assert_eq!(read(&exe).as_deref(), Some("old"));
}

#[test]
fn a_release_without_checksums_is_refused() {
    let s = Scratch::new();
    let exe = make_install(&s.0.join("install"), "old");
    let fake = Fake::new(&s.0, "v0.22.0", "new");
    std::fs::remove_file(fake.dir.join("assets/SHA256SUMS")).unwrap();
    let error = run(&exe, "0.21.0", &fake, Mode::native(), &yes(), true).unwrap_err();
    assert!(error.contains("SHA256SUMS"), "{error}");
    assert_eq!(read(&exe).as_deref(), Some("old"));
}

#[test]
fn an_incomplete_archive_does_not_replace_a_working_install() {
    let s = Scratch::new();
    let install = s.0.join("install");
    let exe = make_install(&install, "old");
    let fake = Fake::new(&s.0, "v0.22.0", "new");
    std::fs::remove_dir_all(
        fake.dir
            .join("trees")
            .join(ASSET)
            .join("jacket-linux-x64/stdlib"),
    )
    .unwrap();
    let error = run(&exe, "0.21.0", &fake, Mode::native(), &yes(), true).unwrap_err();
    assert!(
        error.contains("missing") && error.contains("stdlib"),
        "{error}"
    );
    assert_eq!(read(&exe).as_deref(), Some("old"));
}

#[test]
fn declining_the_prompt_changes_nothing() {
    let s = Scratch::new();
    let exe = make_install(&s.0.join("install"), "old");
    let fake = Fake::new(&s.0, "v0.22.0", "new");
    let outcome = run(
        &exe,
        "0.21.0",
        &fake,
        Mode::native(),
        &Options::default(),
        false,
    )
    .unwrap();
    assert_eq!(outcome, Outcome::Declined);
    assert_eq!(read(&exe).as_deref(), Some("old"));
}

#[test]
fn a_requested_version_is_asked_for_by_tag_and_may_be_older() {
    let s = Scratch::new();
    let exe = make_install(&s.0.join("install"), "current");
    let fake = Fake::new(&s.0, "v0.20.0", "older");
    let options = Options {
        version: Some("0.20.0".into()),
        yes: true,
        ..Options::default()
    };
    let outcome = run(&exe, "0.21.0", &fake, Mode::native(), &options, true).unwrap();
    assert_eq!(
        outcome,
        Outcome::Upgraded {
            to: "0.20.0".into()
        }
    );
    assert_eq!(fake.asked.borrow().as_slice(), [Some("v0.20.0".to_owned())]);
    assert_eq!(read(&exe).as_deref(), Some("older"));
}

#[test]
fn a_bad_version_argument_is_refused_before_any_request() {
    let s = Scratch::new();
    let exe = make_install(&s.0.join("install"), "old");
    let fake = Fake::new(&s.0, "v0.22.0", "new");
    let options = Options {
        version: Some("../../etc".into()),
        yes: true,
        ..Options::default()
    };
    let error = run(&exe, "0.21.0", &fake, Mode::native(), &options, true).unwrap_err();
    assert!(error.contains("not a version"), "{error}");
    assert!(fake.asked.borrow().is_empty());
}

#[test]
fn managed_installs_are_refused_with_the_right_command() {
    let s = Scratch::new();
    let keg = s.0.join("Cellar/jacket/0.21.0/libexec");
    let exe = make_install(&keg, "old");
    let fake = Fake::new(&s.0, "v0.22.0", "new");
    let error = run(&exe, "0.21.0", &fake, Mode::native(), &yes(), true).unwrap_err();
    assert!(error.contains("brew upgrade jacket"), "{error}");
    assert!(fake.asked.borrow().is_empty(), "nothing was fetched");
    assert_eq!(read(&exe).as_deref(), Some("old"));

    // `--check` still reports, and names the command.
    let options = Options {
        check: true,
        ..Options::default()
    };
    let outcome = run(&exe, "0.21.0", &fake, Mode::native(), &options, true).unwrap();
    assert!(matches!(outcome, Outcome::Available { .. }));
}

#[test]
fn a_folder_that_is_not_an_install_is_refused() {
    let s = Scratch::new();
    let exe = s.0.join("target/release").join(format!("jacket{EXE}"));
    write(&exe, "dev build");
    let fake = Fake::new(&s.0, "v0.22.0", "new");
    let error = run(&exe, "0.21.0", &fake, Mode::native(), &yes(), true).unwrap_err();
    assert!(error.contains("not an installed toolchain"), "{error}");
    assert_eq!(read(&exe).as_deref(), Some("dev build"));
}

#[test]
fn leftovers_of_an_earlier_upgrade_are_removed() {
    let s = Scratch::new();
    let install = s.0.join("install");
    let exe = make_install(&install, "old");
    write(
        &swap::sibling(&install, swap::OLD_SUFFIX).join("jacket"),
        "stale",
    );
    write(&install.join(swap::OLD_SUFFIX).join("jacket.exe"), "stale");
    let fake = Fake::new(&s.0, "v0.21.0", "new");
    run(&exe, "0.21.0", &fake, Mode::native(), &yes(), true).unwrap();
    assert!(!swap::sibling(&install, swap::OLD_SUFFIX).exists());
    assert!(!install.join(swap::OLD_SUFFIX).exists());
}

#[test]
fn release_json_is_read_defensively() {
    assert!(Release::parse("not json").is_err());
    assert!(Release::parse("{\"message\":\"Not Found\"}").is_err());
    let release = Release::parse("{\"tag_name\":\"v1.2.3\",\"assets\":[{\"name\":\"a\"},{\"name\":\"b\",\"browser_download_url\":\"https://x/b\"}]}").unwrap();
    assert_eq!(release.tag, "v1.2.3");
    assert_eq!(release.asset("a"), None);
    assert_eq!(release.asset("b"), Some("https://x/b"));
}

#[test]
fn archive_names_follow_the_shared_installer_vectors() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/installers/vectors.json");
    let vectors: serde_json::Value = serde_json::from_str(&read(&path).unwrap()).unwrap();
    let mut checked = 0;
    for entry in vectors["platforms"].as_array().unwrap() {
        let Some(node) = entry.get("node") else {
            continue;
        };
        let os = match node[0].as_str().unwrap() {
            "darwin" => "macos",
            "win32" => "windows",
            other => other,
        };
        let arch = match node[1].as_str().unwrap() {
            "x64" => "x86_64",
            "arm64" => "aarch64",
            other => other,
        };
        assert_eq!(
            asset_name(os, arch).as_deref(),
            entry["asset"].as_str(),
            "{os} {arch}"
        );
        checked += 1;
    }
    assert!(checked >= 8, "{checked}");
    assert_eq!(asset_name("freebsd", "x86_64"), None);
}

#[test]
fn archive_roots_drop_the_extension() {
    assert_eq!(archive_root("jacket-linux-x64.tar.gz"), "jacket-linux-x64");
    assert_eq!(
        archive_root("jacket-windows-arm64.zip"),
        "jacket-windows-arm64"
    );
}
