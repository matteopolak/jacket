//! One-file worker for tools/lsp_partial_edits.py; stdout is a machine-readable report.
#[path = "support/edit_coverage.rs"]
mod edit_coverage;

fn main() {
    // Match jailsp's native compiler thread. Its documented parser depth budget
    // needs more than a small default stack in unoptimized builds.
    std::thread::Builder::new()
        .stack_size(1 << 30)
        .spawn(run)
        .unwrap()
        .join()
        .unwrap();
}

fn run() {
    let mut args = std::env::args().skip(1);
    let file = std::path::PathBuf::from(args.next().expect("FILE [STDLIB|-] [MAX_LINES]"));
    let stdlib = args.next().filter(|s| s != "-").map(|s| {
        std::path::PathBuf::from(s)
            .canonicalize()
            .expect("stdlib directory exists")
    });
    let max_lines = args
        .next()
        .map(|s| s.parse().expect("integer MAX_LINES"))
        .unwrap_or(0);
    edit_coverage::set_progress(args.next().as_deref() == Some("--progress"));
    let file = file.canonicalize().expect("source file exists");
    let bytes = std::fs::read(&file).expect("Jai source");
    // Match Compiler::read_source for files not encoded as UTF-8. LSP documents
    // are Unicode; record the decoding instead of silently skipping the file.
    let source = String::from_utf8_lossy(&bytes);
    let mut report = edit_coverage::check(&file, &source, stdlib, max_lines);
    report.utf8_lossy = std::str::from_utf8(&bytes).is_err();
    println!("{}", serde_json::to_string(&report).unwrap());
    if !report.failures.is_empty() {
        std::process::exit(1);
    }
}
