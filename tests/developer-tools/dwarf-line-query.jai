#import "Basic";
D :: #import "Debug_Info";
#load "../../target/developer-tools-dwarf-line-oracle.jai";

errors: s64;

capture_error :: (_text: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR errors += 1;
}

main :: () {
    info: D.Line_Info;
    empty, empty_ok := D.get_line_mapping_for_address(info, 0);

    assert(!empty_ok && empty.line == 0);
    addresses := u64.[100, 104, 104, 110, 120];
    mappings: [5] D.Line_Mapping;

    for *mapping, i: mappings {
        mapping.line = cast(u32) i + 1;
        mapping.file_id = 1;
    }

    mappings[4].is_end = true;
    sequence: D.Line_Info_Sequence;
    for address: addresses array_add(*sequence.addresses, address);

    defer array_free(sequence.addresses);
    for mapping: mappings array_add(*sequence.mappings, mapping);

    defer array_free(sequence.mappings);
    array_add(*info.sequences, sequence);
    defer array_free(info.sequences);

    for address: 100..119 {
        mapping, ok := D.get_line_mapping_for_address(info, cast(u64) address);
        expected: u32 = ifx address < 104 then 1 else ifx address < 110 then 3 else 4;
        assert(ok && mapping.line == expected && !mapping.is_end && mapping.file_id == 1);
    }

    before, before_ok := D.get_line_mapping_for_address(info, 99);
    after, after_ok := D.get_line_mapping_for_address(info, 120);

    assert(!before_ok && !after_ok && before.line == 0 && after.line == 0);
    range, range_ok := D.get_line_mappings_for_address_range(info, 103, 111);

    assert(range_ok && range.count == 3 && range[0].line == 1 && range[1].line == 3 && range[2].line == 4);
    defer array_free(range);
    none, none_ok := D.get_line_mappings_for_address_range(info, 120, 130);

    assert(none_ok && none.count == 0);
    zero, zero_ok := D.get_line_mappings_for_address_range(info, 110, 110);

    assert(zero_ok && zero.count == 0);

    push_context {
        context.logger = capture_error;
        invalid, invalid_ok := D.get_line_mappings_for_address_range(info, 111, 110);

        assert(!invalid_ok && invalid.count == 0);
        info.sequences[0].mappings.count -= 1;
        malformed, malformed_ok := D.get_line_mapping_for_address(info, 100);

        assert(!malformed_ok && malformed.line == 0);
        info.sequences[0].mappings.count += 1;
        info.sequences[0].addresses[2] = 103;
        unsorted, unsorted_ok := D.get_line_mapping_for_address(info, 100);

        assert(!unsorted_ok && unsorted.line == 0);
        info.sequences[0].addresses[2] = 104;
        array_add(*info.sequences, sequence);
        overlap, overlap_ok := D.get_line_mapping_for_address(info, 105);

        assert(!overlap_ok && overlap.line == 0);
        info.sequences.count -= 1;
    }

    assert(errors == 4);
    check_native_line_queries();
    print("ok\n");
}
