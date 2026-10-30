//! `jacket doc`: the standard library's API documentation as a static website. See
//! `docs/tools/jacket-doc.md`.
//!
//! `jaic -doc_model` reads each library with the compiler's own parser and prints what its pages
//! say as JSON; this module turns that into HTML, a search index and the files the pages load
//! (style, script, the example programs, and optionally the browser compiler that runs them).
mod categories;
mod example;
mod guide;
mod highlight;
mod links;
mod markdown;
mod model;
mod pages;
mod search;
mod shell;
mod since;
mod site;

use crate::cli::DocArgs;
use crate::{test::stdlib::stdlib_dir, tools};
use model::{Kind, Module};
use site::Site;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

const STYLE: &str = include_str!("assets/site.css");
const SCRIPT: &str = include_str!("assets/site.js");
const WORKER: &str = include_str!("assets/run-worker.js");

/// Present in a folder `jacket doc` wrote, so the next run may replace it.
const MARKER: &str = ".jacket-doc";

pub fn run(args: &DocArgs, cwd: &Path) -> ExitCode {
    match build(args, cwd) {
        Ok(code) => code,
        Err(message) => {
            eprintln!("jacket: error: {message}");
            ExitCode::from(2)
        }
    }
}

/// The libraries below `dir`: folders with a `module.jai`, without the library's own internals.
fn discover(dir: &Path, prefix: &str, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut names: Vec<String> = entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| !n.starts_with('.') && !matches!(n.as_str(), "internal" | "prelude" | "tests"))
        .collect();
    names.sort();
    for name in names {
        let id = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}/{name}")
        };
        if dir.join(&name).join("module.jai").is_file() {
            out.push(id.clone());
        }
        discover(&dir.join(&name), &id, out);
    }
}

/// `jaic -doc_model` for each of `ids`, a few at a time.
fn read_models(jaic: &Path, stdlib: &Path, ids: &[String]) -> Result<Vec<Module>, String> {
    let next = AtomicUsize::new(0);
    let found: Mutex<Vec<Result<Module, String>>> = Mutex::new(Vec::new());
    let threads = std::thread::available_parallelism().map_or(2, |n| n.get().min(4));
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| {
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    let Some(id) = ids.get(i) else {
                        break;
                    };
                    let output = Command::new(jaic)
                        .env("JAIC_STDLIB", stdlib)
                        .arg("-doc_model")
                        .arg(stdlib.join(id))
                        .output();
                    let result = match output {
                        Ok(o) if o.status.success() => {
                            model::parse(id, &String::from_utf8_lossy(&o.stdout), stdlib).map(
                                |mut module| {
                                    module.wasm = builds_for_wasm(jaic, stdlib, &module, i);
                                    module
                                },
                            )
                        }
                        Ok(o) => Err(format!(
                            "`jaic -doc_model` failed for {id}: {}",
                            String::from_utf8_lossy(&o.stderr).trim()
                        )),
                        Err(e) => Err(format!("could not start `{}`: {e}", jaic.display())),
                    };
                    found
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .push(result);
                }
            });
        }
    });
    let mut modules = Vec::new();
    for result in found
        .into_inner()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
    {
        modules.push(result?);
    }
    modules.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(modules)
}

/// Whether `module` can be imported when the target is wasm, and most of its items exist there.
/// Found by checking a program that imports it, so a module that asserts another system, or
/// leaves everything to other systems, is out. `n` keeps the scratch files of threads apart.
fn builds_for_wasm(jaic: &Path, stdlib: &Path, module: &Module, n: usize) -> bool {
    let asserted_elsewhere =
        !module.platforms.is_empty() && !module.platforms.iter().any(|p| p == "WASM");
    let there = module
        .items
        .iter()
        .filter(|i| i.platforms.is_empty() || i.platforms.iter().any(|p| p == "WASM"))
        .count();
    let somewhere = there > 0 && there * 2 >= module.items.len();
    if asserted_elsewhere || !somewhere {
        return false;
    }
    let dir = std::env::temp_dir().join(format!("jacket-doc-wasm-{}", std::process::id()));
    if std::fs::create_dir_all(&dir).is_err() {
        return false;
    }
    let file = dir.join(format!("{n}.jai"));
    let program = format!("#import \"{}\";\nmain :: () {{}}\n", module.id);
    let ok = std::fs::write(&file, program).is_ok()
        && Command::new(jaic)
            .env("JAIC_STDLIB", stdlib)
            .args(["-check", "-no_dce", "-os", "wasm"])
            .arg(&file)
            .output()
            .is_ok_and(|o| o.status.success());
    let _ = std::fs::remove_file(&file);
    ok
}

