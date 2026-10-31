// Turns compiler code nodes back into Jai source text.
//
// - [print_expression] writes one expression into a `String_Builder`.
// - [print_declaration] returns a declaration as a string.
// - [get_plugin] is a metaprogram plugin that writes a `pp_<file>.txt` for every file the build loads.
//
// ```jai
// Compiler :: #import "Compiler";
// left := Compiler.make_integer_literal(-13);
// right := Compiler.make_integer_literal(29);
// sum: Compiler.Code_Binary_Operator;
// sum.kind = .BINARY_OPERATOR;
// sum.operator_type = cast(Compiler.Operator_Type) #char "+";
// sum.left = left;
// sum.right = right;
//
// builder: String_Builder;
// init_string_builder(*builder);
// assert(print_expression(*builder, *sum));
// assert(builder_to_string(*builder) == "(-13 + 29)");
// ```

print_procedure_bodies := true;

// Writes `s` to `builder` as a quoted Jai string literal, escaping quotes, backslashes and control characters.
//
// ```jai
// builder: String_Builder;
// init_string_builder(*builder);
// print_string_literal(*builder, "a\"b");
// assert(builder_to_string(*builder) == "\"a\\\"b\"");
// ```
print_string_literal :: (builder: *Basic.String_Builder, s: string) {
    Impl.quoted(builder, s);
}

// Writes `expression` to `builder` as source text. Returns false for a node kind it cannot print, such as inline assembly.
//
// Pass `skip_parens` to leave off the parentheses around a top-level binary operation.
//
// ```jai
// Compiler :: #import "Compiler";
// builder: String_Builder;
// init_string_builder(*builder);
// assert(print_expression(*builder, cast(*Compiler.Code_Node) Compiler.make_integer_literal(7)));
// assert(builder_to_string(*builder) == "7");
// ```
print_expression :: (builder: *Basic.String_Builder, expression: *C.Code_Node, skip_parens := false) -> printed: bool {
    return Impl.expression(builder, expression, skip_parens, print_procedure_bodies, 0);
}

// Returns `decl` as source text, or an empty string if it cannot be printed. The caller frees the string.
//
// ```jai
// Compiler :: #import "Compiler";
// decl: Compiler.Code_Declaration;
// decl.kind = .DECLARATION;
// decl.name = "answer";
// decl.flags = .IS_CONSTANT;
// decl.expression = Compiler.make_integer_literal(42);
// assert(print_declaration(*decl) == "answer:: 42");
// ```
print_declaration :: (decl: *C.Code_Declaration) -> string {
    builder: Basic.String_Builder;

    Basic.init_string_builder(*builder);

    if !Impl.declaration(*builder, decl, true, null, true, print_procedure_bodies, 0) {
        Basic.free_buffers(*builder);
        return "";
    }

    result := Basic.builder_to_string(*builder);

    Basic.free_buffers(*builder);

    return result;
}

// Writes `decl` to `builder`. Returns false if it cannot be printed.
//
// `print_using` includes a `using` prefix, `compound` is the multi-name declaration this one belongs to, and `print_type` includes the type.
print_declaration :: (builder: *Basic.String_Builder, decl: *C.Code_Declaration, print_using := true, compound: *C.Code_Compound_Declaration = null, print_type := true) -> bool {
    return Impl.declaration(builder, decl, print_using, compound, print_type, print_procedure_bodies, 0);
}

// Returns a metaprogram plugin that writes the loaded program back out as source, one `pp_<file>.txt` per file. Set its `workspace` and forward each message to its `message` procedure.
//
// ```jai,no_run
// Compiler :: #import "Compiler";
// plugin := get_plugin();
// plugin.workspace = Compiler.compiler_create_workspace("print");
// ```
get_plugin :: () -> *C.Metaprogram_Plugin {
    plugin.message = program_print_receive_message;
    plugin.finish = program_print_finish;
    return *plugin;
}

// The plugin's message hook: collects each message for the final write.
program_print_receive_message :: (p: *C.Metaprogram_Plugin, message: *C.Message) {
    Impl.collect(message);
}

// The plugin's finish hook: writes the collected files.
program_print_finish :: (p: *C.Metaprogram_Plugin) {
    Impl.finish(print_procedure_bodies);
}

#scope_file
C :: #import "Compiler";
Basic :: #import "Basic";
Impl :: #import "internal/program-print";

plugin: C.Metaprogram_Plugin;
