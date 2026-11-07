//! Private worker for `tools/lsp/scale_sandbox.py`, never a direct untrusted-input runner.
//! The registered scale test always delegates confinement to the owning bench launcher.
use jai_language_server::{DocumentUri, Limits};
use std::path::PathBuf;
use std::time::{Duration, Instant};

const SCALE_BUDGET: Duration = Duration::from_secs(120);

/// Run the unchanged measured case on the launcher's canonical selected inputs.
fn run() {
    let mut args = std::env::args_os().skip(1);
    let root = PathBuf::from(args.next().expect("expected selected source"));
    let stdlib = PathBuf::from(args.next().expect("expected selected stdlib"));
    assert!(args.next().is_none(), "expected exactly source and stdlib");
    assert!(
        root.is_absolute() && root.is_file(),
        "invalid selected source"
    );
    assert!(
        stdlib.is_absolute() && stdlib.is_dir(),
        "invalid selected stdlib"
    );
    let enforce = jacket_config::env::ENFORCE_SCALE_BUDGET.get().as_deref() == Some("1");
    // Missing stdlib grants must fail the harness, not masquerade as expected diagnostics.
    assert!(
        !std::fs::read(stdlib.join("prelude/module.jai"))
            .expect("sandbox grants the selected stdlib spelling")
            .is_empty()
    );
    let text = std::fs::read_to_string(&root).unwrap();
    let mut s = jai_language_server::Session::with_environment(
        Limits::default(),
        test_support::lsp::environment_at(stdlib, None),
    );
    let uri = DocumentUri::from_path(&root.canonicalize().unwrap()).unwrap();
    let started = Instant::now();
    s.open(uri.clone(), 1, text).unwrap();
    let diagnostics = s.diagnostics(&uri).unwrap();
    assert_ne!(diagnostics, [] as [jai_language_server::Diagnostic; 0]);
    let elapsed = started.elapsed();
    let checked = s.compile_counts();
    let coverage = s
        .semantic_coverage(&uri)
        .unwrap()
        .expect("checked Focus context");
    assert!(
        coverage.source_loaded,
        "the checked context must load the selected source"
    );
    assert_eq!(
        s.compile_counts(),
        checked,
        "coverage must reuse the measured analysis"
    );
    eprintln!(
        "Focus coverage: loaded={}, errors={}, references={}, diagnostics={:?}",
        coverage.source_loaded,
        coverage.program_errors,
        coverage.recorded_references,
        diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>()
    );
    // The regression this guards took minutes; a normal run takes seconds, so the budget leaves
    // a shared CI runner plenty of room.
    if enforce {
        assert!(elapsed < SCALE_BUDGET, "took {elapsed:?}");
    } else {
        eprintln!(
            "functional scale case completed in {elapsed:?}; set JAILSP_ENFORCE_SCALE_BUDGET=1 to enforce the {}-second budget",
            SCALE_BUDGET.as_secs()
        );
    }
}

fn main() {
    // Match the original test-thread stack policy, including RUST_MIN_STACK when configured.
    std::thread::spawn(run)
        .join()
        .expect("sandboxed scale worker failed");
}
