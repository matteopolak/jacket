//! Explicit storage casts use checked target extents through #run and native code.
#![cfg(any(target_os = "linux", target_os = "macos"))]
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};
struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let directory = std::env::temp_dir().join(format!(
            "jai-cli-storage-bitcast-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        fs::write(directory.join("main.jai"), source).unwrap();
        Self(directory)
    }
    fn build(&self, optimization: &str) -> std::process::Output {
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
            .arg(self.0.join("program"))
            .arg(optimization)
            .output()
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn storage_casts_preserve_bits_and_prefixes_in_run_and_native_code() {
    let fixture = Fixture::new(
        r#"
Word :: struct { bits:u32; }
evaluate :: ()->int {
    bits:u32 = 0x42280000;
    value := cast,force(float32) bits;
    restored := cast,force(u32) value;
    record := cast,force(Word) restored;
    bytes:[4]u8 = .[42,0,0,0];
    prefix := cast,FORCE(u8) bytes;
    if value!=42.0 || restored!=bits || record.bits!=bits || prefix!=42 return 1;
    return 42;
}
COMPUTED :: #run evaluate();
main :: ()->int { if COMPUTED!=42 return 2; return evaluate(); }
"#,
    );
    for optimization in ["-O0", "-O2"] {
        let build = fixture.build(optimization);
        assert!(
            build.status.success(),
            "{}",
            String::from_utf8_lossy(&build.stderr)
        );
        assert_eq!(
            Command::new(fixture.0.join("program"))
                .status()
                .unwrap()
                .code(),
            Some(42)
        );
    }
}

#[test]
fn incompatible_storage_extents_are_located_and_preserve_existing_outputs() {
    for (source, diagnostic) in [
        (
            "// Authored unequal-size cast.\nmain :: ()->int { value:u32=42; return cast(int) cast,force(u64) value; }\n",
            "storage cast requires equal source and destination sizes (source 4 bytes, destination 8 bytes)",
        ),
        (
            "// Authored lowercase cast cannot read a prefix.\nmain :: ()->int { value:[4]u8=.[42,0,0,0]; return cast(int) cast,force(u8) value; }\n",
            "storage cast requires equal source and destination sizes (source 4 bytes, destination 1 bytes)",
        ),
        (
            "// Authored uppercase cast cannot expand storage.\nmain :: ()->int { value:u32=42; return cast(int) cast,FORCE(u64) value; }\n",
            "storage cast requires a destination no larger than the source (source 4 bytes, destination 8 bytes)",
        ),
    ] {
        let fixture = Fixture::new(source);
        let artifact = fixture.0.join("program");
        fs::write(&artifact, b"preserved authored output").unwrap();
        let build = fixture.build("-O0");
        assert!(!build.status.success());
        let stderr = String::from_utf8_lossy(&build.stderr);
        assert!(stderr.contains(diagnostic), "{stderr}");
        assert!(stderr.contains("main.jai:2:"), "{stderr}");
        assert_eq!(fs::read(artifact).unwrap(), b"preserved authored output");
    }
}
