//! Migrations: rewrites that move code written for an older Jai to the current language.
//!
//! A migration has a feature name (`context-type-directive`), never a version. Each one knows
//! the old form by itself, so `plan` needs no idea of which language the code was written
//! for: it runs every migration and keeps what applies.
//!
//! Safety comes from what a migration may touch. A *token* migration rewrites a form the
//! current language rejects outright (a removed directive), and an *error* migration answers a
//! compile error the current compiler reports on the old form (an identifier that no longer
//! exists). Neither can change code that already means something today. A migration whose old
//! form is also valid current code with another meaning is `ambiguous` and runs only when
//! named.
//!
//! `plan` works in memory: it edits, recompiles over an overlay of the edited text, and
//! repeats until nothing changes. Its [`Plan`] says what would change and whether the result
//! compiles at least as well as the original: a migration whose rewrites add an error is left
//! out and named in [`Plan::rejected`]. Writing the
//! files is the caller's job (`jacket migrate --apply`).
mod rules;

use crate::Edit;
use crate::driver::{self, CompileError, Options};
use jaic::lexer::Token;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub use rules::MIGRATIONS;

/// Edit, recompile and edit again at most this many times.
const MAX_PASSES: usize = 8;

/// One rewrite from an old form to the current one.
pub struct Migration {
    /// What changed, in kebab-case (`context-type-directive`).
    pub name: &'static str,
    /// One line: the old form and the new one.
    pub summary: &'static str,
    /// The old form can also be valid current code that means something else: the migration
    /// runs only when named (`--only`).
    pub ambiguous: bool,
    detect: Detect,
}

enum Detect {
    /// Rewrites tokens of a file before it is compiled.
    Tokens(fn(&Source) -> Vec<Answer>),
    /// Answers each compile error with one of these codes.
    Error {
        codes: &'static [&'static str],
        answer: fn(&CompileError, &Source) -> Option<Answer>,
    },
}

/// A file as a migration sees it.
pub struct Source<'a> {
    pub path: &'a Path,
    pub text: &'a str,
    /// The tokens the current lexer reads; a malformed literal is skipped, not fatal.
    pub tokens: &'a [Token],
    /// Where `#import "Name"` looks, in order: the file's `modules` folder, the import
    /// directories, the standard library.
    pub module_dirs: &'a [PathBuf],
}

impl Source<'_> {
    /// The text of bytes `start..end`.
    fn slice(&self, start: usize, end: usize) -> &str {
        self.text.get(start..end).unwrap_or_default()
    }

    /// Whether `#import "name"` would find a module from this file.
    fn module_exists(&self, name: &str) -> bool {
        self.module_dirs.iter().any(|dir| {
            dir.join(format!("{name}.jai")).is_file() || dir.join(name).join("module.jai").is_file()
        })
    }
}

/// What a migration does about one place.
enum Answer {
    /// Rewrite it.
    Edit {
        start: usize,
        end: usize,
        text: String,
    },
    /// No rewrite can say what the code meant: tell the user what to do instead.
    Manual {
        start: usize,
        end: usize,
        message: String,
    },
}

/// Which migrations run.
#[derive(Clone, Debug, Default)]
pub struct Selection {
    /// Only these, ambiguous ones included. `None`: every migration that is not ambiguous.
    pub only: Option<Vec<String>>,
}

impl Selection {
    /// The migrations this selects, or the first unknown name.
    fn migrations(&self) -> Result<Vec<&'static Migration>, String> {
        let Some(only) = &self.only else {
            return Ok(MIGRATIONS.iter().filter(|m| !m.ambiguous).collect());
        };
        only.iter()
            .map(|name| {
                MIGRATIONS
                    .iter()
                    .find(|m| m.name == name)
                    .ok_or_else(|| unknown_migration(name))
            })
            .collect()
    }
}

fn unknown_migration(name: &str) -> String {
    let names: Vec<&str> = MIGRATIONS.iter().map(|m| m.name).collect();
    match jaic::suggest::closest(name, names.iter().copied()) {
        Some(near) => format!("no migration is called `{name}`\nhelp: did you mean `{near}`?"),
        None => {
            format!("no migration is called `{name}`\nhelp: `jacket migrate --list` lists them")
        }
    }
}

