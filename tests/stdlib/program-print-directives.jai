// Program_Print: directive nodes that used to arrive as .PLACEHOLDER.
#import "Basic";
#import "Compiler";

check :: (code: Code, expected: string, loc := #caller_location) {
    actual := code_to_string(code);
    if actual != expected {
        print("line %: expected [%] got [%]\n", loc.line_number, expected, actual);
        exit(1);
    }
}

#run {
    c1 := #code x := #this;
    check(c1, "x := #this");

    c2 := #code s := #procedure_name(f);
    check(c2, "s := #procedure_name(f)");

    c3 := #code g := #bake_arguments f(a = 1);
    check(c3, "g := #bake_arguments f(a = 1)");

    c4 := #code g := #bake_constants f(T = int);
    check(c4, "g := #bake_constants f(T = int)");

    c5 := #code b := #bytes data;
    check(c5, "b := #bytes data");

    print("ok\n");
}

main :: () {}
