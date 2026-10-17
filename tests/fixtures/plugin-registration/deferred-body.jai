// Reviewed boundary: signature first, body supplied through a scoped placeholder.
using #import "Basic";
C :: #import "Compiler";

#placeholder Generated_Body;
body_scope :: #code Generated_Body;
Callable :: #type (value: int) -> int;
prefix_effects: int;

deferred :: (value: int) -> int {
    #insert Generated_Body;
}

exercise :: () {
    argument := 1;

    prefix_effects += 1;
    C.add_build_string("Generated_Body :: \"return value + 41;\";", -1, body_scope);
    callback: Callable = deferred;
    result := callback(argument);

    assert(result == 42 && argument == 1 && prefix_effects == 1);
    print("deferred body resumed once\n");
}

#run,stallable exercise();

main :: () {}
