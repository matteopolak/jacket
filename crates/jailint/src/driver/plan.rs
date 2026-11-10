//! From paths on the command line to the roots to compile and the files to report on.
use super::Options;
use super::marked_target::{Marked, marked_target};
use crate::config::Configs;
use jaic::project::{
    self, DiskFs, LoadCache, Program, ProgramInputs, ProjectFs, Root, RootOptions, program_for,
    resolve_program,
};
use jaic::target::{TargetCpu, TargetOs};
use jaic::test_notes::{header_notes, header_target_notes};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The identity of a file in a run: its real path, as `jaic::paths::file_identity` gives it.
pub(super) fn canonical(path: &Path) -> PathBuf {
    DiskFs.key(path)
}

/// The `.jai` files under `path`, sorted, excluding what the configuration excludes.
fn expand(path: &Path, config: &Configs, host: (TargetOs, TargetCpu), out: &mut BTreeSet<PathBuf>) {
    if config.excluded(&canonical(path)) {
        return;
    }
    if path.is_dir() {
        let files = jaic::paths::jai_files_under(path, &mut |p| config.excluded(&canonical(p)));
        out.extend(
            files
                .iter()
                .filter(|f| !left_out(f, host))
                .map(|f| canonical(f)),
        );
    } else if path.is_file() {
        out.insert(canonical(path));
    }
}

/// Files a directory scan leaves out: negative and fault fixtures intentionally violate the rules
/// it checks, and a file whose notes name no system the compiler can target (`only(.ANDROID)`)
/// compiles nowhere it would run. Explicit file arguments remain available for investigating
/// either.
fn left_out(path: &Path, host: (TargetOs, TargetCpu)) -> bool {
    let source = std::fs::read_to_string(path).unwrap_or_default();
    header_notes(&source).iter().any(|note| {
        matches!(
            note.split('(').next().unwrap_or(""),
            "compile_fail" | "should_fail" | "xfail"
        )
    }) || marked_target(&header_target_notes(&source).0, host.0, host.1) == Marked::Nowhere
}

/// What the import folders of a program rooted at `path` are made of: the `import_path` of the
/// nearest `jacket.toml`, the `-import_dir`s of the run and the standard library.
fn program_inputs(options: &Options, configs: &Configs, path: &Path) -> ProgramInputs {
    let mut inputs = ProgramInputs::for_file(&jaic::sema::NativeFs, path, &options.stdlib);
    inputs.configured.clone_from(&configs.at(path).import_dirs);
    inputs.extra.clone_from(&options.import_dirs);
    inputs
}

/// The program the linter compiles `file` as, with its import folders.
fn program_of(options: &Options, configs: &Configs, file: &Path, cache: &mut LoadCache) -> Program {
    let inputs = program_inputs(options, configs, file);
    let mut dirs = inputs.configured.clone();
    dirs.extend(inputs.extra.iter().cloned());
    dirs.push(inputs.stdlib.clone());
    resolve_program(&DiskFs, file, &RootOptions::linter(dirs), &inputs, cache)
}

// A flat file under an import directory can also be a standalone test.
// Keep its module checking semantics, but identify its process entry separately
// for lifetime proofs: the virtual importer has a different main module.
pub(super) fn declares_entry(path: &Path, entry: &str) -> bool {
    let Ok(text) = std::fs::read_to_string(path) else {
        return false;
    };
    let Ok(file) = jaic::parser::parse_file(jaic::source::FileId(0), &text) else {
        return false;
    };
    file.stmts.iter().any(|stmt| {
        let jaic::ast::StmtKind::Decl(decl) = &stmt.kind else {
            return false;
        };
        decl.names.iter().any(|n| n.name.as_str() == entry)
            && decl.value.as_ref().is_some_and(
                |v| matches!(&v.kind, jaic::ast::ExprKind::Proc(proc) if proc.body.is_some()),
            )
    })
}

/// The module search path `file`'s program is compiled with, before the compiler gets it.
pub fn root_import_paths(options: &Options, file: &Path) -> Vec<PathBuf> {
    let configs = Configs::new(&options.config);
    program_of(
        options,
        &configs,
        &canonical(file),
        &mut LoadCache::default(),
    )
    .import_dirs
}

/// Roots to compile and the files to report on.
pub(super) fn plan(
    options: &Options,
    configs: &Configs,
    entries: Option<&[PathBuf]>,
) -> (Vec<Program>, BTreeSet<PathBuf>) {
    let mut listed = BTreeSet::new();
    for p in &options.paths {
        expand(
            p,
            configs,
            (options.compiler_options.os, options.compiler_options.cpu),
            &mut listed,
        );
    }
    let mut loaded_by_listed = BTreeSet::new();
    for f in &listed {
        loaded_by_listed.extend(project::reachable_loads(&DiskFs, f, usize::MAX));
    }
    let mut loader_cache = LoadCache::default();
    let mut roots: Vec<Program> = if let Some(entries) = entries {
        entries
            .iter()
            .map(|entry| {
                let entry = canonical(entry);
                let program = program_of(options, configs, &entry, &mut loader_cache);
                match &program.root {
                    Root::Module {
                        entry: module, ..
                    } if *module == entry => program,
                    _ => {
                        let inputs = program_inputs(options, configs, &entry);
                        program_for(Root::Program(entry), &inputs)
                    }
                }
            })
            .collect()
    } else if let Some(source) = &options.source {
        let source = canonical(source);
        let inputs = program_inputs(options, configs, &source);
        vec![program_for(Root::Program(source), &inputs)]
    } else {
        listed
            .iter()
            .filter(|f| !loaded_by_listed.contains(*f))
            .map(|f| program_of(options, configs, f, &mut loader_cache))
            .collect()
    };
    roots.sort_by(|a, b| a.root.cmp(&b.root));
    roots.dedup_by(|a, b| a.root == b.root);
    let mut report = listed;
    report.extend(loaded_by_listed);
    report.retain(|f| !configs.excluded(f));
    (roots, report)
}
