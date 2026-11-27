// test: compile_fail("Web: value hole outside text or quoted attribute") skip_on(.WASM, reason = "Web requires native worker threads")
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
    html(*out, "<{name}></p>");
}
