#import "Basic";

build :: () -> *int {
    local := 42;
    return *local;
}

FROZEN :: #run build();
main :: () { print("%", FROZEN.*); }
