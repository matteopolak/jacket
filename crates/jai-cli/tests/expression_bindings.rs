//! Written initializer order survives source discovery, #run, and native lowering.
#![cfg(any(target_os = "linux", target_os = "macos"))]
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let directory = std::env::temp_dir().join(format!(
            "jai-cli-expression-bindings-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        fs::write(directory.join("main.jai"), source).unwrap();
        Self(directory)
    }
    fn build(&self, optimization: &str) -> PathBuf {
        let artifact = self.0.join(format!("program{optimization}"));
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
            .arg(optimization)
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
            panic!("authored expression binding fixture timed out");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn promoted_initializers_execute_once_in_written_order_in_run_and_native_code() {
    let fixture = Fixture::new(
        r#"
Owner :: struct { struct { x,y:int; } middle:int; }
tick :: (state:*int, n:int)->int { state.* = state.* * 10+n; return n; }
evaluate :: ()->int {
    state := 0;
    value:Owner = .{x=tick(*state,1), middle=tick(*state,2), y=tick(*state,3)};
    if state!=123 || value.x!=1 || value.middle!=2 || value.y!=3 return 1;
    return 42;
}
COMPUTED :: #run evaluate();
main :: ()->int { if COMPUTED!=42 return 2; return evaluate(); }
"#,
    );
    for optimization in ["-O0", "-O2"] {
        execute42(&fixture.build(optimization));
    }
}

#[test]
fn unselected_promoted_initializers_remain_lazy_in_run_and_native_code() {
    let fixture = Fixture::new(
        r#"
Owner :: struct { struct { x,y:int; } }
tick :: (state:*int)->int { state.* += 1; return 21; }
evaluate :: ()->int {
    state := 0;
    value:Owner = ifx false then Owner.{x=tick(*state), y=tick(*state)} else Owner.{x=20,y=22};
    if state!=0 return 1;
    return value.x+value.y;
}
COMPUTED :: #run evaluate();
main :: ()->int { if COMPUTED!=42 return 2; return evaluate(); }
"#,
    );
    for optimization in ["-O0", "-O2"] {
        execute42(&fixture.build(optimization));
    }
}
