#import "Basic";

build :: () -> *int {
    local := 42;
    return *local;
}

select :: () -> (() -> *int) {
    return build;
}

CALLBACK :: #run select();

main :: () {
    print("%", CALLBACK().*);
}
