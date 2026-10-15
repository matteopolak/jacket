#import "Basic";
#import "File";
D :: #import "Debug_Info";
#load "../../target/developer-tools-eh-frame-oracle.jai";

main :: () {
    for fixture: native_eh_frames {
        bytes, read_ok := read_entire_file(tprint("%../../target/%", #filepath, fixture.filename));

        assert(read_ok);
        defer free(bytes);
        cie_ok, cie, cie_end := D.parse_common_information_entry(.{ bytes.count, bytes.data }, cast(s64) fixture.cie_offset);

        assert(cie_ok && cie.version == 1 && cie.id == 0 && cie_end <= fixture.fde_offset);
        defer array_free(cie.initial_instructions);
        extended: [..] u8;

        array_resize(*extended, 12 + cast(s64) cie.length);
        defer array_free(extended);
        for i: 0..3 extended[i] = 0xff;
        for i: 0..7 extended[4 + i] = cast(u8) ((cie.length >> (i * 8)) & 0xff);
        for i: 0..cast(s64) cie.length - 1 extended[12 + i] = bytes[cast(s64) cie.offset + 4 + i];

        extended_ok, extended_cie, extended_end := D.parse_common_information_entry(.{ extended.count, extended.data }, 0);

        assert(extended_ok && extended_end == extended.count && extended_cie.length == cie.length && extended_cie.id == 0 && extended_cie.version == 1);
        assert(extended_cie.initial_instructions.count == cie.initial_instructions.count && extended_cie.augmentation == cie.augmentation);
        defer array_free(extended_cie.initial_instructions);
        assert(cie.augmentation == fixture.augmentation);
        if cie.augmentation.count > 0 assert(cie.augmentation_data.count == 1 && cie.augmentation_data[0] == fixture.encoding);
        else assert(cie.augmentation_data.count == 0);

        ok, frame, next := D.parse_frame_description_entry(cie, fixture.section_address, .{ bytes.count, bytes.data }, fixture.fde_offset);

        assert(ok && next == fixture.fde_end && frame.length == fixture.fde_length);
        assert(frame.cie_offset == fixture.cie_offset && frame.cie_offset != cast(u64) fixture.fde_offset + 4);
        assert(frame.pc_begin == fixture.pc_begin && frame.pc_range == fixture.pc_range);
        assert(frame.augmentation_data.count == 0 && frame.instructions.count == fixture.instruction_types.count);
        for instruction, index: frame.instructions assert(cast(u8) instruction.type == fixture.instruction_types[index]);

        defer array_free(frame.instructions);

        for count: 0..next - 1 {
            rejected, partial, at := D.parse_frame_description_entry(cie, fixture.section_address, .{ count, bytes.data }, fixture.fde_offset);
            assert(!rejected && partial.instructions.count == 0 && partial.augmentation_data.count == 0 && at == fixture.fde_offset);
        }

        wrong := cie;
        wrong.offset += 1;
        rejected, partial, at := D.parse_frame_description_entry(wrong, fixture.section_address, .{ bytes.count, bytes.data }, fixture.fde_offset);

        assert(!rejected && partial.instructions.count == 0 && at == fixture.fde_offset);

        if fixture.pc_begin < fixture.section_address {
            underflow, underflow_frame, underflow_at := D.parse_frame_description_entry(cie, 0, .{ bytes.count, bytes.data }, fixture.fde_offset);
            assert(!underflow && underflow_frame.instructions.count == 0 && underflow_at == fixture.fde_offset);
        }

        overflow, overflow_frame, overflow_at := D.parse_frame_description_entry(cie, 0xffffffffffffffff, .{ bytes.count, bytes.data }, fixture.fde_offset);

        assert(!overflow && overflow_frame.instructions.count == 0 && overflow_at == fixture.fde_offset);

        if cie.augmentation.count > 0 {
            invalid_encoding := u8.[3];

            wrong = cie;
            wrong.augmentation_data = invalid_encoding;
            encoded, encoded_frame, encoded_at := D.parse_frame_description_entry(wrong, fixture.section_address, .{ bytes.count, bytes.data }, fixture.fde_offset);

            assert(!encoded && encoded_frame.instructions.count == 0 && encoded_at == fixture.fde_offset);
        }
    }

    print("ok\n");
}
