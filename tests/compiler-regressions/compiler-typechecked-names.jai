// Names in TYPECHECKED bodies that are not names of the enclosing scope: a member after `.`,
// a named argument and a loop's `it` must not resolve to a global that happens to share the
// name, and a local that shadows a global resolves to the local.
// rules: compiler.15 records.10
#import "Basic";
#import "Compiler";

TARGET :: #string JAI
count :: 5;
it :: 9;
Thing :: struct { count: int; }
scaled :: (by: int, count := 1) -> int { return by * count; }
value :: (t: Thing) -> int {
    a := t.count;
    b := scaled(2, count = 3);
    total := 0;
    for 0..2 total += it;
    count := 7;
    return a + b + total + count;
}
main :: () { value(.{}); }
JAI

// `ident` does not resolve to one of the global constants (`count`, `it`).
not_the_global :: (ident: *Code_Node) -> bool {
    assert(ident.kind == .IDENT);
    decl := (cast(*Code_Ident) ident).resolved_declaration;
    return !decl || !(decl.flags & .IS_CONSTANT);
}

#run {
    w := compiler_create_workspace("typechecked names");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);
    compiler_begin_intercept(w);
    add_build_string(TARGET, w);
    seen := false;

    while true {
        message := compiler_wait_for_message();
        if message.kind == .COMPLETE break;
        if message.kind != .TYPECHECKED continue;

        for (cast(*Message_Typechecked) message).procedure_bodies {
            body := it.expression;
            if body.header.name != "value" continue;

            seen = true;
            statements := body.block.statements;
            // a := t.count;
            member := cast(*Code_Binary_Operator) (cast(*Code_Declaration) statements[0]).expression;

            assert(not_the_global(member.right));
            // b := scaled(2, count = 3);
            call := cast(*Code_Procedure_Call) (cast(*Code_Declaration) statements[1]).expression;

            assert(not_the_global(call.arguments_unsorted[1].name));
            // for 0..2 total += it;
            loop := cast(*Code_For) statements[3];
            add := cast(*Code_Binary_Operator) (cast(*Code_Block) loop.block).statements[0];

            assert(not_the_global(add.right));
            // return a + b + total + count;  (the local `count`, 7)
            sum := cast(*Code_Binary_Operator) (cast(*Code_Return) statements[5]).arguments_unsorted[0].expression;
            last := cast(*Code_Ident) sum.right;

            assert(last.resolved_declaration != null);
            local := last.resolved_declaration.expression;

            assert(local && local.kind == .LITERAL && (cast(*Code_Literal) local)._s64 == 7);
        }
    }

    compiler_end_intercept(w);
    assert(seen);
    set_build_options_dc(.{ do_output = false });
}

main :: () {}
