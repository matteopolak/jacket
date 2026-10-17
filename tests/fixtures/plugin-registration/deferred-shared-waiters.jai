using #import "Basic";
C :: #import "Compiler";

#placeholder Body;
body_scope :: #code Body;
Callback :: #type () -> int;
prefixes: int;

deferred :: () -> int {
    #insert Body;
}

#run,stallable {
    callback: Callback = deferred;
    prefixes += 1;
    assert(callback() == 42 && prefixes == 2);
    print("outer waiter resumed\n");
}

#run,stallable {
    callback: Callback = deferred;
    prefixes += 1;
    assert(callback() == 42 && prefixes == 2);
    print("inner waiter resumed\n");
}

#run C.add_build_string("Body :: \"return 42;\";", -1, body_scope);

main :: () {}
