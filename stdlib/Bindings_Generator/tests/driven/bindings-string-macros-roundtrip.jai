#import "Basic";
G :: #import "developer-tools-string-macros-generated";
C :: #import "developer-tools-string-macros-custom";
O :: #import "developer-tools-original-string-generated";

main :: () {
    assert(O.ORIGINAL_STRING == "value");
    ordinary := string.[G.STRING_SIMPLE, G.STRING_ESCAPED, G.STRING_JOINED, G.STRING_ALIAS, G.STRING_UTF8, G.STRING_EMPTY];
    custom := string.[C.RENAMED_STRING, C.STRING_ESCAPED, C.STRING_JOINED, C.STRING_ALIAS, C.STRING_UTF8, C.STRING_EMPTY];

    for text, index: ordinary {
        assert(G.string_macro_matches(cast(*u8) text.data, cast(u64) text.count, cast(s32) index) == 1);
        assert(text == custom[index]);
    }

    print("ok\n");
}
