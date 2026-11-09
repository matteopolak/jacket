// Run with jacket bench --path stdlib/Extensions/Deflate.
#import "Extensions/Testing";
#import "Basic";
#import "Random";
#import "Extensions/Deflate";
#import "stb_image";
#import "stb_image_write";

// stb_image frees its own buffers. stb_image_write exports no free, and on Windows its static C
// runtime keeps a heap of its own, so there its compressed streams are left allocated.
libc :: #system_library "libc"; @native(external)
c_free :: (memory: *void) #foreign libc "free"; @FreesArgument

free_compressed :: (memory: *void) {
    #if OS != .WINDOWS c_free(memory);
}

WORDS :: string.["the", "of", "and", "compiler", "stream", "window", "literal", "length", "distance", "huffman", "block", "match", "symbol", "table", "inflate", "deflate", "a", "to", "in", "is", "bytes", "output", "Jai", "code"];

pick :: (count: int) -> int {
    return cast(int) (random_get() % cast(u64) count);
}

make_text :: (size: int) -> [] u8 {
    data: [] u8;
    data.data = alloc(size);
    data.count = size;

    at := 0;

    while at < size {
        word := WORDS[pick(WORDS.count)];

        for word {
            if at >= size break;
            data[at] = it;
            at += 1;
        }

        if at < size {
            data[at] = ifx pick(12) == 0 then #char "\n" else #char " ";
            at += 1;
        }
    }

    return data;
}

make_random :: (size: int) -> [] u8 {
    data: [] u8;
    data.data = alloc(size);
    data.count = size;

    for 0..size - 1 data[it] = cast,trunc(u8) random_get();

    return data;
}

// RGB rows of a smooth gradient with a little noise, each prefixed with a Sub filter byte as in PNG.
make_image :: (size: int) -> [] u8 {
    width := 1024;
    stride := 1 + width * 3;
    rows := size / stride;

    data: [] u8;
    data.count = rows * stride;
    data.data = alloc(data.count);

    for y: 0..rows - 1 {
        row := y * stride;
        data[row] = 1;

        previous: [3] u8;

        for x: 0..width - 1 {
            for c: 0..2 {
                value := cast,trunc(u8) ((x * (c + 2)) / 8 + y / 3 + pick(5));
                data[row + 1 + x * 3 + c] = value - previous[c];
                previous[c] = value;
            }
        }
    }

    return data;
}

make_repetitive :: (size: int) -> [] u8 {
    data: [] u8;
    data.data = alloc(size);
    data.count = size;

    for 0..size - 1 {
        data[it] = cast,trunc(u8) ((it % 1000) * 7);
        if pick(500) == 0 data[it] = 0;
    }

    return data;
}

inflate_case :: (b: *Bench, input: [] u8) {
    packed := deflate(input, .ZLIB, 6);

    defer free(packed.data);
    set_bytes(b, input.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        out, _, error := inflate(black_box(packed));
        assert(!error.count, "%", error);
        black_box(out);
        free(out.data);
    }
}

inflate_into_case :: (b: *Bench, input: [] u8) {
    packed := deflate(input, .ZLIB, 6);

    defer free(packed.data);
    scratch := NewArray(input.count, u8);

    defer array_free(scratch);
    set_bytes(b, input.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        written, _, error := inflate_into(black_box(packed), scratch);
        assert(!error.count, "%", error);
        black_box(written);
        black_box(scratch);
    }
}

stb_decode_case :: (b: *Bench, input: [] u8) {
    packed := deflate(input, .ZLIB, 6);

    defer free(packed.data);
    set_bytes(b, input.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        length: s32;
        data := black_box(packed);
        out := stbi_zlib_decode_malloc(data.data, cast(s32) data.count, *length);

        assert(out != null);
        black_box(out[length / 2]);
        stbi_image_free(out);
    }
}

stb_compress_case :: (b: *Bench, input: [] u8) {
    set_bytes(b, input.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        length: s32;
        data := black_box(input);
        out := stbi_zlib_compress(data.data, cast(s32) data.count, *length, 8);

        assert(out != null);
        black_box(out[length / 2]);
        free_compressed(out);
    }
}

crc32_case :: (b: *Bench, input: [] u8) {
    set_bytes(b, input.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        black_box(crc32(black_box(input)));
    }
}

adler32_case :: (b: *Bench, input: [] u8) {
    set_bytes(b, input.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        black_box(adler32(black_box(input)));
    }
}

deflate_1_case :: (b: *Bench, input: [] u8) {
    set_bytes(b, input.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        out := deflate(black_box(input), .ZLIB, 1);
        black_box(out);
        free(out.data);
    }
}

deflate_6_case :: (b: *Bench, input: [] u8) {
    set_bytes(b, input.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        out := deflate(black_box(input), .ZLIB, 6);
        black_box(out);
        free(out.data);
    }
}

deflate_9_case :: (b: *Bench, input: [] u8) {
    set_bytes(b, input.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        out := deflate(black_box(input), .ZLIB, 9);
        black_box(out);
        free(out.data);
    }
}

// Keep every input/operation selectable while sharing the generated fixture setup.
deflate_benches :: () -> string {
    out: String_Builder;

    for input: string.["text", "random", "image", "repetitive"] {
        size := ifx input == "repetitive" then 16000000 else 6000000;

        for operation: string.["inflate", "inflate_into", "stb_decode", "stb_compress", "crc32", "adler32", "deflate_1", "deflate_6", "deflate_9"] {
            print(*out, "%_% :: (b: *Bench) { random_seed(7); input := make_%(%); defer free(input.data); %_case(b, input); } @bench\n", input, operation, input, size, operation);
        }
    }

    return builder_to_string(*out);
}

#insert #run deflate_benches();
