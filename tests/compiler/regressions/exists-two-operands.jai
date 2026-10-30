// rules: scope.17
// `#exists(query, other)` is the answer for `query`; `other` is checked as an ordinary expression.
#import "Basic";

Handler :: struct {
    proc: (x: int) -> int;
}

Holder :: struct {
    handler: Handler;
}

main :: () {
    h: Holder;
    #if #exists(h.handler.proc, h.handler) {
        found := true;
    } else {
        found := false;
    }

    assert(found);

    #if #exists(h.handler.nothing, h.handler) {
        missing := true;
    } else {
        missing := false;
    }

    assert(!missing);

    #if !#exists(h.nothing, h) {
        negated := true;
    } else {
        negated := false;
    }

    assert(negated);
}
