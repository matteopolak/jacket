// Windows_Utf8 converts between UTF-8 and the UTF-16 Windows APIs take. It exists only on
// Windows; elsewhere this program just reports success.
#import "Basic";
#if OS == .WINDOWS {
    #import "Windows_Utf8";
}

main :: () {
    #if OS == .WINDOWS {
        samples :: string.["plain", "é", "日本語", "😀 and more", "a\x00b"];
        for samples {
            wide, ok, bytes := utf8_to_wide(it);
            assert(ok && wide != null, "to wide '%'", it);
            assert(wide[bytes / 2] == 0, "terminated");
            back, back_ok := wide_to_utf8(wide, bytes / 2);
            assert(back_ok && back == it, "round trip '%' gave '%'", it, back);
            free(back);
            free(wide);
        }
        // Lengths: "😀" is one surrogate pair (4 bytes), "é" one unit.
        emoji, emoji_ok, emoji_bytes := utf8_to_wide("😀");
        assert(emoji_ok && emoji_bytes == 4 && emoji[0] == 0xD83D && emoji[1] == 0xDE00, "surrogates % % %", emoji_bytes, emoji[0], emoji[1]);
        // A terminated wide string converts without a length.
        whole, whole_ok := wide_to_utf8(emoji);
        assert(whole_ok && whole == "😀" && whole.count == 4, "terminated conversion '%'", whole);
        free(emoji);

        empty, empty_ok, empty_bytes := utf8_to_wide("");
        assert(empty_ok && empty_bytes == 0);
        nothing, nothing_ok := wide_to_utf8(null, 0);
        assert(nothing_ok && nothing == "");
        _, null_ok := wide_to_utf8(null);
        assert(!null_ok);
    }
    print("ok\n");
}
