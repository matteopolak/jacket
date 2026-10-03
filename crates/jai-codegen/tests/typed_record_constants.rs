#[path = "support/checked_execution.rs"]
mod checked_execution;

#[test]
fn record_typed_constants_keep_annotations_and_ordinary_fields() {
    checked_execution::check(
        "Word::u8; EGL::struct { TRUE:s32:1; NONE:Word:41; value:s32=9; } main::()->int { #assert(type_of(EGL.TRUE)==s32); #assert(type_of(EGL.NONE)==Word); item:EGL; return EGL.TRUE+cast(s32)EGL.NONE+item.value-9; }",
        42,
    );
}
