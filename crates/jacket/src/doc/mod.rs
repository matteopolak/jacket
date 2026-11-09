//! `jacket doc`: the standard library's API documentation as a static website. See
//! `docs/tools/jacket-doc.md`.
//!
//! `jaic -doc_model` reads each library with the compiler's own parser and prints what its pages
//! say as JSON; this module turns that into HTML, a search index and the files the pages load
//! (style, script, the example programs, and optionally the browser compiler that runs them).
mod categories;
mod example;
mod example_check;
mod highlight;
mod markdown;
mod metadata;
mod model;
mod search;
mod since;
mod site;
mod theme;
mod type_links;
mod view;

use crate::cli::DocArgs;
use crate::{test::stdlib::stdlib_dir, tools};
use jacket_config::schema::Doc;
use model::{Kind, Module};
use site::Site;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

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
/// applies only when its `[doc]` is for that folder.
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
        (None, Some((lib, doc, dir))) => Ok((canonical(&lib)?, doc, dir)),
        (None, None) => Ok((stdlib.to_path_buf(), Doc::default(), stdlib.to_path_buf())),
        (Some(_), Some((lib, doc, dir))) if lib.canonicalize().ok().as_ref() == Some(&start) => {
            Ok((start, doc, dir))
        }
        (Some(_), _) => Ok((start.clone(), Doc::default(), start)),
    }
}

