//! Inherited field layout contracts required by reviewed public record shapes.
use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
use std::rc::Rc;

fn compile(source: &str) -> (Compiler, jaic::sema::Result<()>) {
    let mut options = Options::host();
    options.runtime_support = false;
    let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
    let result = c.compile_sources(&[ProgramSource::String(source.into())]);
    (c, result)
}

#[test]
fn overlay_and_place_resolve_inline_promoted_fields() {
    let (c, result) = compile(
        "Base :: struct { padding: u64; storage: [16] u8; } Middle :: struct { using base: Base; } Outer :: struct { lead: u64; #as using base: Middle; #overlay(storage) value: u64; } Placed :: struct { using base: Base; #place storage; value: u64; } #assert(offset_of(Outer, \"value\") == 16); #assert(offset_of(Outer, \"value\") == offset_of(Outer, \"storage\")); #assert(offset_of(Placed, \"value\") == 8); #assert(size_of(Outer) == 32); object: Outer; #assert(cast(*u8) *object.value == *object.base.base.storage[0]);",
    );
    if let Err(e) = result {
        panic!("{}", e.render(&c.sources));
    }
}

#[test]
fn embedded_using_filter_applies_to_member_lookup() {
    let (c, result) = compile(
        "Base :: struct { x: int; y: int; } Outer :: struct { using,only(x) base: Base; } object: Outer; #assert(*object.x == *object.base.x); #assert(*object.y == *object.base.y);",
    );
    let error = result.expect_err("filtered member must be inaccessible");
    assert!(error.message.contains("y"), "{}", error.render(&c.sources));
}

#[test]
fn overlay_cannot_target_storage_behind_using_pointer() {
    let (c, result) = compile(
        "Base :: struct { storage: [16] u8; } Outer :: struct { using base: *Base; #overlay(storage) value: u64; } #assert(size_of(Outer) > 0);",
    );
    let error = result.expect_err("dynamic pointer target has no inline layout offset");
    assert!(
        error.message.contains("storage"),
        "{}",
        error.render(&c.sources)
    );
}
