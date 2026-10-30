//! Queries over a finished analysis: diagnostics, hovers, names and completions.
use super::compile::find_file;
use super::{Analysis, Decl, HoverFacts, SemanticAnalysisContext, SemanticCoverage, SemanticPath};
use jaic::fxhash::HashSet;
use jaic::ide::IdeName;
use jaic::intern::Sym;
use jaic::source::{FileId, Suggestion};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// How many of a program's first errors get their render-time help (see
/// `Analysis::check_findings`) for the editor.
const ENRICHED_ERRORS: usize = 50;

/// A compile error or warning placed in one file.
pub struct CheckFinding {
    /// The byte range it is shown at.
    pub start: usize,
    pub end: usize,
    /// The message alone: what the syntax layer's duplicate is compared with.
    pub message: String,
    /// The message with its unlocated notes and help lines, as the editor shows it.
    pub display: String,
    pub warning: bool,
    /// Notes that have a location.
    pub related: Vec<Related>,
    /// What can fix it, for the quick fixes.
    pub suggestions: Vec<Suggestion>,
}

/// A located note of a finding: a place in any compiled file.
pub struct Related {
    pub path: String,
    pub text: Rc<str>,
    pub start: usize,
    pub end: usize,
    pub message: String,
}

/// What a compile-time evaluation (a `#run`, a constant that needed the interpreter...) of the
/// program cost, summed over the times the compile ran it.
pub struct EvalCost {
    pub path: String,
    pub text: Rc<str>,
    /// The byte range of the code that was evaluated.
    pub start: usize,
    pub end: usize,
    pub blocks: u64,
    pub elapsed: std::time::Duration,
}

/// Which limit stopped compile-time code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LimitKind {
    /// One evaluation ran out of its block budget.
    Budget,
    /// The whole check ran past its time limit.
    Time,
}

/// A compile-time evaluation or check that a limit stopped, found by a compile.
pub struct LimitHit {
    pub kind: LimitKind,
    pub path: String,
    pub text: Rc<str>,
    pub start: usize,
    pub end: usize,
    /// The first line of the diagnostic.
    pub message: String,
}

/// The limits `compiler` and `errors` (a compile's) ran into.
pub(super) fn limit_hits(
    compiler: &jaic::sema::Compiler,
    errors: &[jaic::source::Diagnostic],
) -> Vec<LimitHit> {
    let found = errors
        .iter()
        .filter(|d| d.message.contains("execution budget exhausted"))
        .map(|d| (LimitKind::Budget, d))
        .chain(
            compiler
                .diag
                .warnings
                .iter()
                .filter(|d| d.message.starts_with("checking stopped after"))
                .map(|d| (LimitKind::Time, d)),
        );
    found
        .filter_map(|(kind, d)| {
            let file = compiler.src.sources.try_get(d.span.file)?;
            Some(LimitHit {
                kind,
                path: file.path.to_string(),
                text: file.text.clone(),
                start: d.span.start as usize,
                end: d.span.end as usize,
                message: d.message.lines().next().unwrap_or_default().to_string(),
            })
        })
        .collect()
}

impl Analysis {
    /// The top-level compile-time evaluations of the program and of the programs its
    /// metaprogram built, costliest first. A run inside another counts in the one around it.
    pub fn evaluation_costs(&self) -> Vec<EvalCost> {
        let mut sums: BTreeMap<(String, usize, usize), EvalCost> = BTreeMap::new();
        for (compiler, _) in self.polymorph_compilers() {
            for run in &compiler.meta.perf.runs {
                let (Some(blocks), true) = (run.blocks, run.outermost) else {
                    continue;
                };
                let Some(file) = compiler.src.sources.try_get(run.span.file) else {
                    continue;
                };
                let path = file.path.to_string();
                let key = (path.clone(), run.span.start as usize, run.span.end as usize);
                let entry = sums.entry(key).or_insert_with(|| EvalCost {
                    path,
                    text: file.text.clone(),
                    start: run.span.start as usize,
                    end: run.span.end as usize,
                    blocks: 0,
                    elapsed: std::time::Duration::ZERO,
                });
                entry.blocks += blocks;
                entry.elapsed += run.elapsed;
            }
        }
        let mut costs: Vec<EvalCost> = sums.into_values().collect();
        costs.sort_by_key(|c| std::cmp::Reverse((c.blocks, c.elapsed)));
        costs
    }

