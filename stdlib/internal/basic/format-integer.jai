FormatInt :: Shared_Print.FormatInt;

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

    info = formatter_base_type(info);
    assert(info != null && info.type == .INTEGER, "FormatInt requires an integer value");
    integer := cast(*Type_Info_Integer) info;
    value := get_u64_from_void_pointer(format.value.value_pointer, info.runtime_size);

    if integer.signed && format.base == 10 {
        signed := get_s64_from_void_pointer(format.value.value_pointer, info.runtime_size);
        if signed < 0 {
            append(builder, cast(u8) 45);
            value = 0 - cast(u64) signed;
        }
    }

    print_u64_base_n(builder, value, format);
}

print_u64_base_n :: (builder: *String_Builder, v: u64, format: FormatInt) {
    assert(format.base >= 2 && format.base <= 64);
    assert(format.minimum_digits >= 0);
    digits: [64] u8;
    count: s64;
    value := v;
    base := cast(u64) format.base;

    while true {
        digit := value % base;

        digits[count] = DIGITS[cast(s64) digit];
        count += 1;
        value /= base;
        if value == 0 break;
    }

    remaining := max(count, min(format.minimum_digits, 64));
    while remaining > 0 {
        remaining -= 1;
        if remaining >= count append(builder, format.padding);
        else append(builder, digits[remaining]);
        if remaining > 0 && format.digits_per_comma > 0 && remaining % cast(s64) format.digits_per_comma == 0 append(builder, format.comma_string);
    }
}

#scope_file
DIGITS :: "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_.";
