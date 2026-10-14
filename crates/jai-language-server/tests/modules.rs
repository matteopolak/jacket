//! Module folders found without a `jai.toml`: `modules/` in the folders above the root file up to
//! the workspace folder, `jai.toml`'s `import_path`, and the paths a build metaprogram adds.
use jai_language_server::{DiagnosticCode, DocumentUri, Environment, Limits, Session};
use std::path::{Path, PathBuf};

fn environment() -> Environment {
    let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    Environment {
        fs: std::rc::Rc::new(jaic::sema::NativeFs),
        options: Box::new(move |main| {
            let mut options = jaic::sema::Options::host();
            let dir = main.parent().map(PathBuf::from).unwrap_or_default();
            options.import_paths = vec![dir.join("modules"), stdlib.clone()];
            options.preload = Some(stdlib.join(jaic::PRELOAD_ENTRY));
            options
        }),
    }
}

fn project(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("jailsp-modules-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for (path, text) in files {
        let path = dir.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    dir.canonicalize().unwrap()
}

fn uri_of(path: &Path) -> DocumentUri {
    DocumentUri::parse(&format!("file://{}", path.display())).unwrap()
}

/// The type-checker errors of `file` when the workspace folder is `folder`.
fn errors(folder: &Path, file: &str) -> Vec<String> {
    let mut s = Session::with_environment(Limits::default(), environment());
    s.set_workspace_folders(vec![folder.to_path_buf()]);
    let path = folder.join(file);
    let uri = uri_of(&path);
    s.open(uri.clone(), 1, std::fs::read_to_string(&path).unwrap())
        .unwrap();
    s.diagnostics(&uri)
        .unwrap()
        .into_iter()
        .filter(|d| d.code == DiagnosticCode::Check)
        .map(|d| d.message)
        .collect()
}

const APP: &str =
    "#import \"Basic\";\n#import \"Tools\";\nmain :: () { print(\"%\\n\", tool()); }\n";

const TOOLS: &str = "tool :: () -> int { return 7; }\n";

#[test]
fn modules_folder_of_an_ancestor_is_searched() {
    let dir = project(
        "ancestor",
        &[
            ("modules/Tools/module.jai", TOOLS),
            ("server/app/main.jai", APP),
        ],
    );
    assert_eq!(errors(&dir, "server/app/main.jai"), Vec::<String>::new());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn folders_above_the_workspace_are_not_searched() {
    let dir = project(
        "outside",
        &[
            ("modules/Tools/module.jai", TOOLS),
            ("project/main.jai", APP),
        ],
    );
    let found = errors(&dir.join("project"), "main.jai");
    assert!(
        found.iter().any(|e| e.contains("Tools")),
        "an import outside the workspace stays unresolved: {found:?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn jai_toml_import_path_is_searched() {
    let dir = project(
        "toml",
        &[
            ("jai.toml", "import_path = [\"vendor/mods\"]\n"),
            ("vendor/mods/Tools.jai", TOOLS),
            ("src/main.jai", APP),
        ],
    );
    assert_eq!(errors(&dir, "src/main.jai"), Vec::<String>::new());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn import_path_added_by_a_build_metaprogram_is_searched() {
    let meta = "#run {\n    options := get_build_options();\n    import_path: [..] string;\n    \
                array_add(*import_path, \"third_party\");\n    array_add(*import_path, ..options.import_path);\n    \
                options.import_path = import_path;\n}\n";
    let dir = project(
        "meta",
        &[
            ("first.jai", meta),
            ("third_party/Tools.jai", TOOLS),
            ("src/main.jai", APP),
        ],
    );
    assert_eq!(errors(&dir, "src/main.jai"), Vec::<String>::new());
    let _ = std::fs::remove_dir_all(&dir);
}
