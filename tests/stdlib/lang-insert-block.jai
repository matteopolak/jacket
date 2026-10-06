// `#insert -> string { ... }` / `#insert -> Code { ... }` run a procedure body at compile
// time and insert its result; `#insert,scope()` resolves inserted code at the insertion
// site; `#caller_code` is the Code of the call expression.
// rules: code.3 code.11
#import "Basic";

Fields :: struct(names: [] string) {
    #insert -> string {
        builder: String_Builder;
        defer free_buffers(*builder);
        for names print_to_builder(*builder, "%: int;\n", it);
        return builder_to_string(*builder);
    }
}

total: int;

repeat_with_i :: (count: int, body: Code) #expand {
    #insert -> string {
        builder: String_Builder;
        defer free_buffers(*builder);
        for 0..count - 1 {
            print_to_builder(*builder, "{ i :: %; #insert,scope() body; }\n", it);
        }
        return builder_to_string(*builder);
    }
}

call_text :: (value: int, call := #caller_code) -> Code #expand {
    return call;
}

main :: () -> int {
    fields :: string.["alpha", "beta"];
    record: Fields(fields);
    record.alpha = 3;
    record.beta = 4;
    if record.alpha + record.beta != 7 return 1;

    x := 3;
    #insert -> Code {
        return #code x = (x * 10) + 5;
    }
    if x != 35 return 2;

    total = 0;
    repeat_with_i(4, #code {
        total += i;
    });
    if total != 6 return 3;

    code := call_text(1);
    if type_of(code) != Code return 4;
    return 0;
}
