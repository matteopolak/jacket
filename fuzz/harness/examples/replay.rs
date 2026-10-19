//! Run one harness over input files without libFuzzer, for triage:
//!
//!     cargo run --release --manifest-path fuzz/harness/Cargo.toml --example replay -- check crash-*
//!
//! `show` prints the program the `generated` target builds from each input.
use std::time::Instant;

fn main() {
    let mut args = std::env::args().skip(1);
    let target = args.next().expect("usage: replay <target|show> <files...>");
    let run: fn(&[u8]) = match target.as_str() {
        "lexer" => jai_fuzz_harness::lexer,
        "parser" => jai_fuzz_harness::parser,
        "check" => jai_fuzz_harness::check,
        "interp" => jai_fuzz_harness::interp,
        "generated" => jai_fuzz_harness::generated,
        "lsp" => jai_fuzz_harness::lsp,
        "lsp_json" => jai_fuzz_harness::lsp_json,
        "lsp_edits" => jai_fuzz_harness::lsp_edits,
        "lsp_workspace" => jai_fuzz_harness::lsp_workspace,
        "lsp_typing" => jai_fuzz_harness::lsp_typing,
        "jailint_fix" => jai_fuzz_harness::jailint_fix,
        "jacket_config" => jai_fuzz_harness::jacket_config,
        "metaprogram" => jai_fuzz_harness::metaprogram,
        "jaifmt" => jai_fuzz_harness::jaifmt,
        "show_meta" => |data: &[u8]| print!("{}", jai_fuzz_harness::metaprogram_source(data)),
        "show" => |data: &[u8]| print!("{}", jai_fuzz_harness::generate::program(data)),
        other => panic!("unknown target {other}"),
    };
    for path in args {
        let data = std::fs::read(&path).expect("read input");
        let start = Instant::now();
        run(&data);
        eprintln!("{path}: {:?}", start.elapsed());
    }
}
