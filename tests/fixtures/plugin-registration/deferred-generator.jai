using #import "Basic";
C :: #import "Compiler";

#placeholder Body;
body_scope :: #code Body;
Callback :: #type () -> int;

generate :: () -> string {
    return Body;
}

deferred :: () -> int {
    #insert generate();
}

#run,stallable {
    callback: Callback = deferred;
    C.add_build_string("Body :: \"return 42;\";", -1, body_scope);
    assert(callback() == 42);
    print("generator dependency ready\n");
}

main :: () {}
