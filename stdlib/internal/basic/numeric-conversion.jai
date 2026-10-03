// Exact integer arithmetic is shared by decimal rendering and binary rounding.
#scope_module
portable_float :: (value: float64, conversion: u8, precision: s64, round_up := false) -> string {
    bits := (cast(*u64) *value).*;
    exponent := cast(s64) ((bits >> 52) & 0x7ff);
    mantissa := bits & 0x000fffffffffffff;

    if exponent != 0 mantissa |= 0x0010000000000000;

    binary_scale := ifx exponent == 0 then -1074 else exponent - 1023 - 52;
    number := nat_from(mantissa);

    defer array_free(number.words);
    decimal_scale: s64;

    if binary_scale < 0 {
        decimal_scale = -binary_scale;
        for i: 0..decimal_scale - 1 nat_multiply(*number, 5);
    } else nat_shift(*number, binary_scale);

    exact := nat_decimal(*number);

    defer free(exact);
    assert(precision >= 0 && precision < 0x7fffffffffffffff - exact.count - 4, "float output length exceeds addressable storage");
    p := precision;

    if conversion == 102 {
        rounded := round_decimal(exact, exact.count - decimal_scale + p, round_up);
        defer free(rounded);
        return decimal_layout(rounded, p);
    }

    significant := ifx conversion == 103 then max(p, cast(s64) 1) else p + 1;
    rounded := round_decimal(exact, significant, round_up);

    defer free(rounded);
    decimal_exponent := ifx mantissa == 0 then 0 else exact.count - decimal_scale - 1;

    if rounded.count > significant {
        decimal_exponent += 1;
        rounded.count -= 1;
    }

    if conversion == 103 {
        result: string;
        if decimal_exponent < -4 || decimal_exponent >= significant result = scientific_layout(rounded, decimal_exponent);
        else result = decimal_layout(rounded, significant - decimal_exponent - 1);
        return trim_fraction(result);
    }

    return scientific_layout(rounded, decimal_exponent);
}