/// One rewrite `plan` made.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Applied {
    pub migration: &'static str,
    /// Byte offset in the original text.
    pub start: usize,
}

/// A file `plan` would change.
#[derive(Clone, Debug)]
pub struct Migrated {
    pub path: PathBuf,
    pub before: Arc<str>,
    pub after: String,
    pub applied: Vec<Applied>,
}

/// A place a migration found but could not rewrite.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Manual {
    pub migration: &'static str,
    pub path: PathBuf,
    /// Byte range in the text `plan` ended with ([`Migrated::after`], or the file as it is).
    pub start: usize,
    pub end: usize,
    pub message: String,
}

/// A migration whose rewrites were left out because they made the code fail where it
/// compiled before.
#[derive(Clone, Debug)]
pub struct Rejected {
    pub migration: &'static str,
    /// The errors its rewrites introduced, as the compiler reports them on the rewritten code.
    pub errors: Vec<RejectedError>,
}

/// An error in rewritten code that is not written anywhere, so it carries its own position.
#[derive(Clone, Debug)]
pub struct RejectedError {
    pub path: PathBuf,
    /// One-based line and column in the rewritten text.
    pub line: usize,
    pub column: usize,
    pub code: &'static str,
    pub message: String,
}

/// What migrating would do.
#[derive(Debug, Default)]
pub struct Plan {
    pub files: Vec<Migrated>,
    pub manual: Vec<Manual>,
    /// Migrations that were wrong for this code. Their rewrites are not in `files`; the
    /// rewrites of the others are.
    pub rejected: Vec<Rejected>,
    /// Errors left in the migrated code that it shared with the original.
    pub remaining: Vec<CompileError>,
    /// Files that are not UTF-8 text, which no migration reads or edits.
    pub skipped: Vec<PathBuf>,
}

/// The text of every file a plan edits, and what was done to it so far.
#[derive(Clone)]
struct Work {
    /// The text read from disk, then the text after the edits so far.
    texts: BTreeMap<PathBuf, (Arc<str>, String)>,
    /// Applied rewrites, in the original text's offsets.
    applied: BTreeMap<PathBuf, Vec<Applied>>,
    shifts: BTreeMap<PathBuf, Vec<(usize, isize)>>,
}

