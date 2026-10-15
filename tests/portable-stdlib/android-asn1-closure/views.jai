// Caller storage owns both ASN.1 views; prefix lengths are explicitly LE u32.
#import "Basic";
A :: #import "Android/Toolchain";

main :: () {
    storage: [269] u8;
    storage[0] = 4;
    storage[1] = 0x82;
    storage[2] = 1;
    storage[3] = 4;

    for index: 0..259 storage[index + 4] = cast(u8) (index & 255);

    storage[264] = 0;
    storage[265] = 0;
    storage[266] = 0;
    storage[267] = 0;
    storage[268] = 99;
    input := string.{ storage.count, storage.data };
    value := A.asn1_read_value(*input, .{ universal_type = .OCTET_STRING, length_value = .CONSTRUCTED });

    assert(value.length_value == .PRIMITIVE);
    assert(value.content.count == 260 && value.content.data == storage.data + 4);
    assert(value.value_data.count == 264 && value.value_data.data == storage.data);
    assert(input.count == 5 && input.data == storage.data + 264);
    storage[259] = 77;
    assert(value.content[255] == 77 && value.value_data[259] == 77);
    empty := A.read_length_prefixed_block(*input);

    assert(empty.count == 0 && empty.data == storage.data + 268);
    assert(input.count == 1 && input[0] == 99);

    // 0x00000104 is 260, distinguishing prefix endianness and byte width.
    storage[0] = 4;
    storage[1] = 1;
    storage[2] = 0;
    storage[3] = 0;
    input = .{ 265, storage.data };
    payload := A.read_length_prefixed_block(*input);

    assert(payload.count == 260 && payload.data == storage.data + 4);
    assert(input.count == 1 && input.data == storage.data + 264);
    storage[4] = 42;
    assert(payload[0] == 42);
    print("ok\n");
}
