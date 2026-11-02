//! Embed the complete theme tree, including tests and assets, without another build dependency.
// Build scripts read Cargo's environment before jacket-config is available.
#![allow(clippy::disallowed_methods)]
use std::path::Path;

fn collect(root: &Path, dir: &Path, files: &mut Vec<String>) {
    for entry in std::fs::read_dir(dir).expect("read doc theme") {
        let path = entry.expect("theme entry").path();
        // Editor and OS droppings (`.DS_Store`, `.vscode/`) are not part of the theme.
        if path
            .file_name()
            .is_some_and(|n| n.to_string_lossy().starts_with('.'))
        {
            continue;
        }
        if path.is_dir() {
            collect(root, &path, files);
        } else {
            files.push(
                path.strip_prefix(root)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .replace('\\', "/"),
            );
        }
    }
}

fn main() {
    let root = std::env::current_dir().unwrap().join("src/doc/site");
    println!("cargo:rerun-if-changed={}", root.display());
    let mut files = Vec::new();
    collect(&root, &root, &mut files);
    files.sort();
    let mut table = String::from("pub const FILES: &[(&str, &[u8])] = &[\n");
    for file in files {
        table.push_str(&format!(
            "({file:?}, include_bytes!({:?})),\n",
            root.join(&file).display().to_string()
        ));
    }
    table.push_str("];\n");
    std::fs::write(
        Path::new(&std::env::var_os("OUT_DIR").unwrap()).join("doc_theme.rs"),
        table,
    )
    .expect("write embedded doc theme");
}