    /// The paths the compile asked about so far, and a hash of each open text among them.
    pub(crate) fn inputs_read(&self) -> (super::Reads, BTreeMap<PathBuf, u64>) {
        let reads = self.reads.borrow().clone();
        let buffers = self
            .files
            .iter()
            .filter(|(path, _)| reads.concerns(path))
            .map(|(path, bytes)| (path.clone(), super::text_hash(bytes)))
            .collect();
        (reads, buffers)
    }

    pub(crate) fn polymorph_compilers(
        &self,
    ) -> impl Iterator<Item = (&jaic::sema::Compiler, bool)> {
        std::iter::once((&self.compiler, self.complete))
            .chain(self.others.iter().map(|p| (&p.compiler, p.complete)))
    }

    /// Answer from the program that compiled `path`: the compiled root's own, or one a
    /// metaprogram built. The answering program stays when it compiled `path` too (a module
    /// both import).
    pub fn focus(&mut self, path: &Path) {
        if self.others.is_empty() || self.file(path).is_some() {
            return;
        }
        let Some(at) = self
            .others
            .iter()
            .position(|p| find_file(&p.compiler, path).is_some())
        else {
            return;
        };
        let other = &mut self.others[at];
        std::mem::swap(&mut self.compiler, &mut other.compiler);
        std::mem::swap(&mut self.entry, &mut other.entry);
        std::mem::swap(&mut self.complete, &mut other.complete);
        std::mem::swap(&mut self.errors, &mut other.errors);
        std::mem::swap(&mut self.lints, &mut other.lints);
        std::mem::swap(&mut self.enriched, &mut other.enriched);
        self.found.borrow_mut().clear();
    }

    /// jailint's findings in `path` (computed once per compile).
    pub fn lints(
        &mut self,
        path: &Path,
        config: &jailint::config::Config,
    ) -> Option<Vec<jailint::Lint>> {
        let file = self.file(path)?;
        if let Some(found) = self.lints.get(&file) {
            return Some(found.clone());
        }
        let found = jailint::lint_files(
            &mut self.compiler,
            &[file],
            &jailint::config::Configs::new(config),
            self.complete,
        );
        self.lints.insert(file, found.clone());
        Some(found)
    }

    /// The spans (start, end) in `path` of the compile errors of `kind`.
    pub fn check_error_spans(
        &self,
        path: &Path,
        kind: jaic::source::DiagnosticKind,
    ) -> Vec<(usize, usize)> {
        let Some(file) = self.file(path) else {
            return Vec::new();
        };
        self.errors
            .iter()
            .filter(|e| e.kind == kind && e.span.file == file)
            .map(|e| (e.span.start as usize, e.span.end as usize))
            .collect()
    }

    /// The compile errors and warnings located in `path`, each at its own span when that is in
    /// the file, else at the first of its notes that is (a module procedure the file called).
    /// Errors come first, then warnings.
    pub fn check_findings(&mut self, path: &Path) -> Vec<CheckFinding> {
        let Some(file) = self.file(path) else {
            return Vec::new();
        };
        if self.enriched.is_none() {
            // The render-time help (similar names, the imports that declare a name) of the
            // first errors: working out each one reads the standard library.
            let enriched = self
                .errors
                .iter()
                .enumerate()
                .map(|(i, e)| {
                    let wanted = i < ENRICHED_ERRORS;
                    wanted
                        .then(|| self.compiler.with_name_suggestion(e))
                        .flatten()
                        .unwrap_or_else(|| e.clone())
                })
                .collect();
            self.enriched = Some(enriched);
        }
        let warnings: Vec<_> = self
            .compiler
            .diag
            .warnings
            .iter()
            .map(|w| {
                self.compiler
                    .with_name_suggestion(w)
                    .unwrap_or_else(|| w.clone())
            })
            .collect();
        let errors = self.enriched.as_deref().unwrap_or_default();
        let mut out: Vec<CheckFinding> = Vec::new();
        let mut seen = HashSet::default();
        let tagged = errors
            .iter()
            .map(|d| (d, false))
            .chain(warnings.iter().map(|d| (d, true)));
        for (d, warning) in tagged {
            let Some(finding) = self.finding(file, d, warning) else {
                continue;
            };
            if seen.insert((finding.start, finding.end, finding.display.clone())) {
                out.push(finding);
            }
        }
        out
    }

