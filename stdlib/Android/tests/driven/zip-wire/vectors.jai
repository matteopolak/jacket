// Independent native-zlib CRC vectors, read through reviewed binary cursors.
#import "Basic";
#import "Extensions/Environment";
A :: #import "Android/Toolchain";
F :: #import "File";
S :: #import "String";

main :: () {
    filename := copy_string(output_path("portable-zip-wire/crc-vectors.bin"));

    defer free(filename);
    data, loaded := F.read_entire_file(filename);

    assert(loaded);
    defer free(data);
    cursor := data;
    count := A.read(u32, *cursor);

    assert(count >= 260);

    for 0..cast(s64) count - 1 {
        bytes := A.read_length_prefixed_block(*cursor);
        expected := A.read(u32, *cursor);
        assert(A.zip_crc32(bytes) == expected);
    }

    assert(cursor.count == 0);
    print("ok\n");
}
