//! `jacket migrate`: rewrite code written for an older Jai to the current language. See
//! `docs/tools/jacket-migrate.md`.
//!
//! The rewriting is `jailint::migrate`; this is the command around it: a dry run that prints
//! the diff by default, `--apply` to write, and the guard that keeps `--apply` from changing a
//! file version control could not give back.
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use jailint::migrate::{self, MIGRATIONS, Plan, Selection};

use crate::cli::MigrateArgs;

pub fn run(args: &MigrateArgs, cwd: &Path) -> ExitCode {
    if args.list {
        list();
        return ExitCode::SUCCESS;
    }

    match migrate(args, cwd) {
        Ok(code) => code,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::from(2)
        }
    }
}

fn list() {
    let width = MIGRATIONS.iter().map(|m| m.name.len()).max().unwrap_or(0);

    for m in MIGRATIONS {
        let note = if m.ambiguous {
            " (only with --only)"
        } else {
            ""
        };

        println!("{:width$}  {}{note}", m.name, m.summary);
    }
}

fn migrate(args: &MigrateArgs, cwd: &Path) -> Result<ExitCode, String> {
    let paths = if args.paths.is_empty() {
        let loaded = crate::project::load(cwd).map_err(|e| e.to_string())?;
        vec![loaded.map_or_else(|| cwd.to_path_buf(), |l| l.dir().to_path_buf())]
    } else {
        args.paths.clone()
    };

    for path in &paths {
        std::fs::metadata(path)
            .map_err(|e| format!("cannot read `{}`: {}", path.display(), jaic::io_reason(&e)))?;
    }

    let first = jaic::canonicalize(&paths[0]).map_err(|e| {
        format!(
            "cannot resolve `{}`: {}",
            paths[0].display(),
            jaic::io_reason(&e)
        )
    })?;
    let start = if first.is_dir() {
        first
    } else {
        first
            .parent()
            .map_or_else(|| cwd.to_path_buf(), Path::to_path_buf)
    };
    let config = jailint::Config::find_on_disk(&start)?.unwrap_or_else(|| jailint::Config {
        root: cwd.to_path_buf(),
        ..jailint::Config::default()
    });

    // `--apply` needs git to undo its edits: say so before the slower checks and the plan.
    if args.apply {
        git_toplevel(&start)?;
    }

    let stdlib = jaic::default_stdlib_dir();

    if let Some(message) = jaic::missing_stdlib(&stdlib) {
        return Err(message);
    }

    let options = jailint::driver::Options {
        compiler_options: jaic::sema::Options::host(),
        source: None,
        paths,
        import_dirs: Vec::new(),
        stdlib: jaic::canonicalize(&stdlib).unwrap_or(stdlib),
        config,
        jobs: 0,
        verbose: false,
        add_sources: Vec::new(),
        limits: jaic::ide::lint_compile::CheckLimits::default(),
        overlay: std::collections::BTreeMap::new(),
    };
    let selection = Selection {
        only: (!args.only.is_empty()).then(|| args.only.clone()),
    };
    let plan = migrate::plan(options, &selection)?;

    show(&plan, cwd)?;
    let status = if plan.rejected.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    };

    if plan.files.is_empty() {
        println!("nothing to migrate");
        return Ok(status);
    }

    let rewrites: usize = plan.files.iter().map(|f| f.applied.len()).sum();

    if !args.apply {
        println!(
            "{rewrites} rewrite(s) in {} file(s); nothing written (run with --apply to write them)",
            plan.files.len()
        );
        return Ok(status);
    }

    let targets: Vec<&Path> = plan.files.iter().map(|f| f.path.as_path()).collect();

    guard(&targets, args)?;

    for file in &plan.files {
        write(&file.path, &file.before, &file.after)?;
    }

    println!(
        "migrated {} file(s) ({rewrites} rewrite(s))",
        plan.files.len()
    );
    Ok(status)
}

/// Print the diff of each file, which migrations made it, and what needs a person.
fn show(plan: &Plan, cwd: &Path) -> Result<(), String> {
    for file in &plan.files {
        let shown = crate::shown::path(&file.path, cwd);
        let diff = similar::TextDiff::from_lines(&*file.before, &file.after);

        print!(
            "{}",
            diff.unified_diff()
                .context_radius(3)
                .header(&format!("a/{shown}"), &format!("b/{shown}"))
        );

        for applied in &file.applied {
            let (line, column) = line_column(&file.before, applied.start);

            println!("  {shown}:{line}:{column}: {}", applied.migration);
        }
    }

    for m in &plan.manual {
        eprintln!(
            "note: {}: {}: {}",
            location(&m.path, m.start, plan, cwd)?,
            m.migration,
            m.message
        );
    }

    for path in &plan.skipped {
        eprintln!(
            "note: `{}` is not UTF-8 text; it was left as it is",
            crate::shown::path(path, cwd)
        );
    }

    for r in &plan.rejected {
        eprintln!(
            "error: `{}` was not applied: its rewrites would make code that compiled fail",
            r.migration
        );

        for e in &r.errors {
            eprintln!(
                "  {}:{}:{}: error[{}]: {}",
                crate::shown::path(&e.path, cwd),
                e.line,
                e.column,
                e.code,
                e.message
            );
        }
    }

    if !plan.rejected.is_empty() {
        eprintln!(
            "help: the other migrations are unaffected; please report the code the rejected one came from"
        );
    }

    if !plan.remaining.is_empty() {
        eprintln!(
            "note: {} error(s) no migration covers remain; `jacket check` shows them",
            plan.remaining.len()
        );
    }

    Ok(())
}