    fn finding(
        &self,
        file: FileId,
        d: &jaic::source::Diagnostic,
        warning: bool,
    ) -> Option<CheckFinding> {
        let files = &self.compiler.src.sources;
        let here = d.span.file == file;
        // A diagnostic elsewhere shows at its first note in this file, with that note's text.
        let at_note = (!here)
            .then(|| d.notes.iter().find(|(span, _)| span.file == file))
            .flatten();
        let (span, base) = match at_note {
            Some((span, note)) => (*span, format!("{}\n{note}", d.message)),
            None if here => (d.span, d.message.clone()),
            None => return None,
        };
        let mut display = base.clone();
        let mut related = Vec::new();
        for (span, note) in &d.notes {
            if at_note.is_some_and(|(at, _)| at == span) {
                continue;
            }
            match files.try_get(span.file) {
                Some(found) => related.push(Related {
                    path: found.path.to_string(),
                    text: found.text.clone(),
                    start: span.start as usize,
                    end: span.end as usize,
                    message: note.clone(),
                }),
                None => display.push_str(&format!("\nnote: {note}")),
            }
        }
        for suggestion in d.suggestions().iter().filter(|s| s.listed) {
            display.push_str(&format!("\nhelp: {}", suggestion.title));
        }
        for help in &d.help {
            display.push_str(&format!("\nhelp: {help}"));
        }
        // Edits in another file cannot be applied to this one, and an import goes to the file
        // the diagnostic is in.
        let suggestions = d
            .suggestions()
            .iter()
            .filter(|s| {
                s.replacements().all(|(span, _)| span.file == file)
                    && (here || s.imports().next().is_none())
            })
            .cloned()
            .collect();
        Some(CheckFinding {
            start: span.start as usize,
            end: span.end as usize,
            message: base,
            display,
            warning,
            related,
            suggestions,
        })
    }

    /// `offset` of `path` is code this compile skipped for its target: the file is not part of
    /// it, or an `#if` on `OS` or `CPU` did not take the branch it is in.
    pub fn skipped(&self, path: &Path, offset: usize) -> bool {
        self.skipped_span(path, offset).is_some()
    }

    /// The byte range around `offset` of `path` this compile did not check: an `#if` branch on
    /// `OS` or `CPU` it did not take, or the whole file when the compile did not reach it.
    pub fn skipped_span(&self, path: &Path, offset: usize) -> Option<(usize, usize)> {
        let Some(file) = self.file(path) else {
            return Some((0, usize::MAX));
        };
        self.compiler.ed.facts().and_then(|ide| {
            ide.skipped
                .iter()
                .find(|s| s.file == file && s.start as usize <= offset && offset <= s.end as usize)
                .map(|s| (s.start as usize, s.end as usize))
        })
    }

    /// Byte ranges of `path` no path through the program reaches: code after an exit, in a
    /// branch or loop its condition rules out, and `#if` or `ifx` arms no evaluation took.
    /// A procedure analysed once per instance counts only where every instance agrees.
    pub fn unreachable(&self, path: &Path) -> Vec<(usize, usize)> {
        let (Some(file), Some(ide)) = (self.file(path), self.compiler.ed.facts()) else {
            return Vec::new();
        };
        let dead = ide
            .dead_runs
            .iter()
            .filter(|((body, _), count)| ide.body_evals.get(body) == Some(*count))
            .map(|((_, run), _)| run);
        // An `#if` in a polymorphic procedure may look dead only because the callers so far
        // all pass types that rule an arm out.
        let polymorphic = self.compiler.ide_polymorphic_bodies();
        let inactive = ide
            .static_branches
            .iter()
            .filter(|(_, ran)| !**ran)
            .map(|(s, _)| s)
            .filter(|s| {
                !polymorphic
                    .iter()
                    .any(|b| b.file == s.file && b.start <= s.start && s.end <= b.end)
            });
        let mut found: Vec<(usize, usize)> = dead
            .chain(inactive)
            .filter(|s| s.file == file && s.start < s.end)
            .map(|s| (s.start as usize, s.end as usize))
            .collect();
        found.sort_unstable_by_key(|&(start, end)| (start, std::cmp::Reverse(end)));
        // A dead run inside another needs no range of its own.
        let mut kept: Vec<(usize, usize)> = Vec::new();
        for range in found {
            if kept.last().is_none_or(|last| range.1 > last.1) {
                kept.push(range);
            }
        }
        kept
    }

    pub fn file(&self, path: &Path) -> Option<FileId> {
        if let Some(&found) = self.found.borrow().get(path) {
            return found;
        }
        let found = find_file(&self.compiler, path);
        self.found.borrow_mut().insert(path.to_path_buf(), found);
        found
    }

