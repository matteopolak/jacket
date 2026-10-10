#import "Basic";

Size :: struct {
    width: int;
    height: int;
}

Info :: struct {
    mode: int;
}

// The target is the field brought in by `using`, the value the local declared outside.
fill :: (info: *Info) {
    mode := 3;
    {
        using info;
        mode = mode;
    }
    print("%\n", info.mode);
}

main :: () {
    info: Info;
    fill(*info);
    s: Size;
    width := 4;
    s.width = width;
    t := s;
    s = t;
    print("% %\n", s, width);
}
