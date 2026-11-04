//! Native test builds include the lines their compile-time generators ran.
#[path = "support/installed.rs"]
mod installed;
use installed::{archive, jacket, scratch};

#[test]
fn unit_test_coverage_includes_compile_time_generators() {
    if test_support::toolchain::optional("llvm-dwarfdump").is_none() {
        return;
    }
    let root = scratch("compile-coverage");
    let Some(exe) = archive(&root) else {
        return;
    };
    let project = root.join("project");
    std::fs::create_dir_all(project.join("tests")).unwrap();
    std::fs::write(
        project.join("module.jai"),
        "compute :: () -> int { return 42; }\n",
    )
    .unwrap();
    std::fs::write(
        project.join("tests/check.jai"),
        "#import \"Extensions/Testing\";\n#import \"project\";\nVALUE :: #run compute();\ncheck :: () { expect(VALUE == 42); } @test\n",
    )
    .unwrap();
    let out = jacket(
        &exe,
        &project,
        &["test", "--coverage", "--coverage-out", "coverage.lcov"],
    );
    assert!(
        out.status.success(),
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let report = std::fs::read_to_string(project.join("coverage.lcov")).unwrap();
    let own = report
        .split("end_of_record")
        .find(|record| record.contains("/module.jai\n"))
        .unwrap();
    assert!(
        own.lines()
            .any(|line| line.starts_with("DA:1,") && line != "DA:1,0"),
        "{own}"
    );
}
