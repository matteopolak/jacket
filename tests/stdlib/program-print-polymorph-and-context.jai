// Program_Print: polymorphic variables, `#caller_code`, `#add_context`, `#module_parameters` and
// the `#load`, `#place` and `#overlay` nodes, which used to arrive as .PLACEHOLDER or unchecked.
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
    c0 := #code f :: (a: $T, b: $$U, c: $V/Number) -> T {}
    check(c0, "f :: (a: $T, b: $$U, c: $V/Number) -> T {}");

    c1 := #code g :: (x: $T/interface Number) {}
    check(c1, "g :: (x: $T/interface Number) {}");

    c2 := #code a := #caller_code;
    check(c2, "a := #caller_code");

    c3 := #code #add_context depth: int = 3;
    check(c3, "#add_context depth : int = 3");

    c5 := #code {
        #load "other.jai";
    }
    check(c5, "{\n    #load \"other.jai\";\n}");

    c6 := #code S :: struct {
        a: int;
        #place a;
        b: int;
    }
    check(c6, "S :: struct {\n    a : int;\n    #place a;\n    b : int;\n}");

    c7 := #code O :: struct {
        a: int;
        #overlay(a) b: float32;
    }
    check(c7, "O :: struct {\n    a : int;\n    #overlay(a);\n    b : float32;\n}");

    c8 := #code {
        #module_parameters (A := 1, B: string = "x")(C := 2);
    }
    check(c8, "{\n    #module_parameters(A := 1, B: string = \"x\")(C := 2);\n}");

    // The location directives are the constants they evaluate to.
    c9 := #code x := #line;
    line := #line;
    check(c9, tprint("x := %", line - 1));
    c10 := #code f := #file;
    check(c10, tprint("f := \"%\"", #file));
    c11 := #code d := #filepath;
    check(c11, tprint("d := \"%\"", #filepath));

    print("ok\n");
}

main :: () {}
