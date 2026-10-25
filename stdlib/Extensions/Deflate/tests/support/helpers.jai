// Shared by the Deflate tests: checks, a tracking allocator, inputs and a bit-level stream builder.

#import "Basic";
#import "Extensions/Deflate";

failures := 0;

check :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

Tally :: struct {
    outstanding: int;
}

tally_proc :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, data: *void) -> *void {
    tally := cast(*Tally) data;
    backing := context.default_allocator;
    result := backing.proc(mode, size, old_size, old_memory, backing.data);

    if mode == .ALLOCATE && result {
        tally.outstanding += 1;
    } else if mode == .RESIZE && result && !old_memory {
        tally.outstanding += 1;
    } else if mode == .FREE && old_memory {
        tally.outstanding -= 1;
    }

    return result;
}

bytes_of :: (text: string) -> [] u8 {
    result: [] u8;
    result.data = text.data;
    result.count = text.count;

    return result;
}

same_bytes :: (a: [] u8, b: [] u8) -> bool {
    if a.count != b.count return false;

    for 0..a.count - 1 if a[it] != b[it] return false;

    return true;
}

// A small deterministic generator, so every mode sees the same inputs.
Lcg :: struct {
    state: u64 = 12345;
}

next :: (lcg: *Lcg) -> u32 {
    lcg.state = lcg.state * 6364136223846793005 + 1442695040888963407;

    return cast(u32) (lcg.state >> 33);
}

random_bytes :: (count: int, seed: u64) -> [] u8 {
    lcg := Lcg.{ state = seed };
    result := NewArray(count, u8);

    for 0..count - 1 result[it] = cast,trunc(u8) next(*lcg);

    return result;
}

// Words from a small vocabulary: matches at short and long distances.
text_bytes :: (count: int, seed: u64) -> [] u8 {
    words := string.["alpha ", "beta ", "gamma ", "delta ", "the ", "of ", "stream ", "window ", "\n", "huffman ", "block "];
    lcg := Lcg.{ state = seed };
    result := NewArray(count, u8);
    at := 0;

    while at < count {
        word := words[next(*lcg) % cast(u32) words.count];

        for word {
            if at >= count break;
            result[at] = it;
            at += 1;
        }
    }

    return result;
}

// Rows of a gradient with a little noise, like filtered image data.
image_bytes :: (count: int, seed: u64) -> [] u8 {
    lcg := Lcg.{ state = seed };
    result := NewArray(count, u8);

    for 0..count - 1 {
        result[it] = cast,trunc(u8) ((it % 300) / 3 + (next(*lcg) % 4));
    }

    return result;
}

run_bytes :: (count: int) -> [] u8 {
    result := NewArray(count, u8);

    for 0..count - 1 {
        result[it] = cast,trunc(u8) (it / 700);
    }

    return result;
}

all_byte_values :: () -> [] u8 {
    result := NewArray(256 * 3, u8);

    for *result it.* = cast,trunc(u8) it_index;

    return result;
}

// Writes bits least significant first, as DEFLATE does; Huffman codes go most significant first.
Bit_Builder :: struct {
    bytes: [..] u8;
    accumulator: u32;
    count: int;
}

put :: (b: *Bit_Builder, value: u32, bits: int) {
    for 0..bits - 1 {
        b.accumulator |= ((value >> cast(u32) it) & 1) << cast(u32) b.count;
        b.count += 1;

        if b.count == 8 {
            array_add(*b.bytes, cast,trunc(u8) b.accumulator);
            b.accumulator = 0;
            b.count = 0;
        }
    }
}

put_code :: (b: *Bit_Builder, code: u32, length: int) {
    for 0..length - 1 {
        put(b, (code >> cast(u32) (length - 1 - it)) & 1, 1);
    }
}

finish :: (b: *Bit_Builder) -> [] u8 {
    if b.count > 0 put(b, 0, 8 - b.count);

    return b.bytes;
}

// Canonical code values for the lengths.
canonical_codes :: (lengths: [] u8) -> [] u32 {
    codes := NewArray(lengths.count, u32);
    counts: [16] u32;

    for lengths counts[it] += 1;

    counts[0] = 0;

    next_code: [16] u32;
    code: u32;

    for 1..15 {
        code = (code + counts[it - 1]) << 1;
        next_code[it] = code;
    }

    for lengths {
        if it == 0 continue;
        codes[it_index] = next_code[it];
        next_code[it] += 1;
    }

    return codes;
}

// A dynamic block header for the given code lengths. Each length is sent as its own symbol of a
// code length code in which symbols 0..15 all take 4 bits.
put_dynamic_header :: (b: *Bit_Builder, final: bool, literal_lengths: [] u8, distance_lengths: [] u8) {
    put(b, ifx final then 1 else 0, 1);
    put(b, 2, 2);
    put(b, cast(u32) (literal_lengths.count - 257), 5);
    put(b, cast(u32) (distance_lengths.count - 1), 5);
    put(b, 15, 4); // 19 code length code lengths

    order := u8.[16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];

    for order {
        put(b, ifx it < 16 then 4 else 0, 3);
    }

    for literal_lengths put_code(b, it, 4);
    for distance_lengths put_code(b, it, 4);
}

// A dynamic block header with raw symbols of the code length code, for malformed headers.
put_header_start :: (b: *Bit_Builder, literal_count: int, distance_count: int, code_length_count: int) {
    put(b, 1, 1);
    put(b, 2, 2);
    put(b, cast(u32) (literal_count - 257), 5);
    put(b, cast(u32) (distance_count - 1), 5);
    put(b, cast(u32) (code_length_count - 4), 4);
}

hex :: (text: string) -> [] u8 {
    result: [..] u8;
    high := -1;

    for text {
        c := it;
        value := -1;

        if c >= #char "0" && c <= #char "9" value = c - #char "0";
        else if c >= #char "a" && c <= #char "f" value = c - #char "a" + 10;

        if value < 0 continue;

        if high < 0 {
            high = value;
        } else {
            array_add(*result, cast(u8) (high * 16 + value));
            high = -1;
        }
    }

    return result;
}

// Runs inflate under the tracking allocator and returns whether it succeeded; whatever it
// returned is freed inside the tracked scope.
inflate_tracked :: (data: [] u8, container: Deflate_Container) -> ok: bool, outstanding: int, error: string {
    tally: Tally;
    ok: bool;
    error: string;

    {
        push_allocator(tally_proc, *tally);

        output: [] u8;
        output, ok, error = inflate(data, container);

        if ok free(output.data);
    }

    return ok, tally.outstanding, error;
}
