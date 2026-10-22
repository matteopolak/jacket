// One generator per Type_Info kind. `fill` writes a value of the described type at `dst`.

// Whether `fill` can produce this type. Callbacks, Any, Type and resizable arrays cannot be built.
can_fill :: (info: *Type_Info) -> bool {
    if info.type == {
        case .INTEGER; return true;
        case .FLOAT; return true;
        case .BOOL; return true;
        case .STRING; return true;
        case .ENUM; return true;
        case .POINTER; return true;
        case .ARRAY;
            array := cast(*Type_Info_Array) info;
            if array.array_type == .RESIZABLE return false;

            return can_fill(array.element_type);
        case .STRUCT;
            s := cast(*Type_Info_Struct) info;
            for m: s.members {
                if m.flags & .CONSTANT continue;
                if !can_fill(m.type) return false;
            }

            return true;
    }

    return false;
}

// Whether a value of this type contains a pointer, which targets must opt into.
has_pointer :: (info: *Type_Info) -> bool {
    if info.type == {
        case .POINTER; return true;
        case .ARRAY; return has_pointer((cast(*Type_Info_Array) info).element_type);
        case .STRUCT;
            s := cast(*Type_Info_Struct) info;
            for m: s.members {
                if m.flags & .CONSTANT continue;
                if has_pointer(m.type) return true;
            }
    }

    return false;
}

gen :: (r: *Rng, $T: Type) -> T {
    value: T;
    fill(r, type_info(T), cast(*u8) *value);
    return value;
}

fill :: (r: *Rng, info: *Type_Info, dst: *u8) {
    if info.type == {
        case .INTEGER;
            signed := (cast(*Type_Info_Integer) info).signed;

            store_bits(dst, info.runtime_size, integer_bits(r, info.runtime_size, signed));
        case .FLOAT;
            store_bits(dst, info.runtime_size, float_bits(r, info.runtime_size));
        case .BOOL;
            dst.* = cast,trunc(u8) below(r, 2);
        case .STRING;
            (cast(*string) dst).* = gen_string(r);
        case .ENUM;
            fill_enum(r, cast(*Type_Info_Enum) info, dst);
        case .POINTER;
            fill_pointer(r, cast(*Type_Info_Pointer) info, dst);
        case .ARRAY;
            fill_array(r, cast(*Type_Info_Array) info, dst);
        case .STRUCT;
            s := cast(*Type_Info_Struct) info;
            for m: s.members {
                if m.flags & .CONSTANT continue;
                fill(r, m.type, dst + m.offset_in_bytes);
            }
    }
}

store_bits :: (dst: *u8, size: s64, bits: u64) {
    for i: 0..size - 1 {
        dst[i] = cast,trunc(u8) (bits >> cast(u64) (8 * i));
    }
}

// Boundary values come up often because bugs cluster at them.
integer_bits :: (r: *Rng, size: s64, signed: bool) -> u64 {
    width := 8 * size;
    bits := cast(u64) width;
    all_ones: u64 = 0xFFFF_FFFF_FFFF_FFFF;

    if bits < 64 all_ones = (cast(u64) 1 << bits) - 1;

    sign_bit: u64 = cast(u64) 1 << (bits - 1);

    choice := below(r, 12);

    if choice == {
        case 0; return 0;
        case 1; return 1;
        case 2; return all_ones; // -1, or the unsigned maximum
        case 3; return sign_bit; // signed minimum
        case 4; return sign_bit - 1; // signed maximum
        case 5; return (cast(u64) below(r, 17)) & all_ones; // small
        case 6; return (0 - (cast(u64) below(r, 17))) & all_ones; // small negative
        case 7; return (cast(u64) 1 << cast(u64) below(r, width)) & all_ones;
        case 8; return ((cast(u64) 1 << cast(u64) below(r, width)) - 1) & all_ones;
        case 9; return 0x7F;
        case 10; return 0x80;
    }

    return take(r, size) & all_ones;
}

