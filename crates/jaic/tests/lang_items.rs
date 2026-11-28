//! The compiler's contract with the standard library, checked against the real prelude: every
//! type it names resolves, and the layouts in `jaic::layout` agree with the prelude's structs.
use std::path::Path;
use std::rc::Rc;

use jaic::sema::{Compiler, NativeFs, Options, PreludeType, ProgramSource};
use jaic::source::Span;

fn compiler_with_real_prelude() -> Compiler {
    let stdlib = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();

    options.runtime_support = false;
    options.preload = Some(stdlib.join("prelude/module.jai"));
    options.import_paths = vec![stdlib];
    let mut compiler = Compiler::new(options, Rc::new(NativeFs));

    compiler
        .compile_sources(&[ProgramSource::String(String::new())])
        .expect("the prelude loads");
    compiler
}

#[test]
fn every_prelude_type_resolves_in_the_real_prelude() {
    let mut compiler = compiler_with_real_prelude();

    for ty in PreludeType::ALL {
        assert!(
            compiler.preload_type(*ty, Span::NONE).is_ok(),
            "the prelude does not define `{}`",
            ty.name()
        );
    }
}

#[test]
fn the_builtin_layouts_match_the_prelude() {
    let mut compiler = compiler_with_real_prelude();
    let allocator = compiler
        .preload_type(PreludeType::Allocator, Span::NONE)
        .expect("Allocator");

    assert_eq!(
        compiler.size_of(allocator, Span::NONE).unwrap(),
        compiler.ty.types.builtins().allocator.size
    );
    let info = compiler
        .preload_type(PreludeType::TypeInfo, Span::NONE)
        .expect("Type_Info");
    let pointer = compiler.ty.types.pointer(info);

    assert_eq!(
        compiler.size_of(pointer, Span::NONE).unwrap(),
        compiler.options.layout.ptr_bytes()
    );
}
