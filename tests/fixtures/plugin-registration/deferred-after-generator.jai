using #import "Basic";
C :: #import "Compiler";

#placeholder Body;
body_scope :: #code Body;
Callable :: #type () -> int;

generate :: () -> string {
    print("completed generator once\n");
    return "";
}

deferred :: () -> int {
    #insert generate();
    #insert Body;
}

#run,stallable {
    C.add_build_string("Body :: \"return 42;\";", -1, body_scope);
    callback: Callable = deferred;
    callback();
}

main :: () {}
