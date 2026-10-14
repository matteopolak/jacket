#import "Basic";
M :: #import "pl_mpeg";
F :: #import "File";
S :: #import "String";

main :: () {
    filename := sprint("%/../../stdlib/pl-mpeg-clip.mpg", S.path_strip_filename(#file));

    defer free(filename);
    clip, loaded := F.read_entire_file(filename);

    assert(loaded && clip.count == 6144);
    data := cast(*u8) clip.data;
    data[34] = 0x7f; // The claimed PES extent exceeds this finite source.
    M.create_with_memory((.{ data = data, count = clip.count }), true);
}
