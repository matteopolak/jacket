//! End-to-end native compilation of our own source fixtures.
use std::{fs, process::Command};
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new(files: &[(&str, &str)]) -> Self {
        let root = std::env::temp_dir().join(format!(
            "jai-unit-native-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        for (name, source) in files {
            let path = root.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, source).unwrap();
        }
        Self(root)
    }
    fn command(&self, action: &str) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_jai-rs"));
        command.arg(action).arg(self.0.join("main.jai"));
        command
    }
    fn build(&self) -> PathBuf {
        let output = self.0.join("program");
        let result = self.command("build").arg(&output).output().unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        output
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn exit_code(program: &Path) -> i32 {
    let mut child = Command::new(program).spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            return status.code().unwrap();
        }
        if Instant::now() > deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("generated fixture exceeded execution deadline");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
#[test]
fn native_loaded_procedure() {
    let root = std::env::temp_dir().join(format!("jai-native-load-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("main.jai"),
        "#load \"answer.jai\"; main :: () -> int { return answer(); }",
    )
    .unwrap();
    fs::write(
        root.join("answer.jai"),
        "answer :: () -> int { return 37; }",
    )
    .unwrap();
    let output = root.join("program");
    let result = Command::new(env!("CARGO_BIN_EXE_jai-rs"))
        .arg("build")
        .arg(root.join("main.jai"))
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(exit_code(&output), 37);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn imported_procedures_keep_defining_scopes_and_shared_instance_storage() {
    let fixture = Fixture::new(&[
        (
            "main.jai",
            r#"A :: #import,file "a.jai";
            Again :: #import,file "a.jai";
            B :: #import,file "b.jai";
            Base :: 99;
            main :: () -> int { return A.result() + Again.result() + B.result(); }"#,
        ),
        (
            "a.jai",
            r#"#scope_file
            Base :: 11;
            helper :: (n: int = Base) -> int { return n; }
            #scope_module
            count: int;
            #scope_export
            result :: () -> int { count += 1; return helper() + count; }"#,
        ),
        (
            "b.jai",
            r#"#scope_file
            Base :: 17;
            helper :: () -> int { return Base; }
            #scope_module
            count: int;
            #scope_export
            result :: () -> int { count += 1; return helper() + count; }"#,
        ),
    ]);
    assert_eq!(exit_code(&fixture.build()), 43);
}

#[test]
fn configured_named_imports_compile_and_private_members_are_rejected() {
    let fixture = Fixture::new(&[
        (
            "main.jai",
            r#"Values :: #import "Values"; main :: () -> int { return Values.answer(); }"#,
        ),
        (
            "modules/Values/module.jai",
            "#scope_file secret :: 29; #scope_export answer :: () -> int { return secret; }",
        ),
    ]);
    let output = fixture.0.join("program");
    let result = fixture
        .command("build")
        .env("JAI_RS_MODULE_PATH", fixture.0.join("modules"))
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(exit_code(&output), 29);
    fs::write(
        fixture.0.join("main.jai"),
        r#"Values :: #import "Values"; main :: () -> int { return Values.secret; }"#,
    )
    .unwrap();
    let result = fixture
        .command("check")
        .env("JAI_RS_MODULE_PATH", fixture.0.join("modules"))
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("unknown module member 'secret'"),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn library_policy_checks_bodies_without_fabricating_an_entrypoint() {
    let fixture = Fixture::new(&[("main.jai", "answer :: () -> u8 { return 27; }")]);
    assert!(
        fixture
            .command("check-library")
            .output()
            .unwrap()
            .status
            .success()
    );
    let application = fixture.command("check").output().unwrap();
    assert!(!application.status.success());
    assert!(String::from_utf8_lossy(&application.stderr).contains("application main"));
    fs::write(
        fixture.0.join("main.jai"),
        "answer :: () -> u8 { return absent; }",
    )
    .unwrap();
    let invalid = fixture.command("check-library").output().unwrap();
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("unknown name 'absent'"));
}