parse_float_number :: (text: string, $T := float64) -> (result: T, success: bool, remainder: string) {
    #assert T == float64 || T == float32;
    sign_mask: u64 = ifx T == float64 then 0x8000000000000000 else 0x80000000;
    infinity: u64 = ifx T == float64 then 0x7ff0000000000000 else 0x7f800000;
    quiet_nan: u64 = ifx T == float64 then 0x7ff8000000000000 else 0x7fc00000;
    cursor: s64;

    while cursor < text.count && is_space(text[cursor]) cursor += 1;

    negative := false;

    if cursor < text.count && (text[cursor] == 43 || text[cursor] == 45) {
        negative = text[cursor] == 45;
        cursor += 1;
    }

    sign := ifx negative then sign_mask else cast(u64) 0;

    if word_at(text, cursor, "inf") {
        cursor += 3;
        if word_at(text, cursor, "inity") cursor += 5;
        return float_bits(sign | infinity, T), true, advance(text, cursor);
    }

    if word_at(text, cursor, "nan") {
        cursor += 3;
        end := cursor;
        if end < text.count && text[end] == 40 {
            end += 1;
            while end < text.count && (is_alnum(text[end]) || text[end] == 95) end += 1;
            if end < text.count && text[end] == 41 cursor = end + 1;
        }

        return float_bits(sign | quiet_nan, T), true, advance(text, cursor);
    }

    base: u32 = 10;

    if cursor + 2 < text.count && text[cursor] == 48 && to_lower(text[cursor + 1]) == 120 {
        after := cursor + 2;
        if text[after] == 46 after += 1;

        if after < text.count && digit_value(text[after]) < 16 {
            base = 16;
            cursor += 2;
        }
    }

    number := nat_from(0);

    defer array_free(number.words);
    digits: s64;
    fractional: s64;
    point := false;
    first_nonzero: s64 = -1;

    while cursor < text.count {
        byte := text[cursor];
        if byte == 46 && !point {
            point = true;
            cursor += 1;
            continue;
        }

        digit := digit_value(byte);
        if digit >= base break;
        if digit != 0 && first_nonzero < 0 first_nonzero = digits;

        nat_multiply(*number, base);
        nat_add(*number, digit);
        digits += 1;
        if point fractional += 1;

        cursor += 1;
    }

    if digits == 0 return 0, false, text;

    exponent: s64;
    marker: u8 = ifx base == 16 then 112 else 101;

    if cursor < text.count && to_lower(text[cursor]) == marker {
        end := cursor + 1;
        minus := false;

        if end < text.count && (text[end] == 43 || text[end] == 45) {
            minus = text[end] == 45;
            end += 1;
        }

        start := end;

        while end < text.count && is_digit(text[end]) {
            digit := cast(s64) (text[end] - 48);
            if exponent <= (0x7fffffffffffffff - digit) / 10 exponent = exponent * 10 + digit;
            else exponent = 0x7fffffffffffffff;
            end += 1;
        }

        if end > start {
            cursor = end;
            if minus exponent = -exponent;
        } else exponent = 0;
    }

    rest := advance(text, cursor);
    if first_nonzero < 0 return float_bits(sign, T), true, rest;

    denominator := nat_from(1);

    defer array_free(denominator.words);

    if base == 10 {
        // Compare decimal order before constructing a potentially enormous power.
        order := digits - first_nonzero - fractional - 1;
        if exponent > 310 - order return float_bits(sign | infinity, T), true, rest;
        if exponent < -325 - order return float_bits(sign, T), true, rest;

        scale := exponent - fractional;
        if scale >= 0 for i: 0..scale - 1 nat_multiply(*number, 10);
        else for i: 0..(-scale) - 1 nat_multiply(*denominator, 10);
    } else {
        order := nat_bit_count(number) - 1;
        if exponent > 1024 - order + fractional * 4 return float_bits(sign | infinity, T), true, rest;
        if exponent < -1076 - order + fractional * 4 return float_bits(sign, T), true, rest;

        scale := exponent - fractional * 4;
        if scale >= 0 nat_shift(*number, scale);
        else nat_shift(*denominator, -scale);
    }

    return float_bits(sign | rational_bits(*number, *denominator, T), T), true, rest;
}

#scope_file
Natural :: struct {
    words: [..] u32;
}

nat_from :: (value: u64) -> Natural {
    result: Natural;

    if value != 0 {
        array_add(*result.words, cast, truncate(u32) value);
        if value >> 32 != 0 array_add(*result.words, cast(u32) (value >> 32));
    }

    return result;
}

nat_copy :: (value: Natural) -> Natural {
    result: Natural;
    for word: value.words array_add(*result.words, word);
    return result;
}

nat_trim :: (value: *Natural) {
    while value.words.count > 0 && value.words[value.words.count - 1] == 0 value.words.count -= 1;
}

nat_multiply :: (value: *Natural, factor: u32) {
    carry: u64;

    for *word: value.words {
        product := (cast(u64) word.*) * cast(u64) factor + carry;
        word.* = cast,truncate(u32) product;
        carry = product >> 32;
    }

    if carry != 0 array_add(*value.words, cast(u32) carry);
}

nat_add :: (value: *Natural, amount: u32) {
    carry := cast(u64) amount;

    for *word: value.words {
        sum := cast(u64) word.* + carry;
        word.* = cast,truncate(u32) sum;
        carry = sum >> 32;
        if carry == 0 return;
    }

    if carry != 0 array_add(*value.words, cast(u32) carry);
}

nat_shift :: (value: *Natural, bits: s64) {
    assert(bits >= 0);
    if value.words.count == 0 || bits == 0 return;

    whole := bits / 32;
    part := bits % 32;
    old := value.words.count;

    array_resize(*value.words, old + whole + 1);
    index := old;
    while index > 0 {
        index -= 1;
        value.words[index + whole] = value.words[index];
    }

    for i: 0..whole - 1 value.words[i] = 0;

    carry: u64;

    for i: whole..old + whole - 1 {
        word := ((cast(u64) value.words[i]) << part) | carry;
        value.words[i] = cast,truncate(u32) word;
        carry = word >> 32;
    }

    value.words[old + whole] = cast(u32) carry;
    nat_trim(value);
}

