using #import "Basic";

Callback :: #type () -> int;

source :: () -> string {
    callback: Callback = deferred;
    callback();
    return "return 42;";
}

deferred :: () -> int {
    #insert source();
}

#run,stallable {
    callback: Callback = deferred;
    callback();
}

main :: () {}
