// rules: records.1 records.2 records.5 records.13 records.14 records.15 records.16 records.17 records.21
// Claims of docs/metaprogramming/compiler-records.md: messages of an intercepted workspace
// and node trees from compiler_get_nodes.
#import "Basic";
#import "Compiler";

TARGET :: #string END
#import "Basic";
ratio: float32 = 0.5;
limit :: 9;
main :: () { print("%\n", ratio); }
END

check_messages :: () -> bool {
    w := compiler_create_workspace("records probe");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);
    compiler_begin_intercept(w);
    add_build_string(TARGET, w);

    imports: [..] *Message_Import;
    basic_files := 0;
    saw_ratio := false;
    while true {
        message := compiler_wait_for_message();
        if message.kind == .COMPLETE {
            assert((cast(*Message_Complete) message).error_code == .NONE, "target compiles");
            break;
        }
        if message.kind == .IMPORT array_add(*imports, cast(*Message_Import) message);
        if message.kind == .FILE {
            file := cast(*Message_File) message;
            // records.1: a file's module was announced by an IMPORT before it.
            owner := file.enclosing_import;
            assert(owner != null, "records.1: file has a module");
            announced := false;
            for imports if it == owner announced = true;
            assert(announced, "records.1: IMPORT precedes the module's files");
            if owner.module_name == "Basic" basic_files += 1;
        }
        if message.kind == .TYPECHECKED {
            for (cast(*Message_Typechecked) message).declarations {
                decl := it.expression;
                // records.5: global declarations carry their type.
                if decl.name == "ratio" {
                    saw_ratio = true;
                    assert(decl.type == type_info(float32), "records.5: global declaration type");
                }
            }
        }
    }
    compiler_end_intercept(w);
    // records.2: one FILE message per loaded file (Basic spans several files).
    assert(basic_files >= 2, "records.2: a FILE per file of Basic");
    assert(saw_ratio, "records.5: ratio was reported");
    return true;
}

#run check_messages();

check_nodes :: () -> bool {
    // records.13: untyped literals report their default types.
    whole := compiler_get_nodes(#code 7);
    assert(whole.kind == .LITERAL && whole.type == type_info(s64), "records.13: integer literal is s64");
    fraction := compiler_get_nodes(#code 2.5);
    assert(fraction.kind == .LITERAL && fraction.type == type_info(float32), "records.13: float literal is float32");

    // records.14: a negated literal is one negative literal.
    negative := compiler_get_nodes(#code -1);
    assert(negative.kind == .LITERAL && (cast(*Code_Literal) negative)._s64 == -1, "records.14: -1 is one literal");
    negative_float := compiler_get_nodes(#code -1.5);
    assert(negative_float.kind == .LITERAL && (cast(*Code_Literal) negative_float)._float64 == -1.5, "records.14: -1.5 is one literal");

    // records.15: a backticked identifier carries HAS_SCOPE_MODIFIER.
    block := cast(*Code_Block) compiler_get_nodes(#code {
        `outer_name = 1;
    });
    assignment := cast(*Code_Binary_Operator) block.statements[0];
    ident := cast(*Code_Ident) assignment.left;
    assert(ident.kind == .IDENT && ident.flags & .HAS_SCOPE_MODIFIER, "records.15: backtick flag");

    // records.16, records.17: #assert / #run statements and #exists.
    directives := cast(*Code_Block) compiler_get_nodes(#code {
        #assert true;
        x := #exists(limit);
    });
    assert(directives.statements[0].kind == .DIRECTIVE_RUN, "records.16: #assert is a directive run");
    exists := cast(*Code_Declaration) directives.statements[1];
    assert(exists.expression.kind == .DIRECTIVE_EXISTS, "records.17: #exists node");
    return true;
}

#run check_nodes();

// records.21: a body nothing reaches is never checked, so its error never surfaces.
never_called :: () {
    wrong: int = "not an int";
}

main :: () {

    print("ok\n");
}
