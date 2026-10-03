use super::*;
use crate::host_io::HostLimits;
use jai_vm::{SourceOrigin, WorkspaceId, host_effects::*};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "jai-input-policy-{}-{:?}",
            std::process::id(),
            HostRequestKey::allocate()
        ));
        fs::create_dir_all(path.join("corpus")).unwrap();
        fs::write(
            path.join("corpus/fixture.json"),
            b"trusted fixture inventory",
        )
        .unwrap();
        Self(path.canonicalize().unwrap())
    }
    fn policy(&self, hashes: &[[u8; 32]]) -> OriginalInputPolicy {
        let hash = manifest_fingerprint(&self.0.join("corpus/fixture.json")).unwrap();
        OriginalInputPolicy::load(&self.0, &[("corpus/fixture.json", hash)], hashes).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn origin() -> SourceOrigin {
    SourceOrigin {
        workspace: WorkspaceId::from_raw(1).unwrap(),
        path: "/own-policy-fixture.jai".into(),
        start: 0,
        end: 10,
        body_hash: 1,
        body: vec![],
        specialization: vec![],
    }
}
#[test]
fn compiled_inventory_receipts_load_without_loading_original_native_files() {
    let source = Path::new(file!());
    let workspace = if source.is_absolute() {
        source.ancestors().nth(6).unwrap().to_owned()
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    };
    let policy = OriginalInputPolicy::from_trusted_inventory(&workspace).unwrap();
    assert!(policy.native_fingerprints.len() >= 6);
    assert!(
        policy
            .protected_roots()
            .iter()
            .any(|root| root.ends_with("reference"))
    );
    let mut host = HostIo::default();
    policy.apply(&mut host).unwrap();
    policy.apply(&mut host).unwrap();
}
#[test]
fn changed_manifest_cannot_update_compiled_deny_authority() {
    let fixture = Fixture::new();
    let hash = manifest_fingerprint(&fixture.0.join("corpus/fixture.json")).unwrap();
    fs::write(fixture.0.join("corpus/fixture.json"), b"changed inventory").unwrap();
    assert!(matches!(
        OriginalInputPolicy::load(&fixture.0, &[("corpus/fixture.json", hash)], &[]),
        Err(HostError::Denied(_))
    ));
}
#[test]
fn absent_original_trees_remain_readable_and_immutable_after_creation() {
    let fixture = Fixture::new();
    let policy = fixture.policy(&[]);
    let mut host = HostIo::default();
    policy.apply(&mut host).unwrap();
    let root = host.register_root(&fixture.0, true).unwrap();
    fs::create_dir(fixture.0.join("reference")).unwrap();
    host.begin(origin()).unwrap();
    assert_eq!(
        host.request(HostRequest::ReadEntireFile(path.clone())),
        HostOutcome::Ready(HostResponse::FileBytes(vec![0, 255, 1]))
    );
    assert!(matches!(
        host.request(HostRequest::WriteEntireFile {
            path,
            bytes: vec![2]
        }),
        HostOutcome::Rejected(HostError::Denied(_))
    ));
    host.finish(false).unwrap();
    assert_eq!(
        [0, 255, 1]
    );
}
#[test]
fn whole_policy_install_is_atomic_and_denies_copied_native_bytes() {
    let fixture = Fixture::new();
    let bytes = b"inert authored native inventory fixture";
    fs::write(fixture.0.join("copied-native"), bytes).unwrap();
    let policy = fixture.policy(&[Sha256::digest(bytes).into()]);
    let mut host = HostIo::new(HostLimits {
        requests: 3,
        ..Default::default()
    });
    let root = host.register_root(&fixture.0, true).unwrap();
    assert!(matches!(policy.apply(&mut host), Err(HostError::Budget(_))));
    // Rejected application installed neither roots nor fingerprints.
    host.register_read_only_program(
        &fixture.0.join("copied-native"),
        ProcessArguments::new([]).unwrap(),
        root,
    )
    .unwrap();
    fs::create_dir(fixture.0.join("reference")).unwrap();
    host.begin(origin()).unwrap();
    assert_eq!(
        host.request(HostRequest::WriteEntireFile {
            bytes: b"own".to_vec(),
        }),
        HostOutcome::Ready(HostResponse::WriteStaged)
    );
    host.finish(true).unwrap();
    let mut host = HostIo::default();
    policy.apply(&mut host).unwrap();
    let root = host.register_root(&fixture.0, false).unwrap();
    assert!(matches!(
        host.register_read_only_program(
            &fixture.0.join("copied-native"),
            ProcessArguments::new([]).unwrap(),
            root
        ),
        Err(HostError::Denied(_))
    ));
    host.begin(origin()).unwrap();
    assert_eq!(
        host.request(HostRequest::ReadEntireFile(
            HostPath::new(root, "copied-native").unwrap()
        )),
        HostOutcome::Ready(HostResponse::FileBytes(bytes.to_vec()))
    );
    host.finish(true).unwrap();
    assert!(matches!(
        policy.apply(&mut host),
        Err(HostError::Transaction(_))
    ));
}
#[cfg(unix)]
#[test]
fn dangling_original_root_protects_its_future_target() {
    let fixture = Fixture::new();
    let target = Fixture::new();
    std::os::unix::fs::symlink(target.0.join("missing"), fixture.0.join("reference")).unwrap();
    let policy = fixture.policy(&[]);
    assert!(policy.protected_roots().contains(&target.0.join("missing")));
    let mut host = HostIo::default();
    policy.apply(&mut host).unwrap();
    let root = host.register_root(&target.0, true).unwrap();
    fs::create_dir(target.0.join("missing")).unwrap();
    fs::write(target.0.join("missing/asset"), b"static").unwrap();
    host.begin(origin()).unwrap();
    let path = HostPath::new(root, "missing/asset").unwrap();
    assert_eq!(
        host.request(HostRequest::ReadEntireFile(path.clone())),
        HostOutcome::Ready(HostResponse::FileBytes(b"static".to_vec()))
    );
    assert!(matches!(
        host.request(HostRequest::WriteEntireFile {
            path,
            bytes: vec![]
        }),
        HostOutcome::Rejected(HostError::Denied(_))
    ));
    host.finish(false).unwrap();
}
