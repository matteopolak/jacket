#import "Basic";

Parser :: struct {
    depth: int;
}

main :: () {
    parser := New(Parser);
    parser.depth = 3;
    if parser.depth > 2 print("deep\n");

    buffer := cast(*u8) alloc(64);
    buffer[0] = 1;

    name := copy_string("name");
    if name.count > 2 print("long\n");
}
