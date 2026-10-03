//! Source ABI rejection is checked before any reviewed dependency tool executes.
#![cfg(any(target_os = "macos", target_os = "linux"))]

use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "jai-authored-native-proof-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let root = root.canonicalize().unwrap();
        fs::write(root.join("main.jai"), source).unwrap();
        Self(root)
    }
    fn library(&self) -> PathBuf {
        self.0.join("reviewed-vma-virtual")
    }
    fn build(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_jai-rs"));
        for key in [
            "JAI_RS_MODULE_PATH",
            "JAI_RS_STDLIB",
            "JAI_RS_PRELOAD",
            "JAI_RS_RUNTIME_SUPPORT",
            "JAI_RS_RUNTIME_ENTRY",
            "JAI_RS_RUNTIME_INITIALIZATION",
            "JAI_RS_RUNTIME_BACKTRACE",
            "JAI_RS_TARGET",
            "JAI_RS_CPU",
            "JAI_RS_FEATURES",
            "JAI_RS_OPT",
            "JAI_RS_DEBUG",
            "JAI_RS_CLANG",
            "JAI_RS_AR",
            "JAI_RS_NATIVE_VMA_RECEIPT",
            "JAI_RS_NATIVE_VMA_LIBRARY",
        ] {
            command.env_remove(key);
        }
        command
            .arg("build")
            .arg(self.0.join("main.jai"))
            .arg(self.0.join("program"));
        command
    }
    fn reject_before_receipt(&self, expected: &str) {
        // This absent receipt cannot launch a rebuild. The source ABI diagnostic
        // must precede even the receipt read.
        let output = self
            .build()
            .env(
                "JAI_RS_NATIVE_VMA_RECEIPT",
                self.0.join("must-not-be-read.json"),
            )
            .env("JAI_RS_NATIVE_VMA_LIBRARY", self.library())
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(expected),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!self.0.join("program").exists());
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn unreviewed_symbol_cannot_link_with_a_receipt_path() {
    Fixture::new("VMA :: #library,no_dll \"reviewed-vma-virtual\"; vmaCreateAllocator :: () #foreign VMA; main :: ()->int { vmaCreateAllocator(); return 0; }")
        .reject_before_receipt("outside the reviewed VMA virtual ABI");
}

#[test]
fn external_data_cannot_borrow_the_reviewed_function_only_vma_authority() {
    Fixture::new("VMA :: #library,no_dll \"reviewed-vma-virtual\"; counter:s64 #elsewhere VMA \"vmaCounter\"; main :: ()->int { return cast(int)counter; }")
        .reject_before_receipt("reachable external data is outside the reviewed VMA virtual ABI");
}

#[test]
fn source_record_layout_must_match_the_measured_native_contract() {
    Fixture::new("VMA :: #library,no_dll \"reviewed-vma-virtual\"; Wrong :: struct { size:u32; flags:u32; callbacks:*void; } vmaCreateVirtualBlock :: (info:*Wrong, result:**void)->s32 #foreign VMA; main :: ()->int { value:Wrong; block:*void; return cast(int)vmaCreateVirtualBlock(*value,*block); }")
        .reject_before_receipt("does not match the reviewed native signature and field layout");
}

#[test]
fn protected_library_identity_is_rejected_before_abi_or_receipt() {
    let protected = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
    let fixture = Fixture::new(&format!(
        "VMA :: #library,no_dll {:?}; vmaCreateAllocator :: () #foreign VMA; main :: () {{ vmaCreateAllocator(); }}",
        protected.to_str().unwrap()
    ));
    let output = fixture
        .build()
        .env("JAI_RS_NATIVE_VMA_LIBRARY", protected)
        .env(
            "JAI_RS_NATIVE_VMA_RECEIPT",
            fixture.0.join("must-not-be-read.json"),
        )
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("inside protected source inputs"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!fixture.0.join("program").exists());
}

#[test]
fn partial_configuration_is_not_link_authority() {
    let fixture = Fixture::new("main :: ()->int { return 0; }");
    let output = fixture
        .build()
        .env("JAI_RS_NATIVE_VMA_LIBRARY", fixture.library())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("requires both"));
}

#[test]
fn unused_configured_dependency_never_reads_receipt() {
    let fixture = Fixture::new(
        "VMA :: #library,no_dll \"reviewed-vma-virtual\"; main :: ()->int { return 0; }",
    );
    let output = fixture
        .build()
        .env("JAI_RS_NATIVE_VMA_LIBRARY", fixture.library())
        .env(
            "JAI_RS_NATIVE_VMA_RECEIPT",
            fixture.0.join("must-not-be-read.json"),
        )
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        Command::new(fixture.0.join("program"))
            .status()
            .unwrap()
            .success()
    );
}

#[test]
fn caller_json_cannot_mint_native_link_authority() {
    let fixture = Fixture::new(include_str!("../../../tests/fixtures/vma-virtual-abi.jai"));
    let receipt = fixture.0.join("forged-receipt.json");
    fs::write(&receipt, br#"{"format":1,"link_authority":true}"#).unwrap();
    let output = fixture
        .build()
        .env("JAI_RS_NATIVE_VMA_LIBRARY", fixture.library())
        .env("JAI_RS_NATIVE_VMA_RECEIPT", receipt)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("receipt configuration mismatch"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!fixture.0.join("program").exists());
}

#[test]
fn fresh_reviewed_source_dependency_executes_the_jai_abi_fixture() {
    let Some(receipt) = std::env::var_os("JAI_RS_TEST_VMA_RECEIPT") else {
        eprintln!(
            "VMA native gate requires an explicitly reviewed source-build receipt (JAI_RS_TEST_VMA_RECEIPT)"
        );
        return;
    };
    let fixture = Fixture::new(include_str!("../../../tests/fixtures/vma-virtual-abi.jai"));
    let output = fixture
        .build()
        .env("JAI_RS_NATIVE_VMA_RECEIPT", receipt)
        .env("JAI_RS_NATIVE_VMA_LIBRARY", fixture.library())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        Command::new(fixture.0.join("program"))
            .status()
            .unwrap()
            .code(),
        Some(42)
    );
}
