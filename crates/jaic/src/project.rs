//! Project structure shared by the tools that read a Jai project without compiling all of it:
//! the TOML subset of `jai.toml` and `jailint.toml`, where a project's settings and base folder
//! are, which `#load`/`#import` directives a file writes, which files a program's `#load`s
//! reach, and which root file a source file is compiled from.
//!
//! The language server and the linter both build on this; neither parses settings or walks
//! `#load`s itself. See `docs/compiler/project.md`.
//!
//! - [`parse_toml`]: the one parser; [`ProjectConfig`] reads both settings files from it.
//! - [`scan_directives`] / [`directive_links`]: `#load` and `#import` found from tokens.
//! - [`ProjectFs`]: the files a walk sees (the disk, or an editor's open buffers over it).
//! - [`load_closure`]: every file a root reaches through `#load`.
//! - [`resolve_root`]: the file a source file belongs to, with [`RootOptions`] choosing the
//!   rules (the linter's and the language server's differ on purpose).
use crate::ast::ImportSource;
use crate::lexer::{P, Tok, Token};
use crate::paths::normalize;
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// The project settings file the editor and compiler tools read.
pub const JAI_TOML: &str = "jai.toml";

/// The linter's settings file.
pub const LINT_TOML: &str = "jailint.toml";

/// Entry files tried, in order, when no `jai.toml` names them.
pub const INFERRED_ENTRIES: &[&str] = &["build.jai", "first.jai", "main.jai", "src/main.jai"];

// -------------------------------------------------------------------------------------------
// The TOML subset
// -------------------------------------------------------------------------------------------

/// What [`parse_toml`] hands its visitor.
#[derive(Debug)]
pub enum TomlItem<'a> {
    /// A `[name]` header.
    Table(&'a str),
    /// `key = value` in the current table (`""` before any header). `value` is the text after
    /// the `=`, comment removed, an array continued over several lines joined by spaces.
    Pair {
        table: &'a str,
        key: &'a str,
        value: &'a str,
    },
}

/// Reads the subset of TOML the settings files use: `key = value` lines, `[table]` headers,
/// `#` comments, and arrays continued over several lines. `visit` is called for each header and
/// pair with the 1-based line it started on, and may reject it with a message (the parser puts
/// `line N:` in front of syntax errors it finds itself; `visit` writes its own).
pub fn parse_toml(
    text: &str,
    mut visit: impl FnMut(TomlItem<'_>, usize) -> Result<(), String>,
) -> Result<(), String> {
    let mut table = String::new();
    let mut pending: Option<(usize, String, String)> = None;
    for (number, raw) in text.lines().enumerate() {
        let line = strip_comment(raw).trim().to_string();
        // An array continued over several lines.
        if let Some((at, key, mut value)) = pending.take() {
            value.push(' ');
            value.push_str(&line);
            if value.trim_end().ends_with(']') {
                visit(
                    TomlItem::Pair {
                        table: &table,
                        key: &key,
                        value: &value,
                    },
                    at,
                )?;
            } else {
                pending = Some((at, key, value));
            }
            continue;
        }
        if line.is_empty() {
            continue;
        }
        if !line.contains('=')
            && let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']'))
        {
            table = name.trim().to_string();
            visit(TomlItem::Table(&table), number + 1)?;
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(format!(
                "line {}: expected `key = value`, found `{line}`",
                number + 1
            ));
        };
        let (key, value) = (key.trim().to_string(), value.trim().to_string());
        if value.starts_with('[') && !value.ends_with(']') {
            pending = Some((number + 1, key, value));
            continue;
        }
        visit(
            TomlItem::Pair {
                table: &table,
                key: &key,
                value: &value,
            },
            number + 1,
        )?;
    }
    if let Some((at, ..)) = pending {
        return Err(format!(
            "line {at}: this array is never closed\nhelp: end it with `]`"
        ));
    }
    Ok(())
}

/// The line without a `#` comment (a `#` inside a string stays).
fn strip_comment(line: &str) -> &str {
    let mut quoted = false;
    for (i, c) in line.char_indices() {
        match c {
            '"' => quoted = !quoted,
            '#' if !quoted => return &line[..i],
            _ => {}
        }
    }
    line
}

/// The contents of a `"string"` value (no escapes: paths and levels need none).
pub fn toml_string(text: &str) -> Option<String> {
    let inner = text.trim().strip_prefix('"')?.strip_suffix('"')?;
    (!inner.contains('"')).then(|| inner.to_string())
}

/// The strings of an `["a", "b"]` value; `Err` names the problem (`NotAnArray`, or a non-string
/// item).
pub fn toml_strings(value: &str) -> Result<Vec<String>, TomlArrayError> {
    let inner = value
        .strip_prefix('[')
        .and_then(|v| v.strip_suffix(']'))
        .ok_or(TomlArrayError::NotAnArray)?;
    inner
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|item| toml_string(item).ok_or(TomlArrayError::NotAString))
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TomlArrayError {
    NotAnArray,
    NotAString,
}

// -------------------------------------------------------------------------------------------
// Settings
// -------------------------------------------------------------------------------------------

