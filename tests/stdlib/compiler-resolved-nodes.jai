// Typed compiler_get_nodes from compile-time code (resolved procedures, declarations and
// types), statements a metaprogram edits in place, and compound declarations.
// rules: compiler.15 records.10 records.19 records.20 ws.14
#import "Basic";
#import "Compiler";
#import "Program_Print";
#import "String";

pair :: (a: int) -> int, bool {
    return a, true;
}

// What or_return-style macros read: the header the call resolves to, with its returns.
returns_of :: ($call: Code) -> int #expand {
    return #run -> int {
        root := compiler_get_nodes(call);
        assert(root.kind == .PROCEDURE_CALL);
        resolved := (cast(*Code_Procedure_Call) root).resolved_procedure_expression;
        assert(resolved && resolved.kind == .PROCEDURE_HEADER);
        header := cast(*Code_Procedure_Header) resolved;
        assert(header.name == "pair");
        assert(header.arguments.count == 1 && header.arguments[0].name == "a");
        assert(get_type(header.returns[0].type_inst.result) == int);
        assert(get_type(header.returns[1].type_inst.result) == bool);
        return header.returns.count;
    };
}

#run assert(returns_of(pair(3)) == 2);

// An identifier knows its declaration and type.
#run {
    root := compiler_get_nodes(#code pair);
    assert(root.kind == .IDENT);
    ident := cast(*Code_Ident) root;
    assert(ident.resolved_declaration != null);
    assert(ident.resolved_declaration.expression.kind == .PROCEDURE_HEADER);
}

// `a, b := f()` is a compound declaration and prints as one.
#run {
    root := compiler_get_nodes(#code {
        a, b := pair(1);
        c, d: int = 1, 2;
    });
    block := cast(*Code_Block) root;
    assert(block.statements.count == 2);
    assert(block.statements[0].kind == .COMPOUND_DECLARATION);
    builder: String_Builder;
    print_expression(*builder, block);
    text := builder_to_string(*builder);
    assert(contains(text, "a, b := pair(1);"), "%", text);
    assert(contains(text, "c, d : int = 1, 2;"), "%", text);
}

// Nodes made with New know their kind.
#run {
    ident := New(Code_Ident);
    assert(ident.kind == .IDENT);
}

// A baked variadic takes a spread constant view; a baked Code argument that names a
// Code constant passes that code along (not code of the name).
count_baked :: ($names: ..string) -> int {
    return names.count;
}

NAMES :: string.["a", "b", "c"];
#run assert(count_baked(..NAMES) == 3);

inner :: ($c: Code) -> bool #expand {
    return #run compiler_get_nodes(c).kind == .PROCEDURE_CALL;
}

outer :: ($c: Code) -> bool #expand {
    return inner(c);
}

#run assert(outer(pair(1)));

TARGET :: #string END
#import "Basic";
value :: () -> int {
    a, b := pair(1);
    return a;
}
pair :: (a: int) -> int, bool { return a, true; }
main :: () { value(); }
END

#run {
    w := compiler_create_workspace("resolved nodes");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);
    compiler_begin_intercept(w);
    add_build_string(TARGET, w);
    modified := false;
    while true {
        message := compiler_wait_for_message();
        if message.kind == .COMPLETE break;
        if message.kind == .TYPECHECKED {
            typechecked := cast(*Message_Typechecked) message;
            for typechecked.procedure_bodies {
                body := it.expression;
                if body.header.name != "value" || body.body_flags & .ALREADY_MODIFIED continue;
                // Edit the call inside the compound declaration in place: pair(1) -> pair(42).
                compound := cast(*Code_Compound_Declaration) body.block.statements[0];
                call := cast(*Code_Procedure_Call) compound.declaration_properties.expression;
                literal := cast(*Code_Literal) call.arguments_unsorted[0].expression;
                literal._s64 = 42;
                compiler_modify_procedure(w, body);
                modified = true;
                add_build_string("#run assert(value() == 42);", w);
            }
        }
    }
    compiler_end_intercept(w);
    assert(modified);
    set_build_options_dc(.{ do_output = false });
}

main :: () {}
