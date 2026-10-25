// Throughput of Deflate in MB/s of uncompressed data, against stb_image's zlib decoder and
// stb_image_write's zlib compressor. Native only (stb is linked C).
//
//     jaic -O2 stdlib/Extensions/Deflate/bench/bench.jai -exe /tmp/deflate-bench
//     /tmp/deflate-bench
//
// Four generated inputs (text, random bytes, image rows, one highly repetitive) are compressed
// and decompressed; ratios are compressed size over input size, so smaller is better. The last
// columns of each table are speed relative to stb (above 1.00 is faster).

#import "Basic";
#import "Random";
#import "Extensions/Deflate";
#import "stb_image";
#import "stb_image_write";

// The stb functions allocate with the C library.
libc :: #system_library "libc";
c_free :: (memory: *void) #foreign libc "free";

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

seconds :: () -> float64 {
    return to_float64_seconds(current_time_monotonic());
}

// Best of several runs, in seconds.
MIN_RUNS :: 3;
MAX_RUNS :: 50;
MIN_SECONDS :: 0.7;

measure :: (work: () -> void) -> float64 {
    best: float64 = 1.0e30;
    started := seconds();
    runs := 0;

    while runs < MIN_RUNS || (seconds() - started < MIN_SECONDS && runs < MAX_RUNS) {
        begin := seconds();
        work();

        best = min(best, seconds() - begin);
        runs += 1;
    }

    return best;
}

number :: (value: float64, digits: int) -> string {
    return tprint("%", FormatFloat.{ value = value, trailing_width = digits });
}

input: [] u8;
packed: [] u8;
level: s32;
scratch: [] u8;
sink: u64;

stb_decode_work :: () {
    length: s32;
    out := stbi_zlib_decode_malloc(packed.data, cast(s32) packed.count, *length);
    sink += out[length / 2];
    c_free(out);
}

inflate_work :: () {
    out, _, _ := inflate(packed);
    sink += out[out.count / 2];
    free(out.data);
}

inflate_into_work :: () {
    written, _, _ := inflate_into(packed, scratch);
    sink += scratch[written / 2];
}

deflate_work :: () {
    out := deflate(input, .ZLIB, level);
    sink += out[out.count / 2];
    free(out.data);
}

stb_compress_work :: () {
    length: s32;
    out := stbi_zlib_compress(input.data, cast(s32) input.count, *length, 8);
    sink += out[length / 2];
    c_free(out);
}

crc_work :: () {
    sink += crc32(input);
}

adler_work :: () {
    sink += adler32(input);
}

run_set :: (name: string, data: [] u8) {
    input = data;
    mb := cast(float64) data.count / 1.0e6;

    print("\n%  (% MB)\n", name, number(mb, 1));

    // Decompression of the same stream by both decoders.
    packed = deflate(input, .ZLIB, 6);
    scratch.data = alloc(data.count);
    scratch.count = data.count;

    ours := measure(inflate_work);
    into := measure(inflate_into_work);
    theirs := measure(stb_decode_work);

    print("  inflate        % MB/s   inflate_into % MB/s   stb % MB/s   speed vs stb x% (into x%)\n", number(mb / ours, 0), number(mb / into, 0), number(mb / theirs, 0), number(theirs / ours, 2), number(theirs / into, 2));

    free(packed.data);
    free(scratch.data);

    // Compression.
    length: s32;
    stb_out := stbi_zlib_compress(input.data, cast(s32) input.count, *length, 8);
    stb_size := cast(float64) length;

    c_free(stb_out);

    stb_time := measure(stb_compress_work);

    print("  stb compress   % MB/s   ratio %\n", number(mb / stb_time, 0), number(stb_size / data.count, 4));

    for l: s32.[1, 6, 9] {
        level = l;
        out := deflate(input, .ZLIB, l);
        size := cast(float64) out.count;

        free(out.data);

        time := measure(deflate_work);

        print("  deflate level %  % MB/s   ratio %   speed vs stb x%   size vs stb x%\n", l, number(mb / time, 0), number(size / data.count, 4), number(stb_time / time, 2), number(size / stb_size, 3));
    }

    print("  crc32 % MB/s   adler32 % MB/s\n", number(mb / measure(crc_work), 0), number(mb / measure(adler_work), 0));
}

main :: () {
    random_seed(7);

    run_set("text", make_text(6_000_000));
    run_set("random bytes", make_random(6_000_000));
    run_set("image rows", make_image(6_000_000));
    run_set("repetitive", make_repetitive(16_000_000));

    if sink == 1 print("\n");
}
