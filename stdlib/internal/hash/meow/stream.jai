MeowBegin :: (State: *meow_state, Seed: [128] u8) {
    B.assert(State != null);
    memcpy(State.RegisterFile.data, Seed.data, 128);
    State.TotalLengthInBytes = 0;
    State.BufferLen = 0;
}

MeowAbsorb :: (State: *meow_state, Len: u64, Source: *void) {
    B.assert(State != null && State.BufferLen < 256 && Len <= 0x7fffffffffffffff && (Len == 0 || Source != null));
    State.TotalLengthInBytes = add_length(State.TotalLengthInBytes, Len);
    source := cast(*u8) Source;
    remaining := Len;

    if State.BufferLen != 0 {
        fill := B.min(cast(u64) (256 - State.BufferLen), remaining);

        memcpy(State.Buffer.data + State.BufferLen, source, cast(s64) fill);
        State.BufferLen += cast(u32) fill;
        remaining -= fill;
        source += fill;

        if State.BufferLen == 256 {
            absorb_state_blocks(State, 1, State.Buffer.data);
            State.BufferLen = 0;
        } else return;
    }

    blocks := remaining >> 8;

    absorb_state_blocks(State, blocks, source);
    advance := blocks << 8;

    remaining -= advance;
    source += advance;
    if remaining > 0 memcpy(State.Buffer.data, source, cast(s64) remaining);

    State.BufferLen = cast(u32) remaining;
}

MeowHash :: (Seed: [128] u8, Len: u64, Source: *void, h0: reg) #expand {
    B.assert(Len <= 0x7fffffffffffffff && (Len == 0 || Source != null), "MeowHash invalid source/length");

    #asm {
        pxor h1:, h1;
        pxor h2:, h2;
        pxor h3:, h3;
        pxor h4:, h4;
        pxor h5:, h5;
        pxor h6:, h6;
        pxor h7:, h7;
        pxor r1:, r1;
        pxor r3:, r3;
    }

    MeowBegin(Seed, h0, h1, h2, h3, h4, h5, h6, h7);
    blocks := Len >> 8;

    MeowAbsorbBlocks(blocks, Source, h0, h1, h2, h3, h4, h5, h6, h7);
    tail := cast(*u8) Source + (blocks << 8);

    MeowEnd(Len, tail, r1, r3, h0, h1, h2, h3, h4, h5, h6, h7, null);
}

#scope_file
B :: #import "Basic";

add_length :: (a: u64, b: u64) -> u64 #no_aoc {
    return a + b;
}

absorb_state_blocks :: (State: *meow_state, count: u64, source: *void) {
    if count == 0 return;

    registers := State.RegisterFile.data;

    #asm {
        movdqu.x h0:, [registers];
        movdqu.x h1:, [registers + 16];
        movdqu.x h2:, [registers + 32];
        movdqu.x h3:, [registers + 48];
        movdqu.x h4:, [registers + 64];
        movdqu.x h5:, [registers + 80];
        movdqu.x h6:, [registers + 96];
        movdqu.x h7:, [registers + 112];
    }

    MeowAbsorbBlocks(count, source, h0, h1, h2, h3, h4, h5, h6, h7);

    #asm {
        movdqu.x [registers], h0;
        movdqu.x [registers + 16], h1;
        movdqu.x [registers + 32], h2;
        movdqu.x [registers + 48], h3;
        movdqu.x [registers + 64], h4;
        movdqu.x [registers + 80], h5;
        movdqu.x [registers + 96], h6;
        movdqu.x [registers + 112], h7;
    }
}
