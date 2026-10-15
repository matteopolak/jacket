#import "Basic";
#import "File";
B :: #import "Bindings_Generator";

S :: type_of(context._bindings_generator_session.*);

Capture :: struct {
    cursor: B.CXCursor;
    scalar: B.CXCursor;
    count: int;
}

visit :: (cur: B.CXCursor, _parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    capture := cast(*Capture) data;

    if cur.kind == .StructDecl {
        if capture.count == 0 capture.cursor = cur;
        else capture.scalar = cur;
        capture.count += 1;
    }

    return .Continue;
}

main :: () {
    args := get_command_line_arguments();

    defer array_free(args);
    assert(args.count == 2);
    generator: B.Generator;
    options: B.Generate_Bindings_Options;

    push_context {
        B.init_generator(*generator, options);
        defer B.deinit_generator(*generator);
        S.activate(*generator, *options, context._bindings_generator_session);
        generator.clang_index = B.clang_createIndex(0, 0);
        filename := to_c_string(tprint("%bindings-bitfield-printers.h", #filepath));

        defer free(filename);
        arguments := (*u8).[to_c_string("-x"), to_c_string("c")];

        defer for argument: arguments free(argument);
        status := B.clang_parseTranslationUnit2(generator.clang_index, filename, arguments.data, 2, null, 0, 0, *generator.translation_unit);

        assert(status == .Success);
        capture: Capture;
        root := B.clang_getTranslationUnitCursor(generator.translation_unit);

        B.clang_visitChildren(root, visit, *capture);
        record := B.create_struct(capture.cursor);

        assert(record != null && record.declarations.count == 5);
        status_bits: B.Bitfield_Status;
        status_bits.parent = record;
        first := B.maybe_add_to_bitfield(*status_bits, record.declarations[0]);

        assert(first != null && first.kind == .BITFIELD);
        group := cast(*B.Bitfield) first;

        for index: 1..record.declarations.count - 1 assert(B.maybe_add_to_bitfield(*status_bits, record.declarations[index]) == null);

        assert(status_bits.bitfield == group && group.members.count == 4);
        B.finish_bitfield(group, 23);
        assert(group.members[0] == record.declarations[0] && group.members[3] == record.declarations[4]);
        assert(group.output_name == "__bitfield" && group.bit_offset_in_parent == 0);
        assert(group.type.array_element_count == 5 && group.type.array_element_type == generator.type_def_u8);
        assert(group.alignment_type.size == 4);
        assert(B.maybe_unwrap_bitfield_declaration(first) == first);
        record.output_name = "Bits";
        output: String_Builder;

        defer free_buffers(*output);
        print_to_builder(*output, "Bits :: struct #align % { __bitfield: ", record.type.alignment);
        B.print_type_to_builder(*output, group.type, cast(*B.Declaration) record);
        append(*output, "; }\n");
        B.append_bitfield_helpers(*output, group, record);
        assert(capture.count == 2);
        scalar_record := B.create_struct(capture.scalar);

        assert(scalar_record != null && scalar_record.declarations.count == 2);
        scalar_record.output_name = "Scalar";
        scalar_status: B.Bitfield_Status;
        scalar_status.parent = scalar_record;
        scalar_group := cast(*B.Bitfield) B.maybe_add_to_bitfield(*scalar_status, scalar_record.declarations[0]);

        assert(B.maybe_add_to_bitfield(*scalar_status, scalar_record.declarations[1]) == null);
        B.finish_bitfield(scalar_group, 0);
        assert(scalar_group.type.size == 1 && scalar_group.alignment_type.size == 4);
        append(*output, "Scalar :: struct #align 4 { __bitfield: ");
        B.print_type_to_builder(*output, scalar_group.type, cast(*B.Declaration) scalar_record);
        append(*output, "; }\n");
        B.append_bitfield_helpers(*output, scalar_group, scalar_record);
        append(*output, "native :: #foreign_library \"bitfield_printers\";\ninitialize :: (bits: *Bits, first: u32, second: u32, on: u32, final: u32) #c_call #foreign native \"original_bits_initialize\";\nread :: (bits: *Bits) -> u32 #c_call #foreign native \"original_bits_read\";\n");
        append(*output, "initialize_scalar :: (bits: *Scalar, a: u32, b: u32) #c_call #foreign native \"original_scalar_initialize\";\nread_scalar :: (bits: *Scalar) -> u32 #c_call #foreign native \"original_scalar_read\";\n");
        assert(!output.failed);
        text := builder_to_string(*output);

        defer free(text);
        assert(write_entire_file(tprint("%/developer-tools-bitfields-generated.jai", args[1]), text));
        ordinary: B.Declaration;

        assert(B.maybe_add_to_bitfield(*status_bits, *ordinary) == *ordinary && status_bits.bitfield == null);
        original_type := record.declarations[0].type;
        single_status: B.Bitfield_Status;
        single_status.parent = record;
        single := cast(*B.Bitfield) B.maybe_add_to_bitfield(*single_status, record.declarations[0]);

        B.finish_bitfield(single, 999);
        unwrapped := B.maybe_unwrap_bitfield_declaration(cast(*B.Declaration) single);

        assert(unwrapped == record.declarations[0] && unwrapped.type == single.type && single.type.size == 1);
        unwrapped.type = original_type;
        options.os = .WINDOWS;
        windows_status: B.Bitfield_Status;
        windows_status.parent = record;
        windows_first := B.maybe_add_to_bitfield(*windows_status, record.declarations[0]);
        signed_member := record.declarations[1].*;
        signed_member.type = generator.type_def_s32;
        windows_second := B.maybe_add_to_bitfield(*windows_status, *signed_member);

        assert(windows_first != null && windows_second != null && windows_first != windows_second);
        assert(windows_first.type.size == 4);
    }

    print("ok\n");
}
