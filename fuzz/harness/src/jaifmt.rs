//! Round-trip checks for the `Jai_Format` formatter (`jaifmt`), which is Jai code: each input is
//! formatted by the stdlib module under the playground's interpreter, twice, and the result is
//! checked against the compiler's own lexer and parser.
//!
//! A failure is a panic with the reason, so libFuzzer records it like a crash:
//!
//! - the formatter's internal token check failed (it would have changed the program);
//! - formatting is not idempotent (`format(format(x)) != format(x)`);
//! - the Rust lexer sees a different token stream in the output than in the input;
//! - the input parses but the output does not, or their syntax trees differ (spans aside);
//! - the formatter ran out of its (generous) interpreter budget: it is linear in the input.
//!
//! Input the formatter refuses (text that does not lex, unbalanced brackets) is fine.
use jai_wasm::play::{PlayOptions, run_with};
use jaic::source::FileId;
use std::collections::BTreeMap;

/// The driver: format `/workspace/input.jai` with the config in `/workspace/jaifmt.toml`, twice.
/// Exit codes: 0 formatted (stdout holds the text), 1 refused, 2 internal check failed,
/// 3 second pass refused, 4 not idempotent, 5 bad config.
const DRIVER: &str = r#"
#import "Basic";
#import "File";
#import "String";
#import "Jai_Format";

main :: () -> s32 {
    source, read := read_entire_file("/workspace/input.jai", log_errors = false);
    if !read return 1;
    toml := read_entire_file("/workspace/jaifmt.toml", log_errors = false);
    config, config_ok, config_error := parse_config(toml);
    if !config_ok { print("%\n", config_error, to_standard_error = true); return 5; }
    once, ok, error := format_source(source, config);
    if !ok {
        print("%\n", error, to_standard_error = true);
        return ifx contains(error, "internal error") then cast(s32) 2 else 1;
    }
    twice, ok2, error2 := format_source(once, config);
    if !ok2 { print("%\n", error2, to_standard_error = true); return 3; }
    write_string(once);
    if twice != once return 4;
    return 0;
}
"#;

/// Configs the inputs rotate through (picked by a hash of the input), so the non-default
/// indentation and brace styles get checked too.
const CONFIGS: [&str; 4] = [
    "",
    "indent_width = 2\ncase_indent = 0\n",
    "brace_style = \"preserve\"\nmax_blank_lines = 0\n",
    "indent_width = 8\ncase_body_indent = 2\nmax_blank_lines = 1\n",
];

/// Interpreter blocks for formatting one input twice. Formatting is linear: an 8 KiB input takes
/// a few million blocks, so a run past this limit is a hang.
const FORMAT_BUDGET: u64 = 400_000_000;

/// Tokens as the compiler sees them, positions dropped.
fn tokens(text: &str) -> Option<Vec<String>> {
    let tokens = jaic::lexer::lex(FileId(0), text).ok()?;
    Some(tokens.iter().map(|t| format!("{:?}", t.tok)).collect())
}

/// The syntax tree's debug form with every span offset and node id blanked, so two trees compare
/// equal when they differ only in layout. `None` when the text does not parse.
fn syntax(text: &str) -> Option<String> {
    const BLANKED: [&str; 3] = ["start: ", "end: ", "AstId("];
    let file = jaic::parser::parse_file(FileId(0), text).ok()?;
    let tree = format!("{:?}", file.stmts);
    let mut out = String::with_capacity(tree.len());
    let mut rest = tree.as_str();
    while let Some((at, label)) = BLANKED
        .iter()
        .filter_map(|label| rest.find(label).map(|at| (at, label)))
        .min()
    {
        out.push_str(&rest[..at + label.len()]);
        rest = rest[at + label.len()..].trim_start_matches(|c: char| c.is_ascii_digit());
    }
    out.push_str(rest);
    Some(out)
}

/// Format `data` and check the round trip (see the module docs).
pub fn jaifmt(data: &[u8]) {
    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };
    let config = CONFIGS[data
        .iter()
        .fold(0usize, |h, &b| h.wrapping_mul(31) ^ b as usize)
        % 4];
    let mut files = BTreeMap::new();
    files.insert("main.jai".to_string(), DRIVER.as_bytes().to_vec());
    files.insert("input.jai".to_string(), data.to_vec());
    files.insert("jaifmt.toml".to_string(), config.as_bytes().to_vec());
    let options = PlayOptions {
        budget: Some(FORMAT_BUDGET),
        compile_only: false,
    };
    let result = crate::on_compiler_stack(|| run_with(&files, "main.jai", options));
    let verbose = std::env::var_os("JAI_FUZZ_VERBOSE").is_some();
    if verbose {
        eprintln!(
            "exit {:?}\n{}{}--- formatted ---\n{}",
            result.exit_code, result.rendered, result.stderr, result.stdout
        );
    }
    let why = match result.exit_code {
        Some(0) => None,
        Some(1) => return,
        Some(2) => Some("the formatter's token check failed"),
        Some(3) => Some("the formatter refused its own output"),
        Some(4) => Some("formatting is not idempotent"),
        _ => Some("the formatter did not finish"),
    };
    if let Some(why) = why {
        panic!(
            "{why} (config {config:?}): {}{}",
            result.rendered, result.stderr
        );
    }
    let formatted = result.stdout.as_str();
    if tokens(source) != tokens(formatted) {
        panic!("the formatted text lexes differently (config {config:?})");
    }
    let before = syntax(source);
    let after = syntax(formatted);
    if before.is_some() && before != after {
        if verbose {
            eprintln!("--- input tree ---\n{before:?}\n--- formatted tree ---\n{after:?}");
        }
        panic!("the formatted text parses differently (config {config:?})");
    }
}
