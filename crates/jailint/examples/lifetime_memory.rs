//! Exact retained Rust-allocation deltas on the same roots and facts as `jacket lint`.
//! Run with `--release`, `JAILINT_LIFETIME_MEMORY=1`, and no other heavy job.
#![forbid(unsafe_code)]
use jailint::config::{Config, Level};
use jailint::driver::{Options, run};
use std::path::PathBuf;

#[global_allocator]
static ALLOCATOR: jaic::memory_limit::CountingAllocator =
    jaic::memory_limit::CountingAllocator(std::alloc::System);

fn main() {
    assert!(
        jacket_config::env::LIFETIME_MEMORY.is_set(),
        "set JAILINT_LIFETIME_MEMORY=1 for this destructive profiling run"
    );
    jaic::memory_limit::arm(isize::MAX.unsigned_abs());
    let mut strict = false;
    let paths: Vec<PathBuf> = std::env::args_os()
        .skip(1)
        .filter_map(|arg| {
            if arg == "--strict" {
                strict = true;
                None
            } else {
                Some(PathBuf::from(arg))
            }
        })
        .collect();
    assert!(!paths.is_empty(), "supply lint roots, e.g. stdlib");
    let mut config = Config::find_on_disk(&paths[0]).unwrap().unwrap_or_default();
    config.discover = true;
    config.flags.insert("pointer_lifetime".into(), Level::Warn);
    if strict {
        config.lifetime_mode = Some(jacket_config::LifetimeMode::Strict);
        config.flag_lifetime_mode = config.lifetime_mode;
    }
    let result = run(&Options {
        compiler_options: jaic::sema::Options::host(),
        source: None,
        paths,
        import_dirs: Vec::new(),
        stdlib: jaic::default_stdlib_dir(),
        config,
        jobs: 1,
        verbose: false,
        add_sources: Vec::new(),
        limits: jaic::ide::lint_compile::CheckLimits::new(0, 0.0),
    });
    for finding in &result.lints {
        let labels: Vec<_> = finding.secondary.iter().map(|label| serde_json::json!({"path": label.path, "start": label.start, "end": label.end, "message": label.message})).collect();
        println!(
            "lifetime-finding {}",
            serde_json::json!({"rule": finding.rule, "level": finding.level.as_str(), "path": finding.path, "start": finding.start, "end": finding.end, "message": finding.message, "labels": labels})
        );
    }
    println!(
        "lifetime-memory roots={} incomplete={} findings={}",
        result.roots,
        result.incomplete.len(),
        result.lints.len()
    );
    assert!(
        result.config_errors.is_empty(),
        "{:?}",
        result.config_errors
    );
}
