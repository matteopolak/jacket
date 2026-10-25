#import "Basic";

Style :: enum {
    PLAIN;
    FANCY;
}

render :: (style := Style.PLAIN) -> string {
    return ifx style == .FANCY then "fancy" else "plain";
}

fancy :: #bake_arguments render(style = .FANCY);

main :: () {
    assert(fancy() == "fancy", "a baked argument without a declared type takes the type of its default");
    print("ok\n");
}
