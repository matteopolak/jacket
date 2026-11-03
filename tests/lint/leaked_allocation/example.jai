#import "Basic";

Parser :: struct {
    depth: int;
}

main :: () {
    parser := New(Parser);
    parser.depth = 3;

    if parser.depth > 2 print("deep\n");
}
