//! Authored source identities and actual compiler workspace exercise the host boundary.
use jai_driver::CompilerSession;
use jai_platform::{
    HostServices, Platform, SharedVfs, SourceProvider, VfsHost, VfsHostLimits, VfsLimits,
    VfsPlatform,
};
use jai_vm::{CompilerOutputStream, SourceOrigin, host_effects::*};
use std::hash::{Hash, Hasher};
use std::path::Path;

fn origin(compiler: &CompilerSession, source: &[u8]) -> SourceOrigin {
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    source.hash(&mut hash);
    SourceOrigin {
        workspace: compiler.root(),
        path: "/jai-script/main.jai".into(),
        start: 0,
        end: source.len(),
        body_hash: hash.finish(),
        body: source.to_vec(),
        specialization: vec![],
    }
}
fn setup() -> (SharedVfs, VfsHost, CompilerSession, SourceOrigin) {
    let vfs = SharedVfs::new("/jai-script", VfsLimits::default()).unwrap();
    let source = b"main :: () -> int { return 42; }";
    vfs.insert("main.jai", source.to_vec()).unwrap();
    vfs.insert("data", b"old".to_vec()).unwrap();
    let host = VfsHost::new(
        vfs.clone(),
        Path::new(""),
        true,
        true,
        VfsHostLimits::default(),
    )
    .unwrap();
    let compiler = CompilerSession::new();
    let identity = origin(&compiler, source);
    (vfs, host, compiler, identity)
}
fn path(host: &VfsHost, name: &str) -> HostPath {
    host.file_scope().unwrap().resolve(Path::new(name)).unwrap()
}
fn write(host: &mut VfsHost, name: &str, bytes: &[u8]) {
    let path = path(host, name);
    assert_eq!(
        host.request(HostRequest::WriteEntireFile {
            path,
            bytes: bytes.to_vec()
        }),
        HostOutcome::Ready(HostResponse::WriteStaged)
    );
}
#[test]
fn platform_round_pins_sources_and_closed_requests_cannot_touch_native_files() {
    let (vfs, _, _, _) = setup();
    let platform = VfsPlatform::new(
        vfs.clone(),
        Path::new(""),
        false,
        false,
        VfsHostLimits::default(),
    )
    .unwrap();
    vfs.insert("main.jai", b"main :: () -> int { return 7; }".to_vec())
        .unwrap();
    assert_eq!(
        platform.source_files().read(Path::new("main.jai")).unwrap(),
        b"main :: () -> int { return 42; }"
    );
    assert!(
        platform
            .source_files()
            .read(Path::new("/etc/passwd"))
            .is_err()
    );
}
#[test]
fn failed_journal_rolls_back_file_and_console_then_replays_exact_observation() {
    let (vfs, mut host, _, identity) = setup();
    let read = HostRequest::ReadEntireFile(path(&host, "data"));
    host.begin(identity.clone()).unwrap();
    assert_eq!(
        host.request(read.clone()),
        HostOutcome::Ready(HostResponse::FileBytes(b"old".to_vec()))
    );
    write(&mut host, "data", b"new");
    host.write_console(CompilerOutputStream::StandardOutput, b"42\n".to_vec())
        .unwrap();
    host.finish(false).unwrap();
    assert_eq!(
        vfs.snapshot().unwrap().read(Path::new("data")).unwrap(),
        b"old"
    );
    assert!(host.take_console().is_empty());
    host.begin(identity.clone()).unwrap();
    assert_eq!(
        host.request(read),
        HostOutcome::Ready(HostResponse::FileBytes(b"old".to_vec()))
    );
    write(&mut host, "data", b"new");
    host.write_console(CompilerOutputStream::StandardOutput, b"42\n".to_vec())
        .unwrap();
    host.finish(true).unwrap();
    assert_eq!(
        vfs.snapshot().unwrap().read(Path::new("data")).unwrap(),
        b"new"
    );
    assert_eq!(host.take_console()[0].bytes, b"42\n");
    // Committed replay consumes its receipts without publishing a second time.
    host.begin(identity).unwrap();
    let read = HostRequest::ReadEntireFile(path(&host, "data"));
    assert_eq!(
        host.request(read),
        HostOutcome::Ready(HostResponse::FileBytes(b"old".to_vec()))
    );
    write(&mut host, "data", b"new");
    host.write_console(CompilerOutputStream::StandardOutput, b"42\n".to_vec())
        .unwrap();
    host.finish(true).unwrap();
    assert!(host.take_console().is_empty());
}
#[test]
fn parked_journal_requires_exact_origin_and_concurrent_edits_prevent_publication() {
    let (vfs, mut host, compiler, identity) = setup();
    host.begin(identity.clone()).unwrap();
    write(&mut host, "data", b"staged");
    host.write_console(CompilerOutputStream::StandardError, b"private".to_vec())
        .unwrap();
    host.suspend(&identity).unwrap();
    let other = origin(&compiler, b"other :: () {};");
    assert!(host.resume(&other).is_err());
    assert!(host.cancel(&other).is_err());
    assert!(host.take_console().is_empty());
    vfs.insert("data", b"edited".to_vec()).unwrap();
    host.resume(&identity).unwrap();
    assert!(host.finish(true).is_err());
    assert_eq!(
        vfs.snapshot().unwrap().read(Path::new("data")).unwrap(),
        b"edited"
    );
    assert!(host.take_console().is_empty());
}
#[test]
fn grants_are_bound_to_the_actual_host_and_open_failure_is_data() {
    let (_, mut host, _, identity) = setup();
    let (_, other, _, _) = setup();
    host.begin(identity.clone()).unwrap();
    assert_eq!(
        host.request(HostRequest::ReadFileForOpen(path(&host, "missing"))),
        HostOutcome::Ready(HostResponse::FileOpen(FileOpenObservation::Failed(
            FileOpenFailure::NotFound
        )))
    );
    assert_eq!(
        host.request(HostRequest::ReadEntireFile(path(&other, "data"))),
        HostOutcome::Rejected(HostError::UnknownCapability)
    );
    assert!(host.finish(true).is_err());
}
#[test]
fn atomic_replacements_admit_final_quota_independent_of_file_order() {
    let vfs = SharedVfs::new(
        "/jai-script",
        VfsLimits {
            bytes: 1000,
            files: 2,
        },
    )
    .unwrap();
    vfs.insert("a", vec![1; 400]).unwrap();
    vfs.insert("b", vec![2; 400]).unwrap();
    let mut host = VfsHost::new(
        vfs.clone(),
        Path::new(""),
        true,
        false,
        VfsHostLimits::default(),
    )
    .unwrap();
    let compiler = CompilerSession::new();
    host.begin(origin(&compiler, b"main :: () {}")).unwrap();
    write(&mut host, "a", &vec![3; 800]);
    write(&mut host, "b", &[]);
    host.finish(true).unwrap();
    let snapshot = vfs.snapshot().unwrap();
    assert_eq!(snapshot.read(Path::new("a")).unwrap(), vec![3; 800]);
    assert!(snapshot.read(Path::new("b")).unwrap().is_empty());
}

#[test]
fn rejected_console_operation_prevents_every_staged_publication() {
    let (vfs, _, _, identity) = setup();
    let mut host = VfsHost::new(
        vfs.clone(),
        Path::new(""),
        true,
        true,
        VfsHostLimits {
            console_bytes: 4,
            ..Default::default()
        },
    )
    .unwrap();
    host.begin(identity).unwrap();
    write(&mut host, "data", b"staged");
    host.write_console(CompilerOutputStream::StandardOutput, b"good".to_vec())
        .unwrap();
    assert!(
        host.write_console(CompilerOutputStream::StandardOutput, b"x".to_vec())
            .is_err()
    );
    assert!(host.finish(true).is_err());
    assert_eq!(
        vfs.snapshot().unwrap().read(Path::new("data")).unwrap(),
        b"old"
    );
    assert!(host.take_console().is_empty());
}
