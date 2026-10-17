C :: #import "Compiler"; // jailint: allow(unused_import) the declaration is part of what the test compiles

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
