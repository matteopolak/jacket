//! Which overload a call picks.
use std::path::Path;
use std::process::Command;

/// A fixed-arity header beats a variadic one when the call passes no extra arguments, even if
/// the fixed header is polymorphic. `#modify` can still reject it for a given constant.
#[test]
fn a_fixed_arity_overload_beats_a_packing_one() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/compiler/semantics/overload-fixed-arity.jai");
    let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
        .arg("-interp")
        .arg(source)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "fixed\npacked\npacked\n"
    );
}
