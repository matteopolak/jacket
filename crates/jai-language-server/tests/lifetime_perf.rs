//! Run explicitly with --release, --ignored and --nocapture as the sole heavy job.
use jai_language_server::{DocumentUri, Position};
use std::time::Instant;
use test_support::lsp::{session, uri};

#[test]
#[ignore = "release-profile lifetime performance gate"]
fn warm_single_file_lifetime_lint() {
    const SOURCE: &str = "#import \"Basic\";\nmain :: () {\n    xs: [..] int;\n    array_add(*xs, 1);\n    p := *xs[0];\n    array_add(*xs, 2);\n    print(\"%\\n\", p.*);\n}\n";
    let mut rule_work = Vec::new();
    let mut cached = Vec::new();
    for _ in 0..21 {
        let mut s = session();
        s.open(
            DocumentUri::parse("file:///lsp-test/jacket.toml").unwrap(),
            1,
            "[lint.rules]\npointer_lifetime = \"warn\"\n".into(),
        )
        .unwrap();
        s.open(uri(), 1, SOURCE.into()).unwrap();
        // Hover compiles the program. The first diagnostic request still performs lint work.
        assert!(
            s.hover(
                &uri(),
                Position {
                    line: 4,
                    character: 4
                }
            )
            .unwrap()
            .is_some()
        );
        let start = Instant::now();
        let diagnostics = s.diagnostics(&uri()).unwrap();
        rule_work.push(start.elapsed().as_secs_f64() * 1000.0);
        assert!(
            diagnostics
                .iter()
                .any(|d| d.code == jai_language_server::DiagnosticCode::Lint("pointer_lifetime"))
        );
        let start = Instant::now();
        std::hint::black_box(s.diagnostics(&uri()).unwrap());
        cached.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    // The first sample warms OS/library caches; subsequent samples still run uncached rules.
    rule_work.remove(0);
    cached.remove(0);
    rule_work.sort_by(f64::total_cmp);
    cached.sort_by(f64::total_cmp);
    println!("lifetime-perf warm_rule_work_ms={rule_work:?} cached_diagnostics_ms={cached:?}");
    println!(
        "lifetime-perf median_rule_ms={} max_rule_ms={} median_cached_ms={}",
        rule_work[rule_work.len() / 2],
        rule_work.last().unwrap(),
        cached[cached.len() / 2]
    );
}
