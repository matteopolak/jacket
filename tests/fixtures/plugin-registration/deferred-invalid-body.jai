C :: #import "Compiler";

#placeholder Body;
body_scope :: #code Body;
Callback :: #type () -> int;

deferred :: () -> int {
    #insert Body;
}

#run,stallable {
    callback: Callback = deferred;
    C.add_build_string("Body :: \"return absent;\";", -1, body_scope);
    callback();
}

main :: () {}
