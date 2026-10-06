// rules: code.1 code.2 code.3 code.4 code.6 code.8 code.9 code.10 code.11 code.13
// Pins the claims of docs/metaprogramming/code-values-and-insertion.md.
#import "Basic";
#import "Compiler";

// code.1: `#code` makes a value of type Code.
snippet :: #code seen_count += 1;
#assert type_of(snippet) == Code;

seen_count := 0;

// code.2: insertion at top level (a declaration from a string).
#insert "TOP_LEVEL_VALUE :: 17;";

// code.3: `#insert -> string { ... }` runs at compile time, at top level as well.
#insert -> string {
    return tprint("GENERATED_% :: %;", "SQUARE", 9 * 9);
}

// code.6: an operand that produces nothing inserts nothing.
nothing :: () {}
#insert #run nothing();

// code.8: Code rebuilt with compiler_get_code resolves at the insertion site; names it
// does not find there come from where the nodes were written.
rebuilt :: (code: Code) -> Code {
    root := compiler_get_nodes(code);
    return compiler_get_code(root);
}
base_value := 100;
with_rebuilt :: (code: Code) #expand {
    offset := 7;
    copy :: #run rebuilt(code);
    #insert copy;
}

// code.9: an inserted Code expression is checked where it was written.
double_it :: (expr: Code) -> int #expand {
    local_value := 1000;
    return (#insert expr) * 2;
}

// code.11: `#insert,scope()` checks in the macro's scope, so the macro's names are visible.
index_total := 0;
each_index :: (count: int, body: Code) #expand {
    for 0..count - 1 {
        index := it;
        #insert,scope() body;
    }
}

// code.13: the verified program of the doc, with output collected instead of printed.
log: [..] string;
make_decl :: (name: string) -> string {
    return tprint("% :: 42;\n", name);
}
#insert #run make_decl("ANSWER");
twice_with_defer :: (c: Code) #expand {
    for 0..1 {
        `defer array_add(*log, tprint("d%", it));
        #insert,scope() c;
    }
}
code_val :: #code array_add(*log, "hi");

main :: () {
    // code.2 / code.4: statement position; the Code value contributes its statements.
    #insert snippet;
    #insert snippet;
    assert(seen_count == 2, "code.4: inserting a statement Code twice runs it twice");
    block :: #code {
        seen_count += 10;
        seen_count *= 2;
    };
    #insert block;
    assert(seen_count == 24, "code.4: a block Code yields all its statements");

    // code.2: expression position.
    value := #insert #code 3 + 4;
    assert(value == 7, "code.2: #insert in expression position");
    assert(TOP_LEVEL_VALUE == 17, "code.2: top-level #insert of a string");

    // code.3: `-> Code` and `-> string` bodies at statement level.
    counter := 1;
    #insert -> Code {
        return #code counter += 41;
    };
    assert(counter == 42, "code.3: #insert -> Code");
    #insert -> string {
        return "counter *= 2;";
    };
    assert(counter == 84, "code.3: #insert -> string");
    assert(GENERATED_SQUARE == 81, "code.3: top-level #insert -> string");

    // code.8
    with_rebuilt(#code base_value += offset);
    assert(base_value == 107, "code.8: rebuilt code sees the insertion site and its origin");

    // code.9
    local_value := 5;
    assert(double_it(#code local_value + 1) == 12, "code.9: Code expression uses its defining scope");

    // code.10: a string in expression position is checked in the insertion scope.
    here := 20;
    sum := #insert "here + 1";
    assert(sum == 21, "code.10: string expression sees insertion-scope names");
    product := (#insert "here + 1") * 2;
    assert(product == 42, "code.10: string is parsed as a parenthesized expression");

    // code.11
    index_total = 0;
    each_index(4, #code {
        index_total += index;
    });
    assert(index_total == 6, "code.11: #insert,scope() sees the macro's locals");

    // code.13
    print("%\n", ANSWER);
    assert(ANSWER == 42, "code.13: ANSWER");
    twice_with_defer(#code array_add(*log, "body"));
    #insert code_val;
    #insert -> string {
        return "array_add(*log, \"from string\");";
    };
    x := 5;
    #insert #run tprint("x += %;", 3);
    assert(x == 8, "code.13: x is 8");
    expected :: string.["body", "d0", "body", "d1", "hi", "from string"];
    assert(log.count == expected.count, "code.13: log length");
    for expected assert(log[it_index] == it, "code.13: log order");
}
