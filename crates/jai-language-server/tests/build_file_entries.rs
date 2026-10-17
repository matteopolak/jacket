//! A `jai.toml` that lists the project's `build.jai` among `build_files` must not change how
//! `src/main.jai` (the program that metaprogram builds) is checked and answered.
use jai_language_server::{DiagnosticCode, DocumentUri, Environment, Limits, Position, Session};
use std::path::{Path, PathBuf};

fn environment() -> Environment {
    let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    Environment {
        fs: std::rc::Rc::new(jaic::sema::NativeFs),
        options: Box::new(move |_| {
            let mut options = jaic::sema::Options::host();
            options.import_paths = vec![stdlib.clone()];
            options.preload = Some(stdlib.join(jaic::PRELOAD_ENTRY));
            options
        }),
    }
}

fn project(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("jailsp-buildfile-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for (path, text) in files {
        let path = dir.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    dir.canonicalize().unwrap()
}

fn open(folder: &Path, file: &str) -> (Session, DocumentUri, String) {
    let mut s = Session::with_environment(Limits::default(), environment());
    s.set_workspace_folders(vec![folder.to_path_buf()]);
    let path = folder.join(file);
    let uri = DocumentUri::parse(&format!("file://{}", path.display())).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    s.open(uri.clone(), 1, text.clone()).unwrap();
    (s, uri, text)
}

fn position(text: &str, marker: &str) -> Position {
    let byte = text.find(marker).unwrap();
    let prefix = &text[..byte];
    Position {
        line: prefix.matches('\n').count() as u32,
        character: prefix.rsplit('\n').next().unwrap().len() as u32,
    }
}

const BUILD: &str = "#import \"Basic\";\n#import \"Compiler\";\n#run build();\nbuild :: () {\n    \
    set_build_options_dc(.{do_output = false});\n    w := compiler_create_workspace(\"app\");\n    \
    options := get_build_options(w);\n    options.output_type = .NO_OUTPUT;\n    \
    set_build_options(options, w);\n    add_build_file(\"src/main.jai\", w);\n}\n";

const MAIN: &str = "#import \"Basic\";\nhelper :: (x: int) -> int { return x * 2; }\n\
    main :: () {\n    answer := helper(21);\n    print(\"%\\n\", answer);\n}\n";

fn check(name: &str, toml: Option<&str>) {
    check_with(name, toml, "build.jai", BUILD);
}

fn check_with(name: &str, toml: Option<&str>, build_path: &str, build: &str) {
    check_via(name, toml, build_path, build, &|dir| {
        Some(dir.to_path_buf())
    });
}

/// `via` gives the folder the editor opens the project by (a symlink to it, a case variant);
/// `None` skips the check where the platform cannot do that.
fn check_via(
    name: &str,
    toml: Option<&str>,
    build_path: &str,
    build: &str,
    via: &dyn Fn(&Path) -> Option<PathBuf>,
) {
    let mut files = vec![(build_path, build), ("src/main.jai", MAIN)];
    if let Some(toml) = toml {
        files.push(("jai.toml", toml));
    }
    let dir = project(name, &files);
    let Some(editor_dir) = via(&dir) else {
        let _ = std::fs::remove_dir_all(&dir);
        return;
    };
    let (s, uri, text) = open(&editor_dir, "src/main.jai");
    let errors: Vec<_> = s
        .diagnostics(&uri)
        .unwrap()
        .into_iter()
        .filter(|d| d.code == DiagnosticCode::Check)
        .map(|d| d.message)
        .collect();
    assert_eq!(errors, Vec::<String>::new(), "{name}");
    let hover = s
        .hover(&uri, position(&text, "answer :="))
        .unwrap()
        .unwrap_or_else(|| panic!("{name}: no hover"))
        .contents
        .value;
    assert!(hover.contains("answer: s64"), "{name}: {hover}");
    let def = s
        .definition(&uri, position(&text, "helper(21)"))
        .unwrap_or_else(|e| panic!("{name}: {e:?}"));
    assert!(format!("{def:?}").contains("main.jai"), "{name}: {def:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn without_a_toml() {
    check("none", None);
}

#[test]
fn toml_without_the_build_file() {
    check("main-only", Some("build_files = [\"src/main.jai\"]\n"));
}

#[test]
fn toml_with_only_the_build_file() {
    check("build-only", Some("build_files = [\"build.jai\"]\n"));
}

#[test]
fn toml_with_the_build_file_and_main() {
    check(
        "both",
        Some("build_files = [\"build.jai\", \"src/main.jai\"]\n"),
    );
}

#[test]
fn toml_with_main_then_the_build_file() {
    check(
        "both-reversed",
        Some("build_files = [\"src/main.jai\", \"build.jai\"]\n"),
    );
}

/// The path the metaprogram adds is computed, so no text scan can follow it to `src/main.jai`.
#[test]
fn computed_main_path_with_the_build_file_listed() {
    let build = BUILD.replace(
        "add_build_file(\"src/main.jai\", w);",
        "add_build_file(tprint(\"%/src/main.jai\", #filepath), w);",
    );
    for (name, toml) in [
        ("computed-none", None),
        ("computed-build", Some("build_files = [\"build.jai\"]\n")),
        (
            "computed-both",
            Some("build_files = [\"build.jai\", \"src/main.jai\"]\n"),
        ),
    ] {
        check_with(name, toml, "build.jai", &build);
    }
}

/// The build file lives in a folder of its own and reaches `src/main.jai` with `../`.
#[test]
fn build_file_in_a_subfolder() {
    let build = BUILD.replace("\"src/main.jai\"", "\"../src/main.jai\"");
    for (name, toml) in [
        ("sub-none", None),
        ("sub-build", Some("build_files = [\"tools/build.jai\"]\n")),
    ] {
        check_with(name, toml, "tools/build.jai", &build);
    }
}

const BUILD_TOML: Option<&str> = Some("build_files = [\"build.jai\"]\n");

/// Every spelling of the one `src/main.jai` the build file may add.
#[test]
fn spellings_of_the_added_path() {
    for (name, spelled) in [
        ("dot", "./src/main.jai"),
        ("double-slash", "src//main.jai"),
        ("dot-dot", "src/../src/main.jai"),
        ("up-and-back", "../{DIR}/src/main.jai"),
    ] {
        for toml in [None, BUILD_TOML] {
            // `{DIR}` is the project folder's own name.
            let leaf = format!("jailsp-buildfile-{name}-{}", std::process::id());
            let path = spelled.replace("{DIR}", &leaf);
            let build = BUILD.replace("\"src/main.jai\"", &format!("\"{path}\""));
            check_with(name, toml, "build.jai", &build);
        }
    }
}

#[cfg(unix)]
#[test]
fn project_opened_through_a_symlink() {
    for toml in [None, BUILD_TOML] {
        check_via("symlink", toml, "build.jai", BUILD, &|dir| {
            let leaf = dir.file_name().unwrap().to_string_lossy().into_owned();
            let link = dir.with_file_name(format!("{leaf}-link"));
            let _ = std::fs::remove_file(&link);
            std::os::unix::fs::symlink(dir, &link).ok()?;
            Some(link)
        });
    }
}

/// The editor reports the folder in another case where the disk ignores case.
#[test]
fn project_opened_with_another_case() {
    for toml in [None, BUILD_TOML] {
        check_via("case", toml, "build.jai", BUILD, &|dir| {
            let upper = PathBuf::from(dir.to_string_lossy().to_uppercase());
            // Only where the disk treats both spellings as one folder.
            (upper != *dir && upper.is_dir()).then_some(upper)
        });
    }
}

/// An absolute path with `.` and a doubled slash in it.
#[test]
fn computed_untidy_absolute_path() {
    let build = BUILD.replace(
        "add_build_file(\"src/main.jai\", w);",
        "path := tprint(\"%/./src//main.jai\", #filepath);\n    add_build_file(path, w);",
    );
    check_with("abs-odd", BUILD_TOML, "build.jai", &build);
}