nat_compare :: (a: Natural, b: Natural) -> s64 {
    if a.words.count != b.words.count return ifx a.words.count < b.words.count then -1 else 1;

    index := a.words.count;
    while index > 0 {
        index -= 1;
        if a.words[index] != b.words[index] return ifx a.words[index] < b.words[index] then -1 else 1;
    }

    return 0;
}

nat_subtract :: (a: *Natural, b: Natural) {
    assert(nat_compare(a.*, b) >= 0);
    borrow: u64;

    for *word, index: a.words {
        sub := borrow;
        if index < b.words.count sub += cast(u64) b.words[index];

        original := cast(u64) word.*;

        word.* = cast,truncate(u32) (0x100000000 + original - sub);
        borrow = ifx original < sub then 1 else 0;
    }

    nat_trim(a);
}

nat_bit_count :: (value: Natural) -> s64 {
    if value.words.count == 0 return 0;

    count := (value.words.count - 1) * 32;
    top := value.words[value.words.count - 1];

    while top != 0 {
        count += 1;
        top >>= 1;
    }

    return count;
}

nat_divide_small :: (value: *Natural, divisor: u32) -> u32 {
    remainder: u64;
    index := value.words.count;

    while index > 0 {
        index -= 1;
        combined := (remainder << 32) | cast(u64) value.words[index];
        value.words[index] = cast(u32) (combined / cast(u64) divisor);
        remainder = combined % cast(u64) divisor;
    }

    nat_trim(value);

    return cast(u32) remainder;
}

nat_decimal :: (value: *Natural) -> string {
    if value.words.count == 0 return copy_string("0");

    chunks: [..] u32;

    defer array_free(chunks);
    while value.words.count > 0 array_add(*chunks, nat_divide_small(value, 1000000000));

    builder: String_Builder;

    defer free_buffers(*builder);
    index := chunks.count;
    while index > 0 {
        index -= 1;
        format: FormatInt;
        if index != chunks.count - 1 format.minimum_digits = 9;
        print_u64_base_n(*builder, cast(u64) chunks[index], format);
    }

    return builder_to_string(*builder);
}

round_decimal :: (digits: string, keep: s64, round_up: bool) -> string {
    retained := max(keep, cast(s64) 0);
    up := false;

    if keep >= 0 && keep < digits.count {
        discarded := digits[keep];

        up = discarded > 53;

        if discarded == 53 {
            up = keep > 0 && (digits[keep - 1] & 1) != 0;

            for i: keep + 1..digits.count - 1 if digits[i] != 48 {
                up = true;
                break;
            }
        }
    }

    if round_up {
        up = false;

        for i: max(keep, cast(s64) 0)..digits.count - 1 if digits[i] != 48 {
            up = true;
            break;
        }
    }

    if retained == 0 return copy_string(ifx up then "1" else "0");

    result := alloc_string(retained + 1);
    result[0] = 48;
    for i: 0..retained - 1 result[i + 1] = ifx i < digits.count then digits[i] else cast(u8) 48;

    index := retained;
    while up && index > 0 {
        if result[index] == 57 {
            result[index] = 48;
            index -= 1;
        } else {
            result[index] += 1;
            up = false;
        }
    }

    if up result[0] = 49;
    else {
        for i: 0..retained - 1 result[i] = result[i + 1];
        result.count = retained;
    }

    return result;
}

decimal_layout :: (digits: string, fraction: s64) -> string {
    builder: String_Builder;

    defer free_buffers(*builder);
    integer := digits.count - fraction;
    if integer <= 0 {
        append(*builder, "0.");
        formatter_repeat(*builder, 48, -integer);
        append(*builder, digits);
    } else {
        append(*builder, digits.data, integer);

        if fraction > 0 {
            append(*builder, cast(u8) 46);
            append(*builder, digits.data + integer, fraction);
        }
    }

    return builder_to_string(*builder);
}

