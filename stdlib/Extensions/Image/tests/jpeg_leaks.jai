// The JPEG decoder frees everything it allocates, on success and on every kind of failure. A
// tracking allocator counts the blocks still outstanding after each decode (the returned image
// freed) of every fixture, of each cut off at many offsets, and with bytes replaced at random.
//
//     jaic stdlib/Extensions/Image/tests/jpeg_leaks.jai -interp

#import "Basic";
#import "String";
#import "File";
#import "Extensions/Image";

#load "fixtures/jpeg/fixtures.jai";

failures := 0;
tracked := 0;

check :: (condition: bool, what: string) {
    if condition return;
    print("FAIL: %\n", what);
    failures += 1;
}

Tally :: struct {
    outstanding: int;
    allocations: int;
}

tally_proc :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, data: *void) -> *void {
    tally := cast(*Tally) data;
    backing := context.default_allocator;
    result := backing.proc(mode, size, old_size, old_memory, backing.data);

    if mode == .ALLOCATE && result {
        tally.outstanding += 1;
        tally.allocations += 1;
    } else if mode == .RESIZE && result && !old_memory {
        tally.outstanding += 1;
        tally.allocations += 1;
    } else if mode == .FREE && old_memory {
        tally.outstanding -= 1;
    }

    return result;
}

// Nothing is formatted inside the tracked scope: tprint's temporary storage would be counted.
decode_and_release :: (bytes: [] u8, label: string) {
    tally: Tally;
    ok: bool;
    pixels: int;

    {
        push_allocator(tally_proc, *tally);

        image, decoded, _ := decode_image(bytes);

        ok = decoded;
        pixels = image.pixels.count;
        if decoded deinit(*image);
    }

    tracked += tally.allocations;
    check(tally.outstanding == 0, tprint("% leaves % blocks allocated", label, tally.outstanding));
    check(ok == (pixels > 0), tprint("% returns pixels exactly when it succeeds", label));
}

random_state: u32 = 88172645;

random :: () -> u32 {
    random_state ^= random_state << 13;
    random_state ^= random_state >> 17;
    random_state ^= random_state << 5;
    return random_state;
}

main :: () {
    for fixture: jpeg_fixtures {
        bytes := jpeg_bytes(fixture.jpeg);

        decode_and_release(bytes, fixture.name);

        step := max(bytes.count / 40, 1);

        for length: 0..bytes.count - 1 {
            if length % step != 0 && length > 30 continue;
            cut := bytes;
            cut.count = length;
            decode_and_release(cut, tprint("% cut to % bytes", fixture.name, length));
        }

        copy := NewArray(bytes.count, u8);

        for 1..12 {
            memcpy(copy.data, bytes.data, bytes.count);
            for 1..3 copy[random() % cast(u32) bytes.count] = cast(u8) (random() & 255);
            decode_and_release(copy, tprint("% corrupted (round %)", fixture.name, it));
        }

        array_free(copy);
    }

    check(tracked > 1000, "the tracking allocator saw the decoder's allocations");

    if failures {
        print("% failures\n", failures);
        exit(1);
    }

    print("ok\n");
}
