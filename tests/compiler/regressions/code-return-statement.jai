// A `return` in code that a macro inserts, when the code was written in the caller, returns
// from the caller: the rest of the macro does not run, and pending defers do.
#import "Basic";

log: [..] string;

m :: (c: Code) #expand {
    defer array_add(*log, "macro defer");
    #insert c;
    array_add(*log, "after");
}

block_form :: () -> int {
    defer array_add(*log, "caller defer");
    m(#code {
        return 5;
    });

    return 1;
}

statement_form :: () -> int {
    m(#code return 6);
    return 1;
}

main :: () {
    assert(block_form() == 5);
    assert(log.count == 2);
    assert(log[0] == "macro defer");
    assert(log[1] == "caller defer");

    array_reset(*log);
    assert(statement_form() == 6);
    assert(log.count == 1);
    assert(log[0] == "macro defer");
}
