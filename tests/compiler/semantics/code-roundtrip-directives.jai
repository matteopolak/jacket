// Statement-level directives, loop-variable spellings and statement notes survive
// `compiler_get_nodes` / `compiler_get_code`.
#import "Basic";
#import "Compiler";

rebuild :: (code: Code) -> Code {
    return compiler_get_code(compiler_get_nodes(code), code);
}

Hue :: enum {
    RED;
    GREEN;
    BLUE;
}

counter: int;

bump :: () -> int {
    counter += 1;
    return counter;
}

#insert #run rebuild(#code {
    #scope_file
    file_private :: 5;

    #scope_module
    module_visible :: 6;

    #scope_export
    exported :: 7;

    #placeholder from_elsewhere;
});

main :: () {
    #insert #run rebuild(#code {
        out: [..] string;
        c := Hue.GREEN;

        if c == {
            case .RED;
                array_add(*out, "red");
            case .GREEN;
                array_add(*out, "green");
                #through;
            case .BLUE;
                array_add(*out, "blue");
        }

        assert(out.count == 2 && out[0] == "green" && out[1] == "blue");

        #run bump();
        assert(counter == 0);

        values := int.[3, 4, 5];
        sum := 0;

        for `x, `i: values sum += x * 10 + i;

        assert(sum == 3 * 10 + 4 * 10 + 5 * 10 + 0 + 1 + 2);

        total := 0;
        for #v2 1..4 total += it;

        assert(total == 10);

        assert(file_private + module_visible + exported == 18);
    });
}