float_bits :: (r: *Rng, size: s64) -> u64 {
    if size == 4 {
        choice := below(r, 12);
        if choice == {
            case 0; return 0;
            case 1; return 0x8000_0000; // -0
            case 2; return 0x3F80_0000; // 1
            case 3; return 0xBF80_0000; // -1
            case 4; return 0x7FC0_0000; // NaN
            case 5; return 0x7F80_0000; // inf
            case 6; return 0xFF80_0000; // -inf
            case 7; return 1; // smallest subnormal
            case 8; return 0x7F7F_FFFF; // max
            case 9; return 0xFF7F_FFFF; // lowest
            case 10; return 0x0080_0000; // smallest normal
        }

        return take(r, 4);
    }

    choice := below(r, 12);
    if choice == {
        case 0; return 0;
        case 1; return 0x8000_0000_0000_0000;
        case 2; return 0x3FF0_0000_0000_0000;
        case 3; return 0xBFF0_0000_0000_0000;
        case 4; return 0x7FF8_0000_0000_0000;
        case 5; return 0x7FF0_0000_0000_0000;
        case 6; return 0xFFF0_0000_0000_0000;
        case 7; return 1;
        case 8; return 0x7FEF_FFFF_FFFF_FFFF;
        case 9; return 0xFFEF_FFFF_FFFF_FFFF;
        case 10; return 0x0010_0000_0000_0000;
    }

    return take(r, 8);
}

fill_enum :: (r: *Rng, info: *Type_Info_Enum, dst: *u8) {
    size := info.internal_type.runtime_size;

    if info.values.count > 0 && !chance(r, 4) {
        store_bits(dst, size, cast(u64) info.values[below(r, info.values.count)]);
    } else {
        // A value no member names: neighbours of members and arbitrary bits.
        base: u64 = 0;
        if info.values.count > 0 base = cast(u64) info.values[below(r, info.values.count)];
        if chance(r, 2) store_bits(dst, size, base + 1);
        else store_bits(dst, size, integer_bits(r, size, info.internal_type.signed));
    }
}

fill_pointer :: (r: *Rng, info: *Type_Info_Pointer, dst: *u8) {
    target := cast(**void) dst;
    target.* = null;
    if !r.pointers || (r.nullable && chance(r, 8)) return;

    pointee := info.pointer_to;
    size := pointee.runtime_size;

    // Byte-like and untyped pointees get a buffer, since they usually walk a run of bytes.
    if pointee.type == .VOID || size <= 1 size = r.max_len;

    memory := case_alloc(r, size);

    if pointee.type == .VOID || pointee.runtime_size <= 1 {
        for i: 0..size - 1 memory[i] = cast,trunc(u8) take(r, 1);
    } else {
        fill(r, pointee, memory);
    }

    target.* = memory;
}

fill_array :: (r: *Rng, info: *Type_Info_Array, dst: *u8) {
    element := info.element_type;
    stride := element.runtime_size;

    if info.array_type == .FIXED {
        for i: 0..info.array_count - 1 fill(r, element, dst + i * stride);
        return;
    }

    count := gen_length(r);
    view := cast(*[] u8) dst;
    memory: *u8 = null;

    if count > 0 || !chance(r, 2) memory = case_alloc(r, count * stride);
    for i: 0..count - 1 fill(r, element, memory + i * stride);

    view.data = memory;
    view.count = count;
}

// Mostly short, sometimes empty, sometimes as long as allowed.
gen_length :: (r: *Rng) -> s64 {
    choice := below(r, 8);
    if choice == {
        case 0; return 0;
        case 1; return 1;
        case 2; return r.max_len;
        case 3; return below(r, 4);
    }

    return below(r, r.max_len + 1);
}

// Strings mix well-formed text with the malformed byte runs decoders must reject.
gen_string :: (r: *Rng) -> string {
    kind := below(r, 8);
    if kind == 0 return "";

    length := gen_length(r);
    memory := case_alloc(r, length * 4 + 4);
    used: s64 = 0;

    if kind == {
        case 1; // printable ASCII
            while used < length {
                memory[used] = cast,trunc(u8) (32 + below(r, 95));
                used += 1;
            }
        case 2; // well-formed UTF-8 over every plane
            while used + 4 <= length {
                used += put_code_point(memory + used, random_code_point(r));
            }
        case 3; // arbitrary bytes
            while used < length {
                memory[used] = cast,trunc(u8) take(r, 1);
                used += 1;
            }
        case 4; // truncated, overlong, surrogate and out-of-range sequences, plus stray continuations
            while used + 4 <= length {
                used += put_malformed(r, memory + used);
            }
        case 5; // separators, whitespace and NUL
            while used < length {
                memory[used] = special_byte(r);
                used += 1;
            }
        case 6; // base64 and number alphabets with a little noise
            while used < length {
                memory[used] = alphabet_byte(r);
                used += 1;
            }
        case 7; // one repeated byte
            byte := cast,trunc(u8) take(r, 1);

            while used < length {
                memory[used] = byte;
                used += 1;
            }
    }

    result: string;
    result.data = memory;
    result.count = used;

    return result;
}

