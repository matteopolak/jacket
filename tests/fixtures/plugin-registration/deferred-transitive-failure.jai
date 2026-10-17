using #import "Basic";
C :: #import "Compiler";

#placeholder Body;
body_scope :: #code Body;
Callback :: #type () -> int;

deferred :: () -> int {
    #insert Body;
}

generator :: () -> string {
    print("generator prefix once\n");
    C.add_build_string("Body :: \"return 42;\";", -1, body_scope);
    callback: Callback = deferred;

    assert(callback() == 42);
    assert(false);

    return "";
}

outer :: () {
    #insert generator();
}

#run,stallable outer();
#run print("unrelated run\n");

main :: () {}
