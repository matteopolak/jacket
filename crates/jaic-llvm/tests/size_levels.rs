//! The size pipelines (`OptLevel::Os` and `Oz`) run: LLVM no longer has `default<Os>`, so they are
//! `default<O2>` with the `optsize` (`minsize`) attributes on every function.
use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
use jaic_llvm::{OptLevel, emit_object};
use std::rc::Rc;
use test_support::TempDir;

#[test]
fn the_size_levels_write_an_object() {
    let mut options = Options::host();
    options.runtime_support = false;
    let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
    if let Err(e) = c.compile_sources(&[ProgramSource::String(
        "main :: () {}\n#program_export \"sum\"\nsum :: (a: s64, b: s64) -> s64 #c_call { return a + b; }\n".into(),
    )]) {
        panic!("{}", e.render(&c.src.sources));
    }
    let dir = TempDir::new("size-levels");
    for level in [OptLevel::Os, OptLevel::Oz] {
        let object = dir.join(format!("{level:?}.o"));
        emit_object(
            &c.out.program,
            &jaic_llvm::Options {
                opt_level: level,
                ..Default::default()
            },
            &object,
        )
        .unwrap();
        assert!(object.metadata().unwrap().len() > 0, "{level:?}");
    }
}
