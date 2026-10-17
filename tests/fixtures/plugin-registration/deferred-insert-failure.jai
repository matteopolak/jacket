using #import "Basic";

#placeholder Body;
Callback :: #type () -> int;

deferred :: () -> int {
    #insert Body;
}

emitter :: () -> string {
    print("insert prefix once\n");
    callback: Callback = deferred;
    callback();
    return "";
}

#insert emitter();

main :: () {}
