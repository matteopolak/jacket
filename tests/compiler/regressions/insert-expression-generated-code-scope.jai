// `#insert,scope(call)` in expression position of generated code (`compiler_get_code`) still
// sees the names of the procedure it is written in: here the polymorphic return type
// `$COROUTINE` that the procedure's `#modify` binds and the caller's scope lacks.
#import "Basic";
#import "Compiler";

Pair :: struct {
    a: int;
    b: int;
}

Coroutine :: struct(T: Type) {
    state: T;
    next: (*Coroutine(T)) -> int;
}

Description :: struct {
    Whole: Type;
}

generate :: () -> Code #compile_time {
    root := compiler_get_nodes(#code (self: *COROUTINE) -> int {
        return self.state.a + self.state.b;
    });
    return compiler_get_code(root);
}

make :: ($description: Description, $call := #caller_code) -> $COROUTINE #modify {
    COROUTINE = description.Whole;
    return true;
} {
    return .{ next = #insert,scope(call) #run generate() };
}

main :: () {
    c := make(Description.{ Coroutine(Pair) });
    c.state = .{ a = 1, b = 2 };
    assert(c.next(*c) == 3);
    print("ok\n");
}
