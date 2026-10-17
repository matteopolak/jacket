#import "Basic";
#import "File";
D :: #import "Debug_Info";
BA :: #import "Bucket_Array";
#load "../../../../target/debug-info-sdk.jai";
#load "../../../../target/debug-info-synthetic.jai";

errors: s64;
allocations: s64;

track :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, data: *void) -> *void {
    result := Context_Base.default_allocator.proc(mode, size, old_size, old_memory, data);
    if (mode == .ALLOCATE || mode == .RESIZE) && result && !old_memory allocations += 1;
    if mode == .FREE && old_memory allocations -= 1;
    return result;
}

capture :: (_text: string, _data: *void, info: Log_Info) {
    assert((info.common_flags & .ERROR) != 0);
    errors += 1;
}

load_bytes :: (path: string) -> [] u8 {
    bytes, ok := read_entire_file(path);
    assert(ok);
    return cast([] u8) bytes;
}

release_info :: (info: *D.Debug_Info) {
    for *unit: info.compilation_units {
        for *sub: unit.subprograms {
            for *block: sub.all_blocks {
                array_free(block.declarations);
                array_free(block.children);
            }

            BA.bucket_array_reset(*sub.all_blocks);
            array_free(sub.all_blocks.all_buckets);
            array_free(sub.all_blocks.unfull_buckets);
        }

        BA.bucket_array_reset(*unit.subprograms);
        array_free(unit.subprograms.all_buckets);
        array_free(unit.subprograms.unfull_buckets);
        array_free(unit.types);
        array_free(unit.globals);

        for sequence: unit.line_info.sequences {
            array_free(sequence.addresses);
            array_free(sequence.mappings);
        }

        array_free(unit.line_info.sequences);
        array_free(unit.line_info.dirs);
        array_free(unit.line_info.files);
    }

    array_free(info.compilation_units);
}

check_unit :: (unit: D.Compilation_Unit) {
    assert(unit.name == "stdlib/Debug_Info/tests/driven/dwarf-types-native.c");
    assert(unit.producer.count > 0 && unit.types.count >= 7);
    assert(unit.subprograms.count == 1 && unit.globals.count == 1);
    assert(unit.globals[0].name == "original_type_pair" && D.get_type(unit, unit.globals[0].type_id) != null);
    assert(unit.line_info.sequences.count > 0);

    for sequence: unit.line_info.sequences {
        assert(sequence.addresses.count == sequence.mappings.count);
        assert(sequence.mappings[sequence.mappings.count - 1].is_end);

        for row: sequence.mappings {
            filename := D.get_filename_for_file_id(unit.line_info, row.file_id);
            assert(filename.count > 0);
            if filename.data != unit.line_info.files[row.file_id - 1].path.data free(filename);
        }
    }

    for sub: unit.subprograms {
        assert(sub.name == "original_debug_types" && sub.external && sub.high_pc > sub.low_pc);
        assert(sub.root_block != null && sub.root_block.declarations.count == 2);
        assert(sub.root_block.declarations[0].name == "value" && sub.root_block.declarations[0].kind == .PARAMETER);
        assert(D.get_type(unit, sub.type_id).name == "double");
    }
}

main :: () {
    push_context {
        context.allocator = .{ track, null };
        context.logger = capture;
        check_all();
    }

    assert(allocations == 0 && errors > 0);
    print("ok\n");
}

