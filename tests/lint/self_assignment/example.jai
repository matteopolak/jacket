#import "Basic";

Size :: struct {
    width: int;
    height: int;
}

set_width :: (size: *Size, width: int) {
    width = width;
}

main :: () {
    s: Size;
    set_width(*s, 4);
    print("%\n", s);
}
