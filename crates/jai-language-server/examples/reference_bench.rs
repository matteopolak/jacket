//! Measure owned reference-index extraction for a checked source.
//!
//! `cargo run --release -p jai-language-server --example reference_bench -- STDLIB FILE BINDING [SAMPLES]`
//! BINDING is a declaration snippet beginning with the identifier to query. The compiler
//! oracle and index must agree on a nonempty result before timings are reported.
use jaic::ide::{IdeFacts, IdeRoot};
use jaic::sema::{Compiler, NativeFs, Options};
use jaic::source::FileId;
use std::{path::PathBuf, rc::Rc, time::Instant};

#[cfg(unix)]
#[global_allocator]
static ALLOCATOR: jai_language_server::large_alloc::LargeBlocksToOs =
    jai_language_server::large_alloc::LargeBlocksToOs;

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    assert!(
        (3..=4).contains(&args.len()),
        "usage: reference_bench STDLIB FILE BINDING [SAMPLES]"
    );
    let stdlib = PathBuf::from(&args[0]).canonicalize().expect("stdlib");
    let path = PathBuf::from(&args[1]).canonicalize().expect("file");
    let snippet = &args[2];
    let samples: usize = args.get(3).map_or(5, |s| s.parse().expect("samples"));
    assert!(samples > 0 && !snippet.is_empty());
    let text = std::fs::read_to_string(&path).expect("source");
    let offset = text
        .match_indices(snippet)
        .map(|(at, _)| at)
        .find(|&at| {
            at == 0
                || !text.as_bytes()[at - 1].is_ascii_alphanumeric()
                    && text.as_bytes()[at - 1] != b'_'
        })
        .expect("binding snippet") as u32;
    let fs = Rc::new(NativeFs);
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let guard_options = options.clone();
    let mut compiler = Compiler::new(options, fs);
    let mut facts = IdeFacts::new(vec![IdeRoot::Under(path.parent().unwrap().to_path_buf())]);
    facts.lint = true;
    compiler.ed.record(facts);
    let compile = compiler.compile_program(&path);
    let compile_ok = compile.is_ok();
    println!(
        "file={} compile_ok={} compile_error={:?}",
        path.display(),
        compile_ok,
        compile.err().map(|e| e.message.clone())
    );
    let diagnostics = compiler.ide_check_all();
    compiler.ide_sketch_polymorphs();
    let file = (0..compiler.src.sources.len() as u32)
        .map(FileId)
        .find(|&f| {
            compiler
                .src
                .sources
                .get(f)
                .path
                .as_file()
                .and_then(|p| p.canonicalize().ok())
                .as_ref()
                == Some(&path)
        })
        .expect("input in compiler");
    let expected = compiler.ide_references(file, offset);
    assert!(
        !expected.is_empty(),
        "compiler oracle must have references at the chosen binding"
    );
    let source_bytes: usize = (0..compiler.src.sources.len() as u32)
        .map(FileId)
        .map(|f| compiler.src.sources.get(f).text.len())
        .sum();
    let fact_count = compiler.ed.facts().map_or(0, |f| f.refs.len());
    println!(
        "sources={} source_bytes={} reference_facts={} diagnostics={} oracle_results={} binding={snippet:?} offset={offset}",
        compiler.src.sources.len(),
        source_bytes,
        fact_count,
        diagnostics.len(),
        expected.len()
    );
    for sample in 0..samples {
        let start = Instant::now();
        let index = compiler.ide_reference_index();
        let elapsed = start.elapsed();
        assert_eq!(
            index.references(file, offset),
            expected,
            "index must match live oracle"
        );
        println!(
            "sample={sample} extraction_ms={:.6} retained_bytes={}",
            elapsed.as_secs_f64() * 1000.0,
            index.retained_bytes()
        );
        drop(index);
    }
    drop(compiler);
    if compile_ok && diagnostics.is_empty() {
        input_costs(&path, &text, offset as usize, guard_options, samples);
    } else {
        println!("input_costs_skipped=partial_direct_compile");
    }
}


/// Warm native lookup and request costs; setup, result checks and printing are untimed.
/// An unrelated overlay forces input discovery while compile-count equality excludes rechecking.
fn input_costs(
    path: &std::path::Path,
    text: &str,
    offset: usize,
    options: Options,
    samples: usize,
) {
    use jai_language_server::{
        DiagnosticSeverity, DocumentUri, Environment, Limits, Position, Session,
    };
    let environment = || {
        let options = options.clone();
        Environment {
            fs: Rc::new(NativeFs),
            services: None,
            options: Box::new(move |fs, _| {
                let mut options = options.clone();
                if let Some(stdlib) = options.preload_stdlib.clone() {
                    options.set_stdlib(fs, &stdlib);
                }
                options
            }),
        }
    };
    let mut session = Session::with_environment(Limits::default(), environment());
    let uri = DocumentUri::from_path(path).unwrap();
    session.open(uri.clone(), 1, text.into()).unwrap();
    let diagnostics = session.diagnostics(&uri).unwrap();
    assert!(
        !diagnostics
            .iter()
            .any(|d| d.severity == DiagnosticSeverity::Error),
        "session workload must check"
    );
    let coverage = session
        .semantic_coverage(&uri)
        .unwrap()
        .expect("checked program");
    assert!(
        coverage.source_loaded && coverage.program_complete,
        "session timing requires complete coverage"
    );
    let prefix = &text[..offset];
    let position = Position {
        line: prefix.bytes().filter(|&b| b == b'\n').count() as u32,
        character: prefix.rsplit('\n').next().unwrap().encode_utf16().count() as u32,
    };
    let expected = session.references(&uri, position, true).unwrap();
    assert!(!expected.is_empty());
    println!(
        "session_diagnostics={} session_reference_results={}",
        diagnostics.len(),
        expected.len()
    );
    for miss in [false, true] {
        for sample in 0..samples {
            let other = DocumentUri::from_path(&std::env::temp_dir().join(format!(
                "lsp-input-miss-{}-{sample}.jai",
                std::process::id()
            )))
            .unwrap();
            if miss {
                session
                    .open(other.clone(), 1, "unrelated :: 1;".into())
                    .unwrap();
            }
            let compiles = session.compile_counts().0;
            let start = Instant::now();
            let found = session.references(&uri, position, true).unwrap();
            let elapsed = start.elapsed();
            assert_eq!(found, expected);
            assert_eq!(
                session.compile_counts().0,
                compiles,
                "input-only workload must not compile"
            );
            println!(
                "sample={sample} input_memo_miss={miss} session_references_ms={:.6}",
                elapsed.as_secs_f64() * 1000.0
            );
            if miss {
                session.close(&other).unwrap();
            }
        }
    }
    println!(
        "session_reference_records={:?} discovery_proof_bytes={} build_probes={:?}",
        session.reference_record_counts(),
        session.discovery_proof_bytes(),
        session.build_probe_counts()
    );
}
