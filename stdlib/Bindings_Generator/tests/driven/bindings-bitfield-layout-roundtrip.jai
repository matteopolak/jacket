// The generated bitfield records against the C compiler: sizes, alignments, member offsets and every
// bitfield written and read through the accessors, byte for byte against C doing the same.
#import "Basic";
G :: #import "developer-tools-bitfield-layout-generated";

native :: #library "../../../../target/developer_tools_bitfield_layout";
oracle_fact :: (key: *u8) -> s64 #foreign native;
oracle_field_count :: () -> s64 #foreign native;
oracle_get :: (key: *u8, record: *void) -> s64 #foreign native;
oracle_set :: (key: *u8, record: *void, value: s64) -> s64 #foreign native;

failures: int;
checked_fields: int;

expect :: (condition: bool, format: string, args: ..Any) {
    if condition return;

    failures += 1;
    print(format, ..args);
    print("\n");
}

fact :: (key: string) -> s64 {
    return oracle_fact(temp_c_string(key));
}

check_record :: ($T: Type, name: string) {
    size := fact(tprint("%.size", name));
    alignment := fact(tprint("%.align", name));

    expect(size > 0 && size_of(T) == size, "% size: jai %, c %", name, size_of(T), size);
    expect(alignment > 0 && align_of(T) == alignment, "% alignment: jai %, c %", name, align_of(T), alignment);
}

check_offset :: (name: string, member: string, jai: s64) {
    native_offset := fact(tprint("%.%", name, member));

    expect(native_offset >= 0 && native_offset == jai, "%.% offset: jai %, c %", name, member, jai, native_offset);
}

signed_integer :: ($V: Type) -> bool {
    info := type_info(V);

    if info.type != .INTEGER return false;

    return (cast(*Type_Info_Integer) info).signed;
}

to_bits :: (value: $V) -> s64 {
    #if V == bool return ifx value then 1 else 0;
    else return cast,no_check(s64) value;
}

from_bits :: (bits: s64, $V: Type) -> V {
    #if V == bool return bits != 0;
    else return cast,no_check(V) bits;
}

// Bit patterns that exercise a field of this width; signed fields get them sign-extended.
candidate_values :: (width: int, signed: bool) -> [] s64 {
    result: [..] s64;
    result.allocator = temp;
    mask: u64 = 0xffff_ffff_ffff_ffff;

    if width < 64 mask = (cast(u64) 1 << cast(u64) width) - 1;

    patterns := u64.[0, 1, 2, mask, mask >> 1, (mask >> 1) + 1, 0x5555_5555_5555_5555 & mask, 0xaaaa_aaaa_aaaa_aaaa & mask, 0x0123_4567_89ab_cdef & mask];

    for pattern: patterns {
        value := pattern & mask;

        if signed && width < 64 && (value & ((cast(u64) 1) << cast(u64) (width - 1))) != 0 value |= ~mask;

        bits := cast,no_check(s64) value;

        if !array_find(result, bits) array_add(*result, bits);
    }

    return result;
}

bytes_equal :: (a: *u8, b: *u8, count: s64) -> bool {
    for 0..count - 1 if a[it] != b[it] return false;

    return true;
}

// Writes `width`-bit values through the Jai accessors and through C into records that start filled
// with a byte pattern, then compares the bytes and reads each side's bytes with the other side.
check_field :: ($T: Type, $V: Type, key: string, get: (T) -> V, set: (*T, V), width: int, only: [] s64 = .[]) {
    values := only;

    if !values.count values = candidate_values(width, signed_integer(V));

    c_key := temp_c_string(key);

    checked_fields += 1;

    for fill: u8.[0x00, 0xff, 0xa5] {
        for bits: values {
            jai: T;
            native_record: T;

            for 0..size_of(T) - 1 {
                (cast(*u8) *jai)[it] = fill;
                (cast(*u8) *native_record)[it] = fill;
            }

            set(*jai, from_bits(bits, V));
            expect(oracle_set(c_key, *native_record, bits) == 1, "% is not a field C knows", key);

            expect(bytes_equal(cast(*u8) *jai, cast(*u8) *native_record, size_of(T)), "% = % over %: bytes differ from C", key, bits, fill);
            expect(to_bits(get(jai)) == bits, "% = %: jai reads back %", key, bits, to_bits(get(jai)));
            expect(to_bits(get(native_record)) == bits, "% = %: jai reads % from C bytes", key, bits, to_bits(get(native_record)));
            expect(oracle_get(c_key, *jai) == bits, "% = %: C reads % from jai bytes", key, bits, oracle_get(c_key, *jai));
        }
    }
}

