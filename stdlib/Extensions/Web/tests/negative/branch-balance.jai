// test: compile_fail("Web: unbalanced tags across if branches") skip_on(.WASM, reason = "Web requires native worker threads")
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
    html(*out, "{if true}<p>{else}<div>{end}");
}
