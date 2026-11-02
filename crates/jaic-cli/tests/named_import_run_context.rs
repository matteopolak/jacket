//! Building the context loads every named import, so a later named import can be loaded inside
//! an earlier one's `#run` argument. A failure while loading it is reported as itself, not as a
//! null context trapping at the earlier `#run`'s first allocation.
use std::process::Command;
use test_support::TempDir;

fn check(module: &str, main: &str) -> (bool, String) {
    let dir = TempDir::new("named-import-run-context");
    std::fs::create_dir_all(dir.join("modules/Generated")).unwrap();
    std::fs::write(dir.join("modules/Generated/module.jai"), module).unwrap();
    std::fs::write(dir.join("main.jai"), main).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
        .arg("-check")
        .arg(dir.join("main.jai"))
        .arg("-import_dir")
        .arg(dir.join("modules"))
        .output()
        .unwrap();
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

const MAIN: &str = r#"#import "Basic";
A :: #import "Generated"(NAME = #run sprint("good"));
B :: #import "Generated"(NAME = #run sprint("bad"));
main :: () {}
"#;

#[test]
fn a_failing_import_reports_its_own_error() {
    let (ok, stderr) = check(
        "#module_parameters(NAME: string);\n#import \"Compiler\";\n#insert #run make();\nmake :: () -> string { if NAME == \"bad\" compiler_report(\"rejected bad\"); return \"\"; }\n",
        MAIN,
    );
    assert!(!ok);
    assert!(stderr.contains("rejected bad"), "{stderr}");
    assert!(!stderr.contains("null pointer"), "{stderr}");
}
