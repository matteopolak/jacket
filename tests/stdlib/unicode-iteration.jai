// Unicode at run time: encoding scalars to new strings, stepping through UTF-8 with
// utf8_next_character and unicode_next_character, and utf8_iter forwards and in reverse,
// including how each stops at malformed input.
#import "Basic";
#import "Unicode";

Case :: struct {
    code: u32;
    bytes: string;
}

main :: () {
    {
        // Every encoded length, and the boundaries between them.
        cases :: Case.[
            .{ 0x24, "$" },
            .{ 0x7F, "\x7f" },
            .{ 0x80, "\xc2\x80" },
            .{ 0xE9, "é" },
            .{ 0x7FF, "\xdf\xbf" },
            .{ 0x800, "\xe0\xa0\x80" },
            .{ 0x20AC, "€" },
            .{ 0xFFFD, "\xef\xbf\xbd" },
            .{ 0xFFFF, "\xef\xbf\xbf" },
            .{ 0x10000, "\xf0\x90\x80\x80" },
            .{ 0x1F600, "😀" },
            .{ 0x10FFFF, "\xf4\x8f\xbf\xbf" },
        ];
        for cases {
            encoded := character_utf32_to_utf8(it.code);
            assert(encoded == it.bytes, "U+% encodes as % bytes", formatInt(it.code, base = 16), encoded.count);
            code, consumed, status := character_utf8_to_utf32(it.bytes.data, it.bytes.count);
            assert(status == .CONVERSION_OK && code == it.code && consumed == it.bytes.count, "U+% decodes as %", formatInt(it.code, base = 16), formatInt(code, base = 16));
            free(encoded);
        }
        // Surrogates and values past U+10FFFF have no encoding.
        assert(character_utf32_to_utf8(0xD800) == "");
        assert(character_utf32_to_utf8(0xDFFF) == "");
        assert(character_utf32_to_utf8(0x110000) == "");
    }
    {
        text := "aé€😀";
        rest := text;
        expected :: u32.[0x61, 0xE9, 0x20AC, 0x1F600];
        for expected {
            code, status := utf8_next_character(*rest);
            assert(status == .CONVERSION_OK && code == it, "character %: %", it_index, formatInt(code, base = 16));
        }
        assert(rest.count == 0);
        // At the end there is nothing more.
        _, status := utf8_next_character(*rest);
        assert(status == .SOURCE_EXHAUSTED);

        // A malformed byte is reported and not consumed.
        bad := "a\xffb";
        code: u32;
        code, status = utf8_next_character(*bad);
        assert(status == .CONVERSION_OK && code == #char "a");
        code, status = utf8_next_character(*bad);
        assert(status == .SOURCE_ILLEGAL && code == UNI_REPLACEMENT_CHAR && bad.count == 2);
        // A truncated sequence needs more bytes.
        cut := string.{ 2, "€".data };
        _, status = utf8_next_character(*cut);
        assert(status == .SOURCE_EXHAUSTED && cut.count == 2);
        // Overlong encodings are illegal.
        overlong := "\xe0\x80\xaf";
        _, status = utf8_next_character(*overlong);
        assert(status == .SOURCE_ILLEGAL);
    }
    {
        // unicode_next_character steps by the length its first byte announces.
        text := "aé€😀";
        p := text.data;
        steps: [4] s64;
        for 0..3 {
            next := unicode_next_character(p);
            steps[it] = next - p;
            p = next;
        }
        assert(steps[0] == 1 && steps[1] == 2 && steps[2] == 3 && steps[3] == 4, "steps %", steps);
        assert(p == text.data + text.count);
        // Continuation or invalid bytes step by one; null stays null.
        stray := "\x80\xff";
        assert(unicode_next_character(stray.data) == stray.data + 1);
        assert(unicode_next_character(stray.data + 1) == stray.data + 2);
        assert(unicode_next_character(null) == null);
    }
    {
        text := "hé€😀!";
        forward: [..] u32;
        indices: [..] s64;
        for: utf8_iter * text {
            array_add(*forward, it);
            array_add(*indices, it_index);
        }
        assert(forward.count == 5 && forward[0] == #char "h" && forward[1] == 0xE9 && forward[2] == 0x20AC && forward[3] == 0x1F600 && forward[4] == #char "!", "forward %", forward);
        for indices assert(it == it_index, "index % is %", it_index, it);
        // The string itself is not consumed.
        assert(text.count == 11);

        backward: [..] u32;
        for <: utf8_iter * text array_add(*backward, it);
        assert(backward.count == 5, "% backwards", backward.count);
        for backward assert(it == forward[4 - it_index], "backward % is %", it_index, it);

        // Iteration stops at the first malformed sequence.
        broken := "ab\xffcd";
        seen: [..] u32;
        for: utf8_iter * broken array_add(*seen, it);
        assert(seen.count == 2 && seen[1] == #char "b", "stopped after %", seen.count);
        empty := "";
        count := 0;
        for: utf8_iter * empty count += 1;
        for <: utf8_iter * empty count += 1;
        assert(count == 0);
        // break leaves the loop early.
        count = 0;
        for: utf8_iter * text {
            count += 1;
            if it == 0x20AC break;
        }
        assert(count == 3);
    }
    print("ok\n");
}
