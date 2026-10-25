// Comparing values by what they hold, and saying how two values differ.

#scope_module

values_equal :: (left: Any, right: Any) -> bool {
    a := underlying(left);
    b := underlying(right);
    ta := a.type;
    tb := b.type;

    if ta.type == .INTEGER || ta.type == .ENUM {
        if tb.type != .INTEGER && tb.type != .ENUM return false;
        return integers_equal(a, b);
    }

    if ta.type != tb.type return false;

    if ta.type == {
        case .FLOAT;
            return float_value(a) == float_value(b); // jailint: allow(float_equality) equal means exactly equal
        case .BOOL;
            return (cast(*bool) a.value_pointer).* == (cast(*bool) b.value_pointer).*;
        case .STRING;
            return (cast(*string) a.value_pointer).* == (cast(*string) b.value_pointer).*;
        case .POINTER; #through;
        case .PROCEDURE;
            return (cast(**void) a.value_pointer).* == (cast(**void) b.value_pointer).*;
        case .TYPE;
            return (cast(*Type) a.value_pointer).* == (cast(*Type) b.value_pointer).*;
        case .ARRAY;
            ea, ca, ia := array_view(a);
            eb, cb, ib := array_view(b);

            if ca != cb return false;

            for 0..ca - 1 {
                if !values_equal(element(ea, ia, it), element(eb, ib, it)) return false;
            }

            return true;
        case .STRUCT;
            if ta != tb return false;

            info := cast(*Type_Info_Struct) ta;
            for info.members {
                if it.flags & .CONSTANT continue;
                ma := Any.{ it.type, a.value_pointer + it.offset_in_bytes };
                mb := Any.{ it.type, b.value_pointer + it.offset_in_bytes };
                if !values_equal(ma, mb) return false;
            }

            return true;
    }

    if ta.runtime_size != tb.runtime_size return false;

    return memory_equal(a.value_pointer, b.value_pointer, ta.runtime_size);
}

describe_difference :: (got: Any, want: Any) -> string {
    g := underlying(got);
    w := underlying(want);

    if g.type.type == .STRING && w.type.type == .STRING {
        return string_difference((cast(*string) g.value_pointer).*, (cast(*string) w.value_pointer).*);
    }

    if g.type.type == .ARRAY && w.type.type == .ARRAY return array_difference(g, w);

    if g.type.type != w.type.type && !(is_integral(g) && is_integral(w)) {
        return tprint("got a % (%), want a % (%)", type_name(g), g, type_name(w), w);
    }

    builder: String_Builder;
    builder.allocator = temp;
    print_to_builder(*builder, "got:  %\nwant: %", g, w);

    if g.type.type == .STRUCT && g.type == w.type {
        path := first_member_difference(g, w);
        if path print_to_builder(*builder, "\nfirst difference at %", path);
    }

    return builder_to_string(*builder,, temp);
}

#scope_file

// Variants compare as what they are variants of.
underlying :: (original: Any) -> Any {
    value := original;
    while value.type.type == .VARIANT {
        value.type = (cast(*Type_Info_Variant) value.type).variant_of;
    }

    return value;
}

is_integral :: (value: Any) -> bool {
    return value.type.type == .INTEGER || value.type.type == .ENUM;
}

type_name :: (value: Any) -> string {
    builder: String_Builder;
    builder.allocator = temp;
    print_type_to_builder(*builder, value.type);
    return builder_to_string(*builder,, temp);
}

// An integer or enum as its magnitude and sign, whatever its width.
Integer :: struct {
    negative: bool;
    magnitude: u64;
}

integer_value :: (value: Any) -> Integer {
    info := value.type;
    signed := false;

    if info.type == .ENUM {
        info = (cast(*Type_Info_Enum) info).internal_type;
    }

    signed = (cast(*Type_Info_Integer) info).signed;
    bits: u64;

    if info.runtime_size == {
        case 1; bits = (cast(*u8) value.value_pointer).*;
        case 2; bits = (cast(*u16) value.value_pointer).*;
        case 4; bits = (cast(*u32) value.value_pointer).*;
        case; bits = (cast(*u64) value.value_pointer).*;
    }

    if !signed return .{ false, bits };

    // Sign-extend from the value's width.
    width := cast(u64) info.runtime_size * 8;
    if width < 64 && (bits >> (width - 1)) & 1 {
        bits |= 0xFFFF_FFFF_FFFF_FFFF << width;
    }

    as_signed := cast,no_check(s64) bits;
    if as_signed < 0 return .{ true, cast,no_check(u64) -(as_signed + 1) + 1 };

    return .{ false, bits };
}

integers_equal :: (a: Any, b: Any) -> bool {
    x := integer_value(a);
    y := integer_value(b);
    return x.negative == y.negative && x.magnitude == y.magnitude;
}

float_value :: (value: Any) -> float64 {
    if value.type.runtime_size == 4 return (cast(*float32) value.value_pointer).*;
    return (cast(*float64) value.value_pointer).*;
}

memory_equal :: (a: *void, b: *void, count: s64) -> bool {
    pa := cast(*u8) a;
    pb := cast(*u8) b;
    for 0..count - 1 if pa[it] != pb[it] return false;
    return true;
}

// The element type, count and first element of any array.
array_view :: (value: Any) -> (element: *Type_Info, count: s64, data: *void) {
    info := cast(*Type_Info_Array) value.type;
    if info.array_type == .FIXED return info.element_type, info.array_count, value.value_pointer;
    view := cast(*[] u8) value.value_pointer;
    return info.element_type, view.count, view.data;
}

element :: (type: *Type_Info, data: *void, index: s64) -> Any {
    return Any.{ type, data + index * type.runtime_size };
}