/// A warning for each indented code block in a doc comment of `modules`, with where to find it.
fn indented_code_warnings(modules: &[Module]) -> Vec<String> {
    let mut out = Vec::new();
    let mut note = |what: String, file: &str, line: usize, docs: &str| {
        for first in markdown::indented_blocks(docs) {
            out.push(format!(
                "{what}: indented code block in the doc comment at {file}:{line} (starts `{first}`); fence it with ```jai, or ```text for output"
            ));
        }
    };
    for module in modules {
        note(
            module.id.clone(),
            &module.file,
            module.docs_line.max(1),
            &module.docs,
        );
        for item in &module.items {
            let what = format!("{}.{}", module.id, item.name);
            note(what.clone(), &item.file, item.line, &item.docs);
            for o in &item.overloads {
                note(what.clone(), &o.file, o.docs_line.max(o.line), &o.docs);
            }
            for member in &item.members {
                note(
                    format!("{what}.{}", member.name),
                    &item.file,
                    item.line,
                    &member.doc,
                );
            }
        }
    }
    out
}

/// The commit of the checkout `stdlib` is the `stdlib/` folder of, and the files below it that
/// differ from it. Nothing when it is not such a checkout, or `git` is missing.
fn git_state(stdlib: &Path) -> (Option<String>, std::collections::BTreeSet<String>) {
    let git = |args: &[&str]| -> Option<String> {
        let out = Command::new("git")
            .arg("-C")
            .arg(stdlib)
            .args(args)
            .output()
            .ok()?;
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
    };
    let none = (None, std::collections::BTreeSet::new());
    let Some(prefix) = git(&["rev-parse", "--show-prefix"]) else {
        return none;
    };
    if prefix.trim() != "stdlib/" {
        return none;
    }
    let Some(commit) = git(&["rev-parse", "HEAD"]) else {
        return none;
    };
    let Some(status) = git(&[
        "status",
        "--porcelain",
        "-z",
        "--untracked-files=all",
        "--",
        ".",
    ]) else {
        return none;
    };
    let mut dirty = std::collections::BTreeSet::new();
    let mut entries = status.split('\0').filter(|e| e.len() > 3);
    while let Some(entry) = entries.next() {
        let path = &entry[3..];
        dirty.insert(path.trim_start_matches("stdlib/").to_owned());
        // A rename lists the old name next.
        if entry.starts_with('R') || entry.as_bytes().get(1) == Some(&b'R') {
            entries.next();
        }
    }
    (Some(commit.trim().to_owned()), dirty)
}

fn base_path(text: &str) -> Result<String, String> {
    if !text.starts_with('/') || !text.ends_with('/') {
        return Err(format!(
            "--base-url must start and end with a slash, as in /docs/jai/ (got `{text}`)"
        ));
    }
    Ok(text.to_owned())
}

