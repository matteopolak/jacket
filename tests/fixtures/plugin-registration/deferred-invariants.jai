using #import "Basic";
C :: #import "Compiler";

#placeholder First_Body, Second_Body;
body_scope :: #code First_Body;
Callback :: #type (value: *int, amount: int) -> int;
prefixes: int;

first :: (value: *int, amount: int) -> int {
    #insert First_Body;
}

second :: (value: *int, amount: int) -> int {
    #insert Second_Body;
}

exercise :: () {
    a: Callback = first;
    b: Callback = second;
    local := 30;
    pointer := *local;

    prefixes += 1;
    print("prefix once\n");
    C.add_build_string("First_Body :: \"value.* += amount; return value.*;\";", -1, body_scope);
    result_a := a(pointer, 4);

    assert(result_a == 34 && local == 34 && pointer == *local && prefixes == 1);
    C.add_build_string("Second_Body :: \"value.* += amount; return value.*;\";", -1, body_scope);
    result_b := b(pointer, 8);

    assert(result_a == 34 && result_b == 42 && local == 42 && pointer == *local && prefixes == 1);
    print("two bodies resumed\n");
}

#run,stallable exercise();

main :: () {
    value := 30;
    assert(first(*value, 4) == 34);
    assert(second(*value, 8) == 42 && value == 42);
    print("deferred runtime ok\n");
}
