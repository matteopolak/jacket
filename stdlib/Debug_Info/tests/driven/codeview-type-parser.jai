#import "Basic";
#import "File";
D :: #import "Debug_Info";
#load "codeview-test-support.jai";
#load "../../../../target/debug-info-codeview-sdk.jai";

main :: () {
    push_context {
        context.allocator = .{ count_allocation, null };
        context.logger = capture_error;
        check_all();
    }

    assert(live_allocations == 0);
    print("ok\n");
}

check_all :: () {
    check_sdk_types();

    for size: s64.[size_of(D.Codeview_Arguments) + 12, 8] {
        deny_size = size;
        denied = false;
        before := live_allocations;
        data := u8.[4, 0, 0, 0, 14, 0, 1, 0x12, 2, 0, 0, 0, 0x74, 0, 0, 0, 0x20, 0, 0, 0];
        parsed_ok, parsed := D.parse_codeview_types(data);

        assert(denied && !parsed_ok && parsed.count == 0 && live_allocations == before);
    }

    bytes, read_ok := read_entire_file(tprint("%../../../../target/debug-info-codeview-types.bin", #filepath));

    assert(read_ok);
    defer free(bytes);
    ok, types := D.parse_codeview_types(cast([] u8) bytes);

    assert(ok && types.count > 0);
    defer release_cv_types(types);

    for type, index: types {
        found, found_ok := D.get_type(types, cast(D.Type_Id) (index + 0x1000));

        assert(found_ok && found == type);
        description, described := D.describe_type(types, types, type);

        assert(described && description.count > 0);
        free(description);

        if type.kind == .STRUCTURE {
            record := cast(*D.Codeview_Struct) type;
            assert(record.name == "Original_Type_Pair" && (record.name.data < bytes.data || record.name.data >= bytes.data + bytes.count));
        }
    }

    // Every incomplete record fails; prefixes at complete record boundaries
    // are valid independent type streams and are released by the test.
    for prefix: 0..bytes.count - 1 {
        partial_ok, partial := D.parse_codeview_types(.{ prefix, bytes.data });
        if partial_ok {
            release_cv_types(partial);
        } else assert(partial.count == 0 && partial.data == null);
    }

    primitive, primitive_ok := D.get_type(types, cast(D.Type_Id) 0x74);

    assert(primitive_ok && primitive.kind == .BUILTIN && (cast(*D.Codeview_Builtin) primitive).id == cast(D.Type_Id) 0x74);
    free(primitive);
    invalid, invalid_ok := D.get_type(types, cast(D.Type_Id) 0x7fffffff);

    assert(!invalid_ok && invalid == null);
    symbol: D.Codeview_Symbol;
    symbol.symbol_type = .GPROC32_ID;
    assert(D.is_procedure(*symbol) && !D.is_procedure(null));
    symbol.symbol_type = .LOCAL;
    assert(!D.is_procedure(*symbol));
    tagged, tagged_ok := D.get_id_type(.{}, types, cast(D.Type_Id) 0x80001002);

    assert(tagged_ok && tagged == types[2]);
    id_description, id_ok := D.describe_id_type(types, types, cast(D.Type_Id) 0x1002);

    assert(id_ok && id_description == "original_debug_types");
    free(id_description);
    pointer := D.Codeview_Pointer.{ type = cast(D.Type_Id) 0x1000 };
    cycle := *pointer.base;
    recursive: [1] *D.Codeview_Type = .[cycle];
    text, described := D.describe_type(recursive, .{}, cycle);

    assert(!described && text.count == 0);
}
