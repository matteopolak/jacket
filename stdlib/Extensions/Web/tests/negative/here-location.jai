// test: compile_fail("here-location.jai:17:1: error[metaprogram-report]: Web: unclosed {") skip_on(.WASM, reason = "Web requires native worker threads")
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
    html(*out, #string HTML
<p>
{name
HTML);
}
