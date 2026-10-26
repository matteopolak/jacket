//! From paths on the command line to the roots to compile and the files to report on.
use super::Options;
use crate::config::Configs;
use jaic::project::{
    self, DiskFs, LoadCache, Program, ProgramInputs, ProjectFs, Root, RootOptions, program_for,
    resolve_program,
};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The identity of a file in a run: its real path, as `jaic::paths::file_identity` gives it.
pub(super) fn canonical(path: &Path) -> PathBuf {
    DiskFs.key(path)
}

/// The `.jai` files under `path`, sorted, excluding what the configuration excludes.
fn expand(path: &Path, config: &Configs, out: &mut BTreeSet<PathBuf>) {
    if config.excluded(&canonical(path)) {
        return;
    }
    if path.is_dir() {
        let files = jaic::paths::jai_files_under(path, &mut |p| config.excluded(&canonical(p)));
        out.extend(files.iter().map(|f| canonical(f)));
    } else if path.is_file() {
        out.insert(canonical(path));
    }
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
pub(super) fn plan(options: &Options, configs: &Configs) -> (Vec<Program>, BTreeSet<PathBuf>) {
    let mut listed = BTreeSet::new();
    for p in &options.paths {
        expand(p, configs, &mut listed);
    }
    let mut loaded_by_listed = BTreeSet::new();
    for f in &listed {
        loaded_by_listed.extend(project::reachable_loads(&DiskFs, f, usize::MAX));
    }
    let mut loader_cache = LoadCache::default();
    let mut roots: Vec<Program> = if let Some(source) = &options.source {
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
