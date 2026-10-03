#![cfg(not(target_arch = "wasm32"))]
use jai_native_source::{NativeSourceSnapshot, SourceOverlay};
use jai_source::{SourceProvider, SourceTextSnapshot};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let path = std::env::temp_dir().join(format!(
            "jai-authored-source-observation-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn file(&self) -> PathBuf {
        self.0.join("authored.jai")
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn retention_requires_successful_actual_read_and_matching_decoded_text() {
    let fixture = Fixture::new();
    let path = fixture.file();
    fs::write(&path, b"answer :: 42;").unwrap();
    let provider = NativeSourceSnapshot::default();
    assert!(
        provider
            .retain_decoded_text(&path, "answer :: 42;")
            .is_err()
    );
    provider.read(&path).unwrap();
    assert!(
        provider
            .retain_decoded_text(&path, "answer :: 99;")
            .is_err()
    );
    let first = provider
        .retain_decoded_text(&path, "answer :: 42;")
        .unwrap();
    let replay = provider
        .retain_decoded_text(&path, "answer :: 42;")
        .unwrap();
    assert_eq!(first, replay);
    let failed = fixture.0.join("missing.jai");
    assert!(provider.read(&failed).is_err());
    assert!(provider.retain_decoded_text(&failed, "fabricated").is_err());
}

#[test]
fn a_snapshot_keeps_its_real_first_observation_while_a_new_snapshot_gets_another_owner() {
    let fixture = Fixture::new();
    let path = fixture.file();
    fs::write(&path, b"answer :: 42;").unwrap();
    let first = NativeSourceSnapshot::default();
    first.read(&path).unwrap();
    let owner = first.retain_decoded_text(&path, "answer :: 42;").unwrap();
    fs::write(&path, b"answer :: 99;").unwrap();
    assert_eq!(first.read(&path).unwrap(), b"answer :: 42;");
    assert_eq!(
        owner,
        first.retain_decoded_text(&path, "answer :: 42;").unwrap()
    );
    let second = NativeSourceSnapshot::default();
    assert_eq!(second.read(&path).unwrap(), b"answer :: 99;");
    assert_ne!(
        owner,
        second.retain_decoded_text(&path, "answer :: 99;").unwrap()
    );
}

#[test]
fn overlay_retains_only_actual_snapshot_inputs_and_replacement_discards_the_owner() {
    let path = Path::new("/jai-retained-fixture/source.jai");
    let mut overlay = SourceOverlay::new();
    let owner = SourceTextSnapshot::new("answer :: 42;".into());
    overlay.insert_snapshot(path, owner.clone()).unwrap();
    assert_eq!(
        owner,
        overlay.retain_decoded_text(path, owner.text()).unwrap()
    );
    assert!(overlay.retain_decoded_text(path, "answer :: 99;").is_err());
    overlay.insert(path, b"answer :: 42;".to_vec()).unwrap();
    assert_ne!(
        owner,
        overlay.retain_decoded_text(path, owner.text()).unwrap()
    );
}

#[test]
fn an_overlay_forwards_physical_reads_to_its_actual_retained_base() {
    let fixture = Fixture::new();
    let path = fixture.file();
    fs::write(&path, b"answer :: 42;").unwrap();
    let base = Arc::new(NativeSourceSnapshot::default());
    let first = SourceOverlay::with_base(base.clone());
    first.read(&path).unwrap();
    let owner = first.retain_decoded_text(&path, "answer :: 42;").unwrap();
    let second = SourceOverlay::with_base(base);
    assert_eq!(
        owner,
        second.retain_decoded_text(&path, "answer :: 42;").unwrap()
    );
}
