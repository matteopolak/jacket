using #import "Basic";
C :: #import "Compiler";

#placeholder Body;
body_scope :: #code Body;
Callback :: #type () -> int;

deferred :: () -> int {
    #insert Body;
}

#run,stallable {
    callback: Callback = deferred;
    print("failure prefix once\n");
    C.add_build_string("Body :: \"assert(false); return 42;\";", -1, body_scope);
    callback();
}

main :: () {}