type Edits = BTreeMap<PathBuf, Vec<(Edit, &'static str)>>;

impl Trial {
    /// The introduced errors with their positions in this trial's text.
    fn rejected_errors(&self) -> Vec<RejectedError> {
        self.introduced
            .iter()
            .map(|e| {
                let text = self.work.texts.get(&e.path).map_or("", |(_, t)| t.as_str());
                let before = text.get(..e.start).unwrap_or(text);
                RejectedError {
                    path: e.path.clone(),
                    line: before.matches('\n').count() + 1,
                    column: before.rsplit('\n').next().map_or(0, |l| l.chars().count()) + 1,
                    code: e.code,
                    message: e.message.clone(),
                }
            })
            .collect()
    }
}

impl Work {
    /// Apply the edits that fit together; overlapping ones wait for the next pass.
    fn apply(&mut self, edits: Edits) {
        for (path, file_edits) in edits {
            let Some((_, text)) = self.texts.get_mut(&path) else {
                continue;
            };
            let chosen = non_overlapping(file_edits);
            let file_shifts = self.shifts.entry(path.clone()).or_default();
            let file_applied = self.applied.entry(path.clone()).or_default();
            for (edit, name) in &chosen {
                file_applied.push(Applied {
                    migration: name,
                    start: original_offset(file_shifts, edit.start),
                });
            }
            for (edit, _) in &chosen {
                file_shifts.push((
                    edit.start,
                    edit.text.len() as isize - (edit.end - edit.start) as isize,
                ));
            }
            *text = crate::fix::apply_edits(text, chosen.into_iter().map(|(e, _)| e).collect());
        }
    }

    /// The edited files as an overlay for the compiler.
    fn overlay(&self) -> BTreeMap<PathBuf, Arc<[u8]>> {
        self.texts
            .iter()
            .filter(|(_, (before, after))| **before != **after)
            .map(|(path, (_, after))| (path.clone(), Arc::from(after.as_bytes())))
            .collect()
    }
}

/// The result of compiling a [`Work`] after some edits.
struct Trial {
    work: Work,
    errors: Vec<CompileError>,
    /// The errors the original code did not have.
    introduced: Vec<CompileError>,
}

/// Work out the migrations of the files `options` names, without writing anything.
pub fn plan(options: Options, selection: &Selection) -> Result<Plan, String> {
    let migrations = selection.migrations()?;
    plan_with(options, &migrations)
}

fn plan_with(mut options: Options, migrations: &[&'static Migration]) -> Result<Plan, String> {
    options.overlay.clear();
    let first = driver::run(&options);
    let mut work = Work {
        texts: BTreeMap::new(),
        applied: BTreeMap::new(),
        shifts: BTreeMap::new(),
    };
    let mut skipped = Vec::new();
    for path in &first.files {
        let bytes = std::fs::read(path)
            .map_err(|e| format!("cannot read `{}`: {}", path.display(), jaic::io_reason(&e)))?;
        let Ok(text) = String::from_utf8(bytes) else {
            skipped.push(path.clone());
            continue;
        };
        let before: Arc<str> = Arc::from(text);
        let after = before.to_string();
        work.texts.insert(path.clone(), (before, after));
    }
    let mut manual = Vec::new();
    let mut rejected: Vec<Rejected> = Vec::new();
    let mut active: Vec<&'static Migration> = migrations.to_vec();
    let mut errors = first.errors.clone();
    let trial = |options: &mut Options, work: &Work, edits: Edits| {
        let mut next = work.clone();
        next.apply(edits);
        options.overlay = next.overlay();
        let errors = driver::run(options).errors;
        let (introduced, _) = compare(&first.errors, &errors);
        Trial {
            work: next,
            errors,
            introduced,
        }
    };
    for pass in 0..MAX_PASSES {
        let (edits, found) = detect(&work, &errors, &options, &active);
        manual = found;
        if edits.is_empty() {
            break;
        }
        if pass + 1 == MAX_PASSES {
            return Err(format!(
                "the migrations did not settle after {MAX_PASSES} passes; this is a bug in a migration, please report it"
            ));
        }
        let all = trial(&mut options, &work, edits.clone());
        if all.introduced.is_empty() {
            work = all.work;
            errors = all.errors;
            continue;
        }
        // Something in this pass is wrong. Try each migration on its own to find which, and keep
        // the ones that introduce nothing.
        let mut names: Vec<&'static str> = Vec::new();
        for (_, name) in edits.values().flatten() {
            if !names.contains(name) {
                names.push(name);
            }
        }
        let only = |keep: &dyn Fn(&str) -> bool| -> Edits {
            edits
                .iter()
                .map(|(path, list)| {
                    let list: Vec<_> = list.iter().filter(|(_, n)| keep(n)).cloned().collect();
                    (path.clone(), list)
                })
                .filter(|(_, list)| !list.is_empty())
                .collect()
        };
        let mut good: Vec<Trial> = Vec::new();
        let mut good_names: Vec<&'static str> = Vec::new();
        if names.len() == 1 {
            rejected.push(Rejected {
                migration: names[0],
                errors: all.rejected_errors(),
            });
        } else {
            for name in &names {
                let own = trial(&mut options, &work, only(&|n| n == *name));
                if own.introduced.is_empty() {
                    good_names.push(name);
                    good.push(own);
                } else {
                    rejected.push(Rejected {
                        migration: name,
                        errors: own.rejected_errors(),
                    });
                }
            }
        }
        active.retain(|m| !rejected.iter().any(|r| r.migration == m.name));
        // The accepted migrations together, or the first alone when they disagree.
        if good.len() > 1 {
            let together = trial(&mut options, &work, only(&|n| good_names.contains(&n)));
            if together.introduced.is_empty() {
                good = vec![together];
            }
        }
        if let Some(next) = good.into_iter().next() {
            work = next.work;
            errors = next.errors;
        }
    }
    let (_, remaining) = compare(&first.errors, &errors);
    let files = work
        .texts
        .into_iter()
        .filter(|(_, (before, after))| **before != **after)
        .map(|(path, (before, after))| {
            let mut file_applied = work.applied.remove(&path).unwrap_or_default();
            file_applied.sort_by_key(|a| a.start);
            Migrated {
                path,
                before,
                after,
                applied: file_applied,
            }
        })
        .collect();
    Ok(Plan {
        files,
        manual,
        rejected,
        remaining,
        skipped,
    })
}

/// What `migrations` want to change in the files of `work`, and what they can only point out.
fn detect(
    work: &Work,
    errors: &[CompileError],
    options: &Options,
    migrations: &[&'static Migration],
) -> (Edits, Vec<Manual>) {
    let mut edits = Edits::new();
    let mut manual = Vec::new();
    for (path, (_, text)) in &work.texts {
        let (tokens, _) = jaic::lexer::lex_ide(jaic::source::FileId(0), text);
        // The `modules` folder beside the main file: the file's own folder, or a folder the
        // run was pointed at.
        let main_dirs = options.paths.iter().map(|p| {
            if p.is_dir() {
                p.clone()
            } else {
                p.parent().map(Path::to_path_buf).unwrap_or_default()
            }
        });
        let module_dirs: Vec<PathBuf> = path
            .parent()
            .map(Path::to_path_buf)
            .into_iter()
            .chain(main_dirs)
            .map(|dir| dir.join("modules"))
            .chain(options.import_dirs.iter().cloned())
            .chain([options.stdlib.clone()])
            .collect();
        let source = Source {
            path,
            text,
            tokens: &tokens,
            module_dirs: &module_dirs,
        };
        for m in migrations {
            let answers = match m.detect {
                Detect::Tokens(detect) => detect(&source),
                Detect::Error {
                    codes,
                    answer,
                } => errors
                    .iter()
                    .filter(|e| e.path == *path && codes.contains(&e.code))
                    .filter_map(|e| answer(e, &source))
                    .collect(),
            };
            for a in answers {
                match a {
                    Answer::Edit {
                        start,
                        end,
                        text,
                    } => edits.entry(path.clone()).or_default().push((
                        Edit {
                            start,
                            end,
                            text,
                        },
                        m.name,
                    )),
                    Answer::Manual {
                        start,
                        end,
                        message,
                    } => manual.push(Manual {
                        migration: m.name,
                        path: path.clone(),
                        start,
                        end,
                        message,
                    }),
                }
            }
        }
    }
    (edits, manual)
}

/// The edits of one file that can go together, in order; the rest wait for the next pass.
fn non_overlapping(mut edits: Vec<(Edit, &'static str)>) -> Vec<(Edit, &'static str)> {
    edits.sort_by_key(|(e, _)| (e.start, e.end));
    edits.dedup_by(|(a, _), (b, _)| a == b);
    let mut chosen: Vec<(Edit, &'static str)> = Vec::new();
    for (edit, name) in edits {
        if chosen
            .last()
            .is_some_and(|(last, _)| edit.start < last.end.max(last.start + 1))
        {
            continue;
        }
        chosen.push((edit, name));
    }
    chosen
}

/// Where `offset` in the current text was in the original, given the earlier passes' edits
/// as (start, change in length).
fn original_offset(shifts: &[(usize, isize)], offset: usize) -> usize {
    let mut at = offset as isize;
    for &(start, delta) in shifts.iter().rev() {
        if (start as isize) < at {
            at -= delta;
        }
    }
    at.max(0) as usize
}

/// Split `after`'s errors into those the original did not have and those it did. Errors are
/// matched by file, code and message, since migrated text moves them.
fn compare(
    before: &[CompileError],
    after: &[CompileError],
) -> (Vec<CompileError>, Vec<CompileError>) {
    let mut left: Vec<&CompileError> = before.iter().collect();
    let mut introduced = Vec::new();
    let mut remaining = Vec::new();
    for e in after {
        let same =
            |b: &&CompileError| b.path == e.path && b.code == e.code && b.message == e.message;
        if let Some(i) = left.iter().position(same) {
            left.remove(i);
            remaining.push(e.clone());
        } else {
            introduced.push(e.clone());
        }
    }
    (introduced, remaining)
}

#[cfg(test)]
mod tests;
