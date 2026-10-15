// Reviewed Shared emission/helper policies; canonical supplied model identities.
#import "Basic";
B :: #import "Bindings_Generator";

S :: type_of(context._bindings_generator_session.*);

main :: () {
    generator: B.Generator;
    options: B.Generate_Bindings_Options;

    array_add(*options.strip_prefixes, "API_");
    defer array_free(options.strip_prefixes);
    array_add(*options.typedef_prefixes_to_unwrap, "Mine_");
    defer array_free(options.typedef_prefixes_to_unwrap);
    session := S.open(*generator, B.TARGET_OS);

    defer S.close(*session);

    push_context {
        S.activate(*generator, *options, *session);
        assert(B.stripped_name("API___Value") == "Value");
        repaired := B.stripped_name("API_7Value");

        defer free(repaired);
        assert(repaired == "_7Value");
        assert(B.stripped_name("Unchanged") == "Unchanged");
        assert(B.should_unwrap_typedef("uint32_t"));
        assert(B.should_unwrap_typedef("__something_t"));
        assert(B.should_unwrap_typedef("Mine_Value"));
        assert(!B.should_unwrap_typedef("NotSpecial"));
        outer, flattened, inner: B.Namespace;
        outer.kind = .NAMESPACE;
        outer.output_name = "Outer";
        flattened.kind = .NAMESPACE;
        flattened.output_name = "Flattened";
        flattened.flags = .FLATTEN;
        flattened.parent = cast(*B.Declaration) *outer;
        inner.kind = .NAMESPACE;
        inner.output_name = "Inner";
        inner.parent = cast(*B.Declaration) *flattened;
        builder: String_Builder;

        defer free_buffers(*builder);
        assert(B.append_parent_path(*builder, cast(*B.Declaration) *inner, null, .{}));
        text := builder_to_string(*builder);

        defer free(text);
        assert(text == "Outer.Inner.");
        assert(B.append_parent_path(*builder, cast(*B.Declaration) *inner, null, .{}, "::", false));
        native := builder_to_string(*builder);

        defer free(native);
        assert(native == "Outer::Flattened::Inner::");
        assert(!B.append_parent_path(*builder, cast(*B.Declaration) *outer, cast(*B.Declaration) *inner, .{}));
        assert(builder_string_length(*builder) == 0);
        inner.output_name = "";
        assert(B.append_parent_path(*builder, cast(*B.Declaration) *inner, cast(*B.Declaration) *flattened, .{}));
        assert(builder_string_length(*builder) == 0);
        declaration: B.Declaration;

        assert(!B.should_be_omitted(*declaration, cast(*B.Declaration) *inner));
        outer.decl_flags = .OMIT_FROM_OUTPUT;
        assert(B.should_be_omitted(*declaration, cast(*B.Declaration) *inner));
        assert(!B.should_be_omitted(*declaration, null));
        declaration.decl_flags = .OMIT_FROM_OUTPUT;
        assert(B.should_be_omitted(*declaration, null));

        literal: B.Literal;
        literal.literal_kind = .INTEGER;
        literal.int_value = 5;
        macro: B.Declaration;
        macro.kind = .MACRO_DEFINITION;
        macro.output_name = "5";
        macro.expression = cast(*B.Expression) *literal;
        assert(B.omit_unnecessary_typedefs_and_macros(*macro, null, null) == .STOP);
        assert((macro.decl_flags & .OMIT_FROM_OUTPUT) != 0);
        assert(B.omit_unnecessary_typedefs_and_macros(cast(*B.Declaration) *outer, null, null) == .RECURSE);
        assert(generator.counts.stripped == 0);
    }

    print("ok\n");
}
