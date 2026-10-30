FormatInt :: Shared_Print.FormatInt;

// Wraps an integer for `print` so it comes out in another base, padded, or grouped.
//
// ```jai
// assert(tprint("%", formatInt(255, base = 16)) == "ff");
// assert(tprint("%", formatInt(7, minimum_digits = 3)) == "007");
// assert(tprint("%", formatInt(1234567, digits_per_comma = 3, comma_string = ",")) == "1,234,567");
// ```
//
// - base: from 2 to 64. Digits past 9 are `a` to `z`, then `A` to `Z`.
// - minimum_digits: pads the digits with `padding` up to this many. A minus sign is not counted.
// - padding: the byte used to pad, `0` by default.
// - digits_per_comma: puts `comma_string` between groups of this many digits, counted from the right.
//   0 for no groups.
// - comma_string: what goes between the groups, for example `","`.
formatInt :: (value: Any, base := 10, minimum_digits := 1, padding: u8 = #char "0", digits_per_comma: u16 = 0, comma_string := "") -> FormatInt {
    result: FormatInt;
    result.value = value;
    result.base = base;
    result.minimum_digits = minimum_digits;
    result.padding = padding;
    result.digits_per_comma = digits_per_comma;
    result.comma_string = comma_string;

    return result;
}

print_integer :: (builder: *String_Builder, format: FormatInt) #no_aoc {
    info := format.value.type;
    if !info || !format.value.value_pointer {
        append(builder, "(null Any)");
        return;
    }

    if info.type != .INTEGER info = formatter_base_type(info);

    assert(info != null && info.type == .INTEGER, "FormatInt requires an integer value");
    integer := cast(*Type_Info_Integer) info;
    pointer := format.value.value_pointer;
    value: u64 = ---;

    // One read of the value; signed base-10 values are sign-extended so the minus sign can be split off.
    if integer.signed && format.base == 10 {
        signed: s64 = ---;
        if info.runtime_size == 1 signed = (cast(*s8) pointer).*;
        else if info.runtime_size == 2 signed = (cast(*s16) pointer).*;
        else if info.runtime_size == 4 signed = (cast(*s32) pointer).*;
        else signed = get_s64_from_void_pointer(pointer, info.runtime_size);

        value = cast,no_check(u64) signed;

        if signed < 0 {
            append(builder, cast(u8) 45);
            value = 0 - value;
        }
    } else {
        value = get_u64_from_void_pointer(pointer, info.runtime_size);
    }

    print_u64_base_n(builder, value, format);
}

print_u64_base_n :: (builder: *String_Builder, v: u64, format: FormatInt) #no_abc {
    assert(format.base >= 2 && format.base <= 64);
    assert(format.minimum_digits >= 0);

    // Digits are written from the end of the buffer towards the front, so they come out in order.
    text: [64] u8 = ---;
    start := write_digits(*text, v, cast(u64) format.base);
    count := text.count - start;
    remaining := max(count, min(format.minimum_digits, 64));

    if format.digits_per_comma == 0 {
        // No separators: the padding goes in front of the digits and everything is appended at once.
        while text.count - start < remaining {
            start -= 1;
            text[start] = format.padding;
        }

        append(builder, text.data + start, remaining);

        return;
    }

    // Position `remaining - 1` is the most significant one; digit k from the right is text[63 - k].
    while remaining > 0 {
        remaining -= 1;
        if remaining >= count append(builder, format.padding);
        else append(builder, text[text.count - 1 - remaining]);
        if remaining > 0 && remaining % cast(s64) format.digits_per_comma == 0 append(builder, format.comma_string);
    }
}

#scope_file
DIGITS :: "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_.";

// Writes the digits of `value` at the end of `text` and returns the index of the first one.
write_digits :: (text: *[64] u8, value: u64, base: u64) -> s64 #no_abc {
    position := text.count;
    rest := value;

    if base == 10 {
        // Constant divisors become multiplications, and each one yields two digits.
        while rest >= 100 {
            pair := cast(s64) (rest % 100) * 2;

            rest /= 100;
            position -= 2;
            text.*[position] = DECIMAL_PAIRS[pair];
            text.*[position + 1] = DECIMAL_PAIRS[pair + 1];
        }

        if rest >= 10 {
            pair := cast(s64) rest * 2;
            position -= 2;
            text.*[position] = DECIMAL_PAIRS[pair];
            text.*[position + 1] = DECIMAL_PAIRS[pair + 1];
        } else {
            position -= 1;
            text.*[position] = cast(u8) (#char "0" + rest);
        }

        return position;
    }

    if (base & (base - 1)) == 0 {
        // Powers of two: each digit is a group of bits.
        shift: u64;
        while ((cast(u64) 1) << shift) < base shift += 1;

        mask := base - 1;

        while true {
            position -= 1;
            text.*[position] = DIGITS[cast(s64) (rest & mask)];
            rest >>= shift;
            if rest == 0 break;
        }

        return position;
    }

    while true {
        position -= 1;
        text.*[position] = DIGITS[cast(s64) (rest % base)];
        rest /= base;
        if rest == 0 break;
    }

    return position;
}
