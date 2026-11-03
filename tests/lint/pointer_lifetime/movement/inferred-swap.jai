#import "Basic";

exchange :: (left: *$T, right: *T) {
    saved := left.*;
    left.* = right.*;
    right.* = saved;
}

pretend_exchange :: (left: *int, right: *int) {} @SwapsArguments

bad :: () {
    values: [2] int;
    bad_slot := *values[0];
    exchange(*values[0], *values[1]);
    print("%", bad_slot.*);
}

good_same :: () {
    values: [2] int;
    slot := *values[0];
    exchange(*values[0], *values[0]);
    print("%", slot.*);
}

good_ignored_note :: () {
    values: [2] int;
    slot := *values[0];
    pretend_exchange(*values[0], *values[1]);
    print("%", slot.*);
}

main :: () {}
