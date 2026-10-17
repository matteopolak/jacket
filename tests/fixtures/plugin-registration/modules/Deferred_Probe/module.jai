#module_parameters(VALUE := 41);

#placeholder Body;
body_scope :: #code Body;
Callback :: #type (value: int) -> int;
prefixes: int;

#load "compute.jai";

evaluate :: (value: int) -> int {
    callback: Callback = compute;

    prefixes += 1;
    C.add_build_string("Body :: \"return value + VALUE;\";", -1, body_scope);
    result := callback(value);

    assert(prefixes == 1 && result == value + VALUE);

    return result;
}

#scope_file
using #import "Basic";
C :: #import "Compiler";