/// The settings of a `jai.toml` and a `jailint.toml`: the union of what the tools read. Each
/// file only sets its own fields.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProjectConfig {
    /// `jai.toml` `build_files = ["first.jai"]`: the files given to `jaic` (or
    /// `add_build_file`).
    pub build_files: Vec<String>,
    /// `jai.toml` `import_path = ["vendor"]`: more module folders, as `-import_dir` (and
    /// `Build_Options.import_path`) add.
    pub import_path: Vec<String>,
    /// `jailint.toml` `exclude = ["tests/**"]`: glob patterns of paths not to lint.
    pub exclude: Vec<String>,
    /// `jailint.toml` `max_nesting = 6`.
    pub max_nesting: Option<usize>,
    /// `jailint.toml` `[rules]`: rule name and the level text, in file order.
    pub rules: Vec<(String, String)>,
}

impl ProjectConfig {
    /// The settings in the text of a `jai.toml`, or why it is invalid (with its line).
    pub fn parse_jai(text: &str) -> Result<ProjectConfig, String> {
        let mut config = ProjectConfig::default();
        parse_toml(text, |item, line| match item {
            TomlItem::Table(_) => Err(format!(
                "line {line}: `{JAI_TOML}` has no tables; its settings are `build_files` and `import_path`"
            )),
            TomlItem::Pair {
                key,
                value,
                ..
            } => {
                let list = match key {
                    "build_files" => &mut config.build_files,
                    "import_path" => &mut config.import_path,
                    other => {
                        return Err(format!(
                            "line {line}: unknown setting `{other}`; the settings are `build_files` and `import_path`"
                        ));
                    }
                };
                // A single string is a list of one.
                if let Some(one) = toml_string(value) {
                    list.push(one);
                    return Ok(());
                }
                match toml_strings(value) {
                    Ok(items) => list.extend(items),
                    Err(TomlArrayError::NotAnArray) => {
                        return Err(format!("line {line}: `{key}` takes an array of strings"));
                    }
                    Err(TomlArrayError::NotAString) => {
                        return Err(format!("line {line}: expected a quoted string"));
                    }
                }
                Ok(())
            }
        })?;
        Ok(config)
    }

    /// The settings in the text of a `jailint.toml`. `rule` judges each `[rules]` entry (name,
    /// the value as written, line): it returns the level text to keep, or the error message.
    pub fn parse_lint(
        text: &str,
        mut rule: impl FnMut(&str, &str, usize) -> Result<String, String>,
    ) -> Result<ProjectConfig, String> {
        let mut config = ProjectConfig::default();
        parse_toml(text, |item, line| match item {
            TomlItem::Table(name) => {
                if name != "rules" {
                    return Err(format!(
                        "line {line}: unknown table `[{name}]`\nhelp: the only table is `[rules]`"
                    ));
                }
                Ok(())
            }
            TomlItem::Pair {
                table,
                key,
                value,
            } => match (table, key) {
                ("", "exclude") => {
                    let items = toml_strings(value).map_err(|e| match e {
                        TomlArrayError::NotAnArray => {
                            format!("line {line}: `exclude` takes an array of strings")
                        }
                        TomlArrayError::NotAString => {
                            format!("line {line}: expected a quoted string")
                        }
                    })?;
                    config.exclude.extend(items);
                    Ok(())
                }
                ("", "max_nesting") => {
                    config.max_nesting = Some(value.parse().map_err(|_| {
                        format!(
                            "line {line}: `max_nesting` takes a whole number, such as `max_nesting = 6`"
                        )
                    })?);
                    Ok(())
                }
                ("", other) => Err(format!(
                    "line {line}: unknown setting `{other}`\nhelp: the top-level settings are `exclude = [\"pattern\", ...]` and `max_nesting = 6`; rule levels go under `[rules]`"
                )),
                (_, name) => {
                    let name = name.trim_matches('"');
                    let level = rule(name, value, line)?;
                    config.rules.push((name.to_string(), level));
                    Ok(())
                }
            },
        })?;
        Ok(config)
    }
}

/// The nearest folder at or above `start` (a folder) holding a file called `name`, by
/// `is_file`. An editor passes a test that counts unsaved buffers.
pub fn find_config_dir(
    start: &Path,
    name: &str,
    is_file: impl Fn(&Path) -> bool,
) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|dir| is_file(&dir.join(name)))
        .map(Path::to_path_buf)
}

/// The base folder of the project holding `file`: where the nearest `jai.toml` is, else the
/// innermost of the workspace `folders` that holds `file`.
pub fn project_base(
    config_dir: Option<&Path>,
    folders: &[PathBuf],
    file: &Path,
) -> Option<PathBuf> {
    config_dir.map(Path::to_path_buf).or_else(|| {
        folders
            .iter()
            .filter(|f| file.starts_with(f))
            .max_by_key(|f| f.as_os_str().len())
            .cloned()
    })
}

