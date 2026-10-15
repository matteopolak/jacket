// Actual Python native hashlib digests; production hashing remains independent Jai.
#import "Basic";
A :: #import "Android/Toolchain";
F :: #import "File";
S :: #import "String";

main :: () {
    filename := sprint("%/../../../target/portable-sha/vectors.bin", S.path_strip_filename(#file));

    defer free(filename);
    data, loaded := F.read_entire_file(filename);

    assert(loaded && data.count >= 8);
    defer free(data);
    header := cast(*u32) data.data;

    assert(header[0] == 0x53484132 && data.count == 8 + cast(s64) header[1] * 60);

    for case_index: 0..cast(s64) header[1] - 1 {
        row := data.data + 8 + case_index * 60;
        length := cast(s64) (cast(*u32) row).*;
        mode := (cast(*u32) (row + 4)).*;
        storage := alloc_string(length + 1);
        input: string = .{ length, storage.data + 1 };

        if mode == 1 memset(input.data, 97, length);
        else for index: 0..length - 1 input[index] = cast(u8) ((index * 73 + length * 19) & 255);

        one := A.sha1(input);
        two := A.sha256(input);

        for byte, index: one assert(byte == row[8 + index]);
        for byte, index: two assert(byte == row[28 + index]);

        free(storage);
    }

    print("ok\n");
}
