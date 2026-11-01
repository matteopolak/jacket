// The record format and its checksum.

#import "Basic";
#import "String";
#import "Extensions/Testing";
#import,dir "../store";

round_trips_a_record :: () {
    bytes := encode_record(.PUT, "key", "value");

    defer free(bytes);

    expect_equal(bytes.count, HEADER_SIZE + 3 + 5);

    record, status := decode_record(bytes, 0);

    expect_equal(status, Decode_Status.OK);
    expect_equal(record.kind, Record_Kind.PUT);
    expect_equal(record.key, "key");
    expect_equal(record.value, "value");
    expect_equal(record.size, bytes.count);
} @test

a_delete_record_has_no_value :: () {
    bytes := encode_record(.DELETE, "key", "");

    defer free(bytes);

    record, status := decode_record(bytes, 0);

    expect_equal(status, Decode_Status.OK);
    expect_equal(record.kind, Record_Kind.DELETE);
    expect_equal(record.value, "");
} @test

decodes_from_an_offset :: () {
    first := encode_record(.PUT, "a", "1");
    second := encode_record(.PUT, "b", "22");

    defer free(first);
    defer free(second);

    both := join(first, second,, temp);
    record, status := decode_record(both, first.count);

    expect_equal(status, Decode_Status.OK);
    expect_equal(record.key, "b");
    expect_equal(record.value, "22");
} @test

// Every single-bit change to a record is caught: either the checksum no longer matches, or the
// header no longer describes a record of ours.
any_flipped_bit_is_detected :: () {
    bytes := encode_record(.PUT, "key", "value");

    defer free(bytes);

    for byte: 0..bytes.count - 1 {
        for bit: 0..7 {
            damaged := copy_string(bytes,, temp);
            damaged[byte] ^= cast(u8) (1 << bit);

            _, status := decode_record(damaged, 0);

            // A damaged size can make the record look longer than the data (torn); everything
            // else is a bad checksum or a bad header. None may decode.
            expect(status != .OK, tprint("bit % of byte %", bit, byte));
        }
    }
} @test

every_prefix_of_a_record_is_torn :: () {
    bytes := encode_record(.PUT, "key", "value");

    defer free(bytes);

    for length: 0..bytes.count - 1 {
        _, status := decode_record(slice(bytes, 0, length), 0);

        expect_equal(status, Decode_Status.TORN);
    }
} @test

integers_are_little_endian :: () {
    bytes := encode_record(.PUT, "k", "vvv");

    defer free(bytes);

    expect_equal(bytes[4], 1);
    expect_equal(bytes[5], 1);
    expect_equal(bytes[6], 0);
    expect_equal(bytes[9], 3);
} @test

the_checksum_is_the_start_of_the_md5_digest :: () {
    // md5("abc") = 900150983cd24fb0d6963f7d28e17f72
    expect_equal(checksum("abc"), 0x90015098);
    expect_equal(checksum(""), 0xd41d8cd9);
} @test
