#import "Basic";

Size :: struct {
    width: int;
    height: int;
}

main :: () {
    s: Size;
    width := 4;
    s.width = width;
    t := s;
    s = t;
    print("% %\n", s, width);
}
