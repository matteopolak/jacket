//! `jacket doc` exports compiler-checked documentation facts for the matching Astro builder.
//! `jaic -doc_model` supplies a separately versioned raw model; the semantic exporter owns
//! checked routes, references, example assembly and publication facts. Astro owns HTML and UI.
//! See `docs/tools/jacket-doc.md` and `docs/compiler/doc-site-export.md`.
mod builder;
mod categories;
mod declaration_format;
mod example;
mod example_check;
mod export;
mod guides;
mod highlight;
mod markdown;
mod metadata;
mod model;
mod navigation;
mod since;
mod site;
mod slug;
mod type_links;

use crate::cli::DocArgs;
use crate::{test::stdlib::stdlib_dir, tools};
use jacket_config::schema::Doc;
use model::Module;
use site::Site;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

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
        .filter(|n| !n.starts_with('.') && !matches!(n.as_str(), "internal" | "tests"))
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

/// `-import_dir root`, so a library's modules find each other; nothing for the standard library.
fn import_dir(stdlib: &Path, root: &Path) -> Vec<std::ffi::OsString> {
    if root == stdlib {
        Vec::new()
    } else {
        vec!["-import_dir".into(), root.as_os_str().to_owned()]
    }
}

/// `jaic -doc_model` for each of `ids` (module folders below `root`), a few at a time.
fn read_models(
    jaic: &Path,
    stdlib: &Path,
    root: &Path,
    ids: &[String],
) -> Result<Vec<Module>, String> {
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
                        .env(jacket_config::env::STDLIB.name, stdlib)
                        .args(import_dir(stdlib, root))
                        .arg("-doc_model")
                        .arg(root.join(id))
                        .output();
                    let result = match output {
                        Ok(o) if o.status.success() => {
                            model::parse(id, &String::from_utf8_lossy(&o.stdout), root, stdlib).map(
                                |mut module| {
                                    module.wasm = builds_for_wasm(jaic, stdlib, root, &module, i);
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
fn builds_for_wasm(jaic: &Path, stdlib: &Path, root: &Path, module: &Module, n: usize) -> bool {
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
    let program = format!("#import \"{}\";\nmain :: () {{}}\n", module.import_name());
    let ok = std::fs::write(&file, program).is_ok()
        && Command::new(jaic)
            .env(jacket_config::env::STDLIB.name, stdlib)
            .args(["-check", "-no_dce", "-os", "wasm"])
            .args(import_dir(stdlib, root))
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

/// Where `dir` is in its git checkout, with a trailing slash (`stdlib/`), the checkout's commit,
/// and the files below `dir` that differ from it. Nothing when `dir` is not in a checkout, or
/// `git` is missing.
struct GitState {
    prefix: String,
    commit: String,
    dirty: std::collections::BTreeSet<String>,
}

fn git_state(dir: &Path) -> Option<GitState> {
    let git = |args: &[&str]| -> Option<String> {
        let out = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .ok()?;
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
    };
    let prefix = git(&["rev-parse", "--show-prefix"])?.trim().to_owned();
    let commit = git(&["rev-parse", "HEAD"])?.trim().to_owned();
    let status = git(&[
        "status",
        "--porcelain",
        "-z",
        "--untracked-files=all",
        "--",
        ".",
    ])?;
    let mut dirty = std::collections::BTreeSet::new();
    let mut entries = status.split('\0').filter(|e| e.len() > 3);
    while let Some(entry) = entries.next() {
        // Paths are relative to the checkout's top, whatever folder git ran in.
        let path = &entry[3..];
        dirty.insert(
            path.strip_prefix(prefix.as_str())
                .unwrap_or(path)
                .to_owned(),
        );
        // A rename lists the old name next.
        if entry.starts_with('R') || entry.as_bytes().get(1) == Some(&b'R') {
            entries.next();
        }
    }
    Some(GitState {
        prefix,
        commit,
        dirty,
    })
}

fn base_path(text: &str) -> Result<String, String> {
    if !text.starts_with('/') || !text.ends_with('/') {
        return Err(format!(
            "--base-url must start and end with a slash, as in /docs/jacket/ (got `{text}`)"
        ));
    }
    Ok(text.to_owned())
}

/// What a run documents: the folder its modules' ids are relative to, and their ids.
struct Library {
    /// The folder holding the modules.
    root: PathBuf,
    ids: Vec<String>,
}

/// The library at `path`: a module (its folder has a `module.jai`), with the modules below it,
/// or a folder of modules.
fn library(path: &Path) -> Result<Library, String> {
    let mut ids = Vec::new();
    let root = if path.join("module.jai").is_file() {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| format!("{} has no folder name", path.display()))?
            .to_owned();
        ids.push(name.clone());
        discover(path, &name, &mut ids);
        path.parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| format!("{} has no parent folder", path.display()))?
    } else {
        discover(path, "", &mut ids);
        path.to_path_buf()
    };
    if ids.is_empty() {
        return Err(format!(
            "no modules in {}: a module is a folder with a module.jai",
            path.display()
        ));
    }
    Ok(Library {
        root,
        ids,
    })
}

fn canonical(path: &Path) -> Result<PathBuf, String> {
    path.canonicalize()
        .map_err(|e| format!("cannot read {}: {e}", path.display()))
}

/// The folder to document and the `[doc]` settings for it, with the folder of their
/// `jacket.toml`. Without a path, the `[doc]` of the nearest `jacket.toml` names the library,
/// else it is the bundled standard library. With one, the nearest `jacket.toml` above it
/// applies only when its `[doc]` is for that folder. Both selected roots share canonical
/// identity: `project/child/..` with `path = "lib"` still categorizes `lib/Ext` relatively.
fn target(
    path: Option<&Path>,
    stdlib: &Path,
    cwd: &Path,
) -> Result<(PathBuf, Doc, PathBuf), String> {
    let start = match path {
        Some(path) => canonical(&cwd.join(path))?,
        None => cwd.to_path_buf(),
    };
    let from = if start.is_dir() {
        start.clone()
    } else {
        start
            .parent()
            .map_or_else(|| start.clone(), Path::to_path_buf)
    };
    let loaded = jacket_config::load(&jacket_config::Disk, &from).map_err(|e| e.to_string())?;
    for note in loaded.iter().flat_map(|l| &l.notes) {
        eprintln!("{note}");
    }
    let found = loaded.filter(|l| !l.config.doc.is_empty()).map(|l| {
        let dir = l.dir().to_path_buf();
        let lib = dir.join(l.config.doc.path.as_deref().unwrap_or("."));
        (lib, l.config.doc, dir)
    });
    match (path, found) {
        (None, Some((lib, doc, dir))) => Ok((canonical(&lib)?, doc, canonical(&dir)?)),
        (None, None) => Ok((stdlib.to_path_buf(), Doc::default(), stdlib.to_path_buf())),
        (Some(_), Some((lib, doc, dir))) if lib.canonicalize().ok().as_ref() == Some(&start) => {
            Ok((start, doc, canonical(&dir)?))
        }
        (Some(_), _) => Ok((start.clone(), Doc::default(), start)),
    }
}

/// The logo file's name below `assets/` and its bytes.
fn read_logo(file: &Path) -> Result<(String, Vec<u8>), String> {
    let ext = file
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    let ext = match ext.as_deref() {
        Some(ext @ ("svg" | "png")) => ext.to_owned(),
        _ => return Err(format!("logo {} is not an .svg or .png", file.display())),
    };
    let bytes =
        std::fs::read(file).map_err(|e| format!("cannot read the logo {}: {e}", file.display()))?;
    Ok((format!("logo.{ext}"), bytes))
}

/// Partial module builds omit external pages, so only a full site can validate its links.
/// Prints intra-doc links that lead nowhere and doc-comment links that name nothing, and returns
/// them all. The second kind are errors unless `lenient`.
fn report_broken_links(site: &Site, lenient: bool) -> Vec<String> {
    let mut broken = site.broken_links();
    for link in &broken {
        eprintln!("jacket: warning: intra-doc link to `{link}` leads nowhere");
    }
    let severity = if lenient {
        "warning"
    } else {
        "error"
    };
    for module in &site.modules {
        for link in &module.unresolved {
            eprintln!("jacket: {severity}: {link} names nothing");
            broken.push(link.clone());
        }
    }
    broken
}

fn link_status(args: &DocArgs, broken: &[String], unlinked: &[String], filtered: bool) -> ExitCode {
    if !unlinked.is_empty() && !filtered {
        eprintln!("jacket: error: {} unlinked type name(s)", unlinked.len());
        return ExitCode::from(1);
    }
    if !broken.is_empty() && !args.allow_broken_links && !filtered {
        eprintln!(
            "jacket: error: {} broken intra-doc link(s); fix the doc comment or pass --allow-broken-links",
            broken.len()
        );
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}

/// Runs every example with a Run button the way the browser would; false when one fails.
fn test_examples(
    args: &DocArgs,
    site: &site::Site,
    jaic: &Path,
    stdlib: &Path,
    root: &Path,
) -> Result<bool, String> {
    let programs = site.runnable();
    // With the browser compiler at hand the examples run in it, under Node; otherwise
    // in jaic's interpreter, built for the same target.
    let node = PathBuf::from("node");
    let has_node = Command::new(&node)
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success());
    if args.runtime.is_some() && !has_node {
        eprintln!(
            "jacket: warning: `node` not found; testing the examples in the interpreter, not the browser compiler"
        );
    }
    let engine = if let Some(runtime) = args.runtime.as_deref()
        && root == stdlib
        && has_node
    {
        example_check::Engine::Browser {
            node,
            runtime,
        }
    } else {
        example_check::Engine::Interpreter {
            jaic,
            stdlib,
            import_dir: import_dir(stdlib, root),
        }
    };
    let failed = example_check::run(&engine, &programs)?;
    if !failed.is_empty() {
        eprint!("{}", example_check::report(&failed));
        eprintln!(
            "jacket: error: {} of {} examples failed; mark one that cannot run in the browser (`skip_on(wasm)`, `no_run`, `ignore`), or pass --no-test-examples",
            failed.len(),
            programs.len()
        );
        return Ok(false);
    }
    Ok(true)
}

fn build(args: &DocArgs, cwd: &Path) -> Result<ExitCode, String> {
    let total_start = std::time::Instant::now();
    let mut accounted = std::time::Duration::ZERO;
    let result = (|| {
        if args.theme.is_some() {
            return Err("--theme used Jai source overrides, which were retired; customize website/docs Astro components or select matching builder sources with JACKET_DOCS_BUILDER (docs/tools/astro-docs.md)".into());
        }
        if args.emit_input.is_some() {
            return Err("--emit-input used the retired renderer format; use --export DIR to retain the versioned semantic bundle, or --export-only --out DIR without rendering".into());
        }
        let builder = if args.export_only {
            None
        } else {
            Some(builder::Builder::locate()?)
        };
        let base = base_path(&args.base_url)?;
        let jaic = tools::locate(tools::JAIC)?;
        let stdlib = stdlib_dir(&jaic, cwd)?;
        let (lib, doc, config_dir) = target(args.path.as_deref(), &stdlib, cwd)?;
        if doc.theme.is_some() {
            return Err("[doc] theme used Jai source overrides, which were retired; customize matching Astro builder sources with JACKET_DOCS_BUILDER (docs/tools/astro-docs.md)".into());
        }
        let Library {
            root,
            mut ids,
        } = library(&lib)?;
        // Module folders as the patterns of `jacket.toml` see them.
        let folders: Vec<String> = ids
            .iter()
            .map(|id| {
                let folder = root.join(id);
                jacket_config::slashed(folder.strip_prefix(&config_dir).unwrap_or(&folder))
            })
            .collect();
        let layout =
            categories::layout(&doc.category, &doc.group, &ids, &folders).map_err(|e| {
                format!(
                    "[doc] in {}:\n  {e}",
                    config_dir.join("jacket.toml").display()
                )
            })?;
        let place: BTreeMap<String, (Option<usize>, String)> =
            ids.iter().cloned().zip(layout.of).collect();
        let filtered = !args.modules.is_empty();
        if filtered {
            if let Some(unknown) = args.modules.iter().find(|m| !ids.contains(m)) {
                return Err(format!("no module `{unknown}` in {}", root.display()));
            }
            ids.retain(|id| args.modules.contains(id));
        }
        let models_start = std::time::Instant::now();
        let mut modules = read_models(&jaic, &stdlib, &root, &ids)?;
        accounted += timing(args, "models", models_start);
        for module in &mut modules {
            if let Some((category, group)) = place.get(&module.id) {
                module.category = *category;
                module.group.clone_from(group);
            }
        }
        if let Some(runtime) = &args.runtime {
            for file in ["jai_wasm.wasm", "engine.mjs"] {
                if !runtime.join(file).is_file() {
                    return Err(format!("--runtime: {} has no {file}", runtime.display()));
                }
            }
        }
        let version = args.version.clone().unwrap_or_default();
        if let Some(since) = &doc.since
            && !version.is_empty()
        {
            let file = config_dir.join(since);
            let text = std::fs::read_to_string(&file)
                .map_err(|e| format!("cannot read {}: {e}", file.display()))?;
            since::assign(&text, &mut modules, &version);
        }
        let pick = |flag: &Option<String>, setting: &Option<String>| {
            flag.clone()
                .or_else(|| setting.clone())
                .unwrap_or_default()
                .trim_end_matches('/')
                .to_owned()
        };
        let mut site = Site::new(
            modules,
            layout.groups,
            base,
            version.clone(),
            args.source_ref.clone().unwrap_or_default(),
            pick(&args.playground_url, &doc.playground),
            args.runtime.is_some(),
        );
        site.categories = layout.categories;
        site.name = args
            .name
            .clone()
            .or_else(|| doc.name.clone())
            .unwrap_or_else(|| {
                lib.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default()
            });
        site.breadcrumb = doc.breadcrumb.clone().unwrap_or_else(|| {
            lib.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default()
        });
        site.description = doc
            .description
            .clone()
            .unwrap_or_else(|| format!("API documentation for {}.", site.name));
        site.repo = pick(&args.repo, &doc.repo);
        let logo = match (&args.logo, &doc.logo) {
            (Some(file), _) => Some(read_logo(&cwd.join(file))?),
            (None, Some(file)) => Some(read_logo(&config_dir.join(file))?),
            (None, None) => None,
        };
        site.logo = logo.as_ref().map(|(name, _)| name.clone());
        if !doc.page.is_empty() {
            site.pages_title = doc.pages_title.clone().unwrap_or_else(|| "Guides".into());
        }
        let prefix = git_state(&config_dir).map(|g| g.prefix).unwrap_or_default();
        let extra = guides::collect(&site, &config_dir, &prefix, &doc)?;
        site.pages_href = extra.first_href(&site);
        site.extra_nav.clone_from(&extra.nav);
        if let Some(state) = git_state(&root) {
            site.source_prefix = state.prefix;
            // A given ref wins; else the checkout's commit, for the files it matches.
            if args.source_ref.is_none() {
                site.commit = Some(state.commit);
                site.dirty = state.dirty;
            }
        }

        {
            let input = builder::Input::new(if args.export_only {
                Some(cwd.join(args.export.as_deref().unwrap_or(&args.out)))
            } else {
                args.export.as_ref().map(|path| cwd.join(path))
            })?;
            let publication = export::publication::prepare(args, &doc, &config_dir, cwd, &site)?;
            let destination = &input.path;
            let mut assets = Vec::new();
            if let Some((name, bytes)) = &logo {
                assets.push(export::InputAsset {
                    path: format!("assets/{name}"),
                    bytes: bytes.clone(),
                    provenance: export::contract::AssetProvenance::ConfiguredLogo,
                });
            }
            if let Some(runtime) = &args.runtime {
                for file in ["engine.mjs", "jai_wasm.wasm"] {
                    assets.push(export::InputAsset {
                        path: format!("runtime/{file}"),
                        bytes: std::fs::read(runtime.join(file))
                            .map_err(|e| format!("read runtime {file}: {e}"))?,
                        provenance: export::contract::AssetProvenance::BrowserRuntime,
                    });
                }
            }
            let source = export::source_state(
                &site,
                &doc,
                &publication,
                &assets,
                &config_dir,
                &root,
                &stdlib,
            )?;
            export::write(export::Request {
                site: &site,
                guides: &extra,
                publication: &publication,
                out: destination,
                compiler: &jaic,
                source_root: &root,
                stdlib: &stdlib,
                config: &config_dir,
                partial: filtered,
                allow_broken_links: args.allow_broken_links,
                assets,
                source,
            })?;
            let unlinked = site.unlinked_types();
            for name in &unlinked {
                eprintln!("jacket: warning: unlinked type in {name}");
            }
            let broken = report_broken_links(&site, args.allow_broken_links || filtered);
            for warning in indented_code_warnings(&site.modules) {
                eprintln!("jacket: warning: {warning}");
            }
            if args.validate_examples && !test_examples(args, &site, &jaic, &stdlib, &root)? {
                return Ok(ExitCode::from(1));
            }
            if args.export_only {
                eprintln!(
                    "jacket doc: {} modules exported to {}",
                    site.modules.len(),
                    destination.display()
                );
            } else {
                let started = std::time::Instant::now();
                let status = builder
                    .as_ref()
                    .ok_or("missing Astro builder preflight")?
                    .run(destination, &cwd.join(&args.out), args.check)?;
                accounted += timing(args, "Astro validation/build/check", started);
                if status != ExitCode::SUCCESS {
                    return Ok(status);
                }
                eprintln!(
                    "jacket doc: {} modules{}",
                    site.modules.len(),
                    if args.check {
                        ", production site checked".into()
                    } else {
                        format!(", written to {}", cwd.join(&args.out).display())
                    }
                );
            }
            Ok(link_status(args, &broken, &unlinked, filtered))
        }
    })();
    if args.timings {
        eprintln!(
            "jacket doc timings: rest: {:.3}s",
            total_start
                .elapsed()
                .saturating_sub(accounted)
                .as_secs_f64()
        );
        eprintln!(
            "jacket doc timings: total: {:.3}s",
            total_start.elapsed().as_secs_f64()
        );
    }
    result
}

fn timing(args: &DocArgs, label: &str, start: std::time::Instant) -> std::time::Duration {
    let elapsed = start.elapsed();
    if args.timings {
        eprintln!("jacket doc timings: {label}: {:.3}s", elapsed.as_secs_f64());
    }
    elapsed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn import_context_reaches_native_wasm_probe_and_preserves_status() {
        use std::os::unix::fs::PermissionsExt;
        let temp = test_support::TempDir::new("doc-wasm-import-context");
        let stdlib = temp.join("stdlib");
        let compiler = temp.join("compiler");
        std::fs::write(&compiler, "#!/bin/sh\nset -eu\nfixture=$(dirname \"$0\")\nprintf '%s\\n' \"$@\" > \"$fixture/args\"\nprintf '%s' \"$JAIC_STDLIB\" > \"$fixture/stdlib-env\"\nfor input do :; done\ncp \"$input\" \"$fixture/input.jai\"\nexit \"$(cat \"$fixture/status\")\"\n").unwrap();
        std::fs::set_permissions(&compiler, std::fs::Permissions::from_mode(0o755)).unwrap();
        for (n, (root, id, expected)) in [
            (stdlib.clone(), "Extensions/Widget", "Extensions/Widget"),
            (stdlib.join("Extensions"), "Widget", "Extensions/Widget"),
            (temp.join("project"), "Acme/Widget", "Acme/Widget"),
        ]
        .into_iter()
        .enumerate()
        {
            let directory = root.join(id);
            std::fs::create_dir_all(&directory).unwrap();
            std::fs::write(directory.join("module.jai"), "value :: 1;\n").unwrap();
            let raw = jaic::doc_comments::model_json(&directory, &stdlib);
            let module = model::parse(id, &raw, &root, &stdlib).unwrap();
            for status in [0, 7] {
                std::fs::write(temp.join("status"), status.to_string()).unwrap();
                assert_eq!(
                    builds_for_wasm(&compiler, &stdlib, &root, &module, n),
                    status == 0
                );
                assert_eq!(
                    std::fs::read_to_string(temp.join("input.jai")).unwrap(),
                    format!("#import \"{expected}\";\nmain :: () {{}}\n"),
                    "{id}: native probe import"
                );
                assert_eq!(
                    std::fs::read_to_string(temp.join("stdlib-env")).unwrap(),
                    stdlib.to_str().unwrap()
                );
                let args = std::fs::read_to_string(temp.join("args")).unwrap();
                let args = args.lines().collect::<Vec<_>>();
                assert_eq!(&args[..4], ["-check", "-no_dce", "-os", "wasm"]);
                if root == stdlib {
                    assert_eq!(args.len(), 5);
                } else {
                    assert_eq!(args.len(), 7);
                    assert_eq!(&args[4..6], ["-import_dir", root.to_str().unwrap()]);
                }
                assert!(
                    !Path::new(args.last().unwrap()).exists(),
                    "probe input must be removed after success or failure"
                );
            }
        }
    }

    #[test]
    fn retired_renderer_options_explain_migration_before_extracting_a_library() {
        use clap::Parser;
        for (flag, guidance) in [("--theme", "Astro"), ("--emit-input", "--export")] {
            let cli = crate::cli::Cli::try_parse_from([
                "jacket",
                "doc",
                "/definitely-missing-doc-fixture",
                flag,
                "/unused-old-renderer-input",
            ])
            .unwrap();
            let crate::cli::Command::Doc(args) = cli.command else {
                panic!("doc command")
            };
            let message = build(&args, Path::new("/")).unwrap_err();
            assert!(message.contains(guidance), "{flag}: {message}");
        }
    }

    #[test]
    fn configured_library_and_directory_share_canonical_identity() {
        let temp = test_support::TempDir::new("doc-config-identity");
        std::fs::create_dir_all(temp.join("lib/Ext")).unwrap();
        std::fs::create_dir_all(temp.join("child")).unwrap();
        std::fs::write(temp.join("jacket.toml"), "[doc]\npath = \"lib\"\n").unwrap();
        std::fs::write(temp.join("guide.md"), "# Config-relative guide\n").unwrap();
        let cwd = temp.join("child/..");
        for path in [None, Some(Path::new("lib"))] {
            let (library, _, directory) = target(path, &temp.join("unused-stdlib"), &cwd).unwrap();
            assert_eq!(directory, temp.canonicalize().unwrap());
            assert_eq!(library.strip_prefix(&directory).unwrap(), Path::new("lib"));
            assert_eq!(
                library.join("Ext").strip_prefix(&directory).unwrap(),
                Path::new("lib/Ext")
            );
            assert_eq!(
                std::fs::read_to_string(directory.join("guide.md")).unwrap(),
                "# Config-relative guide\n"
            );
        }
    }

    #[test]
    fn the_base_path_needs_both_slashes() {
        assert_eq!(base_path("/docs/jacket/").unwrap(), "/docs/jacket/");
        assert!(base_path("/docs/jacket").is_err());
        assert!(base_path("docs/").is_err());
    }
}
