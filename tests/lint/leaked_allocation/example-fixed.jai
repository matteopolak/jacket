#import "Basic";

Parser :: struct {
    depth: int;
}

main :: () {
    parser := New(Parser);
    defer free(parser);
    parser.depth = 3;

    if parser.depth > 2 print("deep\n");
}