/// The entry files of the project based at `base`.
pub struct EntryFiles {
    pub paths: Vec<PathBuf>,
    /// They are the `jai.toml`'s `build_files`; otherwise they are the [`INFERRED_ENTRIES`], of
    /// which only those that exist are entries.
    pub declared: bool,
}

/// `base`'s entry files: the `build_files` of its `jai.toml`, else the inferred ones.
pub fn entry_files(base: &Path, config: Option<&ProjectConfig>) -> EntryFiles {
    match config {
        Some(config) if !config.build_files.is_empty() => EntryFiles {
            paths: config
                .build_files
                .iter()
                .map(|f| normalize(&base.join(f)))
                .collect(),
            declared: true,
        },
        _ => EntryFiles {
            paths: INFERRED_ENTRIES.iter().map(|f| base.join(f)).collect(),
            declared: false,
        },
    }
}

// -------------------------------------------------------------------------------------------
// Directives
// -------------------------------------------------------------------------------------------

/// What a directive brings in.
#[derive(Clone, Debug)]
pub enum LinkSource {
    /// `#load "path"`, relative to the loading file.
    Load(String),
    /// `#import "Name"`, `#import,file "x.jai"`, `#import,dir "x"`.
    Import(ImportSource),
}

/// A `#load` or `#import` directive and the string it names.
#[derive(Clone, Debug)]
pub struct DirectiveLink {
    /// Byte range from the directive to the end of its string literal.
    pub directive: Range<usize>,
    /// Byte range of the string literal, quotes included.
    pub string: Range<usize>,
    pub source: LinkSource,
    /// The file it names, normalised, when that is known from the file's folder alone: a
    /// `#load`, `#import,file` or `#import,dir`. A module needs the import paths. Filled by
    /// [`directive_links`], not by [`scan_directives`].
    pub target: Option<PathBuf>,
}

/// The `#load` and `#import` directives of a token stream (found from tokens, so they work
/// while the text does not parse). `#import,string` is no link and is skipped.
pub fn scan_directives(tokens: &[Token]) -> Vec<DirectiveLink> {
    let mut out = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        let Tok::Directive(name) = &token.tok else {
            continue;
        };
        let load = match name.as_str() {
            "load" => true,
            "import" => false,
            _ => continue,
        };
        // `#import,file "x"`: comma-separated flags before the string.
        let mut at = i + 1;
        let mut flag = None;
        while let (Some(Tok::Punct(P::Comma)), Some(Tok::Ident(f))) = (
            tokens.get(at).map(|t| &t.tok),
            tokens.get(at + 1).map(|t| &t.tok),
        ) {
            flag.get_or_insert(f.as_str().to_string());
            at += 2;
        }
        let Some(Token {
            tok: Tok::Str(bytes),
            span,
            ..
        }) = tokens.get(at)
        else {
            continue;
        };
        let path = String::from_utf8_lossy(bytes).into_owned();
        let source = if load {
            LinkSource::Load(path)
        } else {
            LinkSource::Import(match flag.as_deref() {
                Some("file") => ImportSource::File(path.into()),
                Some("dir") => ImportSource::Dir(path.into()),
                Some("string") => continue,
                _ => ImportSource::Module(path.into()),
            })
        };
        out.push(DirectiveLink {
            directive: token.span.start as usize..span.end as usize,
            string: span.start as usize..span.end as usize,
            source,
            target: None,
        });
    }
    out
}

/// The directives of the file `file` with text `text`, targets resolved against its folder.
/// Text that does not lex has none.
pub fn directive_links(file: &Path, text: &str) -> Vec<DirectiveLink> {
    let Ok(tokens) = crate::lexer::lex(crate::source::FileId(0), text) else {
        return Vec::new();
    };
    let mut links = scan_directives(&tokens);
    resolve_targets(&mut links, file.parent().unwrap_or(Path::new(".")));
    links
}

/// Fill the `target` of `links` for a file in `dir`.
pub fn resolve_targets(links: &mut [DirectiveLink], dir: &Path) {
    for link in links {
        link.target = match &link.source {
            LinkSource::Load(path) => Some(normalize(&dir.join(path))),
            LinkSource::Import(ImportSource::File(path) | ImportSource::Dir(path)) => {
                Some(normalize(&dir.join(&**path)))
            }
            LinkSource::Import(_) => None,
        };
    }
}

// -------------------------------------------------------------------------------------------
// Files
// -------------------------------------------------------------------------------------------

/// The files a project walk sees. [`DiskFs`] is the disk; an editor puts its open buffers over
/// it and may cache what it scanned.
pub trait ProjectFs {
    /// The identity of `path` in the walk (the disk resolves links); equal files have equal keys.
    fn key(&self, path: &Path) -> PathBuf {
        path.to_path_buf()
    }

    fn read(&self, path: &Path) -> Option<String>;
    fn is_file(&self, path: &Path) -> bool;
    fn is_dir(&self, path: &Path) -> bool;

    /// Names directly inside `dir`, and whether each is a folder.
    fn list_dir(&self, dir: &Path) -> Vec<(String, bool)>;

