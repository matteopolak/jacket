// Unicode normalization and text boundaries (UCD 17.0.0).
//
// - [normalize] and [is_normalized] for the four normalization forms.
// - [segments] and [next_segment] for grapheme cluster, word and sentence boundaries, with
//   [next_grapheme], [previous_grapheme] and [grapheme_count] for moving a cursor by characters.
//
// ```jai
// count, error := grapheme_count("e\xcc\x81te\xcc\x81");
// assert(error.kind == .NONE && count == 3);
//
// builder: String_Builder;
// normalize("e\xcc\x81te\xcc\x81", .NFC, *builder);
// text := builder_to_string(*builder);
// defer free(text);
// assert(text == "\xc3\xa9t\xc3\xa9");
// ```
//
// Offsets are bytes, and every procedure borrows its input. Nothing allocates except `normalize`,
// which appends to the caller's builder. Inputs and outputs are bounded by
// [Unicode_Text_Options]; error messages are static.

// A Unicode normalization form: canonical composition (`NFC`) or decomposition (`NFD`), or the
// compatibility variants (`NFKC`, `NFKD`).
Normalization_Form :: enum u8 {
    NFC;
    NFD;
    NFKC;
    NFKD;
}

// Why a text operation failed.
//
// - `INVALID_UTF8`: the input is not valid UTF-8.
// - `LIMIT`: an input or output limit from [Unicode_Text_Options] was exceeded.
// - `INVALID_ARGUMENT`: a bad form, kind, offset or negative limit.
// - `ALLOCATION`: the builder could not grow.
Unicode_Text_Error_Kind :: enum u8 {
    NONE;
    INVALID_UTF8;
    LIMIT;
    INVALID_ARGUMENT;
    ALLOCATION;
}

// The outcome of a text operation; `kind` is `.NONE` on success. `message` is static and `offset` is
// the byte offset of the problem.
Unicode_Text_Error :: struct {
    kind: Unicode_Text_Error_Kind;
    message: string;
    offset: int;
}

// Limits for normalization and segmentation. Exceeding one fails with `.LIMIT`; a negative one with
// `.INVALID_ARGUMENT`.
Unicode_Text_Options :: struct {
    max_size := 16 * 1024 * 1024; // input bytes
    max_output := 64 * 1024 * 1024; // output bytes
    stream_safe := false; // Insert CGJ before an NFKD nonstarter sequence would exceed 30.
}

// Appends `text` in normalization form `form` to `builder`. Invalid UTF-8, a limit error or a bad
// argument leaves the builder unchanged; see [is_normalized] to check without allocating.
//
// ```jai
// builder: String_Builder;
// assert(normalize("e\xcc\x81", .NFC, *builder).kind == .NONE);
// text := builder_to_string(*builder);
// defer free(text);
// assert(text == "\xc3\xa9");
// ```
normalize :: (text: string, form: Normalization_Form, builder: *String_Builder, options: Unicode_Text_Options = .{}) -> Unicode_Text_Error {
    error := arguments_error(text, form, options);
    if error.kind != .NONE return error;

    codes: [..] u32;

    defer array_free(codes);
    scratch: [..] u32;

    defer array_free(scratch);
    at := 0;
    nonstarters := 0;

    while at < text.count {
        cp, size, valid := decode(text, at);
        if !valid return .{ .INVALID_UTF8, "invalid UTF-8", at };

        if options.stream_safe {
            expanded: [3] u32;
            sequence := decomposition(cp, true, *expanded);
            leading := 0;
            trailing := 0;

            for sequence {
                if (property(it) & 255) == 0 break;
                leading += 1;
            }

            for sequence {
                if (property(it) & 255) == 0 trailing = 0;
                else trailing += 1;
            }

            if nonstarters + leading > 30 {
                if codes.count >= options.max_output return .{ .LIMIT, "output limit exceeded", at };
                array_add(*codes, 0x034F);
                nonstarters = 0;
            }

            if leading == sequence.count nonstarters += leading;
            else nonstarters = trailing;
        }

        hangul: [3] u32;
        sequence := decomposition(cp, form == .NFKC || form == .NFKD, *hangul);

        if sequence.count > options.max_output - codes.count return .{ .LIMIT, "output limit exceeded", at };
        for sequence array_add(*codes, it);

        at += size;
    }

    // Counting sort only disordered runs: stable, linear even for adversarial combining sequences.
    begin := 0;
    while begin < codes.count {
        if (property(codes[begin]) & 255) == 0 {
            begin += 1;
            continue;
        }

        end := begin;
        previous: u32 = 0;
        disordered := false;

        while end < codes.count {
            ccc := property(codes[end]) & 255;
            if ccc == 0 break;
            if ccc < previous disordered = true;

            previous = ccc;
            end += 1;
        }

        if disordered {
            counts: [256] int;
            for i: begin..end - 1 counts[property(codes[i]) & 255] += 1;

            total := 0;

            for *counts {
                count := it.*;
                it.* = total;
                total += count;
            }

            array_resize(*scratch, end - begin);

            for i: begin..end - 1 {
                ccc := property(codes[i]) & 255;
                scratch[counts[ccc]] = codes[i];
                counts[ccc] += 1;
            }

            for scratch codes[begin + it_index] = it;
        }

        begin = end;
    }

    if form == .NFC || form == .NFKC {
        starter := -1;
        previous: u32 = 0;
        out := 0;

        for codes {
            ccc := property(it) & 255;
            composite: u32;

            if starter >= 0 && (previous == 0 || previous < ccc) composite = compose(codes[starter], it);

            if composite != 0 {
                codes[starter] = composite;
            } else {
                if ccc == 0 starter = out;
                previous = ccc;
                codes[out] = it;
                out += 1;
            }
        }

        codes.count = out;
    }

    bytes := 0;

    for codes {
        bytes += utf8_size(it);
        if bytes > options.max_output return .{ .LIMIT, "output limit exceeded", text.count };
    }

    for codes {
        buffer: [4] u8;
        encoded := string.{ 4, buffer.data };
        character_utf32_to_utf8(it, *encoded);
        append(builder, encoded);
    }

    if builder.failed return .{ .ALLOCATION, "builder allocation failed", text.count };

    return .{};
}

