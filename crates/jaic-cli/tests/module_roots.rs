//! `-package_import_dir`: folders that only one package's files search. A dependency's own
//! `import-path` finds the dependency's modules for its files and for nobody else, so it can't
//! replace `Basic` (or any other module) for the rest of the program.
use std::path::Path;
use std::process::{Command, Output};
use test_support::TempDir;

const JAIC: &str = env!("CARGO_BIN_EXE_jaic");

fn write(root: &Path, files: &[(&str, &str)]) {
    for (path, text) in files {
        let full = root.join(path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(full, text).unwrap();
    }
}

/// `main.jai` imports `dep` and `other`, which are both submodules under `modules/`. `dep`
/// carries its own `vendor/` folder with a `Probe` module and a `Basic` that is not the library's.
fn project(main: &str) -> TempDir {
    let root = TempDir::new("package-roots");
    write(
        &root,
        &[
            ("main.jai", main),
            (
                "modules/dep/module.jai",
                "#import \"Probe\";\n#import \"Basic\";\ndep_value :: () -> s64 { return probe() + not_basic(); }\n",
            ),
            (
                "modules/dep/src/inner.jai",
                "#import \"Probe\";\ninner_value :: () -> s64 { return probe(); }\n",
            ),
            (
                "modules/dep/vendor/Probe/module.jai",
                "probe :: () -> s64 { return 7; }\n",
            ),
            (
                "modules/dep/vendor/Basic/module.jai",
                "not_basic :: () -> s64 { return 1; }\n",
            ),
            (
                "modules/other/module.jai",
                "#import \"Basic\";\nother_value :: () { print(\"other\\n\"); }\n",
            ),
        ],
    );
    root
}

fn check(root: &Path, scoped: bool) -> Output {
    let mut command = Command::new(JAIC);
    command.arg(root.join("main.jai")).arg("-check");
    if scoped {
        command.arg("-package_import_dir").arg(format!(
            "{}={}",
            root.join("modules/dep").display(),
            root.join("modules/dep/vendor").display()
        ));
    }
    command.output().unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

const USES_BOTH: &str = "#import \"Basic\";\n#import \"dep\";\n#import \"other\";\nmain :: () {\n    print(\"%\\n\", dep_value());\n    other_value();\n}\n";

#[test]
fn a_package_finds_its_own_modules_and_the_program_keeps_the_library() {
    let root = project(USES_BOTH);
    let output = check(&root, true);
    assert!(output.status.success(), "{}", stderr(&output));
}

#[test]
fn without_the_flag_the_package_has_no_extra_folders() {
    let root = project(USES_BOTH);
    let output = check(&root, false);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("Probe"), "{}", stderr(&output));
}

#[test]
fn the_package_folders_do_not_reach_the_program() {
    let root = project(
        "#import \"Basic\";\n#import \"Probe\";\nmain :: () {\n    print(\"%\\n\", probe());\n}\n",
    );
    let output = check(&root, true);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("Probe"), "{}", stderr(&output));
}

#[test]
fn a_package_basic_does_not_replace_the_library_one_for_other_files() {
    // `other` and the program use `print`, which only the library's `Basic` has.
    let root = project(USES_BOTH);
    let output = check(&root, true);
    assert!(output.status.success(), "{}", stderr(&output));
    let leaked =
        project("#import \"Basic\";\nmain :: () {\n    print(\"%\\n\", not_basic());\n}\n");
    let output = check(&leaked, true);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("not_basic"), "{}", stderr(&output));
}

#[test]
fn files_in_subfolders_of_the_package_share_its_folders() {
    let root = project(USES_BOTH);
    write(
        &root,
        &[(
            "modules/dep/module.jai",
            "#import \"Probe\";\n#import \"Basic\";\n#load \"src/inner.jai\";\ndep_value :: () -> s64 { return probe() + not_basic() + inner_value(); }\n",
        )],
    );
    let output = check(&root, true);
    assert!(output.status.success(), "{}", stderr(&output));
}
