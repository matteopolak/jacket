C :: #import "Compiler";

#placeholder Body;
Callback :: #type () -> int;

deferred :: () -> int {
    #insert Body;
}

#run,stallable {
    callback: Callback = deferred;
    callback();
}

main :: () {}
