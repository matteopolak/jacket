// Compile-time code of a 32-bit target reads files through the interpreter's own libc, also when
// a plugin made the workspace it runs in.
#import "Basic";
#import "File";

own_text :: #run -> string {
    text, ok := read_entire_file(#file);
    assert(ok);
    return text;
};

main :: () {
    print("%\n", own_text.count > 0);
}
