#import "Basic";
#import "File";
#import "String";
B :: #import "Bindings_Generator";

errors: int;

capture :: (_text: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR errors += 1;
}

main :: () {
    args := get_command_line_arguments();

    defer array_free(args);
    assert(args.count == 2);
    library: B.Library_Info;
    library.identifier = "native";
    declaration: B.Declaration;
    declaration.library = *library;
    declaration.kind = .FUNCTION;
    builder: String_Builder;

    defer free_buffers(*builder);
    append(*builder, "native :: #foreign_library \"linkage_emission\";\nread :: () -> s32 #c_call ");
    B.append_elsewhere(*builder, *declaration, "read", "original_read");
    append(*builder, ";\noriginal_same :: () -> s32 #c_call ");
    B.append_elsewhere(*builder, *declaration, "original_same", "original_same");
    append(*builder, ";\nvalue: s32 ");
    declaration.kind = .DECLARATION;
    B.append_elsewhere(*builder, *declaration, "value", "original_value");
    append(*builder, ";\n");
    assert(!builder.failed);
    text := builder_to_string(*builder);

    defer free(text);
    assert(write_entire_file(tprint("%/developer-tools-linkage-generated.jai", args[1]), text));
    declaration.library = null;

    push_context {
        context.logger = capture;
        B.append_elsewhere(*builder, *declaration, "value", "original_value");
    }

    unknown := builder_to_string(*builder);

    defer free(unknown);
    assert(errors == 1 && contains(unknown, "__UnknownLib"));
    print("ok\n");
}
