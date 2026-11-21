//! Computed `using` filters are arrays of strings, read back at the target's string stride.
use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
use jaic::target::Target;
use std::rc::Rc;

/// A string is 12 bytes on i686 (`s64` aligns to 4), not 16.
#[test]
fn a_computed_filter_reads_strings_at_the_targets_stride() {
    let mut options = Options::host();
    options.layout = Target::parse("i686-unknown-linux-gnu")
        .unwrap()
        .data_layout();
    options.runtime_support = false;
    let source = r#"
        Point :: struct { x: s32; y: s32; z: s32; }
        main :: () {
            p: Point;
            using,only(.["y", "z"]) p;
            a := y;
            b := z;
        }
    "#;
    let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
    if let Err(e) = c.compile_sources(&[ProgramSource::String(source.into())]) {
        panic!("{}", e.render(&c.src.sources));
    }
}
