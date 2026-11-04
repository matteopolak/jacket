#import "Basic";

build :: () -> *int {
    local := 42;
    return *local;
}

FROZEN :: #run build();

main :: () {
    runtime := build();
    print("%", runtime.*);
}
