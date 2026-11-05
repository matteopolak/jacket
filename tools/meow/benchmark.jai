// Deterministic padded inputs let the historical implementation run safely.
// Each output row contains a case identity, elapsed seconds, and both digest sums.
#import "Basic";
M :: #import "meow_hash";

black_box :: (value: $T) -> T #intrinsic;

// Hash an input in chunks, writing the digest to the caller's register.
stream_hash :: (seed: [128] u8, source: *u8, length: s64, chunk: s64, hash: M.reg) #expand {
    state: M.meow_state;

    M.MeowBegin(*state, seed);
    pos: s64;
    while pos < length {
        amount := min(chunk, length - pos);
        M.MeowAbsorb(*state, xx amount, source + pos);
        pos += amount;
    }

    M.MeowEnd(*state, hash);
}

main :: () {
    #if CPU != .X64 {
        print("requires x64 vector-assembly support\n");
        exit(1);
    }

    args := get_command_line_arguments();
    full := args.count > 1 && args[1] == "--full";
    smoke := args.count > 1 && args[1] == "--smoke";
    lengths := s64.[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 127, 128, 129, 255, 256, 257, 1024, 1048576];
    capacity := 16 * 1024 * 1024;
    data, base := NewArray(capacity + 64, u8, alignment = 4096);

    defer free(base);
    for i: 0..capacity + 63 data[i] = cast,trunc(u8) (i * 73 + 19);

    custom: [128] u8;
    for i: 0..127 custom[i] = cast,trunc(u8) (i * 17 + 11);

    for length: lengths {
        if smoke && !(length == 0 || length == 1 || length == 15 || length == 16 || length == 31 || length == 32 || length == 255 || length == 256 || length == 257) continue;
        if !full && !(length == 0 || length == 1 || length == 15 || length == 16 || length == 31 || length == 32 || length == 63 || length == 64 || length == 127 || length == 128 || length == 129 || length == 255 || length == 256 || length == 257 || length == 1024 || length == 1048576) continue;

        iterations := 5000;
        if length >= 1024 iterations = 2000;
        if length >= 1048576 iterations = 4;

        for offset: 0..15 {
            if smoke && !(offset == 0 || offset == 15) continue;
            if !full && !(offset == 0 || offset == 1 || offset == 7 || offset == 15) continue;

            for seed_id: 0..1 {
                seed := M.MeowDefaultSeed;
                if seed_id == 1 seed = custom;

                // warm, disjoint, edit, streaming in 31-byte and 256-byte chunks
                for mode: 0..4 {
                    sum0: u64;
                    sum1: u64;
                    stride := ((length + 63) / 64) * 64 + 64;
                    slots := (capacity - length - 64) / stride;
                    start := current_time_monotonic();

                    for i: 0..iterations - 1 {
                        slot := 0;
                        if mode == 1 slot = i % slots;

                        source := data.data + 32 + offset + slot * stride;
                        if mode == 2 && length > 0 source[i % length] ^= 1;

                        // Hide both pointers every iteration, including zero-length inputs.
                        // The barriers add loop overhead but do not alter the bytes hashed.
                        source = black_box(source);
                        iteration_seed := black_box(*seed);

                        #asm { hash: vec; }

                        if mode < 3 {
                            M.MeowHash(iteration_seed.*, xx length, source, hash);
                        } else {
                            chunk := 31;
                            if mode == 4 chunk = 256;
                            stream_hash(iteration_seed.*, source, length, chunk, hash);
                        }

                        sum0 += M.MeowU64From(hash, 0);
                        sum1 += M.MeowU64From(hash, 1);
                    }

                    elapsed, _ := to_float64_seconds(current_time_monotonic() - start);

                    print("% % % % % % % %\n", length, offset, seed_id, mode, iterations, elapsed, sum0, sum1);
                }
            }
        }
    }
}
