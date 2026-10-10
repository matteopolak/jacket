//! Replays every saved crash input in `fuzz/regressions/<target>/` through the same harness
//! function its libFuzzer target calls, so fixed crashes stay fixed without a sanitizer build.
use std::path::Path;

const REPLAY_STACK: usize = 128 << 20;

fn replay(target: &str, run: fn(&[u8])) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../regressions")
        .join(target);
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return;
    };
    let mut paths: Vec<_> = entries.filter_map(Result::ok).map(|e| e.path()).collect();
    paths.sort();
    for path in paths {
        if path
            .file_name()
            .is_some_and(|n| n.to_string_lossy().starts_with('.'))
        {
            continue;
        }
        let data = std::fs::read(&path).expect("read regression input");
        eprintln!("replaying {}", path.display());
        // libFuzzer runs `lexer`/`parser` on its 8 MiB main thread in a release build. Debug
        // frames are several times larger, and a test thread has 2 MiB, so give it room.
        std::thread::Builder::new()
            .stack_size(REPLAY_STACK)
            .spawn(move || run(&data))
            .expect("spawn replay thread")
            .join()
            .unwrap_or_else(|panic| std::panic::resume_unwind(panic));
    }
}

#[test]
fn lexer() {
    replay("lexer", jai_fuzz_harness::lexer);
}

#[test]
fn parser() {
    replay("parser", jai_fuzz_harness::parser);
}

#[test]
fn check() {
    replay("check", jai_fuzz_harness::check);
}

#[test]
fn interp() {
    replay("interp", jai_fuzz_harness::interp);
}

#[test]
fn generated() {
    replay("generated", jai_fuzz_harness::generated);
}

#[test]
fn lsp() {
    replay("lsp", jai_fuzz_harness::lsp);
}

#[test]
fn lsp_json() {
    replay("lsp_json", jai_fuzz_harness::lsp_json);
}

#[test]
fn lsp_edits() {
    replay("lsp_edits", jai_fuzz_harness::lsp_edits);
}

#[test]
fn jaifmt() {
    replay("jaifmt", jai_fuzz_harness::jaifmt);
}

/// A metaprogram that answers every `FAILED_IMPORT` with a file the sandbox does not have must
/// end with the import error, not retry until the interpreter budget is gone (a nightly `check`
/// timeout: the replay above only shows it does not crash, not how long it takes).
#[test]
fn unanswerable_provide_import_ends() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../regressions/check/provide-import-missing-file.jai");
    let source = std::fs::read(path).expect("read regression input");
    let files = std::collections::BTreeMap::from([("main.jai".to_string(), source)]);
    let options = jai_wasm::play::PlayOptions {
        budget: Some(jai_fuzz_harness::BLOCK_BUDGET),
        compile_only: true,
        styled: false,
    };
    let result = std::thread::Builder::new()
        .stack_size(REPLAY_STACK)
        .spawn(move || jai_wasm::play::run_with(&files, "main.jai", options))
        .expect("spawn replay thread")
        .join()
        .expect("compile");
    let messages: Vec<_> = result.diagnostics.iter().map(|d| &d.message).collect();
    assert!(
        messages.iter().all(|m| !m.contains("budget exhausted")),
        "{messages:?}"
    );
}
