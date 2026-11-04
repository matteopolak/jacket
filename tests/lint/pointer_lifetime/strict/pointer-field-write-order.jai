#import "Basic";

saved: string;

get_empty :: () -> string {
    view: [] u8;
    pointer := view.data;
    view.data = talloc(4);
    return to_string(pointer, 0);
}

keep :: () {
    saved = get_empty();
}

main :: () {
    keep();
}
