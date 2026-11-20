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
//! compiles at least as well as the original ([`Plan::introduced`] is empty). Writing the
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

/// What migrating would do.
#[derive(Debug, Default)]
pub struct Plan {
    pub files: Vec<Migrated>,
    pub manual: Vec<Manual>,
    /// Errors the migrated code has that the original did not. Non-empty means a migration
    /// is wrong for this code: nothing should be written.
    pub introduced: Vec<CompileError>,
    /// Errors left in the migrated code that it shared with the original.
    pub remaining: Vec<CompileError>,
}

/// Work out the migrations of the files `options` names, without writing anything.
pub fn plan(mut options: Options, selection: &Selection) -> Result<Plan, String> {
    let migrations = selection.migrations()?;
    options.overlay.clear();
    let first = driver::run(&options);
    let mut texts: BTreeMap<PathBuf, (Arc<str>, String)> = BTreeMap::new();
    for path in &first.files {
        let before: Arc<str> = match std::fs::read_to_string(path) {
            Ok(text) => Arc::from(text),
            Err(e) => {
                return Err(format!(
                    "cannot read `{}`: {}",
                    path.display(),
                    jaic::io_reason(&e)
                ));
            }
        };
        let after = before.to_string();
        texts.insert(path.clone(), (before, after));
    }
    // Offsets of each file's applied rewrites are kept in the original text's terms.
    let mut applied: BTreeMap<PathBuf, Vec<Applied>> = BTreeMap::new();
    let mut shifts: BTreeMap<PathBuf, Vec<(usize, isize)>> = BTreeMap::new();
    let mut manual = Vec::new();
    let mut errors = first.errors.clone();
    for pass in 0..MAX_PASSES {
        let mut edits: BTreeMap<PathBuf, Vec<(Edit, &'static str)>> = BTreeMap::new();
        manual.clear();
        for (path, (_, text)) in &texts {
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
            for m in &migrations {
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
        if edits.is_empty() {
            break;
        }
        if pass + 1 == MAX_PASSES {
            return Err(format!(
                "the migrations did not settle after {MAX_PASSES} passes; this is a bug in a migration, please report it"
            ));
        }
        for (path, file_edits) in edits {
            let Some((_, text)) = texts.get_mut(&path) else {
                continue;
            };
            let chosen = non_overlapping(file_edits);
            let file_shifts = shifts.entry(path.clone()).or_default();
            let file_applied = applied.entry(path.clone()).or_default();
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
        options.overlay = texts
            .iter()
            .filter(|(_, (before, after))| **before != **after)
            .map(|(path, (_, after))| (path.clone(), Arc::from(after.as_bytes())))
            .collect();
        errors = driver::run(&options).errors;
    }
    let (introduced, remaining) = compare(&first.errors, &errors);
    let files = texts
        .into_iter()
        .filter(|(_, (before, after))| **before != **after)
        .map(|(path, (before, after))| {
            let mut file_applied = applied.remove(&path).unwrap_or_default();
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
        introduced,
        remaining,
    })
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
