//! Compiler-only syntax is distinct from runtime storage and native publication.
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "jai-cli-source-boundaries-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        fs::write(path.join("main.jai"), source).unwrap();
        Self(path)
    }

    fn run(&self, action: &str) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_jai-rs"));
        for (key, _) in std::env::vars_os() {
            if key.to_string_lossy().starts_with("JAI_RS_") {
                command.env_remove(key);
            }
        }
        command
            .env("JAI_RS_PRELOAD", "off")
            .env("JAI_RS_RUNTIME_SUPPORT", "off")
            .arg(action)
            .arg(self.0.join("main.jai"));
        if action == "emit-llvm" {
            command.arg(self.0.join("output.ll"));
        }
        command.output().unwrap()
    }

    fn rejected(&self, site: &str, message: &str) {
        let artifact = self.0.join("output.ll");
        fs::write(&artifact, b"prior output remains intact").unwrap();
        for action in ["check", "emit-llvm"] {
            let output = self.run(action);
            let diagnostic = String::from_utf8_lossy(&output.stderr);
            assert_eq!(output.status.code(), Some(1), "{action}: {diagnostic}");
            assert!(
                diagnostic.contains(&format!("main.jai:{site}: error: {message}")),
                "{action}: {diagnostic}"
            );
            assert_eq!(fs::read(&artifact).unwrap(), b"prior output remains intact");
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn unused_deferred_quote_does_not_demand_its_unbound_body() {
    let fixture = Fixture::new(include_str!(
        "../../../tests/fixtures/source-boundaries/code-deferred-quote.jai"
    ));
    let output = fixture.run("check");
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).starts_with("checked "));
}

#[test]
fn code_cannot_enter_runtime_local_storage() {
    Fixture::new(include_str!(
        "../../../tests/fixtures/source-boundaries/code-runtime-local.jai"
    ))
    .rejected("3:5", "IR value has no runtime representation");
}

#[test]
fn null_code_cannot_supply_inserted_syntax() {
    Fixture::new(include_str!(
        "../../../tests/fixtures/source-boundaries/code-null-insertion.jai"
    ))
    .rejected(
        "3:5",
        "#insert requires captured syntax; #code,null has no body or scope",
    );
}

#[test]
fn expression_insertion_rejects_a_quoted_statement_at_the_use_site() {
    Fixture::new(include_str!(
        "../../../tests/fixtures/source-boundaries/code-statement-expression.jai"
    ))
    .rejected(
        "3:12",
        "expression insertion requires exactly one quoted expression",
    );
}
