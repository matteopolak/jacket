#import "Basic";

Size :: struct {
    width: int;
    height: int;
}

main :: () {
    s: Size;
    width := 4;
    width = width;
    s.height = s.height;
    print("% %\n", s, width);
}
