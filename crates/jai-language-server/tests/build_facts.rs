//! Project discovery follows what a build metaprogram did when it ran, not a scan of its
//! text: computed paths, files added in a loop, and import paths built with `tprint` or kept
//! in a variable are all found, and a metaprogram that fails later leaves the last good facts.
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
    let dir = std::env::temp_dir().join(format!("jailsp-facts-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for (path, text) in files {
        let path = dir.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    dir.canonicalize().unwrap()
}

fn open(s: &mut Session, dir: &Path, file: &str) -> (DocumentUri, String) {
    let path = dir.join(file);
    let text = std::fs::read_to_string(&path).unwrap();
    let uri = DocumentUri::from_path(&path).unwrap();
    s.open(uri.clone(), 1, text.clone()).unwrap();
    (uri, text)
}

fn session(dir: &Path) -> Session {
    let mut s = Session::with_environment(Limits::default(), environment());
    s.set_workspace_folders(vec![dir.to_path_buf()]);
    s
}

fn at(text: &str, marker: &str, offset: usize) -> Position {
    let byte = text.find(marker).unwrap_or_else(|| panic!("no {marker:?}")) + offset;
    let prefix = &text[..byte];
    Position {
        line: prefix.matches('\n').count() as u32,
        character: prefix.rsplit('\n').next().unwrap().len() as u32,
    }
}

fn check_errors(s: &Session, uri: &DocumentUri) -> Vec<String> {
    s.diagnostics(uri)
        .unwrap()
        .into_iter()
        .filter(|d| d.code == DiagnosticCode::Check)
        .map(|d| d.message)
        .collect()
}

const HEAD: &str = "#import \"Basic\";\n#import \"Compiler\";\n#run build();\nbuild :: () {\n    \
    set_build_options_dc(.{do_output = false});\n    w := compiler_create_workspace(\"app\");\n    \
    options := get_build_options(w);\n    options.output_type = .NO_OUTPUT;\n";

const TAIL: &str = "    set_build_options(options, w);\n{ADD}}\n";

fn build_with(options: &str, add: &str) -> String {
    format!("{HEAD}{options}{}", TAIL.replace("{ADD}", add))
}

const TOML: &str = "build_files = [\"build.jai\"]\n";

const MAIN: &str = "#import \"Basic\";\nhelper :: (x: int) -> int { return x * 2; }\n\
    main :: () {\n    answer := helper(21);\n    print(\"%\\n\", answer);\n}\n";

/// `main.jai` is checked, hovered and navigated as part of the program the build file builds.
fn works(s: &Session, uri: &DocumentUri, text: &str, what: &str) {
    assert_eq!(check_errors(s, uri), Vec::<String>::new(), "{what}");
    let hover = s
        .hover(uri, at(text, "answer :=", 0))
        .unwrap()
        .unwrap_or_else(|| panic!("{what}: no hover"))
        .contents
        .value;
    assert!(hover.contains("answer: s64"), "{what}: {hover}");
    let def = s.definition(uri, at(text, "helper(21)", 0)).unwrap();
    assert!(format!("{def:?}").contains("main.jai"), "{what}: {def:?}");
}

#[test]
fn path_built_with_tprint_and_filepath() {
    let build = build_with(
        "",
        "    add_build_file(tprint(\"%/src/main.jai\", #filepath), w);\n",
    );
    let dir = project(
        "tprint",
        &[
            ("jai.toml", TOML),
            ("build.jai", &build),
            ("src/main.jai", MAIN),
        ],
    );
    let mut s = session(&dir);
    let (uri, text) = open(&mut s, &dir, "src/main.jai");
    works(&s, &uri, &text, "tprint");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn path_kept_in_a_variable() {
    let build = build_with(
        "",
        "    entry := \"src\";\n    full := tprint(\"%/main.jai\", entry);\n    add_build_file(full, w);\n",
    );
    let dir = project(
        "variable",
        &[
            ("jai.toml", TOML),
            ("build.jai", &build),
            ("src/main.jai", MAIN),
        ],
    );
    let mut s = session(&dir);
    let (uri, text) = open(&mut s, &dir, "src/main.jai");
    works(&s, &uri, &text, "variable");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn files_added_in_a_loop() {
    let build = build_with(
        "",
        "    names := string.[\"main\", \"extra\"];\n    for names add_build_file(tprint(\"src/%.jai\", it), w);\n",
    );
    let main = MAIN.replace(
        "answer := helper(21);",
        "answer := helper(21) + extra_value;",
    );
    let dir = project(
        "loop",
        &[
            ("jai.toml", TOML),
            ("build.jai", &build),
            ("src/main.jai", &main),
            ("src/extra.jai", "extra_value :: 1;\n"),
        ],
    );
    let mut s = session(&dir);
    let (uri, text) = open(&mut s, &dir, "src/main.jai");
    works(&s, &uri, &text, "loop");
    // The other added file belongs to the same program.
    let (extra, extra_text) = open(&mut s, &dir, "src/extra.jai");
    assert_eq!(check_errors(&s, &extra), Vec::<String>::new());
    let hover = s
        .hover(&extra, at(&extra_text, "extra_value", 0))
        .unwrap()
        .expect("hover");
    assert!(hover.contents.value.contains("extra_value"), "{hover:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

const TOOLS_MAIN: &str = "#import \"Basic\";\n#import \"Tools\";\nmain :: () {\n    \
    answer := tool_value + 1;\n    print(\"%\\n\", answer);\n}\n";

fn import_path_options(extra: &str) -> String {
    format!(
        "{extra}    dirs: [..] string;\n    array_add(*dirs, ..options.import_path);\n    \
         array_add(*dirs, vendor);\n    options.import_path = dirs;\n"
    )
}

fn import_path_project(name: &str, vendor: &str) -> (PathBuf, String) {
    let options = import_path_options(vendor);
    let build = build_with(
        &options,
        "    add_build_file(tprint(\"%/src/main.jai\", #filepath), w);\n",
    );
    let dir = project(
        name,
        &[
            ("jai.toml", TOML),
            ("build.jai", &build),
            ("vendor/Tools.jai", "tool_value :: 41;\n"),
            ("src/main.jai", TOOLS_MAIN),
        ],
    );
    (dir, build)
}

#[test]
fn import_path_added_with_tprint_resolves_imports() {
    let (dir, _) = import_path_project(
        "path-tprint",
        "    vendor := tprint(\"%vendor\", #filepath);\n",
    );
    let mut s = session(&dir);
    let (uri, text) = open(&mut s, &dir, "src/main.jai");
    assert_eq!(check_errors(&s, &uri), Vec::<String>::new());
    let hover = s
        .hover(&uri, at(&text, "answer :=", 0))
        .unwrap()
        .expect("hover")
        .contents
        .value;
    assert!(hover.contains("answer: s64"), "{hover}");
    // Goto on the module string finds the file in the computed folder.
    let found = s.definition(&uri, at(&text, "\"Tools\"", 2)).unwrap();
    assert!(
        found.len() == 1 && found[0].uri.ends_with("/vendor/Tools.jai"),
        "{found:?}"
    );
    // So does completion of module names.
    let labels: Vec<String> = s
        .completion(&uri, at(&text, "\"Tools\"", 2))
        .unwrap()
        .items
        .into_iter()
        .map(|i| i.label)
        .collect();
    assert!(labels.iter().any(|l| l == "Tools"), "{labels:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn import_path_kept_in_a_variable_resolves_imports() {
    let (dir, _) = import_path_project(
        "path-variable",
        "    base := #filepath;\n    folder := \"vendor\";\n    vendor := tprint(\"%%\", base, folder);\n",
    );
    let mut s = session(&dir);
    let (uri, text) = open(&mut s, &dir, "src/main.jai");
    assert_eq!(check_errors(&s, &uri), Vec::<String>::new());
    let found = s.definition(&uri, at(&text, "\"Tools\"", 2)).unwrap();
    assert!(
        found.len() == 1 && found[0].uri.ends_with("/vendor/Tools.jai"),
        "{found:?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_metaprogram_that_fails_later_keeps_the_last_good_facts() {
    let (dir, build) =
        import_path_project("keeps", "    vendor := tprint(\"%vendor\", #filepath);\n");
    let mut s = session(&dir);
    let (uri, text) = open(&mut s, &dir, "src/main.jai");
    assert_eq!(check_errors(&s, &uri), Vec::<String>::new());
    // The build file now fails before it does anything (a file that does not exist).
    let broken = build.replace(
        "add_build_file(tprint(\"%/src/main.jai\", #filepath), w);",
        "add_build_file(\"src/missing.jai\", w);",
    );
    let build_uri = DocumentUri::from_path(&dir.join("build.jai")).unwrap();
    s.open(build_uri, 2, broken).unwrap();
    // The folder the last good run added is still searched for module names.
    let labels: Vec<String> = s
        .completion(&uri, at(&text, "\"Tools\"", 2))
        .unwrap()
        .items
        .into_iter()
        .map(|i| i.label)
        .collect();
    assert!(labels.iter().any(|l| l == "Tools"), "{labels:?}");
    let found = s.definition(&uri, at(&text, "\"Tools\"", 2)).unwrap();
    assert!(
        found.len() == 1 && found[0].uri.ends_with("/vendor/Tools.jai"),
        "{found:?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// Running the metaprogram must not touch the disk: its writes stay in the sandbox.
#[test]
fn the_run_cannot_write_files() {
    let dir_name = format!("jailsp-facts-sandbox-{}", std::process::id());
    let target = std::env::temp_dir().join(&dir_name).join("leaked.txt");
    let write = format!(
        "    write_entire_file(\"{}\", \"leak\");\n",
        target.display()
    );
    let build = build_with(
        "",
        &format!("{write}    add_build_file(tprint(\"%/src/main.jai\", #filepath), w);\n"),
    )
    .replace(
        "#import \"Compiler\";",
        "#import \"Compiler\";\n#import \"File\";",
    );
    let dir = project(
        "sandbox",
        &[
            ("jai.toml", TOML),
            ("build.jai", &build),
            ("src/main.jai", MAIN),
        ],
    );
    let mut s = session(&dir);
    let (uri, text) = open(&mut s, &dir, "src/main.jai");
    works(&s, &uri, &text, "sandbox");
    assert!(!target.exists(), "the metaprogram wrote {target:?}");
    let _ = std::fs::remove_dir_all(&dir);
}