random_code_point :: (r: *Rng) -> u32 {
    choice := below(r, 8);
    if choice == {
        case 0; return cast(u32) below(r, 0x80);
        case 1; return 0x80 + cast(u32) below(r, 0x780);
        case 2; return 0x800 + cast(u32) below(r, 0xF800);
        case 3; return 0x10000 + cast(u32) below(r, 0x100000);
        case 4; return 0x10FFFF;
        case 5; return 0xFFFD;
        case 6; return 0xD7FF;
    }

    code := cast(u32) below(r, 0x110000);
    if code >= 0xD800 && code < 0xE000 code = 0x41;

    return code;
}

put_code_point :: (dst: *u8, code: u32) -> s64 {
    if code < 0x80 {
        dst[0] = cast,trunc(u8) code;
        return 1;
    }

    if code < 0x800 {
        dst[0] = cast,trunc(u8) (0xC0 | (code >> 6));
        dst[1] = cast,trunc(u8) (0x80 | (code & 0x3F));
        return 2;
    }

    if code < 0x10000 {
        dst[0] = cast,trunc(u8) (0xE0 | (code >> 12));
        dst[1] = cast,trunc(u8) (0x80 | ((code >> 6) & 0x3F));
        dst[2] = cast,trunc(u8) (0x80 | (code & 0x3F));
        return 3;
    }

    dst[0] = cast,trunc(u8) (0xF0 | (code >> 18));
    dst[1] = cast,trunc(u8) (0x80 | ((code >> 12) & 0x3F));
    dst[2] = cast,trunc(u8) (0x80 | ((code >> 6) & 0x3F));
    dst[3] = cast,trunc(u8) (0x80 | (code & 0x3F));

    return 4;
}

// Writes one malformed sequence of at most four bytes and returns its length.
put_malformed :: (r: *Rng, dst: *u8) -> s64 {
    choice := below(r, 10);
    if choice == {
        case 0;
            dst[0] = 0x80;
            return 1; // stray continuation
        case 1;
            dst[0] = 0xC0;
            dst[1] = 0x80;
            return 2; // overlong NUL
        case 2;
            dst[0] = 0xE0;
            dst[1] = 0x80;
            dst[2] = 0x80;

            return 3; // overlong
        case 3;
            dst[0] = 0xED;
            dst[1] = 0xA0;
            dst[2] = 0x80;

            return 3; // surrogate
        case 4;
            dst[0] = 0xF4;
            dst[1] = 0x90;
            dst[2] = 0x80;
            dst[3] = 0x80;

            return 4; // above U+10FFFF
        case 5;
            dst[0] = 0xF8;
            dst[1] = 0x88;
            dst[2] = 0x80;
            dst[3] = 0x80;

            return 4; // five-byte lead
        case 6;
            dst[0] = 0xE2;
            dst[1] = 0x82;
            return 2; // truncated before the end
        case 7;
            dst[0] = 0xF0;
            return 1; // lead with nothing after
        case 8;
            dst[0] = 0xFF;
            return 1;
    }

    dst[0] = 0xC3;
    dst[1] = 0x28;

    return 2; // continuation replaced
}

special_byte :: (r: *Rng) -> u8 {
    SPECIAL :: "\0 \t\r\n/\\.:;,|*?-_=+%\"'";
    return SPECIAL[below(r, SPECIAL.count)];
}

alphabet_byte :: (r: *Rng) -> u8 {
    ALPHABET :: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=-_.eE xX\xff";
    return ALPHABET[below(r, ALPHABET.count)];
}