    /// The files `file` `#load`s, keyed, whether or not they exist.
    fn loads(&self, file: &Path) -> Vec<PathBuf> {
        let Some(text) = self.read(file) else {
            return Vec::new();
        };
        directive_links(file, &text)
            .into_iter()
            .filter(|l| matches!(l.source, LinkSource::Load(_)))
            .filter_map(|l| l.target)
            .map(|t| self.key(&t))
            .collect()
    }

    /// The files a program that includes `file` also includes: its existing `#load`s. An
    /// editor adds what a build metaprogram compiles.
    fn edges(&self, file: &Path) -> Vec<PathBuf> {
        self.loads(file)
            .into_iter()
            .filter(|t| self.is_file(t))
            .collect()
    }

    /// The direct loaders of each file under `folder` (see [`build_loader_graph`]) when this
    /// file system keeps them between calls and knows they are current. An editor that asks
    /// for a program after every keystroke answers from a graph it rebuilds only when a
    /// `#load` changes, instead of reading every `#load` of the tree again.
    fn loader_graph(&self, _folder: &Path) -> Option<LoaderGraph> {
        None
    }

    /// The `.jai` files under `folder`, for [`LoaderSearch::Tree`].
    fn project_files(&self, folder: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let mut pending = vec![(folder.to_path_buf(), 0usize)];
        while let Some((dir, depth)) = pending.pop() {
            for (name, is_dir) in self.list_dir(&dir) {
                if files.len() >= MAX_TREE_FILES {
                    return files;
                }
                let path = dir.join(&name);
                if is_dir {
                    if depth < MAX_TREE_DEPTH
                        && !name.starts_with('.')
                        && !TREE_SKIPPED.contains(&name.as_str())
                    {
                        pending.push((path, depth + 1));
                    }
                } else if name.ends_with(".jai") {
                    files.push(self.key(&path));
                }
            }
        }
        files
    }
}

/// For each file, the files under a folder that `#load` it.
pub type LoaderGraph = Rc<BTreeMap<PathBuf, Vec<PathBuf>>>;

/// The most `.jai` files a [`ProjectFs::project_files`] walk collects, and how deep it goes.
pub const MAX_TREE_FILES: usize = 4000;

pub const MAX_TREE_DEPTH: usize = 8;

/// Folders a project walk skips besides hidden ones (module folders are indexed as modules).
pub const TREE_SKIPPED: &[&str] = &["modules", "node_modules", "target", "build", "bin"];

/// The disk, with links resolved in keys.
pub struct DiskFs;

impl ProjectFs for DiskFs {
    fn key(&self, path: &Path) -> PathBuf {
        crate::canonicalize(path)
            .or_else(|_| std::path::absolute(path))
            .unwrap_or_else(|_| path.to_path_buf())
    }

    fn read(&self, path: &Path) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }

    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn list_dir(&self, dir: &Path) -> Vec<(String, bool)> {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return Vec::new();
        };
        entries
            .flatten()
            .map(|e| {
                (
                    e.file_name().to_string_lossy().into_owned(),
                    e.path().is_dir(),
                )
            })
            .collect()
    }
}

/// Files reachable from `root` through [`ProjectFs::edges`], not `root` itself unless a cycle
/// loads it back; at most `limit` files are followed.
pub fn reachable_loads(fs: &dyn ProjectFs, root: &Path, limit: usize) -> BTreeSet<PathBuf> {
    let mut seen = BTreeSet::new();
    let mut pending = vec![fs.key(root)];
    while let Some(file) = pending.pop() {
        for target in fs.edges(&file) {
            let target = fs.key(&target);
            if seen.len() < limit && seen.insert(target.clone()) {
                pending.push(target);
            }
        }
    }
    seen
}

/// `root` and every file its `#load`s reach (a program's files).
pub fn load_closure(fs: &dyn ProjectFs, root: &Path, limit: usize) -> BTreeSet<PathBuf> {
    let mut seen = reachable_loads(fs, root, limit.saturating_sub(1));
    seen.insert(fs.key(root));
    seen
}

// -------------------------------------------------------------------------------------------
// Roots
// -------------------------------------------------------------------------------------------

/// What a source file is compiled as.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Root {
    /// A module: its entry file, the name to import it by and the directory holding it.
    Module {
        entry: PathBuf,
        name: String,
        dir: PathBuf,
    },
    /// A program compiled from this file.
    Program(PathBuf),
}

impl Root {
    pub fn path(&self) -> &Path {
        match self {
            Root::Module {
                entry, ..
            } => entry,
            Root::Program(p) => p,
        }
    }
}

/// Where [`resolve_root`] looks for the files that `#load` a file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoaderSearch {
    /// The `.jai` files in the file's folder and this many folders above it (itself included).
    Nearby { levels: usize },
    /// Every `.jai` file under `folder` (or under the file's folder when `None`), through
    /// [`ProjectFs::project_files`].
    Tree { folder: Option<PathBuf> },
}

/// How [`resolve_root`] picks among several files that load a file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TieBreak {
    /// The loader that no other loader loads; ties go to the first path. The linter's rule.
    Outermost,
    /// Climb: the `module.jai` first, then the shallowest path, then the first by name, and
    /// repeat from the chosen loader until nothing loads it. The language server's rule, which
    /// gives a file the same owner however it is reached.
    ModuleFirstShallowest,
}

