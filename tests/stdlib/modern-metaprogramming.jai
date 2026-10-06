// Language and Compiler-module features newer metaprogramming libraries rely on
// (match-jai, yield-jai, epic-fail): see docs/metaprogramming/compiler-records.md.
// rules: macro.13 macro.14 macro.15
#import "Basic";
#import "Compiler";
#import "Program_Print";
#import "String";

printed :: (node: *Code_Node) -> string {
    builder: String_Builder;
    print_expression(*builder, node);
    return builder_to_string(*builder);
}

// Per-name backticks in a multi-declaration.
two :: () -> int, bool {
    return 7, true;
}
declare_two :: () #expand {
    a, `b := two();
}

// `a, b :: f()` evaluates f once and binds each value.
pair_consts :: () -> int, string {
    return 3, "three";
}
N, NAME :: #run pair_consts();
#assert N == 3 && NAME == "three";

// `#exists(`name)` inside a macro looks in the caller.
declare_once :: () #expand {
    #if !#exists(`once) {
        `once := 1;
    } else {
        `once += 1;
    }
}

// `using` an enum-typed value brings its members into scope.
Fruit :: enum {
    APPLE;
    MANGO;
}
Basket :: struct {
    tag: Fruit;
}
is_mango :: (basket: Basket) -> bool {
    using basket.tag;
    if basket.tag == {
        case MANGO; return true;
    }
    return false;
}

// Code made by compiler_get_code without a scope resolves where it is inserted, falling
// back on where its nodes were written (`helper` is not visible at the insertion site).
#scope_file
helper :: () -> int {
    return 40;
}
#scope_export
made_code :: () -> Code {
    root := compiler_get_nodes(#code helper() + LOCAL_ONLY);
    return compiler_get_code(root);
}

main :: () {
    declare_two();
    assert(b);

    declare_once();
    declare_once();
    assert(once == 2);

    assert(is_mango(.{ .MANGO }) && !is_mango(.{ .APPLE }));

    LOCAL_ONLY :: 2;
    made := #insert #run made_code();
    assert(made == 42);

    // `-1` in a `#code` is one negative literal, and backticks survive printing.
    #run {
        c: Code;
        c = #code case -1;
        node := compiler_get_nodes(c).(*Code_Case);
        assert(node.condition.kind == .LITERAL && node.condition.(*Code_Literal)._s64 == -1);
        assert(printed(compiler_get_nodes(#code `x + y)) == "`x + y");
        assert(contains(printed(compiler_get_nodes(#code {
            `z := 1;
        })), "`z := 1;"));
    }

    // Printing of #assert, #exists and named struct-literal fields.
    #run {
        text := printed(compiler_get_nodes(#code {
            #assert(1 < 2, "math");
            a := #exists(foo);
        }));
        assert(contains(text, "#assert(1 < 2, \"math\")"), "%", text);
        assert(contains(text, "#exists(foo)"), "%", text);
    }

    // Typechecked expressions carry their type.
    x := 2;
    #insert -> string {
        root := compiler_get_nodes(#code x + 2 == 4).(*Code_Binary_Operator);
        assert(root.left.type == type_info(int), "% %", root.left.type, root.left.kind);
        assert(root.right.type == type_info(int));
        return "";
    };

    // code_of: the Code of a procedure, header included.
    #run {
        root := compiler_get_nodes(code_of(two));
        assert(root.kind == .PROCEDURE_HEADER);
        assert((cast(*Code_Procedure_Header) root).returns.count == 2);
    }
}