    pub fn coverage(&self, path: &Path) -> SemanticCoverage {
        let file = self.file(path);
        let ide = self.compiler.ed.facts();
        SemanticCoverage {
            context_id: format!("{:016x}", self.key),
            analysis: SemanticAnalysisContext {
                root: Some(SemanticPath::observed(self.root.clone())),
                entry: self.entry.clone().map(SemanticPath::observed),
                import_dirs: Some(
                    self.compiler
                        .options
                        .import_paths
                        .iter()
                        .cloned()
                        .map(SemanticPath::observed)
                        .collect(),
                ),
                workspace_id: Some(self.compiler.meta.workspace),
            },
            source_loaded: file.is_some(),
            recorded_references: ide.map_or(0, |i| {
                i.refs.iter().filter(|r| Some(r.span.file) == file).count()
            }),
            recorded_fields: ide.map_or(0, |i| {
                i.fields.keys().filter(|s| Some(s.file) == file).count()
            }),
            program_complete: self.complete,
            program_errors: self.errors.len(),
            target: format!(
                "{:?}/{:?}",
                self.compiler.options.os, self.compiler.options.cpu
            ),
        }
    }

    /// A span of any compiled file as (path, text, start, end); `None` for a span in no file
    /// (`Span::NONE`, which builtins carry).
    pub fn location(&self, span: jaic::source::Span) -> Option<(String, Rc<str>, usize, usize)> {
        let source = self.compiler.src.sources.try_get(span.file)?;
        Some((
            source.path.to_string(),
            source.text.clone(),
            span.start as usize,
            span.end as usize,
        ))
    }

    /// Hover at `offset`: what it covers, its description and memory layout, and where what
    /// it describes is declared (one per line of the description for overloads).
    pub fn hover(&mut self, path: &Path, offset: usize) -> Option<HoverFacts> {
        let file = self.file(path)?;
        let hover = self.compiler.ide_hover(file, offset as u32)?;
        let spans = self.compiler.ide_hover_decls(file, offset as u32);
        let decls = spans
            .into_iter()
            .map(|span| self.compiler.ide_doc_site(span))
            .collect::<Vec<_>>()
            .into_iter()
            .filter_map(|span| self.location(span))
            .collect();
        Some(HoverFacts {
            start: hover.span.start as usize,
            end: hover.span.end as usize,
            text: hover.text,
            layout: hover.layout,
            decls,
        })
    }

    /// The declarations `names` (`print`, `Basic.print`, `Type.field`) name as seen from
    /// `offset` of the file at `path`: where doc comment links lead. Falls back to the file's
    /// own scope when the file has no recorded scopes.
    pub fn resolve_names(&mut self, path: &Path, offset: usize, names: &[String]) -> Vec<Decl> {
        let Some(file) = self.file(path) else {
            return Vec::new();
        };
        let Some(scope) = self.compiler.ide_scope_at(file, offset as u32) else {
            return Vec::new();
        };
        let names: Vec<Sym> = names.iter().map(|n| Sym::intern(n)).collect();
        self.compiler
            .ide_resolve_path(scope, &names)
            .into_iter()
            .filter_map(|span| self.location(span))
            .collect()
    }

    /// The path of each file `names` are declared in, by file number.
    /// Moves the declarations of `names` in the file at `path` from the text this analysis
    /// compiled to where they are in `text`, the document as it is now: past the edited
    /// stretch (the longest common prefix and suffix of the two) by the change in length.
    pub fn shift_to(&self, path: &Path, text: &str, names: &mut [IdeName]) {
        let Some(file) = self.file(path) else {
            return;
        };
        let Some(source) = self.compiler.src.sources.try_get(file) else {
            return;
        };
        let (old, new) = (source.text.as_bytes(), text.as_bytes());
        if old == new {
            return;
        }
        let prefix = old.iter().zip(new).take_while(|(a, b)| a == b).count();
        let room = old.len().min(new.len()) - prefix;
        let suffix = old
            .iter()
            .rev()
            .zip(new.iter().rev())
            .take(room)
            .take_while(|(a, b)| a == b)
            .count();
        let tail = (old.len() - suffix) as u32;
        let delta = new.len() as i64 - old.len() as i64;
        let moved = |at: u32| {
            if at >= tail {
                (at as i64 + delta) as u32
            } else {
                at
            }
        };
        for name in names.iter_mut() {
            for span in std::iter::once(&mut name.decl).chain(name.docs.as_mut()) {
                if span.file == file {
                    span.start = moved(span.start);
                    span.end = moved(span.end);
                }
            }
        }
    }

