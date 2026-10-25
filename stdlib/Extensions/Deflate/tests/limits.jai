// inflate_into with an exact, too small and larger buffer (and nothing written outside it),
// and the max_output limit of inflate.
//
//     jaic stdlib/Extensions/Deflate/tests/limits.jai -interp

#load "support/helpers.jai";

into_buffer :: (stream: [] u8, container: Deflate_Container, size: int, expected: [] u8, label: string) {
    guard := NewArray(size + 64, u8);
    for *guard it.* = 0xCD;

    window: [] u8;
    window.data = guard.data + 32;
    window.count = size;

    written, ok, error := inflate_into(stream, window, container);

    if size >= expected.count {
        check(ok && written == expected.count, tprint("%: size % should fit (%)", label, size, error));

        if ok {
            for 0..expected.count - 1 {
                if window[it] != expected[it] {
                    check(false, tprint("%: size % differs at %", label, size, it));
                    break;
                }
            }
        }
    } else {
        check(!ok, tprint("%: size % is too small but was accepted", label, size));
        check(error == "output is too small", tprint("%: size %: error was \"%\"", label, size, error));
        check(written == 0, tprint("%: size %: written should be 0", label, size));
    }

    for 0..31 {
        check(guard[it] == 0xCD && guard[32 + size + it] == 0xCD, tprint("%: size % wrote outside the buffer", label, size));
    }

    free(guard.data);
}

inflate_into_sizes :: () {
    inputs: [3][] u8;
    inputs[0] = text_bytes(3000, 21);
    inputs[1] = random_bytes(700, 22);
    inputs[2] = run_bytes(2500);

    for data: inputs {
        for container: Deflate_Container.[.RAW, .ZLIB, .GZIP] {
            for level: s32.[0, 1, 6] {
                stream := deflate(data, container, level);
                label := tprint("% bytes % level %", data.count, container, level);

                sizes := int.[0, 1, 2, 17, data.count / 2, data.count - 1, data.count, data.count + 1, data.count + 300];

                for sizes into_buffer(stream, container, it, data, label);

                free(stream.data);
            }
        }
    }

    // An empty result fits an empty buffer.
    empty := deflate(.[], .ZLIB);

    into_buffer(empty, .ZLIB, 0, .[], "empty stream");
}

limit_checks :: () {
    data := text_bytes(5000, 31);

    for container: Deflate_Container.[.RAW, .ZLIB, .GZIP] {
        for level: s32.[0, 6] {
            stream := deflate(data, container, level);

            output, ok := inflate(stream, container, data.count);

            check(ok && output.count == data.count, tprint("limit equal to the size (% level %)", container, level));
            if ok free(output.data);

            _, ok2, error2 := inflate(stream, container, data.count - 1);

            check(!ok2 && error2 == "output exceeds the limit", tprint("limit one below the size (% level %): %", container, level, error2));

            _, ok3, error3 := inflate(stream, container, 0);

            check(!ok3 && error3 == "output exceeds the limit", tprint("limit 0 (% level %)", container, level));

            _, ok4 := inflate(stream, container, -1);

            check(!ok4, "negative limit");

            free(stream.data);
        }
    }

    // A small stream that expands a great deal stops at the limit.
    zeros := NewArray(3_000_000, u8);
    bomb := deflate(zeros, .ZLIB, 1);

    check(bomb.count < 10000, "zeros compress well");

    _, ok5, error5 := inflate(bomb, .ZLIB, 100_000);

    check(!ok5 && error5 == "output exceeds the limit", "expansion stops at max_output");

    output, ok6 := inflate(bomb, .ZLIB);

    check(ok6 && output.count == zeros.count, "the default limit allows it");
    if ok6 free(output.data);

    small: [] u8;
    small.data = alloc(1000);
    small.count = 1000;

    _, ok7, error7 := inflate_into(bomb, small, .ZLIB);

    check(!ok7 && error7 == "output is too small", "inflate_into stops at its buffer");
}

main :: () {
    inflate_into_sizes();
    limit_checks();

    assert(failures == 0);
    print("ok\n");
}
