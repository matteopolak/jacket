#import "Basic";
Box :: struct { pointer: *int; }
saved: Box;

local_sink :: () {
    box: Box;
    alias := *box;
    value := 42;
    alias.pointer = *value;
    print("%", box.pointer.*);
}

rebound_sink :: (replace: (p: **Box)) {
    box: Box;
    alias := *box;
    replace(*alias);
    value := 42;
    alias.pointer = *value;
}

reassigned_sink :: () {
    box: Box;
    alias := *box;
    alias = *saved;
    value := 42;
    alias.pointer = *value;
}

real_escape :: () {
    value := 42;
    saved.pointer = *value;
}
