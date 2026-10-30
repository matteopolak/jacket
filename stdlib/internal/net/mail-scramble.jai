// Obfuscates `text` with `secret` and returns a string of the same length; running it again with the same secret gives the
// text back. This hides text from a casual look and is not encryption.
//
// Both strings may only contain space, `!`, the digits, and ASCII letters; any other character fails an assertion, as does a
// non-empty `text` with an empty `secret`. Each character is XORed with the matching character of `secret`, which repeats
// as needed, within that 64-character alphabet. The result is in temporary storage: it is not freed by the caller and is
// invalid after the temporary storage is reset, so copy it to keep it.
//
// ```jai
// text := "Az 19!aZ0";
// hidden := scramble(text, "A!z");
//
// assert(hidden.count == text.count);
// assert(hidden != text);
// assert(scramble(hidden, "A!z") == text);
// ```
scramble :: (text: string, secret: string) -> string {
    if text.count == 0 return "";

    if secret.count == 0 {
        assert(false, "Mail.scramble requires a secret for nonempty text");
        return "";
    }

    for byte: secret {
        if alphabet_index(byte) < 0 {
            assert(false, "Mail.scramble secret contains an unsupported character");
            return "";
        }
    }

    for byte: text {
        if alphabet_index(byte) < 0 {
            assert(false, "Mail.scramble text contains an unsupported character");
            return "";
        }
    }

    data := cast(*u8) temporary_alloc(text.count);
    if !data {
        assert(false, "Mail.scramble temporary allocation failed");
        return "";
    }

    for byte, index: text {
        left := alphabet_index(byte);
        right := alphabet_index(secret[index % secret.count]);
        data[index] = ALPHABET[left ^ right];
    }

    return .{ count = text.count, data = data };
}

#scope_file
using #import "Basic";

ALPHABET :: " !0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

alphabet_index :: (byte: u8) -> s64 {
    for allowed, index: ALPHABET if allowed == byte return index;
    return -1;
}
