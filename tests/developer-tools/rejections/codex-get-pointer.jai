#import "Basic";
C :: #import "Codex";

main :: () {
    input := "invalid pointer payload";
    value: *u8;
    C.get(*input, *value);
}
