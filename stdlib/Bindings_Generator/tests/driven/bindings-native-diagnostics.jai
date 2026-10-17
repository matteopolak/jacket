#import "Basic";
#import "String";
B :: #import "Bindings_Generator";

message: string;
info: Log_Info;

capture :: (text: string, _data: *void, metadata: Log_Info) {
    free(message);
    message = copy_string(text);
    info = metadata;
}

main :: () {
    generator: B.Generator;

    push_context {
        B.init_generator(*generator, .{});
        generator.clang_index = B.clang_createIndex(0, 0);
        filename := to_c_string(tprint("%bindings-diagnostics-SDK.h", #filepath));

        defer free(filename);
        arguments := (*u8).[to_c_string("-x"), to_c_string("c")];

        defer for argument: arguments free(argument);
        status := B.clang_parseTranslationUnit2(generator.clang_index, filename, arguments.data, 2, null, 0, 0, *generator.translation_unit);

        assert(status == .Success);
        file := B.clang_getFile(generator.translation_unit, filename);
        cursor := B.clang_getCursor(generator.translation_unit, B.clang_getLocation(generator.translation_unit, file, 1, 12));
        name := B.as_str(cursor);

        defer free(name);
        assert(name == "OriginalDiagnostic");
        type := B.as_str(B.clang_getCursorType(cursor));

        defer free(type);
        assert(type == "int");
        description := B.as_str(cursor, true);

        defer free(description);
        assert(contains(description, "OriginalDiagnostic") && contains(description, "VarDecl") && contains(description, "int"));
        location := B.Location.{ file = file, line = 1, column = 12 };
        first := B.to_string(location);
        second := B.to_string(location);

        defer free(first);
        defer free(second);
        assert(first == second && contains(first, "bindings-diagnostics-sdk.h:1:12"));
        caller := Source_Code_Location.{ fully_pathed_filename = "caller-owned.jai", line_number = 900, character_number = 13 };

        context.logger = capture;
        B.log(location, "value %", 4, loc = caller, flags = .WARNING, user_flags = 77);
        assert(contains(message, first) && contains(message, "value 4"));
        assert(info.location.line_number == 900 && info.location.fully_pathed_filename == "caller-owned.jai");
        assert(info.common_flags == .WARNING && info.user_flags == 77);
        B.log_error(cursor, "failed %", 5, loc = caller, flags = .WARNING, user_flags = 88);
        assert(contains(message, "failed 5"));
        assert((info.common_flags & (.WARNING | .ERROR)) == (.WARNING | .ERROR));
        assert(info.user_flags == 88 && info.location.character_number == 13);
        B.log(cursor, "cursor", loc = caller);
        B.log_error(location, "location", loc = caller);
        B.deinit_generator(*generator);
        assert(name == "OriginalDiagnostic" && type == "int" && contains(first, "bindings-diagnostics-sdk.h"));
    }

    free(message);
    print("ok\n");
}
