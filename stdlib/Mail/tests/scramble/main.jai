// Reviewed Mail scramble: 64-character XOR alphabet, cyclic secret, temporary result.
using #import "Basic";
M :: #import "Mail";

main :: () {
    args := get_command_line_arguments();
    if args.count > 1 {
        if args[1] == "bad-text" M.scramble("@", "A");
        if args[1] == "bad-secret" M.scramble("A", "@");
        if args[1] == "empty-secret" M.scramble("A", "");
        return;
    }

    mark := get_temporary_storage_mark();

    defer set_temporary_storage_mark(mark);
    alphabet := " !0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

    assert(alphabet.count == 64);
    assert(M.scramble("", "") == "");

    for key: alphabet {
        secret: string = .{ count = 1, data = *key };
        encrypted := M.scramble(alphabet, secret);

        assert(encrypted.count == alphabet.count);
        assert(M.scramble(encrypted, secret) == alphabet);
        seen: [256] bool;

        for byte: encrypted {
            assert(!seen[byte]);
            seen[byte] = true;
            permitted := false;
            for valid: alphabet if valid == byte permitted = true;

            assert(permitted);
        }
    }

    text := "Az 19!aZ0";
    encrypted := M.scramble(text, "A!z");

    assert(M.scramble(encrypted, "A!z") == text);

    for byte, i: text {
        input: string = .{ count = 1, data = *byte };
        key := "A!z"[i % 3];
        secret: string = .{ count = 1, data = *key };
        expected := M.scramble(input, secret);

        assert(encrypted[i] == expected[0]);
    }

    assert(text == "Az 19!aZ0");
    print("ok\n");
}
