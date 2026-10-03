// Separately adapted from permitted Molly Rocket native C (zlib license).
// See third_party/meow_hash for the full unmodified notice and provenance.
MeowU64From :: (Hash: reg, $INDEX: int) -> u64 #expand {
    #assert INDEX >= 0 && INDEX < 2;
    result: u64;
    #asm { pextrq result, Hash, INDEX; }
    return result;
}

MeowU32From :: (Hash: reg, $INDEX: int) -> u32 #expand {
    #assert INDEX >= 0 && INDEX < 4;
    result: u32;
    #asm { pextrd result, Hash, INDEX; }
    return result;
}

MEOW_MIX_REG :: (r1: reg, r2: reg, r3: reg, r4: reg, r5: reg, i1: reg, i2: reg, i3: reg, i4: reg) #expand {
    #asm {
        aesdec r1, r2;
        paddq r3, i1;
        pxor r2, i2;
        aesdec r2, r4;
        paddq r5, i3;
        pxor r4, i4;
    }
}

MEOW_SHUFFLE :: (r1: reg, r2: reg, r3: reg, r4: reg, r5: reg, r6: reg) #expand {
    #asm {
        aesdec r1, r4;
        paddq r2, r5;
        pxor r4, r6;
        aesdec r4, r2;
        paddq r5, r6;
        pxor r2, r3;
    }
}

MEOW_MIX :: (r1: reg, r2: reg, r3: reg, r4: reg, r5: reg, ptr: *void, $OFFSET: s32) #expand {
    source := cast(*u8) ptr + OFFSET;

    #asm {
        movdqu.x a:, [source + 15];
        movdqu.x b:, [source];
        movdqu.x c:, [source + 1];
        movdqu.x d:, [source + 16];
    }

    MEOW_MIX_REG(r1, r2, r3, r4, r5, a, b, c, d);
}

MeowBegin :: (Seed: [128] u8, h0: reg, h1: reg, h2: reg, h3: reg, h4: reg, h5: reg, h6: reg, h7: reg) #expand {
    source := Seed.data;

    #asm {
        movdqu.x h0, [source];
        movdqu.x h1, [source + 16];
        movdqu.x h2, [source + 32];
        movdqu.x h3, [source + 48];
        movdqu.x h4, [source + 64];
        movdqu.x h5, [source + 80];
        movdqu.x h6, [source + 96];
        movdqu.x h7, [source + 112];
    }
}

MeowAbsorbBlocks :: (BlockCount: u64, Source: *void, h0: reg, h1: reg, h2: reg, h3: reg, h4: reg, h5: reg, h6: reg, h7: reg) #expand {
    B.assert(BlockCount <= 0x7fffffffffffffff / 256 && (BlockCount == 0 || Source != null));
    remaining := BlockCount;
    source := cast(*u8) Source;

    while remaining > 0 {
        MEOW_MIX(h0, h4, h6, h1, h2, source, 0x00);
        MEOW_MIX(h1, h5, h7, h2, h3, source, 0x20);
        MEOW_MIX(h2, h6, h0, h3, h4, source, 0x40);
        MEOW_MIX(h3, h7, h1, h4, h5, source, 0x60);
        MEOW_MIX(h4, h0, h2, h5, h6, source, 0x80);
        MEOW_MIX(h5, h1, h3, h6, h7, source, 0xa0);
        MEOW_MIX(h6, h2, h4, h7, h0, source, 0xc0);
        MEOW_MIX(h7, h3, h5, h0, h1, source, 0xe0);
        source += 256;
        remaining -= 1;
    }
}

MEOW_DUMP_STATE :: (T: string, r0: reg, r1: reg, r2: reg, r3: reg, r4: reg, r5: reg, r6: reg, r7: reg) #expand {
    B.print("%\n", T);
    B.print("0: % %\n", MeowU64From(r0, 0), MeowU64From(r0, 1));
    B.print("1: % %\n", MeowU64From(r1, 0), MeowU64From(r1, 1));
    B.print("2: % %\n", MeowU64From(r2, 0), MeowU64From(r2, 1));
    B.print("3: % %\n", MeowU64From(r3, 0), MeowU64From(r3, 1));
    B.print("4: % %\n", MeowU64From(r4, 0), MeowU64From(r4, 1));
    B.print("5: % %\n", MeowU64From(r5, 0), MeowU64From(r5, 1));
    B.print("6: % %\n", MeowU64From(r6, 0), MeowU64From(r6, 1));
    B.print("7: % %\n", MeowU64From(r7, 0), MeowU64From(r7, 1));
}

#scope_file
B :: #import "Basic";