main :: () {
    check_record(G.BfIssue, "BfIssue");
    check_record(G.BfSigned, "BfSigned");
    check_record(G.BfBool, "BfBool");
    check_record(G.BfEnum, "BfEnum");
    check_record(G.BfUnnamed, "BfUnnamed");
    check_record(G.BfZeroWidth, "BfZeroWidth");
    check_record(G.BfStraddle, "BfStraddle");
    check_record(G.BfWide, "BfWide");
    check_record(G.BfTrail, "BfTrail");
    check_record(G.BfGap, "BfGap");
    check_record(G.BfComments, "BfComments");
    check_record(G.BfInner, "BfInner");
    check_record(G.BfNested, "BfNested");
    check_record(G.BfClash, "BfClash");
    check_record(G.BfAnonymous, "BfAnonymous");
    check_record(G.BfUnion, "BfUnion");
    check_record(G.BfUnionBits, "BfUnionBits");
    check_record(G.BfPack1, "BfPack1");
    check_record(G.BfPack2, "BfPack2");
    check_record(G.BfPackedBits, "BfPackedBits");
    check_record(G.BfPackedUnion, "BfPackedUnion");

    // The reporter's record is 16 bytes, not 24.
    expect(size_of(G.BfIssue) == 16, "BfIssue is % bytes", size_of(G.BfIssue));

    check_offset("BfIssue", "section", offset_of(G.BfIssue, "section"));
    check_offset("BfIssue", "offset", offset_of(G.BfIssue, "offset"));
    check_offset("BfZeroWidth", "tail", offset_of(G.BfZeroWidth, "tail"));
    check_offset("BfWide", "after", offset_of(G.BfWide, "after"));
    check_offset("BfTrail", "before", offset_of(G.BfTrail, "before"));
    check_offset("BfTrail", "after", offset_of(G.BfTrail, "after"));
    check_offset("BfGap", "between", offset_of(G.BfGap, "between"));
    check_offset("BfComments", "tail", offset_of(G.BfComments, "tail"));
    check_offset("BfNested", "inner", offset_of(G.BfNested, "inner"));
    check_offset("BfAnonymous", "before", offset_of(G.BfAnonymous, "before"));
    check_offset("BfUnion", "whole", offset_of(G.BfUnion, "whole"));
    check_offset("BfUnion", "parts", offset_of(G.BfUnion, "parts"));
    check_offset("BfPack1", "a", offset_of(G.BfPack1, "a"));
    check_offset("BfPack1", "b", offset_of(G.BfPack1, "b"));
    check_offset("BfPack1", "c", offset_of(G.BfPack1, "c"));
    check_offset("BfPack2", "a", offset_of(G.BfPack2, "a"));
    check_offset("BfPack2", "b", offset_of(G.BfPack2, "b"));
    check_offset("BfPackedBits", "a", offset_of(G.BfPackedBits, "a"));
    check_offset("BfPackedBits", "z", offset_of(G.BfPackedBits, "z"));
    check_offset("BfPackedUnion", "byte", offset_of(G.BfPackedUnion, "byte"));

    check_field(G.BfIssue, u32, "BfIssue.type", G.BfIssue_type, G.BfIssue_set_type, 8);
    check_field(G.BfIssue, u32, "BfIssue.size", G.BfIssue_size, G.BfIssue_set_size, 24);

    check_field(G.BfSigned, s8, "BfSigned.a", G.BfSigned_a, G.BfSigned_set_a, 3);
    check_field(G.BfSigned, s16, "BfSigned.b", G.BfSigned_b, G.BfSigned_set_b, 5);
    check_field(G.BfSigned, u32, "BfSigned.c", G.BfSigned_c, G.BfSigned_set_c, 7);
    check_field(G.BfSigned, s32, "BfSigned.d", G.BfSigned_d, G.BfSigned_set_d, 17);
    check_field(G.BfSigned, s64, "BfSigned.e", G.BfSigned_e, G.BfSigned_set_e, 33);
    check_field(G.BfSigned, u64, "BfSigned.f", G.BfSigned_f, G.BfSigned_set_f, 31);

    check_field(G.BfBool, bool, "BfBool.flag", G.BfBool_flag, G.BfBool_set_flag, 1);
    check_field(G.BfBool, bool, "BfBool.other", G.BfBool_other, G.BfBool_set_other, 1);
    check_field(G.BfBool, u8, "BfBool.rest", G.BfBool_rest, G.BfBool_set_rest, 6);
    check_field(G.BfBool, bool, "BfBool.last", G.BfBool_last, G.BfBool_set_last, 1);

    check_field(G.BfEnum, G.BfColor, "BfEnum.color", G.BfEnum_color, G.BfEnum_set_color, 2, s64.[0, 1, 2]);
    check_field(G.BfEnum, G.BfLevel, "BfEnum.level", G.BfEnum_level, G.BfEnum_set_level, 3, s64.[-2, 0, 1]);
    check_field(G.BfEnum, bool, "BfEnum.on", G.BfEnum_on, G.BfEnum_set_on, 1);
    check_field(G.BfEnum, s32, "BfEnum.small", G.BfEnum_small, G.BfEnum_set_small, 2);

    check_field(G.BfUnnamed, u32, "BfUnnamed.a", G.BfUnnamed_a, G.BfUnnamed_set_a, 3);
    check_field(G.BfUnnamed, u32, "BfUnnamed.b", G.BfUnnamed_b, G.BfUnnamed_set_b, 4);
    check_field(G.BfUnnamed, u32, "BfUnnamed.c", G.BfUnnamed_c, G.BfUnnamed_set_c, 2);
    check_field(G.BfUnnamed, u32, "BfUnnamed.d", G.BfUnnamed_d, G.BfUnnamed_set_d, 9);

    check_field(G.BfZeroWidth, u8, "BfZeroWidth.a", G.BfZeroWidth_a, G.BfZeroWidth_set_a, 3);
    check_field(G.BfZeroWidth, u8, "BfZeroWidth.b", G.BfZeroWidth_b, G.BfZeroWidth_set_b, 2);
    check_field(G.BfZeroWidth, u16, "BfZeroWidth.c", G.BfZeroWidth_c, G.BfZeroWidth_set_c, 11);

    check_field(G.BfStraddle, u8, "BfStraddle.a", G.BfStraddle_a, G.BfStraddle_set_a, 6);
    check_field(G.BfStraddle, u8, "BfStraddle.b", G.BfStraddle_b, G.BfStraddle_set_b, 6);
    check_field(G.BfStraddle, u16, "BfStraddle.c", G.BfStraddle_c, G.BfStraddle_set_c, 6);
    check_field(G.BfStraddle, u16, "BfStraddle.d", G.BfStraddle_d, G.BfStraddle_set_d, 12);
    check_field(G.BfStraddle, u32, "BfStraddle.e", G.BfStraddle_e, G.BfStraddle_set_e, 20);
    check_field(G.BfStraddle, u32, "BfStraddle.f", G.BfStraddle_f, G.BfStraddle_set_f, 20);

    check_field(G.BfWide, u64, "BfWide.whole", G.BfWide_whole, G.BfWide_set_whole, 64);
    check_field(G.BfWide, s64, "BfWide.signed_whole", G.BfWide_signed_whole, G.BfWide_set_signed_whole, 64);
    check_field(G.BfWide, u64, "BfWide.high", G.BfWide_high, G.BfWide_set_high, 40);
    check_field(G.BfWide, u64, "BfWide.low", G.BfWide_low, G.BfWide_set_low, 24);

    check_field(G.BfTrail, u32, "BfTrail.bits", G.BfTrail_bits, G.BfTrail_set_bits, 3);

    check_field(G.BfGap, u8, "BfGap.a", G.BfGap_a, G.BfGap_set_a, 1);
    check_field(G.BfGap, u8, "BfGap.b", G.BfGap_b, G.BfGap_set_b, 3);
    check_field(G.BfGap, u8, "BfGap.c", G.BfGap_c, G.BfGap_set_c, 7);

    check_field(G.BfComments, u16, "BfComments.count", G.BfComments_count, G.BfComments_set_count, 10);
    check_field(G.BfComments, u16, "BfComments.flag", G.BfComments_flag, G.BfComments_set_flag, 1);
    check_field(G.BfComments, u16, "BfComments.level", G.BfComments_level, G.BfComments_set_level, 5);

    check_field(G.BfInner, s8, "BfInner.x", G.BfInner_x, G.BfInner_set_x, 4);
    check_field(G.BfInner, u8, "BfInner.y", G.BfInner_y, G.BfInner_set_y, 4);
    check_field(G.BfNested, u16, "BfNested.head", G.BfNested_head, G.BfNested_set_head, 5);
    check_field(G.BfNested, u16, "BfNested.foot", G.BfNested_foot, G.BfNested_set_foot, 9);

    // The constant BfClash_x took that accessor name, so the pair for `x` carries a suffix.
    check_field(G.BfClash, s32, "BfClash.x", G.BfClash_x_1, G.BfClash_set_x_1, 3);
    check_field(G.BfClash, s32, "BfClash.y", G.BfClash_y, G.BfClash_set_y, 4);
    expect(G.BfClash_x == 5, "the constant BfClash_x changed");

    check_field(G.BfAnonymous, u8, "BfAnonymous.p", G.BfAnonymous_p, G.BfAnonymous_set_p, 1);
    check_field(G.BfAnonymous, u8, "BfAnonymous.q", G.BfAnonymous_q, G.BfAnonymous_set_q, 2);

    check_field(G.BfUnion, u32, "BfUnion.parts_lo", G.BfUnion_parts_lo, G.BfUnion_set_parts_lo, 8);
    check_field(G.BfUnion, u32, "BfUnion.parts_hi", G.BfUnion_parts_hi, G.BfUnion_set_parts_hi, 24);
    check_field(G.BfUnion, u16, "BfUnion.narrow", G.BfUnion_narrow, G.BfUnion_set_narrow, 9);
    check_field(G.BfUnionBits, u32, "BfUnionBits.a", G.BfUnionBits_a, G.BfUnionBits_set_a, 5);
    check_field(G.BfUnionBits, s32, "BfUnionBits.b", G.BfUnionBits_b, G.BfUnionBits_set_b, 17);

    check_field(G.BfPack1, u32, "BfPack1.f", G.BfPack1_f, G.BfPack1_set_f, 3);
    check_field(G.BfPack1, u32, "BfPack1.g", G.BfPack1_g, G.BfPack1_set_g, 13);
    check_field(G.BfPack2, u16, "BfPack2.f", G.BfPack2_f, G.BfPack2_set_f, 5);
    check_field(G.BfPack2, u16, "BfPack2.g", G.BfPack2_g, G.BfPack2_set_g, 9);
    check_field(G.BfPack2, u32, "BfPack2.h", G.BfPack2_h, G.BfPack2_set_h, 20);
    check_field(G.BfPackedBits, u32, "BfPackedBits.x", G.BfPackedBits_x, G.BfPackedBits_set_x, 4);
    check_field(G.BfPackedBits, u32, "BfPackedBits.y", G.BfPackedBits_y, G.BfPackedBits_set_y, 12);
    check_field(G.BfPackedUnion, u32, "BfPackedUnion.wide", G.BfPackedUnion_wide, G.BfPackedUnion_set_wide, 21);

    // Every field C knows about has a check above.
    expect(checked_fields == oracle_field_count(), "% fields checked, C has %", checked_fields, oracle_field_count());

    // One record written field by field matches C doing the same, so neighbours do not disturb each other.
    jai: G.BfIssue;
    native_record: G.BfIssue;

    G.BfIssue_set_type(*jai, 0x9c);
    G.BfIssue_set_size(*jai, 0xabcdef);
    jai.offset = 77;
    expect(oracle_set(temp_c_string("BfIssue.type"), *native_record, 0x9c) == 1, "BfIssue.type");
    expect(oracle_set(temp_c_string("BfIssue.size"), *native_record, 0xabcdef) == 1, "BfIssue.size");
    native_record.offset = 77;
    expect(bytes_equal(cast(*u8) *jai, cast(*u8) *native_record, size_of(G.BfIssue)), "BfIssue written field by field differs from C");
    expect(G.BfIssue_type(jai) == 0x9c && G.BfIssue_size(jai) == 0xabcdef && jai.offset == 77, "BfIssue reads back wrongly");

    assert(failures == 0, "% bitfield checks failed", failures);
    print("ok\n");
}