/// The choices [`resolve_root`] makes.
#[derive(Clone, Debug)]
pub struct RootOptions {
    /// Import folders (the standard library last): a file in one is a module. `None` does not
    /// look for modules at all.
    pub import_dirs: Option<Vec<PathBuf>>,
    pub search: LoaderSearch,
    pub tie_break: TieBreak,
}

impl RootOptions {
    /// The linter's rules: modules by import folder and `module.jai`, loaders nearby (three
    /// levels), the outermost loader wins.
    pub fn linter(import_dirs: Vec<PathBuf>) -> RootOptions {
        RootOptions {
            import_dirs: Some(import_dirs),
            search: LoaderSearch::Nearby {
                levels: LINTER_LOADER_LEVELS,
            },
            tie_break: TieBreak::Outermost,
        }
    }

    /// The language server's rules: no module detection (a module's parts are found by their
    /// loader), loaders anywhere under `folder`, climbing with `module.jai` first.
    pub fn language_server(folder: Option<PathBuf>) -> RootOptions {
        RootOptions {
            import_dirs: None,
            search: LoaderSearch::Tree {
                folder,
            },
            tie_break: TieBreak::ModuleFirstShallowest,
        }
    }
}

/// How far above a file the linter looks for a program that loads it.
pub const LINTER_LOADER_LEVELS: usize = 3;

/// Loaded-file sets by loader, kept between calls for the same [`ProjectFs`].
#[derive(Default)]
pub struct LoadCache {
    reached: BTreeMap<PathBuf, BTreeSet<PathBuf>>,
}

impl LoadCache {
    fn reached(&mut self, fs: &dyn ProjectFs, file: &Path) -> &BTreeSet<PathBuf> {
        self.reached
            .entry(file.to_path_buf())
            .or_insert_with(|| reachable_loads(fs, file, usize::MAX))
    }
}

/// The root `file` (a key of `fs`) is compiled as: the module whose `module.jai` loads it, the
/// module it is the entry of, the program that loads it, or itself as a program.
pub fn resolve_root(
    fs: &dyn ProjectFs,
    file: &Path,
    options: &RootOptions,
    cache: &mut LoadCache,
) -> Root {
    if let Some(import_dirs) = &options.import_dirs
        && let Some(module) = enclosing_module(fs, file, import_dirs, cache)
            .or_else(|| module_of(fs, file, import_dirs))
    {
        return module;
    }
    match enclosing_program(fs, file, &options.search, options.tie_break, cache) {
        Some(program) => Root::Program(program),
        None => Root::Program(file.to_path_buf()),
    }
}

/// The module `file` is the entry of, if it is one: `module.jai`, or a file directly in an
/// import directory.
pub fn module_of(fs: &dyn ProjectFs, file: &Path, import_dirs: &[PathBuf]) -> Option<Root> {
    let mut dirs: Vec<_> = import_dirs.iter().map(|dir| fs.key(dir)).collect();
    dirs.sort_by_key(|dir| std::cmp::Reverse(dir.components().count()));
    for dir in dirs {
        if let Ok(relative) = file.strip_prefix(&dir) {
            let name = if relative
                .file_name()
                .is_some_and(|name| name == "module.jai")
            {
                relative.parent()?.to_path_buf()
            } else {
                relative.with_extension("")
            };
            return Some(Root::Module {
                entry: file.to_path_buf(),
                name: name.to_string_lossy().replace('\\', "/"),
                dir,
            });
        }
    }
    let dir = file.parent()?;
    if file.file_name().is_some_and(|n| n == "module.jai") {
        return Some(Root::Module {
            entry: file.to_path_buf(),
            name: dir.file_name()?.to_string_lossy().into_owned(),
            dir: dir.parent()?.to_path_buf(),
        });
    }
    let in_import_dir = import_dirs.iter().any(|d| fs.key(d) == dir)
        || dir.file_name().is_some_and(|n| n == "modules");
    in_import_dir.then(|| Root::Module {
        entry: file.to_path_buf(),
        name: file
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned(),
        dir: dir.to_path_buf(),
    })
}

/// For a file that is not a module entry: the module whose `module.jai` (in the file's folder or
/// above it) `#load`s it.
pub fn enclosing_module(
    fs: &dyn ProjectFs,
    file: &Path,
    import_dirs: &[PathBuf],
    cache: &mut LoadCache,
) -> Option<Root> {
    for dir in file.parent()?.ancestors() {
        let entry = dir.join("module.jai");
        if !fs.is_file(&entry) {
            continue;
        }
        let entry = fs.key(&entry);
        return if cache.reached(fs, &entry).contains(file) {
            module_of(fs, &entry, import_dirs)
        } else {
            None
        };
    }
    None
}

