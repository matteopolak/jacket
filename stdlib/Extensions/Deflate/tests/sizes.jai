// Round trips through every container and level for empty, one-byte, repetitive, 70 KB text and
// 70 KB incompressible inputs. Repetitive input must shrink a lot; incompressible input must not
// grow past the stored-block overhead.
#import "Basic";
D :: #import "Extensions/Deflate";

failures := 0;

expect :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

same_bytes :: (a: [] u8, b: [] u8) -> bool {
    if a.count != b.count return false;
    for 0..a.count - 1 if a[it] != b[it] return false;
    return true;
}

round_trip :: (name: string, data: [] u8) {
    for container: D.Deflate_Container.[.RAW, .ZLIB, .GZIP] {
        for level: s32.[0, 1, 6, 9] {
            packed := D.deflate(data, container, level);
            output, ok, error := D.inflate(packed, container);

            expect(ok, tprint("% % level %: inflate failed: %", name, container, level, error));
            expect(same_bytes(output, data), tprint("% % level %: output differs", name, container, level));
        }
    }
}

// A byte buffer holding `text` repeated until it is `size` bytes long.
repeat_text :: (text: string, size: int) -> [] u8 {
    result := alloc(size);
    bytes := cast(*u8) result;
    for i: 0..size - 1 bytes[i] = text[i % text.count];

    return .{ size, bytes };
}

// Pseudo-random bytes from a fixed xorshift stream, so the input is the same on every run.
noise :: (size: int) -> [] u8 {
    bytes := cast(*u8) alloc(size);
    state: u64 = 0x9E3779B97F4A7C15;

    for i: 0..size - 1 {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        bytes[i] = cast,trunc(u8) (state >> 24);
    }

    return .{ size, bytes };
}

main :: () {
    round_trip("empty", .[]);
    round_trip("one byte", cast([] u8) "x");
    round_trip("repetitive 4096 bytes", repeat_text("a", 4096));

    text := repeat_text("the quick brown fox jumps over the lazy dog\n", 70 * 1024);

    round_trip("70 KB text", text);

    // The text repeats every 44 bytes, so LZ77 matches cover almost all of it.
    packed := D.deflate(text, .ZLIB, 6);

    expect(packed.count < text.count / 20, tprint("70 KB text compressed to % bytes", packed.count));

    random := noise(70 * 1024);

    round_trip("70 KB noise", random);

    // Stored blocks add 5 bytes per 64 KB block plus the container framing.
    stored := D.deflate(random, .RAW, 6);

    expect(stored.count <= random.count + random.count / 100 + 64, tprint("noise grew to % bytes", stored.count));

    if failures > 0 {
        print("% failures\n", failures);
        exit(1);
    }

    print("ok\n");
}