check_all :: () {
    check_synthetic_dwarf();
    bad_directory: D.Line_Info;

    array_add(*bad_directory.dirs, string.{ 1, null });
    defer array_free(bad_directory.dirs);
    assert(D.get_filename(bad_directory, .{ dir_id = 1, path = "a" }) == "");
    // Malformed interpreted attributes must not turn into invented empty text.
    malformed := u8.[9, 0, 0, 0, 4, 0, 0, 0, 0, 0, 8, 1, 42];
    wrong_name_form := u8.[1, 0x11, 0, 3, 0x0b, 0, 0, 0];
    bad, bad_ok := D.parse_dwarf_debug_info(malformed, wrong_name_form, .{}, .{});

    assert(!bad_ok && bad.compilation_units.count == 0);
    negative_high := u8.[17, 0, 0, 0, 4, 0, 0, 0, 0, 0, 8, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0x7f];
    high_abbrev := u8.[1, 0x11, 0, 0x11, 1, 0x12, 0x0d, 0, 0, 0];

    bad, bad_ok = D.parse_dwarf_debug_info(negative_high, high_abbrev, .{}, .{});
    assert(!bad_ok && bad.compilation_units.count == 0);
    check_sdk_units();
    info, info_ok := read_entire_file(tprint("%../../../../target/debug-info-info.bin", #filepath));
    abbrev, abbrev_ok := read_entire_file(tprint("%../../../../target/debug-info-abbrev.bin", #filepath));
    lines, lines_ok := read_entire_file(tprint("%../../../../target/debug-info-line.bin", #filepath));
    strings, strings_ok := read_entire_file(tprint("%../../../../target/debug-info-str.bin", #filepath));

    assert(info_ok && abbrev_ok && lines_ok && strings_ok);
    defer free(info);
    defer free(abbrev);
    defer free(lines);
    defer free(strings);

    parsed, ok := D.parse_dwarf_debug_info(cast([] u8) info, cast([] u8) abbrev, cast([] u8) lines, cast([] u8) strings);

    assert(ok && parsed.compilation_units.count == 1);
    defer release_info(*parsed);
    unit := parsed.compilation_units[0];

    assert(unit.name == "stdlib/Debug_Info/tests/driven/dwarf-types-native.c");
    assert(unit.producer.count > 0 && unit.types.count >= 7);
    assert(unit.subprograms.count == 1 && unit.globals.count == 1);
    assert(unit.globals[0].name == "original_type_pair" && D.get_type(unit, unit.globals[0].type_id) != null);
    assert(unit.line_info.sequences.count > 0);

    for sequence: unit.line_info.sequences {
        assert(sequence.addresses.count == sequence.mappings.count);
        assert(sequence.mappings[sequence.mappings.count - 1].is_end);

        for row: sequence.mappings {
            filename := D.get_filename_for_file_id(unit.line_info, row.file_id);
            assert(filename.count > 0);
            if filename.data != unit.line_info.files[row.file_id - 1].path.data free(filename);
        }
    }

    for sub: unit.subprograms {
        assert(sub.name == "original_debug_types" && sub.external && sub.high_pc > sub.low_pc);
        assert(sub.root_block != null && sub.root_block.declarations.count == 2);
        assert(sub.root_block.declarations[0].name == "value" && sub.root_block.declarations[0].kind == .PARAMETER);
        assert(D.get_type(unit, sub.type_id).name == "double");
    }

    for prefix: 1..info.count - 1 {
        rejected, rejected_ok := D.parse_dwarf_debug_info(.{ prefix, info.data }, cast([] u8) abbrev, cast([] u8) lines, cast([] u8) strings);
        assert(!rejected_ok && rejected.compilation_units.count == 0);
    }

    for prefix: 0..abbrev.count - 1 {
        before := allocations;
        rejected, rejected_ok := D.parse_dwarf_debug_info(cast([] u8) info, .{ prefix, abbrev.data }, cast([] u8) lines, cast([] u8) strings);
        assert(!rejected_ok && rejected.compilation_units.count == 0 && allocations == before);
    }

    for prefix: 0..lines.count - 1 {
        before := allocations;
        rejected, rejected_ok := D.parse_dwarf_debug_info(cast([] u8) info, cast([] u8) abbrev, .{ prefix, lines.data }, cast([] u8) strings);
        assert(!rejected_ok && rejected.compilation_units.count == 0 && allocations == before);
    }

    doubled: [..] u8;

    array_resize(*doubled, info.count * 2);
    defer array_free(doubled);
    memcpy(doubled.data, info.data, info.count);
    memcpy(doubled.data + info.count, info.data, info.count);
    two, two_ok := D.parse_dwarf_debug_info(doubled, cast([] u8) abbrev, cast([] u8) lines, cast([] u8) strings);

    assert(two_ok && two.compilation_units.count == 2);
    defer release_info(*two);

    for sub: two.compilation_units[1].subprograms {
        assert(sub.type_id >= info.count && D.get_type(two.compilation_units[1], sub.type_id).name == "double");
    }
}