/// The program that `#load`s `file`, found as `search` and `tie_break` say; `None` when nothing
/// loads it.
pub fn enclosing_program(
    fs: &dyn ProjectFs,
    file: &Path,
    search: &LoaderSearch,
    tie_break: TieBreak,
    cache: &mut LoadCache,
) -> Option<PathBuf> {
    match tie_break {
        TieBreak::Outermost => {
            let loaders = match search {
                LoaderSearch::Nearby {
                    levels,
                } => nearby_loaders(fs, file, *levels, cache),
                LoaderSearch::Tree {
                    folder,
                } => tree_loaders(fs, file, folder.as_deref(), cache),
            };
            outermost(&loaders, cache)
        }
        TieBreak::ModuleFirstShallowest => {
            let root = climb(&gather_loaders(fs, file, search), file);
            (root != file).then_some(root)
        }
    }
}

/// The `.jai` files under `folder` (the file's folder when `None`) that reach `file`.
fn tree_loaders(
    fs: &dyn ProjectFs,
    file: &Path,
    folder: Option<&Path>,
    cache: &mut LoadCache,
) -> Vec<PathBuf> {
    let Some(folder) = folder.or(file.parent()) else {
        return Vec::new();
    };
    let mut candidates = fs.project_files(folder);
    candidates.sort();
    candidates.retain(|c| c != file && cache.reached(fs, c).contains(file));
    candidates
}

/// The `.jai` files near `file` that reach it through `#load`, nearest folder first.
fn nearby_loaders(
    fs: &dyn ProjectFs,
    file: &Path,
    levels: usize,
    cache: &mut LoadCache,
) -> Vec<PathBuf> {
    let mut loaders = Vec::new();
    let Some(parent) = file.parent() else {
        return loaders;
    };
    for dir in parent.ancestors().take(levels) {
        let mut candidates: Vec<PathBuf> = fs
            .list_dir(dir)
            .into_iter()
            .map(|(name, _)| dir.join(name))
            .filter(|p| p.extension().is_some_and(|e| e == "jai") && fs.is_file(p))
            .map(|p| fs.key(&p))
            .filter(|p| p != file)
            .collect();
        candidates.sort();
        for c in candidates {
            if cache.reached(fs, &c).contains(file) {
                loaders.push(c);
            }
        }
    }
    loaders
}

/// Of `loaders`, the first that none of the others reaches (else the first).
fn outermost(loaders: &[PathBuf], cache: &mut LoadCache) -> Option<PathBuf> {
    loaders
        .iter()
        .find(|l| {
            !loaders
                .iter()
                .any(|o| o != *l && cache.reached.get(o).is_some_and(|set| set.contains(*l)))
        })
        .or(loaders.first())
        .cloned()
}

/// Direct loaders of each file under the search area, the tree's way.
fn gather_loaders(fs: &dyn ProjectFs, file: &Path, search: &LoaderSearch) -> LoaderGraph {
    let folder = match search {
        LoaderSearch::Tree {
            folder,
        } => folder
            .clone()
            .or_else(|| file.parent().map(Path::to_path_buf)),
        LoaderSearch::Nearby {
            levels,
        } => file
            .parent()
            .and_then(|p| p.ancestors().take(*levels).last())
            .map(Path::to_path_buf),
    };
    let Some(folder) = folder else {
        return LoaderGraph::default();
    };
    fs.loader_graph(&folder)
        .unwrap_or_else(|| Rc::new(build_loader_graph(fs, &fs.project_files(&folder))))
}

/// The direct loaders of each file that `files` `#load`.
pub fn build_loader_graph(
    fs: &dyn ProjectFs,
    files: &[PathBuf],
) -> BTreeMap<PathBuf, Vec<PathBuf>> {
    let mut loaders: BTreeMap<PathBuf, Vec<PathBuf>> = BTreeMap::new();
    for f in files {
        for target in fs.loads(f) {
            loaders.entry(fs.key(&target)).or_default().push(f.clone());
        }
    }
    loaders
}

/// The loader order of preference for [`TieBreak::ModuleFirstShallowest`].
fn preference(candidate: &Path) -> (bool, usize, PathBuf) {
    (
        candidate.file_name().is_none_or(|n| n != "module.jai"),
        candidate.components().count(),
        candidate.to_path_buf(),
    )
}

fn climb(loaders: &BTreeMap<PathBuf, Vec<PathBuf>>, file: &Path) -> PathBuf {
    let mut root = file.to_path_buf();
    let mut visited = BTreeSet::from([root.clone()]);
    while let Some(found) = loaders.get(&root).and_then(|candidates| {
        candidates
            .iter()
            .filter(|c| !visited.contains(*c))
            .min_by_key(|c| preference(c))
            .cloned()
    }) {
        visited.insert(found.clone());
        root = found;
    }
    root
}

/// The file a fragment is analysed from: `file`, then whoever `#load`s it, and whoever loads
/// that, until a file nothing loads. Loaders are looked for among the files under `folder`.
/// Where several files load one, the `module.jai` goes first, then the shallowest path, then
/// the first by name, so the same file always gets the same owner.
pub fn climb_loaders(fs: &dyn ProjectFs, file: &Path, folder: Option<&Path>) -> PathBuf {
    let loaders = gather_loaders(
        fs,
        file,
        &LoaderSearch::Tree {
            folder: folder.map(Path::to_path_buf),
        },
    );
    climb(&loaders, file)
}

