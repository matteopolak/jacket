// Independently generated Python integer/base128 wire vectors.
#import "Basic";
A :: #import "Android/Toolchain";
F :: #import "File";
S :: #import "String";

word :: (s: *string, count: s64) -> u64 {
    assert(s.count >= count);
    result: u64;
    for index: 0..count - 1 result |= (cast(u64) s.data[index]) << (8 * index);

    s.data += count;
    s.count -= count;

    return result;
}

block :: (s: *string) -> string {
    count := word(s, 4);

    assert(count <= cast(u64) s.count);
    result := string.{ cast(s64) count, s.data };

    s.data += count;
    s.count -= cast(s64) count;

    return result;
}

main :: () {
    filename := sprint("%/../../../target/portable-asn1/vectors.bin", S.path_strip_filename(#file));

    defer free(filename);
    data, loaded := F.read_entire_file(filename);

    assert(loaded);
    defer free(data);
    cursor := data;
    integers := word(*cursor, 4);

    for 0..cast(s64) integers - 1 {
        input := block(*cursor);
        bits := word(*cursor, 8);
        expected: s64;

        memcpy(*expected, *bits, 8);
        assert(A.asn1_decode_integer(input) == expected);
    }

    identifiers := word(*cursor, 4);
    for 0..cast(s64) identifiers - 1 {
        input := block(*cursor);
        count := word(*cursor, 4);
        expected: A.ASN1_Object_Identifier;
        expected.parts_count = cast(int) count;
        for index: 0..cast(s64) count - 1 expected.parts[index] = word(*cursor, 8);

        text := block(*cursor);
        actual := A.asn1_decode_object_identifier(input);

        assert(A.asn1_oid_equals(actual, expected));
        printed := A.asn1_to_string(actual);

        assert(printed == text);
        free(printed);
    }

    assert(cursor.count == 0);
    print("ok\n");
}
