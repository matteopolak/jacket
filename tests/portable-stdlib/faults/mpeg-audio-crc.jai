#import "Basic";
M :: #import "pl_mpeg";
F :: #import "File";
S :: #import "String";

main :: () {
    filename := sprint("%/../../../target/portable-tests/mpeg/joint-crc.mp2", S.path_strip_filename(#file));

    defer free(filename);
    bytes, loaded := F.read_entire_file(filename);

    assert(loaded && bytes.count == 834);
    data := cast(*u8) bytes.data;
    data[4] ^= 1;
    buffer := M.buffer_create_with_memory((.{ data = data, count = bytes.count }), true);
    audio := M.audio_create_with_buffer(buffer, true);

    assert(audio.has_header);
    M.audio_decode(audio);
}
