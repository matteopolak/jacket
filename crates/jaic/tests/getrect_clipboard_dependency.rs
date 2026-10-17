//! The text actions compile to the clipboard calls of the Linux backend; nothing is executed.
use jaic::sema::{Compiler, NativeFs, Options, TargetCpu, TargetOs};
use std::{path::PathBuf, rc::Rc};

#[test]
fn public_getrect_text_actions_reach_canonical_linux_clipboard_calls() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for cpu in [TargetCpu::X64, TargetCpu::Arm64] {
        let mut options = Options::host();
        options.os = TargetOs::Linux;
        options.cpu = cpu;
        options.preload = Some(root.join("stdlib/prelude/module.jai"));
        options.import_paths = vec![root.join("stdlib")];
        let mut compiler = Compiler::new(options, Rc::new(NativeFs));
        let input = root.join("tests/numerics/clipboard_text_actions_linux.jai");
        if let Err(error) = compiler.compile_program(&input) {
            panic!("{}", error.render(&compiler.sources));
        }
        for symbol in [
            "portable_clipboard_text",
            "portable_clipboard_read",
            "portable_clipboard_free",
        ] {
            assert!(
                compiler
                    .program
                    .foreigns
                    .iter()
                    .any(|foreign| foreign.symbol == symbol),
                "public text action did not reach canonical {symbol} for {cpu:?}"
            );
        }
    }
}
