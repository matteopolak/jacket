Info :: struct {
    env: *Dict(*Text, *Text);
    count :: (self: *Info) -> *Dict {
        f: (obj: *void) -> *Dict #c_call = null;
        if f != null return f(self);
        return null;
    }
}
Dict :: struct (key: Type, value: Type) { size: int; }
Text :: struct { length: int; }
main :: () -> int {
    info: Info;
    g: (a: int) -> *Dict = null;
    if g != null return 2;
    if info.env != null return 1;
    return 17;
}
