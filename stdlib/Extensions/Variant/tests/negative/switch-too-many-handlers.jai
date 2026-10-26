// test: compile_fail
#import "Basic";
#import "Extensions/Variant";

// variant_switch takes at most MAX_SWITCH_CASES handlers.
main :: () {
    v: Variant(s64, string);

    variant_switch(*v,
        (it: *s64) {}, (it: *string) {}, (it: *bool) {}, (it: *u8) {},
        (it: *u16) {}, (it: *u32) {}, (it: *u64) {}, (it: *s8) {},
        (it: *s16) {}, (it: *s32) {}, (it: *float32) {}, (it: *float64) {},
        (it: *Any) {}, (it: *Type) {}, (it: *[2] int) {}, (it: *[3] int) {},
        (it: *[4] int) {},
    );
}
