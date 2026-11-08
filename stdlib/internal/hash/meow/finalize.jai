// Public finalization operates on the trailing (less than 256-byte) input region and never reads
// outside it: the 16-byte tail window is a direct load when it lies inside the region, a load of
// the region's last 16 bytes shifted into place when it runs past the end, and a bounded copy only
// for regions under 16 bytes.
// Finishes a hash from the state registers and exactly `Len & 255` readable bytes at `Source`; the
// result is left in `h0`. `r1` and `r3` are scratch registers.
MeowEnd :: (Len: u64, Source: *void, r1: reg, r3: reg, h0: reg, h1: reg, h2: reg, h3: reg, h4: reg, h5: reg, h6: reg, h7: reg, Store128: *void) #expand {
    finish(Len, Source, r1, r3, h0, h1, h2, h3, h4, h5, h6, h7, Store128, false);
}

// Finishes a streamed hash and leaves the 128-bit result in `h0`. When `Store128` is set, the final
// 128 bytes of internal state are written there too.
MeowEnd :: (State: *meow_state, h0: reg, Store128: *void = null) #expand {
    B.assert(State != null && State.BufferLen == (State.TotalLengthInBytes & 255));
    registers := State.RegisterFile.data;

    #asm {
        movdqu.x h0, [registers];
        movdqu.x h1:, [registers + 16];
        movdqu.x h2:, [registers + 32];
        movdqu.x h3:, [registers + 48];
        movdqu.x h4:, [registers + 64];
        movdqu.x h5:, [registers + 80];
        movdqu.x h6:, [registers + 96];
        movdqu.x h7:, [registers + 112];
        pxor r1:, r1;
        pxor r3:, r3;
    }

    finish(State.TotalLengthInBytes, State.Buffer.data, r1, r3, h0, h1, h2, h3, h4, h5, h6, h7, Store128, true);
}

#scope_file
B :: #import "Basic";

// `pshufb` controls that move a loaded vector down by 0..15 bytes and zero the rest.
MeowTailShift: [32] u8 = u8.[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80];

finish :: (Len: u64, Source: *void, r1: reg, r3: reg, h0: reg, h1: reg, h2: reg, h3: reg, h4: reg, h5: reg, h6: reg, h7: reg, Store128: *void, $PADDED: bool) #expand {
    tail := Len & 255;

    B.assert(tail == 0 || Source != null);
    source := cast(*u8) Source;
    last := source + (tail & 0xf0);
    count := tail & 15;

    #asm {
        pxor r1, r1;
        pxor r3, r3;
        pxor a:, a;
        pxor c:, c;
        pxor l0:, l0;
        pxor l1:, l1;
        pxor l2:, l2;
        pxor l3:, l3;
    }

    if count != 0 {
        mask := MeowMaskLen.data + (16 - count);

        #asm { movdqu.x a, [mask]; }

        #if PADDED {
            #asm { movdqu.x r1, [last]; }
        } else {
            last_ok := ((cast(u64) source + tail - 1) | (MEOW_PAGESIZE - 1)) - 16;
            alignment: u64;

            if cast(u64) last > last_ok alignment = cast(u64) last & 15;

            adjustment := MeowShiftAdjust.data + alignment;
            // The shuffle window is the 16 bytes at `start`, zero outside the residual object:
            // neither the first block nor allocator or page padding belongs to it.
            start := cast(s64) (tail & 0xf0) - cast(s64) alignment;
            input: *u8;
            window: [16] u8;

            if start >= 0 && start + 16 <= cast(s64) tail {
                input = source + start;
            } else if start >= 0 && tail >= 16 {
                // The window runs past the end: load the last 16 bytes and shift them down.
                shift := start + 16 - cast(s64) tail;
                last16 := source + tail - 16;
                shuffle := MeowTailShift.data + shift;
                out := window.data;

                #asm {
                    movdqu.x w:, [last16];
                    movdqu.x s:, [shuffle];
                    pshufb w, s;
                    movdqu.x [out], w;
                }

                input = out;
            } else {
                // Fewer than 16 bytes are readable at all: copy them into the zeroed window.
                first := start;
                stop := start + 16;

                if first < 0 first = 0;
                if stop > cast(s64) tail stop = cast(s64) tail;

                if stop > first memcpy(window.data + (first - start), source + first, stop - first);

                input = window.data;
            }

            #asm {
                movdqu.x c, [adjustment];
                movdqu.x r1, [input];
                pshufb r1, c;
            }
        }

        #asm { pand r1, a; }
    }

    if (tail & 16) != 0 {
        before := last - 16;
        #asm { movdqu.x r3, r1; movdqu.x r1, [before]; }
    }

    #asm {
        movdqu.x a, r1;
        movdqu.x c, r1;
        palignr a, r3, 15;
        palignr c, r3, 1;
        movq l3, Len;
        palignr l0, l3, 15;
        palignr l2, l3, 1;
    }

    MEOW_MIX_REG(h0, h4, h6, h1, h2, a, r1, c, r3);
    MEOW_MIX_REG(h1, h5, h7, h2, h3, l0, l1, l2, l3);
    lanes := tail >> 5;
    if lanes > 0 MEOW_MIX(h2, h6, h0, h3, h4, source, 0x00);
    if lanes > 1 MEOW_MIX(h3, h7, h1, h4, h5, source, 0x20);
    if lanes > 2 MEOW_MIX(h4, h0, h2, h5, h6, source, 0x40);
    if lanes > 3 MEOW_MIX(h5, h1, h3, h6, h7, source, 0x60);
    if lanes > 4 MEOW_MIX(h6, h2, h4, h7, h0, source, 0x80);
    if lanes > 5 MEOW_MIX(h7, h3, h5, h0, h1, source, 0xa0);
    if lanes > 6 MEOW_MIX(h0, h4, h6, h1, h2, source, 0xc0);

    MEOW_SHUFFLE(h0, h1, h2, h4, h5, h6);
    MEOW_SHUFFLE(h1, h2, h3, h5, h6, h7);
    MEOW_SHUFFLE(h2, h3, h4, h6, h7, h0);
    MEOW_SHUFFLE(h3, h4, h5, h7, h0, h1);
    MEOW_SHUFFLE(h4, h5, h6, h0, h1, h2);
    MEOW_SHUFFLE(h5, h6, h7, h1, h2, h3);
    MEOW_SHUFFLE(h6, h7, h0, h2, h3, h4);
    MEOW_SHUFFLE(h7, h0, h1, h3, h4, h5);
    MEOW_SHUFFLE(h0, h1, h2, h4, h5, h6);
    MEOW_SHUFFLE(h1, h2, h3, h5, h6, h7);
    MEOW_SHUFFLE(h2, h3, h4, h6, h7, h0);
    MEOW_SHUFFLE(h3, h4, h5, h7, h0, h1);

    if Store128 {
        output := cast(*u8) Store128;

        #asm {
            movdqu.x [output], h0;
            movdqu.x [output + 16], h1;
            movdqu.x [output + 32], h2;
            movdqu.x [output + 48], h3;
            movdqu.x [output + 64], h4;
            movdqu.x [output + 80], h5;
            movdqu.x [output + 96], h6;
            movdqu.x [output + 112], h7;
        }
    }

    #asm {
        paddq h0, h2;
        paddq h1, h3;
        paddq h4, h6;
        paddq h5, h7;
        pxor h0, h1;
        pxor h4, h5;
        paddq h0, h4;
    }
}
