// rules: proc.1 proc.2 proc.3 proc.4 proc.5 proc.6 proc.7 proc.8 proc.9 proc.10 proc.11
// Self-checking program for the claims in docs/language/procedures.md.
#import "Basic";
#import "Compiler";
#import "String";

named :: (a: int, b := 7, c := "z") -> int, string {
    return a * 100 + b, c;
}

pair :: () -> int, string {
    return 4, "four";
}

sum :: (args: ..int) -> int {
    t := 0;
    for args t += it;
    return t;
}

show :: (x: int) -> string {
    return "int";
}

show :: (x: float) -> string {
    return "float";
}

show :: (x: string) -> string {
    return "string";
}

from_int :: (x: int) -> int {
    return 1;
}

from_float :: (x: float) -> int {
    return 2;
}

either :: from_int;
either :: from_float;

caller_line :: (l := #caller_location) -> int {
    return l.line_number;
}

caller_file :: (l := #caller_location) -> string {
    return l.fully_pathed_filename;
}

// Kind of the root node of the code a plain procedure receives.
root_kind :: (code: Code) -> Code_Node.Kind {
    return compiler_get_nodes(code).kind;
}

STORED :: #code 4 + 5;
PASSED_EXPRESSION :: #run root_kind(1 + 2 * 3);
PASSED_CODE :: #run root_kind(STORED);

old_api :: (x: int) -> int #deprecated "use new_api" {
    return x + 1;
}

quiet :: (x: int) -> int #no_debug {
    return x * 3;
}

main :: () {
    // proc.1: default and named arguments.
    v, c := named(1, c = "q");
    assert(v == 107 && c == "q", "named(1, c = q)");
    w := named(b = 1, a = 2);
    assert(w == 201, "named(b = 1, a = 2)");
    // proc.2: multiple return values.
    i, s := pair();
    assert(i == 4 && s == "four", "pair()");
    // proc.3: variadic arguments.
    assert(sum(1, 2, 3) == 6 && sum() == 0, "sum(..int)");
    assert(sum(..int.[4, 5]) == 9, "spread an array into a variadic");

    // proc.4: overloads chosen by argument types.
    assert(show(1) == "int" && show(2.5) == "float" && show("s") == "string", "overload by type");
    // proc.5: constant aliases join one overload set.
    assert(either(1) == 1 && either(1.0) == 2, "alias overload set");

    // proc.6: #caller_location is the call site.
    here := #line;
    at := caller_line();
    assert(at == here + 1, "caller location line"); // The call is on the line after `here`.
    assert(ends_with(caller_file(), "rules-procedures.jai"), "caller location file");
    // proc.7: #location(), #file and #line give their own position.
    loc := #location();
    line := #line;
    assert(loc.line_number + 1 == line, "#location() line");
    assert(ends_with(#file, "rules-procedures.jai"), "#file");
    assert(ends_with(loc.fully_pathed_filename, "rules-procedures.jai"), "#location() file");

    // proc.8: an expression passed to a plain Code parameter arrives as code.
    assert(PASSED_EXPRESSION == .BINARY_OPERATOR, "1 + 2 * 3 as code");
    // proc.9: a Code-valued argument passes its value, not an identifier.
    assert(PASSED_CODE == .BINARY_OPERATOR, "#code constant passes its value");

    // proc.10 / proc.11: #deprecated and #no_debug procedures compile and run.
    assert(old_api(1) == 2, "#deprecated call runs");
    assert(quiet(2) == 6, "#no_debug call runs");

    print("ok\n");
}
