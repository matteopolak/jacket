// test: compile_fail("Web: hole `{1.25}` has unsupported type float32") skip_on(.WASM, reason = "Web requires native worker threads")
#import "Basic";
#import "Extensions/Web";

Point :: struct {
    x: int;
}

Card :: (out: *Out) {}

main :: () {
    out: Out;
    name := "x";
    point: Point;
    html(*out, "<p>{1.25}</p>");
}