fn build(args: &DocArgs, cwd: &Path) -> Result<ExitCode, String> {
    let base = base_path(&args.base_url)?;
    let jaic = tools::locate(tools::JAIC)?;
    let stdlib = stdlib_dir(&jaic, cwd)?;
    let root = stdlib
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "the standard library has no parent folder".to_owned())?;
    let mut ids = Vec::new();
    discover(&stdlib, "", &mut ids);
    let categories = categories::Categories::load(&stdlib, &ids)?;
    let filtered = !args.modules.is_empty();
    if filtered {
        if let Some(unknown) = args.modules.iter().find(|m| !ids.contains(m)) {
            return Err(format!("no module `{unknown}` in {}", stdlib.display()));
        }
        ids.retain(|id| args.modules.contains(id));
    }
    let mut modules = read_models(&jaic, &stdlib, &ids)?;
    for module in &mut modules {
        module.category = categories.of(&module.id).to_owned();
    }
    since::assign(&mut modules, env!("CARGO_PKG_VERSION"));
    if let Some(runtime) = &args.runtime {
        for file in ["jai_wasm.wasm", "engine.mjs"] {
            if !runtime.join(file).is_file() {
                return Err(format!("--runtime: {} has no {file}", runtime.display()));
            }
        }
    }
    let version = env!("CARGO_PKG_VERSION").to_owned();
    let mut site = Site::new(
        modules,
        categories.order.clone(),
        base,
        version.clone(),
        args.source_ref
            .clone()
            .unwrap_or_else(|| format!("v{version}")),
        args.playground_url.trim_end_matches('/').to_owned(),
        args.runtime.is_some(),
    );

    if args.source_ref.is_none() {
        (site.commit, site.dirty) = git_state(&stdlib);
    }

    let mut files: Vec<(String, Vec<u8>)> = Vec::new();
    let mut add = |path: String, text: String| files.push((path, text.into_bytes()));
    add("index.html".into(), pages::index_page(&site));
    let mut items = 0;
    for (mi, module) in site.modules.iter().enumerate() {
        add(Site::module_path(&module.id), pages::module_page(&site, mi));
        for item in &module.items {
            items += 1;
            add(
                Site::item_path(&module.id, &item.name),
                pages::item_page(&site, mi, item),
            );
        }
    }
    for (path, html) in guide::pages(&site, &root)? {
        add(path, html);
    }
    add("search-index.json".into(), search::index(&site));
    add("assets/site.css".into(), STYLE.to_owned());
    add("assets/site.js".into(), SCRIPT.to_owned());
    add("assets/run-worker.js".into(), WORKER.to_owned());
    add(
        "assets/logo.svg".into(),
        include_str!("../../../../editors/vscode/images/icon.svg").to_owned(),
    );
    let examples = site.examples();
    for (id, program) in &examples {
        add(format!("examples/{id}.jai"), program.clone());
    }
    if site.runtime {
        add(
            "runtime.json".into(),
            "{\"engine\":\"runtime/engine.mjs\",\"wasm\":\"runtime/jai_wasm.wasm\"}\n".into(),
        );
    }

    let mut problems = site.path_clashes();
    problems.extend(links::check(&files, &site.base, !filtered));
    for problem in &problems {
        eprintln!("jacket: error: {problem}");
    }
    if !problems.is_empty() {
        return Ok(ExitCode::from(1));
    }
    let broken = site.broken_links();
    for link in &broken {
        eprintln!("jacket: warning: intra-doc link to `{link}` leads nowhere");
    }
    let unfenced = indented_code_warnings(&site.modules);
    for warning in &unfenced {
        eprintln!("jacket: warning: {warning}");
    }
    let procedures = site
        .modules
        .iter()
        .flat_map(|m| &m.items)
        .filter(|i| i.kind == Kind::Procedure)
        .count();
    let total: usize = files.iter().map(|(_, bytes)| bytes.len()).sum();
    if !args.check {
        write_site(&args.out, &files, args.runtime.as_deref())?;
    }
    eprintln!(
        "jacket doc: {} modules, {items} items ({procedures} procedures), {} example programs, {} files, {} KiB{}",
        site.modules.len(),
        examples.len(),
        files.len(),
        total / 1024,
        if args.check {
            String::new()
        } else {
            format!(", written to {}", args.out.display())
        }
    );
    if !broken.is_empty() && !args.allow_broken_links && !filtered {
        eprintln!(
            "jacket: error: {} broken intra-doc link(s); fix the doc comment or pass --allow-broken-links",
            broken.len()
        );
        return Ok(ExitCode::from(1));
    }
    Ok(ExitCode::SUCCESS)
}

/// Replaces `out` with `files`, and copies the browser compiler in when there is one.
fn write_site(
    out: &Path,
    files: &[(String, Vec<u8>)],
    runtime: Option<&Path>,
) -> Result<(), String> {
    if out.exists() {
        let is_ours = out.join(MARKER).is_file()
            || std::fs::read_dir(out).is_ok_and(|mut entries| entries.next().is_none());
        if !is_ours {
            return Err(format!(
                "{} exists and `jacket doc` did not make it; remove it or choose another --out",
                out.display()
            ));
        }
        std::fs::remove_dir_all(out).map_err(|e| format!("cannot clear {}: {e}", out.display()))?;
    }
    let write = |path: PathBuf, bytes: &[u8]| -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
        }
        std::fs::write(&path, bytes).map_err(|e| format!("cannot write {}: {e}", path.display()))
    };
    write(out.join(MARKER), b"")?;
    for (path, bytes) in files {
        write(out.join(path), bytes)?;
    }
    if let Some(runtime) = runtime {
        for file in ["jai_wasm.wasm", "engine.mjs"] {
            let bytes = std::fs::read(runtime.join(file))
                .map_err(|e| format!("cannot read {}: {e}", runtime.join(file).display()))?;
            write(out.join("runtime").join(file), &bytes)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_base_path_needs_both_slashes() {
        assert_eq!(base_path("/docs/jai/").unwrap(), "/docs/jai/");
        assert!(base_path("/docs/jai").is_err());
        assert!(base_path("docs/").is_err());
    }
}
