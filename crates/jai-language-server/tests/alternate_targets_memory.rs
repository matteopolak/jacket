//! Compiles for the other targets (asked about code the target skips) count toward the
//! compiles kept, so a large program is not held several times over.
use jai_language_server::{Limits, Position, Session};
use jaic::target::TargetOs;
use test_support::lsp::{file_uri, modules_environment, uri};

/// Past the size under which a compile counts as small and is kept besides the others.
const PADDING: usize = 1 << 21;

#[test]
fn compiles_for_other_targets_do_not_pile_up() {
    let source = format!(
        "// {}\n#if OS == .WINDOWS {{\n    windows_only :: () -> int {{ return 1; }}\n}}\n\
         #if OS == .MACOS {{\n    macos_only :: () -> int {{ return 2; }}\n}}\n\
         #if OS == .WASM {{\n    wasm_only :: () -> int {{ return 3; }}\n}}\n\
         main :: () {{}}\n",
        "x".repeat(PADDING)
    );
    let mut s = Session::with_environment(
        Limits::default(),
        modules_environment(Some(TargetOs::Linux)),
    );
    s.open(
        file_uri("/lsp-test/jacket.toml"),
        1,
        "[project]\nentry = [\"main.jai\"]\n".into(),
    )
    .unwrap();
    s.set_auto_import(false);
    s.open(uri(), 1, source.clone().into()).unwrap();
    let _ = s.diagnostics(&uri());
    // Each hover asks the targets in turn until one compiles the arm: Windows, then macOS
    // (which has to pass Windows), then Wasm.
    for name in ["windows_only", "macos_only", "wasm_only"] {
        let (line, text) = source
            .lines()
            .enumerate()
            .find(|(_, l)| l.contains(name))
            .unwrap();
        let at = Position {
            line: line as u32,
            character: text.find(name).unwrap() as u32,
        };
        let hover = s.hover(&uri(), at).unwrap();
        assert!(hover.is_some(), "{name} is described");
        let (_, kept) = s.compile_counts();
        assert!(kept <= 2, "{kept} compiles kept after asking about {name}");
    }
}