/// An image's file name below `assets/` (`<what>.svg` or `<what>.png`) and its bytes.
fn read_image(what: &str, file: &Path) -> Result<(String, Vec<u8>), String> {
    let ext = file
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    let ext = match ext.as_deref() {
        Some(ext @ ("svg" | "png")) => ext.to_owned(),
        _ => return Err(format!("{what} {} is not an .svg or .png", file.display())),
    };
    let bytes = std::fs::read(file)
        .map_err(|e| format!("cannot read the {what} {}: {e}", file.display()))?;
    Ok((format!("{what}.{ext}"), bytes))
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

/// What runs the examples: the browser compiler under Node with `--runtime`, else jaic's
/// interpreter. A missing `node`, or a library the bundle does not hold, is an error, not a
/// quiet switch to the other one.
fn engine<'a>(
    runtime: Option<&'a Path>,
    root: &Path,
    stdlib: &'a Path,
    jaic: &'a Path,
    has_node: bool,
) -> Result<example_check::Engine<'a>, String> {
    let Some(runtime) = runtime else {
        return Ok(example_check::Engine::Interpreter {
            jaic,
            stdlib,
            import_dir: import_dir(stdlib, root),
        });
    };
    if !has_node {
        return Err(
            "--runtime tests the examples in the browser compiler under Node, and `node` is not on PATH: install Node, or drop --runtime to test them in jaic's interpreter".into(),
        );
    }
    if root != stdlib {
        return Err(format!(
            "--runtime tests the examples in the browser compiler, which holds only the standard library, so it cannot run the examples of {}: drop --runtime to test them in jaic's interpreter, or pass --no-test-examples",
            root.display()
        ));
    }
    Ok(example_check::Engine::Browser {
        node: PathBuf::from("node"),
        runtime,
    })
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
    let has_node = Command::new("node")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success());
    let engine = engine(args.runtime.as_deref(), root, stdlib, jaic, has_node)?;
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
        let base = base_path(&args.base_url)?;
        let jaic = tools::locate(tools::JAIC)?;
        let stdlib = stdlib_dir(&jaic, cwd)?;
        let (lib, doc, config_dir) = target(args.path.as_deref(), &stdlib, cwd)?;
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
        let image = |what, arg: &Option<PathBuf>, key: &Option<String>| match (arg, key) {
            (Some(file), _) => read_image(what, &cwd.join(file)).map(Some),
            (None, Some(file)) => read_image(what, &config_dir.join(file)).map(Some),
            (None, None) => Ok(None),
        };
        let logo = image("logo", &args.logo, &doc.logo)?;
        let icon = image("icon", &args.icon, &doc.icon)?;
        site.logo = logo.as_ref().map(|(name, _)| name.clone());
        site.icon = icon
            .as_ref()
            .or(logo.as_ref())
            .map(|(name, _)| name.clone());
        if !doc.page.is_empty() {
            site.pages_title = doc.pages_title.clone().unwrap_or_else(|| "Guides".into());
        }
        let prefix = git_state(&config_dir).map(|g| g.prefix).unwrap_or_default();
        let extra = view::guide::collect(&site, &config_dir, &prefix, &doc)?;
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

        let overrides = args
            .theme
            .as_ref()
            .map(|p| cwd.join(p))
            .or_else(|| doc.theme.as_ref().map(|p| config_dir.join(p)));
        let theme = theme::Theme::new(overrides.as_deref())?;
        let input_dir = theme::Input::new(args.emit_input.as_deref())?;
        let view_start = std::time::Instant::now();
        let mut shared = view::site(&site);
        let mut input = Vec::new();
        let mut guides = extra.pages(&site, &shared.nav);
        use_home(&mut guides, &mut shared.index);
        let metadata::Publication {
            metadata,
            mut urls,
            image_file,
        } = metadata::prepare(args, &doc, &config_dir, cwd, &site, &mut shared)?;
        let mut items = 0;
        for mi in 0..site.modules.len() {
            let mut module = view::module(&site, mi);
            metadata.apply_module(&mut module, site.modules[mi].items.len(), &mut urls);
            items += module.items.len();
            let file = format!("modules/{mi}.json");
            input.push((file.clone(), json_bytes(&module)?));
            shared.module_files.push(file);
        }
        for (n, guide) in guides.iter_mut().enumerate() {
            metadata.apply(guide);
            urls.push(metadata.url(&guide.path, false));
            let file = format!("guides/{n}.json");
            input.push((file.clone(), json_bytes(guide)?));
            shared.guide_files.push(file);
        }
        let view_elapsed = view_start.elapsed();
        let examples = site.examples();
        let mut files = auxiliary_files(&site, logo.into_iter().chain(icon), &version, &examples);
        files.extend(metadata.files(&urls, image_file));
        shared.written_files = files.iter().map(|(path, _)| path.clone()).collect();
        if site.runtime {
            shared
                .written_files
                .extend(["runtime/engine.mjs".into(), "runtime/jai_wasm.wasm".into()]);
        }
        shared.written_files.push(MARKER.into());
        shared.assets = theme.assets();
        shared.resolve_links = !filtered;
        input.push(("site.json".into(), json_bytes(&shared)?));
        let input_start = std::time::Instant::now();
        write_site(&input_dir.path, &input, None)?;
        // Serialization and input I/O are part of the view phase's end-to-end cost.
        let view_elapsed = view_elapsed + input_start.elapsed();
        accounted += view_elapsed;
        if args.timings {
            eprintln!(
                "jacket doc timings: view: {:.3}s",
                view_elapsed.as_secs_f64()
            );
        }
        drop(input);

        let problems = site.path_clashes();
        for problem in &problems {
            eprintln!("jacket: error: {problem}");
        }
        if !problems.is_empty() {
            return Ok(ExitCode::from(1));
        }
        let unlinked = site.unlinked_types();
        for name in &unlinked {
            eprintln!("jacket: warning: unlinked type in {name}");
        }
        let broken = report_broken_links(&site, args.allow_broken_links || filtered);
        let unfenced = indented_code_warnings(&site.modules);
        for warning in &unfenced {
            eprintln!("jacket: warning: {warning}");
        }
        let procedures = site
            .modules
            .iter()
            .flat_map(|m| &m.items)
            .filter(|i| i.kind == Kind::Procedure && !i.binding)
            .count();
        if !args.check && !args.no_test_examples {
            let started = std::time::Instant::now();
            if !test_examples(args, &site, &jaic, &stdlib, &root)? {
                return Ok(ExitCode::from(1));
            }
            accounted += timing(args, "examples", started);
        }
        let compile_start = std::time::Instant::now();
        let prepared = theme.prepare(&jaic, &stdlib, args.timings)?;
        accounted += compile_start.elapsed();
        let render_start = std::time::Instant::now();
        if !args.check {
            write_site(&args.out, &files, args.runtime.as_deref())?;
        }
        let rendered = prepared.run(&stdlib, &input_dir.path, &cwd.join(&args.out), args.check)?;
        accounted += timing(args, "render and write", render_start);
        if rendered != ExitCode::SUCCESS {
            return Ok(rendered);
        }
        eprintln!(
            "jacket doc: {} modules, {items} items ({procedures} procedures), {} example programs{}",
            site.modules.len(),
            examples.len(),
            if args.check {
                String::new()
            } else {
                format!(", written to {}", args.out.display())
            }
        );
        Ok(link_status(args, &broken, &unlinked, filtered))
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

