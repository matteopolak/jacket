//! Standalone syntax evidence must not imply imports were resolved or code executed.
use std::{fs, path::PathBuf, process::Command};

struct Fixture(PathBuf);
impl Fixture {
    fn new(name: &str, bytes: &[u8]) -> Self {
        let path =
            std::env::temp_dir().join(format!("jai-cli-parse-{}-{name}.jai", std::process::id()));
        fs::write(&path, bytes).unwrap();
        Self(path)
    }
    fn invoke(&self, action: &str) -> std::process::Output {
        Command::new(env!("CARGO_BIN_EXE_jai-rs"))
            .arg(action)
            .arg(&self.0)
            .output()
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
#[test]
fn syntax_imports_and_scopes_pass_without_loading_or_checking_modules() {
    let source = Fixture::new(
        "imports",
        b"#scope_file; using Basic :: #import \"Missing_Module_For_Syntax_Test\"; #load \"missing_file_for_syntax_test.jai\"; #scope_export; answer :: 42;",
    );
    let parsed = source.invoke("parse");
    assert!(
        parsed.status.success(),
        "{}",
        String::from_utf8_lossy(&parsed.stderr)
    );
    assert!(String::from_utf8_lossy(&parsed.stdout).contains("syntax stage only"));
    assert!(!source.invoke("check").status.success());
}
#[test]
fn malformed_syntax_has_file_and_line_diagnostic() {
    let source = Fixture::new("malformed", b"#scope_file;\nanswer :: ;\n");
    let parsed = source.invoke("parse");
    assert!(!parsed.status.success());
    let diagnostic = String::from_utf8_lossy(&parsed.stderr);
    assert!(
        diagnostic.contains(&format!("{}:2:", source.0.display())),
        "{diagnostic}"
    );
    assert!(diagnostic.contains("expected expression"), "{diagnostic}");
}
#[test]
fn binary_is_rejected_by_shared_decoder_before_parsing() {
    let source = Fixture::new("binary", b"\x7fELF\0\0\0\0");
    let parsed = source.invoke("parse");
    assert!(!parsed.status.success());
    assert!(String::from_utf8_lossy(&parsed.stderr).contains("binary"));
}
#[test]
fn legacy_comment_bytes_use_shared_decoder() {
    let source = Fixture::new("legacy-comment", b"// comment \xff\xfe\nanswer :: 42;");
    let parsed = source.invoke("parse");
    assert!(
        parsed.status.success(),
        "{}",
        String::from_utf8_lossy(&parsed.stderr)
    );
}
#[test]
fn invalid_utf8_code_is_rejected() {
    let source = Fixture::new("invalid-utf8", b"answer :: \xff;");
    let parsed = source.invoke("parse");
    assert!(!parsed.status.success());
    assert!(String::from_utf8_lossy(&parsed.stderr).contains("invalid UTF-8 outside a comment"));
}
