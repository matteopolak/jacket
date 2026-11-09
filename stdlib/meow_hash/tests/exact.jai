#import "Basic";
M :: #import "meow_hash";
P :: #import "Extensions/Meow_Portable";

#if CPU == .X64 {
    libc :: #system_library "libc"; @native(external)
    c_malloc :: (size: u64) -> *void #foreign libc "malloc";
    c_free :: (memory: *void) #foreign libc "free"; @FreesArgument

    // Basic's allocator may reserve alignment slack; malloc makes ASan see the
    // requested end for every size, including large and unaligned inputs.
    exact_alloc :: (size: int) -> *u8 {
        if size == 0 return null;
        result := cast(*u8) c_malloc(cast(u64) size);
        assert(result != null);
        return result;
    }

    exact_free :: (memory: *void) {
        c_free(memory);
    }

    // Compare independent scalar output with every native entry point. The direct
    // finalizer receives its own allocation, so it cannot borrow earlier blocks.
    check :: (source: *u8, count: int, seed: [128] u8) {
        expected := P.hash_bytes(([] u8).{ count, source }, seed);

        #asm { pxor hash:, hash; }
        M.MeowHash(seed, cast(u64) count, source, hash);
        assert(M.MeowU64From(hash, 0) == expected[0] && M.MeowU64From(hash, 1) == expected[1]);
        tail := count & 255;
        residual := exact_alloc(tail);

        defer exact_free(residual);
        if tail != 0 memcpy(residual, source + (count & ~255), tail);

        #asm {
            pxor h0:, h0; pxor h1:, h1; pxor h2:, h2; pxor h3:, h3;
            pxor h4:, h4; pxor h5:, h5; pxor h6:, h6; pxor h7:, h7;
            pxor r1:, r1; pxor r3:, r3;
        }

        M.MeowBegin(seed, h0, h1, h2, h3, h4, h5, h6, h7);
        M.MeowAbsorbBlocks(cast(u64) (count >> 8), source, h0, h1, h2, h3, h4, h5, h6, h7);
        M.MeowEnd(cast(u64) count, residual, r1, r3, h0, h1, h2, h3, h4, h5, h6, h7, null);
        assert(M.MeowU64From(h0, 0) == expected[0] && M.MeowU64From(h0, 1) == expected[1]);

        state: M.meow_state;

        for stride: int.[1, 17, 255, 256, 257] {
            memset(state.Buffer.data, 0xA5, 256);
            M.MeowBegin(*state, seed);
            at := 0;
            while at < count {
                n := min(stride, count - at);
                chunk := exact_alloc(n);

                memcpy(chunk, source + at, n);
                M.MeowAbsorb(*state, cast(u64) n, chunk);
                exact_free(chunk);
                M.MeowAbsorb(*state, 0, null);
                at += n;
            }

            assert(state.BufferLen == tail && state.TotalLengthInBytes == cast(u64) count);

            for 0..1 {
                M.MeowEnd(*state, hash);
                assert(M.MeowU64From(hash, 0) == expected[0] && M.MeowU64From(hash, 1) == expected[1]);
            }
        }

        if count <= 64 for split: 0..count {
            M.MeowBegin(*state, seed);
            M.MeowAbsorb(*state, cast(u64) split, source);
            M.MeowAbsorb(*state, cast(u64) (count - split), source + split);
            M.MeowEnd(*state, hash);
            assert(M.MeowU64From(hash, 0) == expected[0] && M.MeowU64From(hash, 1) == expected[1]);
        }
    }

    // A mutable shuffle selecting outside the logical input sees zero, even
    // when the mask would expose it. Portable is an oracle only after restore.
    bounded_mutations :: () {
        source := exact_alloc(1);

        defer exact_free(source);
        source[0] = 42;
        saved := M.MeowShiftAdjust[0];

        M.MeowShiftAdjust[0] = 15;
        #asm { pxor hash:, hash; }
        M.MeowHash(M.MeowDefaultSeed, 1, source, hash);
        low := M.MeowU64From(hash, 0);
        high := M.MeowU64From(hash, 1);

        M.MeowShiftAdjust[0] = saved;
        zero: [1] u8;
        expected := P.hash_bytes(zero);

        assert(low == expected[0] && high == expected[1]);
    }

    #if OS == .LINUX || OS == .MACOS {
        getpagesize :: () -> s32 #foreign libc;
        mmap :: (address: *void, length: u64, protection: s32, flags: s32, fd: s32, offset: s64) -> *void #foreign libc;
        mprotect :: (address: *void, length: u64, protection: s32) -> s32 #foreign libc;
        munmap :: (address: *void, length: u64) -> s32 #foreign libc; @MaybeFreesArgument

        guards :: () {
            page := cast(int) getpagesize();

            #if OS == .MACOS flags: s32 = 0x1002;
            else flags: s32 = 0x22;
            base := cast(*u8) mmap(null, cast(u64) (page * 3), 3, flags, -1, 0);

            assert(base != cast(*u8) -1);
            defer munmap(base, cast(u64) (page * 3));
            assert(mprotect(base, cast(u64) page, 0) == 0);
            assert(mprotect(base + page * 2, cast(u64) page, 0) == 0);

            for count: 1..513 {
                for side: 0..1 {
                    source := base + page;
                    if side == 1 source = base + page * 2 - count;
                    for 0..count - 1 source[it] = cast,trunc(u8) (it * 73 + count);
                    check(source, count, M.MeowDefaultSeed);
                }
            }
        }
    }
}

main :: () {
    #if CPU == .X64 {
        bounded_mutations();
        check(null, 0, M.MeowDefaultSeed);

        for count: 1..513 {
            for offset: 0..15 {
                allocation := exact_alloc(count + offset);

                defer exact_free(allocation);
                source := allocation + offset;
                for 0..count - 1 source[it] = cast,trunc(u8) (it * 37 + count);

                seed := M.MeowDefaultSeed;
                if (offset & 1) != 0 seed[0] ^= 0x5A;

                check(source, count, seed);
            }
        }

        for count: int.[1023, 1024, 1025, 4095, 4096, 4097, 8193] {
            source := exact_alloc(count);
            defer exact_free(source);
            for 0..count - 1 source[it] = cast,trunc(u8) (it * 73 + count);
            check(source, count, M.MeowDefaultSeed);
        }

        stack: [1] u8;
        stack[0] = 42;
        check(stack.data, 1, M.MeowDefaultSeed);
        #if OS == .LINUX || OS == .MACOS guards();
    }

    print("ok\n");
}