first_member_difference :: (a: Any, b: Any) -> string {
    info := cast(*Type_Info_Struct) a.type;
    for info.members {
        if it.flags & .CONSTANT continue;

        ma := Any.{ it.type, a.value_pointer + it.offset_in_bytes };
        mb := Any.{ it.type, b.value_pointer + it.offset_in_bytes };

        if values_equal(ma, mb) continue;

        if it.type.type == .STRUCT {
            inner := first_member_difference(ma, mb);
            if inner return tprint(".%1%2", it.name, inner);
        }

        return tprint(".%", it.name);
    }

    return "";
}

MAX_LISTED :: 10;

array_difference :: (got: Any, want: Any) -> string {
    eg, cg, dg := array_view(got);
    ew, cw, dw := array_view(want);
    builder: String_Builder;
    builder.allocator = temp;
    if cg != cw print_to_builder(*builder, "got % elements, want %\n", cg, cw);

    listed := 0;

    for 0..min(cg, cw) - 1 {
        a := element(eg, dg, it);
        b := element(ew, dw, it);

        if values_equal(a, b) continue;

        if listed == MAX_LISTED {
            append(*builder, "  ...\n");
            break;
        }

        print_to_builder(*builder, "  [%]: got %, want %\n", it, a, b);
        listed += 1;
    }

    for cw..cg - 1 {
        if it - cw == MAX_LISTED break;
        print_to_builder(*builder, "  [%]: extra %\n", it, element(eg, dg, it));
    }

    for cg..cw - 1 {
        if it - cg == MAX_LISTED break;
        print_to_builder(*builder, "  [%]: missing %\n", it, element(ew, dw, it));
    }

    text := builder_to_string(*builder,, temp);
    while text && text[text.count - 1] == #char "\n" text.count -= 1;

    return text;
}

has_newline :: (s: string) -> bool {
    for s if it == #char "\n" return true;
    return false;
}

// A string as a quoted literal, with the characters that would hide a difference escaped.
quoted :: (s: string) -> string {
    builder: String_Builder;
    builder.allocator = temp;
    append(*builder, "\"");

    for s {
        c := it;
        if c == {
            case #char "\n"; append(*builder, "\\n");
            case #char "\t"; append(*builder, "\\t");
            case #char "\r"; append(*builder, "\\r");
            case #char "\""; append(*builder, "\\\"");
            case #char "\\"; append(*builder, "\\\\");
            case;
                if c < 32 || c == 127 {
                    print_to_builder(*builder, "\\x%", formatInt(c, base = 16, minimum_digits = 2));
                } else {
                    append(*builder, c);
                }
        }
    }

    append(*builder, "\"");

    return builder_to_string(*builder,, temp);
}

string_difference :: (got: string, want: string) -> string {
    if has_newline(got) || has_newline(want) return line_difference(got, want);

    first := 0;
    while first < got.count && first < want.count && got[first] == want[first] first += 1;

    builder: String_Builder;
    builder.allocator = temp;
    print_to_builder(*builder, "got:  %\nwant: %\n", quoted(got), quoted(want));
    // The caret sits under the first byte that differs in `got`'s quoted form.
    column := quoted(slice_of(got, first)).count - 1;
    for 1..column + "got:  ".count append(*builder, " ");

    print_to_builder(*builder, "^ first difference at byte %", first);

    return builder_to_string(*builder,, temp);
}

slice_of :: (s: string, count: s64) -> string {
    return string.{ count, s.data };
}

lines_of :: (s: string) -> [..] string {
    lines: [..] string;
    lines.allocator = temp;
    start := 0;

    for s {
        if it != #char "\n" continue;
        array_add(*lines, string.{ it_index - start, s.data + start });
        start = it_index + 1;
    }

    if start < s.count array_add(*lines, string.{ s.count - start, s.data + start });

    return lines;
}

// Larger inputs are shown whole: the table below is quadratic.
MAX_DIFF_CELLS :: 4_000_000;

// A line diff (longest common subsequence): ` ` for lines both have, `-` for lines only `want`
// has, `+` for lines only `got` has.
line_difference :: (got: string, want: string) -> string {
    g := lines_of(got);
    w := lines_of(want);
    builder: String_Builder;
    builder.allocator = temp;
    append(*builder, "--- want\n+++ got\n");

    if (g.count + 1) * (w.count + 1) > MAX_DIFF_CELLS {
        print_to_builder(*builder, "want:\n%\ngot:\n%", want, got);
        return builder_to_string(*builder,, temp);
    }

    columns := g.count + 1;
    table := NewArray((w.count + 1) * columns, s32,, temp);
    // table[i * columns + j]: common lines of w[i..] and g[j..].
    i := w.count - 1;

    while i >= 0 {
        j := g.count - 1;
        while j >= 0 {
            if w[i] == g[j] {
                table[i * columns + j] = table[(i + 1) * columns + j + 1] + 1;
            } else {
                table[i * columns + j] = max(table[(i + 1) * columns + j], table[i * columns + j + 1]);
            }

            j -= 1;
        }

        i -= 1;
    }

    i = 0;
    j := 0;
    while i < w.count || j < g.count {
        if i < w.count && j < g.count && w[i] == g[j] {
            print_to_builder(*builder, "  %\n", w[i]);
            i += 1;
            j += 1;
        } else if j < g.count && (i == w.count || table[i * columns + j + 1] >= table[(i + 1) * columns + j]) {
            print_to_builder(*builder, "+ %\n", g[j]);
            j += 1;
        } else {
            print_to_builder(*builder, "- %\n", w[i]);
            i += 1;
        }
    }

    text := builder_to_string(*builder,, temp);
    while text && text[text.count - 1] == #char "\n" text.count -= 1;

    return text;
}