// Reports whether `text` is already in form `form`. Most text is answered from the quick-check
// tables without allocating; text they cannot decide is normalized into temporary storage and
// compared.
//
// ```jai
// yes, error := is_normalized("\xc3\xa9", .NFC);
// assert(yes && error.kind == .NONE);
// yes, error = is_normalized("e\xcc\x81", .NFC);
// assert(!yes);
// ```
is_normalized :: (text: string, form: Normalization_Form, options: Unicode_Text_Options = .{}) -> bool, Unicode_Text_Error {
    error := arguments_error(text, form, options);
    if error.kind != .NONE return false, error;

    at := 0;
    previous: u32;
    maybe := false;
    yes := true;

    while at < text.count {
        cp, size, valid := decode(text, at);
        if !valid return false, .{ .INVALID_UTF8, "invalid UTF-8", at };

        p := property(cp);
        ccc := p & 255;
        qc := (p >> (24 + 2 * cast(u32) form)) & 3;

        if qc == 1 || (ccc != 0 && previous > ccc) yes = false;
        if qc == 2 maybe = true;

        previous = ccc;
        at += size;
    }

    if yes && maybe {
        b: String_Builder;

        defer free_buffers(*b);
        standard := options;
        standard.stream_safe = false;
        error = normalize(text, form, *b, standard);
        if error.kind != .NONE return false, error;

        normalized := builder_to_string(*b);

        defer free(normalized);
        yes = normalized == text;
    }

    return yes, .{};
}

#load "segmentation.jai";
#load "properties_data.jai";
#load "normalization_data.jai";

#scope_module
#import "Basic";
#import "Unicode";

property :: inline (cp: u32) -> u32 {
    return properties_values[cast(int) properties_index[cp >> 8] * 256 + (cp & 255)];
}

decode :: (text: string, at: int) -> u32, int, bool {
    cp, size, status := character_utf8_to_utf32(text.data + at, text.count - at, true);
    return cp, size, status == .CONVERSION_OK;
}

utf8_size :: (cp: u32) -> int {
    if cp < 0x80 return 1;
    if cp < 0x800 return 2;
    if cp < 0x10000 return 3;
    return 4;
}

arguments_error :: (text: string, form: Normalization_Form, options: Unicode_Text_Options) -> Unicode_Text_Error {
    if cast(int) form > 3 return .{ .INVALID_ARGUMENT, "invalid normalization form", 0 };
    if options.max_size < 0 || options.max_output < 0 return .{ .INVALID_ARGUMENT, "negative limit", 0 };
    if text.count > options.max_size return .{ .LIMIT, "input limit exceeded", 0 };
    return .{};
}

decomposition :: (cp: u32, compat: bool, buffer: *[3] u32) -> [] u32 {
    scalars := cast(*u32) buffer;

    if cp >= 0xAC00 && cp <= 0xD7A3 {
        n := cp - 0xAC00;

        scalars[0] = 0x1100 + n / 588;
        scalars[1] = 0x1161 + (n % 588) / 28;
        scalars[2] = 0x11A7 + n % 28;

        return .{ ifx n % 28 == 0 then 2 else 3, scalars };
    }

    entry: u32;
    if compat entry = compatibility_values[cast(int) compatibility_index[cp >> 8] * 256 + (cp & 255)];
    else entry = canonical_values[cast(int) canonical_index[cp >> 8] * 256 + (cp & 255)];

    if entry == 0 {
        scalars[0] = cp;
        return .{ 1, scalars };
    }

    return .{ cast(int) (entry >> 24), decompositions.data + (entry & 0xFFFFFF) };
}

compose :: (a: u32, b: u32) -> u32 {
    if a >= 0x1100 && a <= 0x1112 && b >= 0x1161 && b <= 0x1175 return 0xAC00 + (a - 0x1100) * 588 + (b - 0x1161) * 28;
    if a >= 0xAC00 && a <= 0xD7A3 && (a - 0xAC00) % 28 == 0 && b >= 0x11A8 && b <= 0x11C2 return a + b - 0x11A7;

    key := ((cast(u64) a) << 21) | (cast(u64) b);
    lo := 0;
    hi := composition_keys.count;

    while lo < hi {
        mid := lo + (hi - lo) / 2;
        if composition_keys[mid] < key lo = mid + 1;
        else hi = mid;
    }

    if lo < composition_keys.count && composition_keys[lo] == key return composition_values[lo];

    return 0;
}
