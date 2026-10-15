#import "Basic";
#import "File";
D :: #import "Debug_Info";

main :: () {
    data: [10] u8 = .[0x0c, 7, 8, 0x90, 1, 0x44, 0x11, 6, 0x78, 0x00];
    rows, ok := D.decode_cfa_instructions(data);

    defer array_free(rows);
    assert(ok && rows.count == 5);
    assert(rows[0].type == .DEF_CFA && rows[0].arg1.uvalue == 7 && rows[0].arg2.uvalue == 8);
    assert(rows[1].type == .OFFSET && rows[1].arg1.uvalue == 16 && rows[1].arg2.uvalue == 1);
    assert(rows[2].type == .ADVANCE_LOC && rows[2].arg1.uvalue == 4);
    assert(rows[3].type == .DEF_CFA_SF && rows[3].arg1.uvalue == 6 && rows[3].arg2.svalue == -8);
    wide: [8] u8 = .[0x03, 0x34, 0x12, 0x04, 0x78, 0x56, 0x34, 0x12];
    wide_rows, wide_ok := D.decode_cfa_instructions(wide);

    defer array_free(wide_rows);
    assert(wide_ok && wide_rows.count == 2);
    assert(wide_rows[0].arg1.uvalue == 0x1234 && wide_rows[1].arg1.uvalue == 0x12345678);
    truncated: [1] u8 = .[0x0c];
    rejected, truncated_ok := D.decode_cfa_instructions(truncated);

    assert(!truncated_ok && rejected.count == 0);
    overflow: [11] u8 = .[0x0d, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 2];
    invalid, overflow_ok := D.decode_cfa_instructions(overflow);

    assert(!overflow_ok && invalid.count == 0);
    native, read_ok := read_entire_file(tprint("%../../target/developer-tools-eh-frame.bin", #filepath));

    defer free(native);
    assert(read_ok);
    parsed, cie, next := D.parse_common_information_entry(.{ native.count, native.data }, 0);

    defer array_free(cie.initial_instructions);
    assert(parsed && next > 0 && cie.version == 1);
    assert(cie.code_alignment_factor == 1 && cie.data_alignment_factor == -8);
    assert(cie.return_address_register == 16 && cie.initial_instructions.count > 0);

    for count: 0..next - 1 {
        prefix_ok, prefix, prefix_end := D.parse_common_information_entry(.{ count, native.data }, 0);
        assert(!prefix_ok && prefix.initial_instructions.count == 0 && prefix_end == 0);
    }

    unsupported: [1] u8 = .[0x0f];
    no_expression, expression_ok := D.decode_cfa_instructions(unsupported);

    assert(!expression_ok && no_expression.count == 0);
    signed_min: [11] u8 = .[0x12, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x7f];
    minimum, min_ok := D.decode_cfa_instructions(signed_min);

    defer array_free(minimum);
    assert(min_ok && minimum[0].arg1.svalue == -9223372036854775808);
    print("ok\n");
}
