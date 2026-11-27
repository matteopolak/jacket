// test: compile_fail("Web: control flow is only allowed in text or between attributes") skip_on(.WASM, reason = "Web requires native worker threads")
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
    html(*out, "<a title=\"{if true}x{end}\"></a>");
}