/// `path:line:column` of `offset` in the text the plan ended with for `path`, which is the file
/// on disk when the plan does not change it.
fn location(path: &Path, offset: usize, plan: &Plan, cwd: &Path) -> Result<String, String> {
    let text = match plan.files.iter().find(|f| f.path == path) {
        Some(file) => file.after.clone(),
        None => std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read `{}`: {}", path.display(), jaic::io_reason(&e)))?,
    };
    let (line, column) = line_column(&text, offset);

    Ok(format!("{}:{line}:{column}", crate::shown::path(path, cwd)))
}

fn line_column(text: &str, offset: usize) -> (usize, usize) {
    let before = text.get(..offset).unwrap_or(text);
    let line = before.matches('\n').count() + 1;
    let column = before.rsplit('\n').next().map_or(0, |l| l.chars().count()) + 1;

    (line, column)
}

/// Refuse to write unless version control can give back every file `--apply` changes: each
/// must be tracked by git with no staged or unstaged changes. `--allow-dirty` and
/// `--allow-no-vcs` lift the two halves.
fn guard(targets: &[&Path], args: &MigrateArgs) -> Result<(), String> {
    let dir = targets[0].parent().unwrap_or(Path::new("."));
    let Some(top) = git_toplevel(dir)? else {
        if args.allow_no_vcs {
            return Ok(());
        }

        return Err(format!(
            "`{}` is not in a git repository, so these edits could not be undone\nhelp: commit the code to git first, or pass --allow-no-vcs",
            dir.display()
        ));
    };

    if args.allow_dirty {
        return Ok(());
    }

    let dirty = dirty_files(&top, targets)?;

    if dirty.is_empty() {
        return Ok(());
    }

    let list: Vec<String> = dirty.iter().map(|p| format!("  {}", p.display())).collect();

    Err(format!(
        "these files have changes git has not committed, which the migration would mix with its own:\n{}\nhelp: commit or stash them first, or pass --allow-dirty",
        list.join("\n")
    ))
}

/// The top of the git work tree holding `dir`, or `None` when it is in none.
fn git_toplevel(dir: &Path) -> Result<Option<PathBuf>, String> {
    use crate::git::{Failure, Repo};

    match Repo::at(dir).top() {
        Ok(top) => Ok(Some(jaic::canonicalize(&top).unwrap_or(top))),
        Err(Failure::Failed {
            said,
        }) if said.contains("not a git repository") => Ok(None),
        Err(Failure::Failed {
            said,
        }) => Err(format!(
            "git could not tell where the repository is: {said}"
        )),
        Err(Failure::Missing) if dir.ancestors().any(|d| d.join(".git").exists()) => {
            Err(crate::prerequisite::missing(
                &jaic_toolchain::package::Need::Git,
                "git",
                "jacket migrate",
            ))
        }
        Err(Failure::Missing) => Ok(None),
        Err(Failure::Unusable(why)) => Err(format!("cannot run git: {why}")),
    }
}

/// The targets with staged or unstaged changes, or untracked by git.
fn dirty_files(top: &Path, targets: &[&Path]) -> Result<Vec<PathBuf>, String> {
    let dirty = crate::git::Repo::at(top)
        .dirty(targets)
        .map_err(|e| e.message)?;
    Ok(dirty.iter().map(|path| top.join(path)).collect())
}

/// Replace `path`'s text, which must still be `before`, with `after`, through a temporary file
/// in the same folder so a failed write leaves the original.
fn write(path: &Path, before: &str, after: &str) -> Result<(), String> {
    let now = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read `{}`: {}", path.display(), jaic::io_reason(&e)))?;

    if now != before {
        return Err(format!(
            "`{}` changed while migrating; nothing more was written\nhelp: run `jacket migrate` again",
            path.display()
        ));
    }

    let name = path
        .file_name()
        .ok_or_else(|| format!("cannot write `{}`: it names no file", path.display()))?
        .to_string_lossy();
    let temp = path.with_file_name(format!(".{name}.migrate"));
    let failed = |e: std::io::Error| {
        // The original is untouched; the temporary file is ours and is all there is to undo.
        std::fs::remove_file(&temp).ok();
        format!("cannot write `{}`: {}", path.display(), jaic::io_reason(&e))
    };
    let permissions = std::fs::metadata(path).map_err(failed)?.permissions();

    std::fs::write(&temp, after).map_err(failed)?;
    std::fs::set_permissions(&temp, permissions).map_err(failed)?;
    std::fs::rename(&temp, path).map_err(failed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_location_in_a_file_that_cannot_be_read_is_an_error() {
        let plan = Plan::default();
        let missing = Path::new("/nowhere/jacket-migrate/a.jai");
        let error = location(missing, 0, &plan, Path::new("/")).unwrap_err();

        assert!(
            error.starts_with("cannot read `/nowhere/jacket-migrate/a.jai`: "),
            "{error}"
        );
        let dir = std::env::temp_dir().join(format!("jacket-migrate-{}", std::process::id()));

        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("a.jai");

        std::fs::write(&file, "a\nbc").unwrap();
        assert_eq!(
            location(&file, 3, &plan, &dir).unwrap(),
            format!("{}:2:2", crate::shown::path(&file, &dir))
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
