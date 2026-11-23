// test: only(.MACOS)
// A CFString made from bytes keeps embedded NULs and multi-byte UTF-8, reports UTF-16 length, and converts back to the same bytes.
#import "Basic";
M :: #import "macOS";

main :: () {
    text: [6] u8 = .[#char "a", 0, #char "b", #char " ", 0xC3, 0xA9];
    input := string.{ 6, *text[0] };

    cf := M.cfstring_from_string(input);

    assert(cf != null);
    print("length: %\n", M.CFStringGetLength(cf));

    out := M.to_string(cf);

    print("bytes: %\n", out.count);
    print("nul kept: %\n", out[1] == 0);
    print("same bytes: %\n", out == input);
    free(out.data);
    M.CFRelease(cf);

    empty := M.cfstring_from_string("");

    print("empty length: %\n", M.CFStringGetLength(empty));
    empty_text := M.to_string(empty);

    print("empty bytes: %\n", empty_text.count);
    M.CFRelease(empty);
}
