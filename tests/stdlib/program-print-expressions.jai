// Program_Print: code_to_string and print_expression over #code snippets.
#import "Basic";
#import "Compiler";
#import "Program_Print";

check :: (code: Code, expected: string, loc := #caller_location) {
    actual := code_to_string(code);
    if actual != expected {
        print("line %: expected [%] got [%]\n", loc.line_number, expected, actual);
        exit(1);
    }
}

#run {
    c1 := #code a_constant :: 10;
    check(c1, "a_constant :: 10");

    c2 := #code a := Vector3.{ 1, 2, 3 };
    check(c2, "a := Vector3.{1, 2, 3}");

    c3 := #code x + y * 2;
    check(c3, "x + y * 2");

    c4 := #code (x + y) * 2;
    check(c4, "(x + y) * 2");

    c5 := #code -x;
    check(c5, "-x");

    c6 := #code f(a, b = 3);
    check(c6, "f(a, b = 3)");

    c7 := #code s := "hi\n";
    check(c7, "s := \"hi\\n\"");

    c8 := #code n: int = 5;
    check(c8, "n : int = 5");

    c9 := #code {
        if a == b then_do();
        else other();
    };
    check(c9, "{\n    if a == b {\n        then_do();\n    } else {\n        other();\n    }\n}");

    c10 := #code {
        while i < 10 {
            i += 1;
        }
    };
    check(c10, "{\n    while i < 10 {\n        i += 1;\n    }\n}");

    c11 := #code {
        for i: 0..9 {
            total += i;
        }
    };
    check(c11, "{\n    for i: 0..9 {\n        total += i;\n    }\n}");

    c12 := #code {
        for *it, idx: arr {
            it.x = idx;
        }
    };
    check(c12, "{\n    for *it, idx: arr {\n        it.x = idx;\n    }\n}");

    c13 := #code {
        a := 1;
        b := a[0];
    };
    check(c13, "{\n    a := 1;\n    b := a[0];\n}");

    c14 := #code cast(float) n;
    check(c14, "cast(float) n");

    c15 := #code p.*;
    check(c15, "p.*");

    c16 := #code .[1, 2, 3];
    check(c16, ".[1, 2, 3]");

    c17 := #code flag && !other || true;
    check(c17, "flag && !other || true");

    c18 := #code a.b.c;
    check(c18, "a.b.c");

    c19 := #code f :: (x: int, y := 2) -> int {
        return x + y;
    };
    check(c19, "f :: (x: int, y := 2) -> int {\n    return x + y;\n}");

    c20 := #code V :: struct {
        x: float;
        y := 3;
    };
    check(c20, "V :: struct {\n    x : float;\n    y := 3;\n}");

    c21 := #code {
        v := ifx a > b then 1 else 2;
        defer free(p);
    };
    check(c21, "{\n    v := ifx a > b then 1 else 2;\n    defer free(p);\n}");

    // Parentheses follow Jai's precedence: bitwise and shift operators share a level above
    // `*`, `%` sits between `*` and `+`, and a cast operand of a bitwise operator keeps its
    // parentheses.
    p1 := #code (a << 2) | b;
    check(p1, "a << 2 | b");
    p2 := #code a | (b << 2);
    check(p2, "a | (b << 2)");
    p3 := #code (a + b) & c;
    check(p3, "(a + b) & c");
    p4 := #code a * (b & c);
    check(p4, "a * b & c");
    p5 := #code (a % b) * c;
    check(p5, "(a % b) * c");
    p6 := #code a - (b % c);
    check(p6, "a - b % c");
    p7 := #code (a == b) < c;
    check(p7, "(a == b) < c");
    p8 := #code (cast(u32) b) << 4;
    check(p8, "(cast(u32) b) << 4");
    p9 := #code (cast(u32) b).x;
    check(p9, "(cast(u32) b).x");

    // print_expression on a node below the root.
    root, exprs := compiler_get_nodes(c3);
    builder: String_Builder;
    print_expression(*builder, root);
    assert(builder_to_string(*builder) == "x + y * 2");
}

main :: () {}
