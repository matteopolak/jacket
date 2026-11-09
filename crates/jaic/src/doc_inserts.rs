//! `jaic -doc_model` with the top-level `#insert`s of the library run: the model of a module
//! reads source and cannot know what an `#insert` declares, so a module that has a public one is
//! compiled once with its default module parameters and each insert's text goes back into the
//! model as source it declares.
use crate::fxhash::HashMap;
use crate::ide::lint_compile::{CheckLimits, LintCompile, LintSetup};
use crate::ide::meta::IdeExpansionKind;
use crate::ide::{IdeFacts, IdeRoot};
use crate::interp::SandboxHost;
use crate::project::{ProgramInputs, module_import};
use crate::sema::{FileSystem, NativeFs, Options, ProgramSource};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// What compiling the module expanded.
struct Expanded {
    /// The text of each `#insert` that ran, by file and line.
    texts: HashMap<(PathBuf, usize), String>,
    /// Why the compile stopped, when it did.
    error: Option<String>,
}

fn canonical(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

/// The line `at` is on in `text`, counting from 1.
fn line_of(text: &str, at: usize) -> usize {
    text[..at.min(text.len())].matches('\n').count() + 1
}

/// The text of every `#insert` the module in `dir` runs when a program imports it without
/// module parameters, searching `import_dirs` and then `stdlib` for modules.
fn compile(dir: &Path, stdlib: &Path, import_dirs: &[PathBuf]) -> Expanded {
    let fs: Rc<dyn FileSystem> = Rc::new(NativeFs);
    let entry = canonical(&crate::paths::module_entry(dir));
    let inputs = ProgramInputs {
        configured: Vec::new(),
        extra: import_dirs.to_vec(),
        stdlib: stdlib.to_path_buf(),
    };
    let options = Options::for_program(&*fs, &entry, &inputs);
    let Some((import, _)) = module_import(&entry, &options.import_paths, true, &|p| canonical(p))
    else {
        return Expanded {
            texts: HashMap::default(),
            error: Some(format!("{} is not a module", entry.display())),
        };
    };
    let host = Rc::new(RefCell::new(SandboxHost::with_files(
        fs.clone(),
        &dir.to_string_lossy(),
    )));
    let mut started = LintCompile::start(LintSetup {
        fs,
        options,
        host,
        observer: None,
        report: Box::new(|_, _| {}),
        // A documentation build must not depend on how fast the machine is.
        limits: CheckLimits::new(CheckLimits::DEFAULT_EVAL_STEPS, 0.0),
    });
    let compiler = &mut started.compiler;
    compiler
        .ed
        .record(IdeFacts::new(vec![IdeRoot::under(canonical(dir))]));
    let program = format!("{import}\nmain :: () {{}}\n");
    let result = compiler.begin_sources(&[ProgramSource::String(program)]);
    let mut texts = HashMap::default();
    for expansion in compiler.ed.facts().map_or(&[][..], |f| &f.expansions) {
        if expansion.kind != IdeExpansionKind::Insert {
            continue;
        }
        let Some(source) = compiler.src.sources.try_get(expansion.span.file) else {
            continue;
        };
        let Some(file) = source.path.as_file() else {
            continue;
        };
        if let Some(text) = expansion.texts.first() {
            let line = line_of(&source.text, expansion.span.start as usize);
            texts.insert((canonical(file), line), text.clone());
        }
    }
    let error = result.err().map(|e| {
        let at = compiler
            .src
            .sources
            .try_get(e.span.file)
            .map(|s| format!(" ({}:{})", s.path, line_of(&s.text, e.span.start as usize)))
            .unwrap_or_default();
        format!("{}{at}", e.message)
    });
    Expanded {
        texts,
        error,
    }
}

/// What `jaic -doc_model` prints for the library in `dir`, with the text of its public top-level
/// `#insert`s read as source. Compile-time code of the library runs in the compiler's sandbox,
/// once, and only for a library that has such an `#insert`.
pub fn model_json(dir: &Path, stdlib: &Path, import_dirs: &[PathBuf]) -> String {
    let (dir, stdlib) = (canonical(dir), canonical(stdlib));
    let mut expanded: Option<Expanded> = None;
    crate::doc_comments::model_json_expanding(&dir, &stdlib, &mut |file, line| {
        let expanded = expanded.get_or_insert_with(|| {
            // Compiling needs a large stack, as `jaic` itself runs on.
            std::thread::scope(|scope| {
                std::thread::Builder::new()
                    .stack_size(1 << 30)
                    .spawn_scoped(scope, || compile(&dir, &stdlib, import_dirs))
                    .ok()
                    .and_then(|worker| worker.join().ok())
                    .unwrap_or(Expanded {
                        texts: HashMap::default(),
                        error: Some("the compiler stopped unexpectedly".to_owned()),
                    })
            })
        });
        match expanded.texts.get(&(canonical(file), line)) {
            Some(text) => Ok(text.clone()),
            // Not reached: under an `#if` that is off for this system, or in a file the module
            // does not load.
            None => expanded.error.clone().map_or(Ok(String::new()), Err),
        }
    })
}