    /// Notes, on each of `names` that is an alias with no docs of its own, the declaration it
    /// names, where its docs are written.
    pub fn forward_docs(&mut self, names: &mut [IdeName]) {
        for name in names {
            let site = self.compiler.ide_doc_site(name.decl);
            name.docs = (site != name.decl).then_some(site);
        }
    }

    pub fn decl_files(&self, names: &[IdeName]) -> BTreeMap<u32, String> {
        let mut out = BTreeMap::new();
        for name in names {
            for file in std::iter::once(name.decl.file).chain(name.docs.map(|d| d.file)) {
                if let std::collections::btree_map::Entry::Vacant(slot) = out.entry(file.0)
                    && let Some(source) = self.compiler.src.sources.try_get(file)
                {
                    slot.insert(source.path.to_string());
                }
            }
        }
        out
    }

    /// Declarations the identifier at `offset` names, as (path, text, start, end) with byte
    /// offsets into that file's text (which may be a module or stdlib file, not an open document).
    pub fn definition(
        &mut self,
        path: &Path,
        offset: usize,
    ) -> Vec<(String, Rc<str>, usize, usize)> {
        let Some(file) = self.file(path) else {
            return Vec::new();
        };
        self.compiler
            .ide_definition(file, offset as u32)
            .into_iter()
            .filter_map(|span| self.location(span))
            .collect()
    }

    /// The members of the enum an inferred `.NAME` at `offset` is expected to be.
    pub fn complete_inferred(&mut self, path: &Path, offset: usize) -> Option<Vec<IdeName>> {
        let file = self.file(path)?;
        Some(self.compiler.ide_inferred_members(file, offset as u32))
    }

    /// The fields of the struct literal whose field name starts at `offset`.
    pub fn complete_literal_fields(&mut self, path: &Path, offset: usize) -> Option<Vec<IdeName>> {
        let file = self.file(path)?;
        Some(self.compiler.ide_literal_fields(file, offset as u32))
    }

    /// Of the names visible at `offset`, those whose type fits the one the placeholder `.NAME`
    /// at `offset + 1` was expected to have; `None` when that is not known.
    pub fn complete_fitting(&mut self, path: &Path, offset: usize) -> Option<BTreeSet<String>> {
        let file = self.file(path)?;
        let scope = self.compiler.ide_scope_at(file, offset as u32)?;
        let names = self.compiler.ide_visible(scope, file, offset as u32);
        let fitting = self
            .compiler
            .ide_fitting_names(file, offset as u32 + 1, &names)?;
        Some(fitting.into_iter().collect())
    }

    /// The members of the value whose member name starts at `offset`, by the type the compile
    /// recorded for it.
    pub fn complete_recorded(&mut self, path: &Path, offset: usize) -> Option<Vec<IdeName>> {
        let file = self.file(path)?;
        self.compiler.ide_recorded_members(file, offset as u32)
    }

    /// Completion candidates at `offset`, members of `chain` when it is not empty.
    pub fn complete(&mut self, path: &Path, offset: usize, chain: &[&str]) -> Option<Vec<IdeName>> {
        let file = self.file(path)?;
        let scope = self.compiler.ide_scope_at(file, offset as u32)?;
        if chain.is_empty() {
            return Some(self.compiler.ide_visible(scope, file, offset as u32));
        }
        let names: Vec<Sym> = chain.iter().map(|n| Sym::intern(n)).collect();
        // A polymorphic body is checked once per instance, and the receiver is a different type
        // in each (`a.text` where `T` is `Numeric`, `a.count` where it is a string): offer the
        // members of all of them.
        let mut scopes = self.compiler.ide_innermost_scopes_at(file, offset as u32);
        if !scopes.contains(&scope) {
            scopes.insert(0, scope);
        }
        let mut members: Option<Vec<IdeName>> = None;
        for scope in scopes {
            let Some(receiver) =
                self.compiler
                    .ide_receiver_at(scope, Some((file, offset as u32)), &names)
            else {
                continue;
            };
            let found = self.compiler.ide_members(receiver);
            let all = members.get_or_insert_with(Vec::new);
            for name in found {
                if !all.contains(&name) {
                    all.push(name);
                }
            }
        }
        members
    }
}
