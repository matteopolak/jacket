#![cfg(not(target_arch = "wasm32"))]
//! Actual registered native roots and policies paired with private console journals.
use jai_driver::{
    CompilerSession,
    host_io::{HostIo, platform::NativeHostServices},
};
use jai_platform::{HostServices, VfsHostLimits};
use jai_vm::{CompilerOutputStream, SourceOrigin, host_effects::*};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "jai-native-platform-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        fs::write(path.join("data"), b"sentinel").unwrap();
        Self(path)
    }
    fn host(&self) -> (HostIo, FileRootId) {
        let mut host = HostIo::default();
        let root = host.register_root(&self.0, true).unwrap();
        (host, root)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn origin(compiler: &CompilerSession) -> SourceOrigin {
    let body = b"main :: () { /* independently authored host fixture */ }";
    SourceOrigin {
        workspace: compiler.root(),
        path: "native-fixture/main.jai".into(),
        start: 0,
        end: body.len(),
        body_hash: 0,
        body: body.to_vec(),
        specialization: vec![],
    }
}
#[test]
fn native_console_rejection_prevents_the_real_file_rename() {
    let fixture = Fixture::new();
    let (host, root) = fixture.host();
    let mut adapter = NativeHostServices::new(
        host,
        root,
        Path::new(""),
        VfsHostLimits {
            console_bytes: 3,
            ..Default::default()
        },
    )
    .unwrap();
    let compiler = CompilerSession::new();
    adapter.begin(origin(&compiler)).unwrap();
    let path = adapter
        .file_scope()
        .unwrap()
        .resolve(Path::new("data"))
        .unwrap();
    assert_eq!(
        adapter.request(HostRequest::WriteEntireFile {
            path,
            bytes: b"unpublished".to_vec()
        }),
        HostOutcome::Ready(HostResponse::WriteStaged)
    );
    assert!(
        adapter
            .write_console(CompilerOutputStream::StandardOutput, b"four".to_vec())
            .is_err()
    );
    assert!(adapter.finish(true).is_err());
    assert_eq!(fs::read(fixture.0.join("data")).unwrap(), b"sentinel");
    assert!(adapter.take_console().is_empty());
}
#[test]
fn native_adapter_keeps_original_input_protection() {
    let fixture = Fixture::new();
    let (mut host, root) = fixture.host();
    host.protect_original_inputs(&fixture.0.join("data"))
        .unwrap();
    let mut adapter =
        NativeHostServices::new(host, root, Path::new(""), VfsHostLimits::default()).unwrap();
    let compiler = CompilerSession::new();
    adapter.begin(origin(&compiler)).unwrap();
    let path = adapter
        .file_scope()
        .unwrap()
        .resolve(Path::new("data"))
        .unwrap();
    assert!(matches!(
        adapter.request(HostRequest::WriteEntireFile {
            path,
            bytes: b"denied".to_vec()
        }),
        HostOutcome::Rejected(HostError::Denied(_))
    ));
    assert!(adapter.finish(true).is_err());
    assert_eq!(fs::read(fixture.0.join("data")).unwrap(), b"sentinel");
    assert!(adapter.take_console().is_empty());
}
#[test]
fn native_adapter_cannot_adopt_an_already_active_source_journal() {
    let fixture = Fixture::new();
    let (mut host, root) = fixture.host();
    let compiler = CompilerSession::new();
    host.begin(origin(&compiler)).unwrap();
    assert!(matches!(
        NativeHostServices::new(host, root, Path::new(""), VfsHostLimits::default()),
        Err(HostError::Transaction(_))
    ));
}
