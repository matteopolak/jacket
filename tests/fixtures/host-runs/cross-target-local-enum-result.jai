// A plain #run whose result is an enum declared inside a struct has no counterpart that the
// target program can find by name: in a cross build the run stays on the target compiler.
#import "Basic";

Holder :: struct {
    Mode :: enum {
        FIRST;
        SECOND;
    }
}

pick :: () -> Holder.Mode {
    return .SECOND;
}

CHOSEN :: #run pick();

#assert CHOSEN == .SECOND;

main :: () {}
