#import "Basic";
#import "File";
D :: #import "Debug_Info";
#load "../../../../target/developer-tools-dwarf-frame-oracle.jai";

main :: () {
    for fixture: native_frames {
        bytes, read_ok := read_entire_file(tprint("%../../../../target/%", #filepath, fixture.filename));

        assert(read_ok);
        defer free(bytes);
        cie_ok, cie, cie_end := D.parse_common_information_entry(.{ bytes.count, bytes.data }, 0);

        assert(cie_ok && cie.id == 0xffffffff && cie.version == 4 && cie_end == fixture.fde_offset);
        defer array_free(cie.initial_instructions);
        ok, fde, next := D.parse_frame_description_entry(cie, 0, .{ bytes.count, bytes.data }, fixture.fde_offset);

        assert(ok && next == fixture.fde_end && fde.length == fixture.fde_length);
        assert(fde.cie_offset == cie.offset && fde.pc_begin == fixture.pc_begin && fde.pc_range == fixture.pc_range);
        assert(fde.augmentation_data.count == 0);
        assert(fde.instructions.count == fixture.instruction_types.count);
        for instruction, index: fde.instructions assert(cast(u8) instruction.type == fixture.instruction_types[index]);

        defer array_free(fde.instructions);

        for count: 0..next - 1 {
            rejected, partial, at := D.parse_frame_description_entry(cie, 0, .{ count, bytes.data }, fixture.fde_offset);
            assert(!rejected && partial.instructions.count == 0 && at == fixture.fde_offset);
        }

        invalid_cie := cie;
        invalid_cie.id = 0;
        unsupported, empty, at := D.parse_frame_description_entry(invalid_cie, 0, .{ bytes.count, bytes.data }, fixture.fde_offset);

        assert(!unsupported && empty.instructions.count == 0 && at == fixture.fde_offset);
        invalid_cie = cie;
        invalid_cie.offset = cast(u64) bytes.count + 1;
        bad_cie, bad, bad_at := D.parse_frame_description_entry(invalid_cie, 0, .{ bytes.count, bytes.data }, fixture.fde_offset);

        assert(!bad_cie && bad.instructions.count == 0 && bad_at == fixture.fde_offset);
        original := bytes.data[fixture.fde_offset + 4];

        bytes.data[fixture.fde_offset + 4] = 1;
        wrong_reference, reference, reference_at := D.parse_frame_description_entry(cie, 0, .{ bytes.count, bytes.data }, fixture.fde_offset);

        assert(!wrong_reference && reference.instructions.count == 0 && reference_at == fixture.fde_offset);
        bytes.data[fixture.fde_offset + 4] = original;
    }

    print("ok\n");
}
