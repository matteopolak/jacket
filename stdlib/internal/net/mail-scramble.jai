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
