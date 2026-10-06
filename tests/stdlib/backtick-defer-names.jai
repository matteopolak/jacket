// rules: macro.5 macro.6
#import "Basic";

log: [..] int;

open_thing :: () #expand {
    `thing := New(int);
    `thing.* = 5;
    `defer {
        array_add(*log, `thing.*);
        free(`thing);
    }
}

main :: () {
    {
        open_thing();
        thing.* = 9;
    }
    assert(log.count == 1 && log[0] == 9);
    print("ok\n");
}
