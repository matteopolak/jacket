print_procedure_bodies := true;

print_string_literal :: (builder: *Basic.String_Builder, s: string) {
    Impl.quoted(builder, s);
}

print_expression :: (builder: *Basic.String_Builder, expression: *C.Code_Node, skip_parens := false) -> printed: bool {
    return Impl.expression(builder, expression, skip_parens, print_procedure_bodies, 0);
}

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

print_declaration :: (builder: *Basic.String_Builder, decl: *C.Code_Declaration, print_using := true, compound: *C.Code_Compound_Declaration = null, print_type := true) -> bool {
    return Impl.declaration(builder, decl, print_using, compound, print_type, print_procedure_bodies, 0);
}

get_plugin :: () -> *C.Metaprogram_Plugin {
    plugin.message = program_print_receive_message;
    plugin.finish = program_print_finish;
    return *plugin;
}

program_print_receive_message :: (p: *C.Metaprogram_Plugin, message: *C.Message) {
    Impl.collect(message);
}

program_print_finish :: (p: *C.Metaprogram_Plugin) {
    Impl.finish(print_procedure_bodies);
}

#scope_file
C :: #import "Compiler";
Basic :: #import "Basic";
Impl :: #import "internal/program-print";

plugin: C.Metaprogram_Plugin;