scientific_layout :: (digits: string, exponent: s64) -> string {
    builder: String_Builder;

    defer free_buffers(*builder);
    append(*builder, digits[0]);

    if digits.count > 1 {
        append(*builder, cast(u8) 46);
        append(*builder, digits.data + 1, digits.count - 1);
    }

    append(*builder, ifx exponent < 0 then "e-" else "e+");
    magnitude := ifx exponent < 0 then -exponent else exponent;

    print_u64_base_n(*builder, cast(u64) magnitude, FormatInt.{ minimum_digits = 2 });

    return builder_to_string(*builder);
}

trim_fraction :: (text: string) -> string {
    end := text.count;
    point: s64 = -1;

    for byte, index: text {
        if byte == 46 point = index;

        if byte == 101 {
            end = index;
            break;
        }
    }

    if point < 0 return text;

    kept := end;
    while kept > point + 1 && text[kept - 1] == 48 kept -= 1;
    if kept == point + 1 kept = point;
    for i: 0..text.count - end - 1 text[kept + i] = text[end + i];

    text.count -= end - kept;

    return text;
}

digit_value :: (byte: u8) -> u32 {
    if byte >= 48 && byte <= 57 return cast(u32) (byte - 48);
    lower := to_lower(byte);
    if lower >= 97 && lower <= 102 return cast(u32) (lower - 97 + 10);
    return 255;
}

word_at :: (text: string, cursor: s64, word: string) -> bool {
    if cursor + word.count > text.count return false;
    for byte, index: word if to_lower(text[cursor + index]) != byte return false;
    return true;
}

float_bits :: (bits: u64, $T: Type) -> T {
    #if T == float64 return (cast(*float64) *bits).*;
    else {
        narrow := cast,truncate(u32) bits;
        return (cast(*float32) *narrow).*;
    }
}

rational_bits :: (numerator: *Natural, denominator: *Natural, $T: Type) -> u64 {
    mantissa_bits: s64 = ifx T == float64 then 52 else 23;
    bias: s64 = ifx T == float64 then 1023 else 127;
    minimum := 1 - bias;
    infinity: u64 = ifx T == float64 then 0x7ff0000000000000 else 0x7f800000;
    exponent := nat_bit_count(numerator.*) - nat_bit_count(denominator.*);
    shifted := ifx exponent >= 0 then nat_copy(denominator.*) else nat_copy(numerator.*);

    defer array_free(shifted.words);
    nat_shift(*shifted, ifx exponent >= 0 then exponent else -exponent);
    if (ifx exponent >= 0 then nat_compare(numerator.*, shifted) else nat_compare(shifted, denominator.*)) < 0 exponent -= 1;
    if exponent > bias return infinity;
    if exponent < minimum - mantissa_bits - 1 return 0;

    unit := max(exponent - mantissa_bits, minimum - mantissa_bits);
    if unit < 0 nat_shift(numerator, -unit);
    else nat_shift(denominator, unit);

    quotient: u64;
    bit := nat_bit_count(numerator.*) - nat_bit_count(denominator.*);

    assert(bit < 64);

    while bit >= 0 {
        array_free(shifted.words);
        shifted = nat_copy(denominator.*);
        nat_shift(*shifted, bit);

        if nat_compare(numerator.*, shifted) >= 0 {
            nat_subtract(numerator, shifted);
            quotient |= ((cast(u64) 1) << bit);
        }

        bit -= 1;
    }

    nat_shift(numerator, 1);
    comparison := nat_compare(numerator.*, denominator.*);
    if comparison > 0 || (comparison == 0 && (quotient & 1) != 0) quotient += 1;
    if exponent < minimum return quotient;

    if quotient == ((cast(u64) 1) << (mantissa_bits + 1)) {
        quotient >>= 1;
        exponent += 1;
    }

    if exponent > bias return infinity;

    return ((cast(u64) (exponent + bias)) << mantissa_bits) | (quotient & (((cast(u64) 1) << mantissa_bits) - 1));
}