/// Files owned by the model step; the theme only registers them as link targets.
fn auxiliary_files(
    site: &Site,
    images: impl Iterator<Item = (String, Vec<u8>)>,
    version: &str,
    examples: &BTreeMap<String, String>,
) -> Vec<(String, Vec<u8>)> {
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();
    for (name, bytes) in images {
        files.push((format!("assets/{name}"), bytes));
    }
    let mut add = |path: String, text: String| files.push((path, text.into_bytes()));
    add("search-index.json".into(), search::index(site));
    if !version.is_empty() {
        add(
            "versions.json".into(),
            serde_json::json!([{ "version": version, "url": "./" }]).to_string(),
        );
    }
    for (id, parts) in examples {
        add(format!("examples/{id}.json"), parts.clone());
    }
    if site.runtime {
        add(
            "runtime.json".into(),
            "{\"engine\":\"runtime/engine.mjs\",\"wasm\":\"runtime/jai_wasm.wasm\"}\n".into(),
        );
    }
    files
}

fn timing(args: &DocArgs, label: &str, start: std::time::Instant) -> std::time::Duration {
    let elapsed = start.elapsed();
    if args.timings {
        eprintln!("jacket doc timings: {label}: {:.3}s", elapsed.as_secs_f64());
    }
    elapsed
}

fn json_bytes(value: &impl serde::Serialize) -> Result<Vec<u8>, String> {
    serde_json::to_vec(value).map_err(|e| format!("cannot serialize documentation view: {e}"))
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


/// Makes the home file the front page: it keeps the front page's place, title and sidebar.
fn use_home(guides: &mut Vec<view::PageView>, index: &mut view::PageView) {
    let Some(i) = guides.iter().position(|g| g.path == view::guide::HOME) else {
        return;
    };
    let mut home = guides.remove(i);
    home.path = String::new();
    home.title.clone_from(&index.title);
    home.description.clone_from(&index.description);
    home.side_label.clone_from(&index.side_label);
    std::mem::swap(&mut home.sidebar, &mut index.sidebar);
    *index = home;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_base_path_needs_both_slashes() {
        assert_eq!(base_path("/docs/jacket/").unwrap(), "/docs/jacket/");
        assert!(base_path("/docs/jacket").is_err());
        assert!(base_path("docs/").is_err());
    }

    #[test]
    fn the_engine_follows_the_flags_and_never_what_is_installed() {
        let (jaic, stdlib) = (Path::new("jaic"), Path::new("/s"));
        let pick = |runtime: Option<&Path>, root: &Path, node: bool| {
            engine(runtime, root, stdlib, jaic, node)
                .map(|e| matches!(e, example_check::Engine::Browser { .. }))
        };
        // No flag: the interpreter, with or without Node.
        assert_eq!(pick(None, stdlib, false), Ok(false));
        assert_eq!(pick(None, stdlib, true), Ok(false));
        // `--runtime` asks for the browser compiler.
        assert_eq!(pick(Some(Path::new("rt")), stdlib, true), Ok(true));
        let no_node = pick(Some(Path::new("rt")), stdlib, false).unwrap_err();
        assert!(
            no_node.contains("install Node") && no_node.contains("--runtime"),
            "{no_node}"
        );
        // The bundle holds the standard library only.
        let other = pick(Some(Path::new("rt")), Path::new("/lib"), true).unwrap_err();
        assert!(
            other.contains("--runtime") && other.contains("standard library"),
            "{other}"
        );
    }
}
