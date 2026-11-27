//! What `--coverage` records and where its report goes.

use super::build::Toolchain;
use super::{coverage, discover, prerequisites, watch};
use crate::cli::{CoverageFormat, TestArgs};
use std::path::{Path, PathBuf};

/// The folder a coverage report goes to without `--coverage-out`: `JACKET_OUTPUT_DIR` (a
/// relative one is below the checkout the standard library is in, as everywhere else), else
/// `target/` in the project.
fn report_dir(project_dir: &Path, stdlib: &Path, over: Option<std::ffi::OsString>) -> PathBuf {
    match over {
        Some(dir) => prerequisites::output_in(stdlib.parent().unwrap_or(project_dir), Some(dir)),
        None => project_dir.join("target"),
    }
}

/// What `--coverage` records and where it goes, or why this run cannot.
pub(super) fn coverage_for(
    args: &TestArgs,
    cross: bool,
    roots: &[discover::Root],
    toolchain: &Toolchain,
    work: &Path,
    project_dir: &Path,
) -> Result<Option<coverage::Coverage>, String> {
    if !args.coverage {
        return Ok(None);
    }
    let (flags, jaic) = (&toolchain.flags, &toolchain.jaic);
    if cross {
        return Err("`--coverage` records programs that run on this machine; it does not work with another `-os` or `-cpu`".into());
    }
    let out = args.coverage_out.clone().unwrap_or_else(|| {
        let name = match args.coverage_format {
            CoverageFormat::Lcov => "lcov.info",
            CoverageFormat::Html => "coverage.html",
        };
        report_dir(
            project_dir,
            &toolchain.stdlib,
            jacket_config::env::OUTPUT_DIR.non_empty(),
        )
        .join(name)
    });
    let canonical = |dir: &Path| watch::canonical(dir);
    let include = roots
        .iter()
        .map(|r| canonical(&r.dir))
        .chain(discover::import_folders(flags))
        .chain(args.coverage_include.iter().map(|dir| canonical(dir)));
    let exclude = roots.iter().map(|r| canonical(&r.dir.join("tests")));
    coverage::Coverage::new(coverage::Config {
        output: coverage::Output {
            out,
            format: args.coverage_format,
            by: args.coverage_by,
            baseline: args.coverage_baseline.clone(),
            margin: args.coverage_margin,
            record: args.coverage_record.clone(),
            summary: args.coverage_summary.clone(),
        },
        dir: work.join("coverage"),
        include: coverage::outermost(include),
        exclude: exclude.collect(),
        base: project_dir.to_path_buf(),
        jaic: jaic.clone(),
        stdlib: toolchain.stdlib.clone(),
        jaic_flags: flags.clone(),
        native: !args.interp,
    })
    .map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_relative_output_dir_puts_the_coverage_report_below_the_checkout() {
        let (project, stdlib) = (Path::new("/work/app"), Path::new("/opt/jacket/stdlib"));
        let dir = |over: Option<&str>| report_dir(project, stdlib, over.map(Into::into));
        assert_eq!(dir(None), Path::new("/work/app/target"));
        assert_eq!(dir(Some("out")), Path::new("/opt/jacket/out"));
        assert_eq!(dir(Some("/elsewhere")), Path::new("/elsewhere"));
    }
}
