using #import "Basic";
C :: #import "Compiler";

#placeholder Body;
body_scope :: #code Body;
Comparator :: #type (left: *void, right: *void) -> s32 #c_call;
abs :: (value: s32) -> s32 #foreign;
qsort :: (base: *void, count: u64, width: u64, compare: Comparator) #foreign;

compare :: (left: *void, right: *void) -> s32 #c_call {
    #insert Body;
}

generate :: () -> string {
    assert(abs(-7) == 7);
    return "return (cast(*s32) left).* - (cast(*s32) right).*;";
}

generated_compare :: (left: *void, right: *void) -> s32 #c_call {
    #insert generate();
}

#run,stallable {
    callback: Comparator = compare;

    assert(abs(-7) == 7);
    C.add_build_string("Body :: \"return (cast(*s32) left).* - (cast(*s32) right).*;\";", -1, body_scope);
    values: [3] s32 = .[3, 1, 2];

    qsort(values.data, 3, size_of(s32), callback);
    assert(values[0] == 1 && values[1] == 2 && values[2] == 3);
    values = .[3, 1, 2];
    qsort(values.data, 3, size_of(s32), generated_compare);
    assert(values[0] == 1 && values[1] == 2 && values[2] == 3);
    print("native callback ready on exposure\n");
}

main :: () {}
