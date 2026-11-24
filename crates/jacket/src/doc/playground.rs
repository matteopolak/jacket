//! The playground page of the generated site, and the browser-compiler bundle it runs on.
use super::site::Site;
use super::view::{Content, NavCategory, PageHead, PageView, Sidebar};
use std::path::Path;

/// The playground page's file.
pub const PATH: &str = "playground.html";

/// What a playground needs of a bundle, as `tools/browser/build_scripting_wasm.py` stages it.
const REQUIRED: [&str; 4] = [
    "jai_wasm.wasm",
    "engine.mjs",
    "jaifmt-playground.jai",
    "examples.json",
];

/// How to get a bundle, for the page and for the error.
pub const BUILD_HINT: &str = "python3 tools/browser/build_scripting_wasm.py --release --output target/doc-runtime\njacket doc --runtime target/doc-runtime";

/// The files of `dir`, a bundle, as `(path below runtime/, bytes)`: the four it must have and
/// every file `examples.json` lists.
pub fn bundle(dir: &Path) -> Result<Vec<(String, Vec<u8>)>, String> {
    let read = |name: &str| {
        std::fs::read(dir.join(name)).map_err(|e| {
            format!(
                "--runtime: cannot read {} ({e}); build the bundle with\n  {}",
                dir.join(name).display(),
                BUILD_HINT.replace('\n', "\n  ")
            )
        })
    };
    let mut files = Vec::new();
    for name in REQUIRED {
        files.push((name.to_owned(), read(name)?));
    }
    let index = std::str::from_utf8(&files[3].1)
        .ok()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(text).ok())
        .ok_or_else(|| {
            format!(
                "--runtime: {} is not valid JSON",
                dir.join("examples.json").display()
            )
        })?;
    let listed = |entry: &serde_json::Value, key: &str| -> Vec<String> {
        entry[key]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|v| v.as_str().map(str::to_owned))
            .collect()
    };
    for entry in index["examples"].as_array().into_iter().flatten() {
        let name = entry["name"].as_str().unwrap_or_default();
        for file in listed(entry, "files")
            .into_iter()
            .chain(listed(entry, "assets"))
        {
            let path = format!("{name}/{file}");
            if !safe(&path) {
                return Err(format!(
                    "--runtime: examples.json lists an unsafe path `{path}`"
                ));
            }
            let bytes = read(&path)?;
            files.push((path, bytes));
        }
    }
    Ok(files)
}

/// A relative path that stays inside the folder it is joined to.
fn safe(path: &str) -> bool {
    !path.is_empty()
        && !path.contains(['\\', ':'])
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
}

/// `runtime.json`: where the page finds the files of the bundle.
pub fn runtime_json() -> String {
    serde_json::json!({
        "engine": "runtime/engine.mjs",
        "wasm": "runtime/jai_wasm.wasm",
        "formatter": "runtime/jaifmt-playground.jai",
        "examples": "runtime/examples.json",
    })
    .to_string()
        + "\n"
}

/// The playground page, with the module list beside it.
pub fn page(site: &Site, nav: &[NavCategory]) -> PageView {
    let href = site.page_href(PATH);
    let mut nav = nav.to_vec();
    for link in nav
        .iter_mut()
        .flat_map(|c| c.groups.iter_mut())
        .flat_map(|g| g.links.iter_mut())
    {
        link.current = link.href == href;
    }
    PageView {
        head: PageHead::default(),
        not_found: false,
        title: site.title("Playground"),
        description: format!("Write, run and share {} code in the browser.", site.name),
        path: PATH.into(),
        side_label: "Modules".into(),
        sidebar: Sidebar::Modules {
            categories: nav,
        },
        outline: Vec::new(),
        content: Content::Playground {
            ready: site.runtime,
            hint: BUILD_HINT.into(),
            runtime_url: site.href("runtime.json"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use test_support::TempDir;

    fn staged(extra: &[(&str, &str)]) -> TempDir {
        let dir = TempDir::new("doc-playground");
        for name in REQUIRED {
            fs::write(dir.path().join(name), "{\"examples\":[]}").unwrap();
        }
        for (name, text) in extra {
            let path = dir.path().join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }
        dir
    }

    #[test]
    fn a_bundle_is_the_required_files_and_the_listed_examples() {
        let dir = staged(&[
            (
                "examples.json",
                r#"{"examples":[{"name":"hello","files":["main.jai"],"assets":["assets/a.png"]}]}"#,
            ),
            ("hello/main.jai", "main :: () {}"),
            ("hello/assets/a.png", "png"),
        ]);
        let names: Vec<_> = bundle(dir.path())
            .unwrap()
            .into_iter()
            .map(|f| f.0)
            .collect();
        assert_eq!(
            names,
            [
                "jai_wasm.wasm",
                "engine.mjs",
                "jaifmt-playground.jai",
                "examples.json",
                "hello/main.jai",
                "hello/assets/a.png"
            ]
        );
    }

    #[test]
    fn a_missing_file_says_how_to_build_the_bundle() {
        let dir = staged(&[]);
        fs::remove_file(dir.path().join("engine.mjs")).unwrap();
        let error = bundle(dir.path()).unwrap_err();
        assert!(error.contains("engine.mjs"), "{error}");
        assert!(error.contains("build_scripting_wasm.py"), "{error}");
    }

    #[test]
    fn an_example_cannot_name_a_file_outside_the_bundle() {
        let dir = staged(&[(
            "examples.json",
            r#"{"examples":[{"name":"x","files":["../secret"],"assets":[]}]}"#,
        )]);
        assert!(bundle(dir.path()).unwrap_err().contains("unsafe"));
    }
}
