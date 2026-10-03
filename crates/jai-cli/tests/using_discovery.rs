//! Parsed using decisions flow through the driver and freshly emitted native code.
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str, library: &str) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "jai-cli-using-discovery-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        fs::write(path.join("main.jai"), source).unwrap();
        fs::write(path.join("library.jai"), library).unwrap();
        Self(path)
    }
    fn build(&self) -> PathBuf {
        let artifact = self.0.join("authored-using-program");
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
        let output = command
            .arg("build")
            .arg(self.0.join("main.jai"))
            .arg(&artifact)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        artifact
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn execute42(path: &Path) {
    let mut child = Command::new(path).spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert_eq!(status.code(), Some(42));
            return;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("fresh authored using fixture timed out");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn computed_using_filters_and_map_execute_fresh_native_42() {
    for (source, library) in [
        (
            "Lib::#import,file \"library.jai\"; names::()->[]string{return .[\"value\"]; } using,only(names()) Lib; main::()->int{return value;}",
            "value::42; omitted::5;",
        ),
        (
            "Lib::#import,file \"library.jai\"; names::()->[]string{return .[\"omitted\"]; } using,except(names()) Lib; main::()->int{return value;}",
            "value::42; omitted::5;",
        ),
        (
            "Lib::#import,file \"library.jai\"; mapper::(names:[]string){names[0]=\"renamed\";} using,map(mapper) Lib; main::()->int{return renamed;}",
            "value::42;",
        ),
    ] {
        let fixture = Fixture::new(source, library);
        execute42(&fixture.build());
    }
}

#[test]
fn mapped_mutable_field_alias_writes_original_native_storage() {
    let fixture = Fixture::new(
        "Record::struct{original:int;} mapper::(names:[]string){names[0]=\"renamed\";} main::()->int{record:Record; using,map(mapper) record; renamed=42; return record.original;}",
        "",
    );
    execute42(&fixture.build());
}