// -------------------------------------------------------------------------------------------
// Tests
// -------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str, files: &[(&str, &str)]) -> PathBuf {
        // Nested so the linter's search two folders up never reaches the shared temporary folder.
        let base = std::env::temp_dir().join(format!("jaic-project-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let dir = base.join("outer").join("root");
        for (path, text) in files {
            let path = dir.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        DiskFs.key(&dir)
    }

    #[test]
    fn parses_jai_toml() {
        let config = ProjectConfig::parse_jai(
            "# The editor's settings\nbuild_files = [\"first.jai\", # metaprogram\n  \"src/main.jai\"]\nimport_path = \"vendor\"\n",
        )
        .unwrap();
        assert_eq!(config.build_files, ["first.jai", "src/main.jai"]);
        assert_eq!(config.import_path, ["vendor"]);
        assert!(
            ProjectConfig::parse_jai("roots = [\"a.jai\"]")
                .unwrap_err()
                .contains("unknown setting `roots`")
        );
        assert!(ProjectConfig::parse_jai("[project]").is_err());
        assert!(
            ProjectConfig::parse_jai("build_files = [\"a\"")
                .unwrap_err()
                .contains("never closed")
        );
        assert!(ProjectConfig::parse_jai("build_files = 3").is_err());
    }

    #[test]
    fn parses_jailint_toml() {
        let text = "# settings\nexclude = [\n  \"tests/**\", # fixtures\n  \"a.jai\",\n]\nmax_nesting = 4\n\n[rules]\nfloat_equality = \"deny\" # strict\n";
        let config = ProjectConfig::parse_lint(text, |name, value, _| {
            assert_eq!(name, "float_equality");
            toml_string(value).ok_or_else(|| "bad".to_string())
        })
        .unwrap();
        assert_eq!(config.exclude, ["tests/**", "a.jai"]);
        assert_eq!(config.max_nesting, Some(4));
        assert_eq!(config.rules, [("float_equality".into(), "deny".into())]);
        let reject = |_: &str, _: &str, line: usize| Err(format!("line {line}: no"));
        assert_eq!(
            ProjectConfig::parse_lint("[rules]\nx = 1", reject).unwrap_err(),
            "line 2: no"
        );
        let err = ProjectConfig::parse_lint("[lint]\n", reject).unwrap_err();
        assert!(err.contains("unknown table `[lint]`"));
        assert!(ProjectConfig::parse_lint("max_nesting = x", reject).is_err());
        assert!(ProjectConfig::parse_lint("exclude = \"a\"", reject).is_err());
    }

    #[test]
    fn base_and_entries() {
        let folders = [PathBuf::from("/w"), PathBuf::from("/w/inner")];
        assert_eq!(
            project_base(None, &folders, Path::new("/w/inner/a.jai")),
            Some(PathBuf::from("/w/inner"))
        );
        assert_eq!(
            project_base(Some(Path::new("/w/p")), &folders, Path::new("/w/p/a.jai")),
            Some(PathBuf::from("/w/p"))
        );
        assert_eq!(project_base(None, &folders, Path::new("/x/a.jai")), None);
        let config = ProjectConfig {
            build_files: vec!["src/../first.jai".into()],
            ..ProjectConfig::default()
        };
        let declared = entry_files(Path::new("/p"), Some(&config));
        assert!(declared.declared);
        assert_eq!(declared.paths, [PathBuf::from("/p/first.jai")]);
        let inferred = entry_files(Path::new("/p"), None);
        assert!(!inferred.declared);
        assert_eq!(inferred.paths[0], Path::new("/p/build.jai"));
    }

    #[test]
    fn directives_with_flags_and_targets() {
        let text = "#import \"Basic\"; B :: #import,file \"b.jai\"; #load \"x/../y.jai\"; #import,string \"x :: 1;\"; #import,dir \"d\";";
        let found = directive_links(Path::new("/p/a.jai"), text);
        assert_eq!(found.len(), 4);
        assert!(
            matches!(&found[0].source, LinkSource::Import(ImportSource::Module(n)) if &**n == "Basic")
        );
        assert_eq!(found[0].target, None);
        assert!(
            matches!(&found[1].source, LinkSource::Import(ImportSource::File(n)) if &**n == "b.jai")
        );
        assert_eq!(found[1].target.as_deref(), Some(Path::new("/p/b.jai")));
        assert!(matches!(&found[2].source, LinkSource::Load(p) if p == "x/../y.jai"));
        assert_eq!(found[2].target.as_deref(), Some(Path::new("/p/y.jai")));
        assert_eq!(&text[found[2].string.clone()], "\"x/../y.jai\"");
        assert!(text[found[2].directive.clone()].starts_with("#load"));
        assert_eq!(found[3].target.as_deref(), Some(Path::new("/p/d")));
        assert!(directive_links(Path::new("/p/a.jai"), "\"unterminated").is_empty());
    }

    #[test]
    fn closure_follows_loads_and_cycles() {
        let dir = fixture(
            "closure",
            &[
                ("main.jai", "#load \"a.jai\"; #load \"gone.jai\";"),
                ("a.jai", "#load \"sub/b.jai\";"),
                ("sub/b.jai", "#load \"../main.jai\";"),
                ("other.jai", ""),
            ],
        );
        let main = dir.join("main.jai");
        let closure = load_closure(&DiskFs, &main, 100);
        let names: Vec<_> = closure
            .iter()
            .map(|p| {
                p.strip_prefix(&dir)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect();
        assert_eq!(names, ["a.jai", "main.jai", "sub/b.jai"]);
        // A cycle loads the root back; without one, the root is not among what it reaches.
        assert!(reachable_loads(&DiskFs, &main, 100).contains(&main));
        let a = dir.join("a.jai");
        assert!(!reachable_loads(&DiskFs, &dir.join("other.jai"), 100).contains(&a));
        // The limit counts the root.
        assert_eq!(load_closure(&DiskFs, &main, 2).len(), 2);
    }

    #[test]
    fn linter_roots() {
        let dir = fixture(
            "linter",
            &[
                ("prog/main.jai", "#load \"a.jai\";"),
                ("prog/a.jai", "#load \"deep/b.jai\";"),
                ("prog/deep/b.jai", ""),
                ("prog/alone.jai", ""),
                ("mods/Thing/module.jai", "#load \"part.jai\";"),
                ("mods/Thing/part.jai", ""),
                ("mods/Flat.jai", ""),
            ],
        );
        let options = RootOptions::linter(vec![dir.join("mods")]);
        let mut cache = LoadCache::default();
        let mut root = |path: &str| resolve_root(&DiskFs, &dir.join(path), &options, &mut cache);
        assert_eq!(
            root("prog/deep/b.jai"),
            Root::Program(dir.join("prog/main.jai"))
        );
        assert_eq!(root("prog/a.jai"), Root::Program(dir.join("prog/main.jai")));
        assert_eq!(
            root("prog/alone.jai"),
            Root::Program(dir.join("prog/alone.jai"))
        );
        assert_eq!(
            root("mods/Thing/part.jai"),
            Root::Module {
                entry: dir.join("mods/Thing/module.jai"),
                name: "Thing".into(),
                dir: dir.join("mods"),
            }
        );
        assert!(matches!(root("mods/Flat.jai"), Root::Module { name, .. } if name == "Flat"));
    }

    /// Where the linter and the language server are meant to agree, they do; the tie-break is
    /// the option that makes them differ.
    #[test]
    fn linter_and_language_server_agree_on_the_same_tree() {
        let dir = fixture(
            "agree",
            &[
                ("main.jai", "#load \"a.jai\";"),
                ("a.jai", "#load \"deep/b.jai\";"),
                ("deep/b.jai", ""),
                ("lone.jai", ""),
                ("Thing/module.jai", "#load \"part.jai\";"),
                ("Thing/part.jai", ""),
            ],
        );
        let linter = RootOptions::linter(Vec::new());
        let server = RootOptions::language_server(Some(dir.clone()));
        for file in [
            "deep/b.jai",
            "a.jai",
            "main.jai",
            "lone.jai",
            "Thing/part.jai",
        ] {
            let file = dir.join(file);
            let by_linter = resolve_root(&DiskFs, &file, &linter, &mut LoadCache::default());
            let by_server = resolve_root(&DiskFs, &file, &server, &mut LoadCache::default());
            assert_eq!(by_linter.path(), by_server.path(), "{}", file.display());
        }
    }

    #[test]
    fn tie_breaks_differ_when_loaders_disagree() {
        // `sub/leaf.jai` is loaded by `sub/zeta.jai` and by `top.jai`, neither of which loads
        // the other. The linter's rule takes the nearest-first, first-by-path outermost loader;
        // the language server's takes the shallowest.
        let dir = fixture(
            "tie",
            &[
                ("top.jai", "#load \"sub/leaf.jai\";"),
                ("sub/zeta.jai", "#load \"leaf.jai\";"),
                ("sub/leaf.jai", ""),
            ],
        );
        let leaf = dir.join("sub/leaf.jai");
        let resolve = |options: &RootOptions| {
            resolve_root(&DiskFs, &leaf, options, &mut LoadCache::default())
        };
        let linter = RootOptions::linter(Vec::new());
        let server = RootOptions::language_server(Some(dir.clone()));
        assert_eq!(resolve(&linter), Root::Program(dir.join("sub/zeta.jai")));
        assert_eq!(resolve(&server), Root::Program(dir.join("top.jai")));
        // The tie-break is an option of its own, whichever search finds the loaders.
        let mut mixed = server.clone();
        mixed.tie_break = TieBreak::Outermost;
        assert_eq!(resolve(&mixed), Root::Program(dir.join("sub/zeta.jai")));
        let mut mixed = linter.clone();
        mixed.tie_break = TieBreak::ModuleFirstShallowest;
        assert_eq!(resolve(&mixed), Root::Program(dir.join("top.jai")));
    }
}
