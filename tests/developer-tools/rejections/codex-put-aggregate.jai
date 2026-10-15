#import "Basic";
C :: #import "Codex";

Record :: struct {
    value: s64;
}

main :: () {
    builder: String_Builder;
    C.put(*builder, Record.{ 42 });
}
