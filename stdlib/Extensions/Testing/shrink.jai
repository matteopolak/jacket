// Shrinking: from an input that fails a property to a smaller one that still fails. Every value
// has a short list of smaller candidates (an integer toward 0, a string or array cut down or
// with one element simplified); the first candidate that still fails replaces the value, and
// the search starts over from it, until no candidate fails or the attempts run out.

#scope_module

// Smaller inputs for each argument in turn, until none fails. Returns how many were accepted.
shrink :: (t: *Test, invoke: (t: *Test, slots: [] Property_Arg) -> bool, slots: [] Property_Arg, custom: [] Custom_Generator) -> s64 {
    attempts := 0;
    steps := 0;
    progress := true;

    while progress && attempts < MAX_SHRINK_ATTEMPTS {
        progress = false;

        for slot: slots {
            size := slot.info.runtime_size;
            saved := talloc(max(size, 1));

            candidates := smaller_values(slot.info, slot.pointer, custom);

            for candidate: candidates {
                if attempts >= MAX_SHRINK_ATTEMPTS break;

                attempts += 1;

                copy_bytes(saved, slot.pointer, size);
                copy_bytes(slot.pointer, candidate, size);

                if !holds(t, invoke, slots) {
                    steps += 1;
                    progress = true;
                    break;
                }

                copy_bytes(slot.pointer, saved, size);
            }

            if progress break;
        }
    }

    return steps;
}

// Copies the value at `source` to `destination` (room for the same type) with its arrays,
// and what pointers lead to, in new memory.
deep_copy :: (info: *Type_Info, source: *void, destination: *void) {
    copy_bytes(destination, source, info.runtime_size);

    if info.type == {
        case .ARRAY;
            array := cast(*Type_Info_Array) info;
            element_type, count, data := array_view(Any.{ info, source });
            stride := element_type.runtime_size;
            target := ifx array.array_type == .FIXED then destination else talloc(max(count * stride, 1));

            for 0..count - 1 deep_copy(element_type, data + it * stride, target + it * stride);

            if array.array_type == .VIEW {
                (cast(*[] u8) destination).data = target;
            } else if array.array_type == .RESIZABLE {
                resizable := cast(*[..] u8) destination;
                resizable.data = target;
                resizable.allocated = count;
                resizable.allocator = temp;
            }
        case .STRUCT;
            for (cast(*Type_Info_Struct) info).members {
                if it.flags & .CONSTANT continue;
                deep_copy(it.type, source + it.offset_in_bytes, destination + it.offset_in_bytes);
            }
        case .POINTER;
            pointee := (cast(*Type_Info_Pointer) info).pointer_to;
            target := (cast(**void) source).*;

            if target && pointee && pointee.type != .VOID {
                copy := talloc(max(pointee.runtime_size, 1));
                deep_copy(pointee, target, copy);
                (cast(**void) destination).* = copy;
            }
        case .VARIANT;
            deep_copy((cast(*Type_Info_Variant) info).variant_of, source, destination);
    }
}

#scope_file

Blocks :: [..] *void;

copy_bytes :: (destination: *void, source: *void, count: s64) {
    memcpy(destination, source, count);
}

// A new block holding a copy of the value at `source`.
copy_of :: (info: *Type_Info, source: *void) -> *void {
    block := talloc(max(info.runtime_size, 1));
    copy_bytes(block, source, info.runtime_size);

    return block;
}

// The values smaller than the one at `value`, as new blocks, smallest first.
smaller_values :: (info: *Type_Info, value: *void, custom: [] Custom_Generator) -> Blocks {
    blocks: Blocks;
    blocks.allocator = temp;

    for custom if it.info == info return blocks;

    if info.type == {
        case .INTEGER;
            smaller_integers(*blocks, cast(*Type_Info_Integer) info, value);
        case .BOOL;
            if (cast(*bool) value).* {
                block := cast(*bool) talloc(1);
                block.* = false;
                array_add(*blocks, block);
            }
        case .ENUM;
            smaller_enums(*blocks, cast(*Type_Info_Enum) info, value);
        case .FLOAT;
            smaller_floats(*blocks, info, value);
        case .STRING;
            smaller_strings(*blocks, (cast(*string) value).*);
        case .ARRAY;
            smaller_arrays(*blocks, cast(*Type_Info_Array) info, value, custom);
        case .STRUCT;
            for member: (cast(*Type_Info_Struct) info).members {
                if member.flags & .CONSTANT continue;

                inner := smaller_values(member.type, value + member.offset_in_bytes, custom);

                for inner {
                    block := copy_of(info, value);
                    copy_bytes(block + member.offset_in_bytes, it, member.type.runtime_size);
                    array_add(*blocks, block);
                }
            }
        case .POINTER;
            if (cast(**void) value).* {
                block := cast(**void) talloc(size_of(*void));
                block.* = null;
                array_add(*blocks, block);
            }
        case .VARIANT;
            return smaller_values((cast(*Type_Info_Variant) info).variant_of, value, custom);
    }

    return blocks;
}

add_integer :: (blocks: *Blocks, info: *Type_Info_Integer, value: Integer) {
    block := talloc(max(info.runtime_size, 1));
    write_integer(info, block, value);
    array_add(blocks, block);
}

smaller_integers :: (blocks: *Blocks, info: *Type_Info_Integer, value: *void) {
    current := integer_value(Any.{ info, value });
    if current.magnitude == 0 return;

    add_integer(blocks, info, .{ false, 0 });

    largest := largest_positive(info);

    // A negative number is simpler as its positive.
    if current.negative && current.magnitude <= largest {
        add_integer(blocks, info, .{ false, current.magnitude });
    }

    // Closer to 0 by half, a quarter, an eighth... of the way, then by one.
    step := current.magnitude / 2;
    last := cast(u64) 0;

    while step > 0 {
        smaller := current.magnitude - step;
        if smaller != last && smaller != 0 {
            add_integer(blocks, info, .{ current.negative, smaller });
            last = smaller;
        }

        step /= 2;
    }

    if current.magnitude > 1 && current.magnitude - 1 != last {
        add_integer(blocks, info, .{ current.negative, current.magnitude - 1 });
    }
}

smaller_enums :: (blocks: *Blocks, info: *Type_Info_Enum, value: *void) {
    if !info.values.count return;

    current := integer_value(Any.{ info, value });
    first := integer_of_bits(info.values[0]);

    if current.negative == first.negative && current.magnitude == first.magnitude return;

    block := talloc(max(info.runtime_size, 1));

    write_integer(info.internal_type, block, first);
    array_add(blocks, block);
}

add_float :: (blocks: *Blocks, info: *Type_Info, value: float64) {
    block := talloc(max(info.runtime_size, 1));

    if info.runtime_size == 4 {
        (cast(*float32) block).* = cast(float32) value;
    } else {
        (cast(*float64) block).* = value;
    }

    array_add(blocks, block);
}

// Floats up to this are cast to an integer to drop their fraction.
LARGEST_WHOLE :: 9.0e18;

smaller_floats :: (blocks: *Blocks, info: *Type_Info, value: *void) {
    current := float_value(Any.{ info, value });

    // NaN and infinities are not smaller than anything and not worth chasing.
    if current != current || current - current != 0 return;

    magnitude := ifx current < 0 then -current else current;
    if magnitude == 0 return;

    add_float(blocks, info, 0);
    if current < 0 add_float(blocks, info, magnitude);

    if magnitude < LARGEST_WHOLE {
        whole := cast(float64) cast(s64) current;
        if whole != current add_float(blocks, info, whole); // jailint: allow(float_equality) exactly whole or not
    }

    // Closer to 0 by half, a quarter, an eighth... of the way, down to one.
    step := magnitude / 2;
    while step >= 1 {
        add_float(blocks, info, ifx current < 0 then current + step else current - step);
        step /= 2;
    }
}

// How many bytes the character at `index` of `text` takes.
character_length :: (text: string, index: s64) -> s64 {
    lead := text[index];
    length := 1;

    if (lead & 0xE0) == 0xC0 length = 2;
    else if (lead & 0xF0) == 0xE0 length = 3;
    else if (lead & 0xF8) == 0xF0 length = 4;

    return min(length, text.count - index);
}

// The byte offsets where characters of `text` start.
character_starts :: (text: string) -> [] s64 {
    starts: [..] s64;
    starts.allocator = temp;

    i := 0;
    while i < text.count {
        array_add(*starts, i);
        i += character_length(text, i);
    }

    return starts;
}

// Which of `count` positions to try changing: all when there are few, else the ends.
POSITIONS :: 6;

position_list :: (count: s64) -> [] s64 {
    positions: [..] s64;
    positions.allocator = temp;

    if count <= POSITIONS * 2 {
        for 0..count - 1 array_add(*positions, it);
    } else {
        for 0..POSITIONS - 1 array_add(*positions, it);
        for count - POSITIONS..count - 1 array_add(*positions, it);
    }

    return positions;
}

add_string :: (blocks: *Blocks, text: string) {
    block := cast(*string) talloc(size_of(string));
    block.* = text;
    array_add(blocks, block);
}

smaller_strings :: (blocks: *Blocks, text: string) {
    if !text.count return;

    add_string(blocks, "");

    starts := character_starts(text);

    if starts.count >= 2 {
        middle := starts[starts.count / 2];
        add_string(blocks, copy_string(string.{ middle, text.data },, temp));
        add_string(blocks, copy_string(string.{ text.count - middle, text.data + middle },, temp));
    }

    // One character left out, or made an `a`.
    for position: position_list(starts.count) {
        start := starts[position];
        length := character_length(text, start);
        builder: String_Builder;
        builder.allocator = temp;
        append(*builder, string.{ start, text.data });
        append(*builder, string.{ text.count - start - length, text.data + start + length });
        add_string(blocks, builder_to_string(*builder,, temp));
    }

    for position: position_list(starts.count) {
        start := starts[position];
        length := character_length(text, start);

        if length == 1 && text[start] == #char "a" continue;

        builder: String_Builder;
        builder.allocator = temp;
        append(*builder, string.{ start, text.data });
        append(*builder, "a");
        append(*builder, string.{ text.count - start - length, text.data + start + length });
        add_string(blocks, builder_to_string(*builder,, temp));
    }
}

// An array of `info`'s kind holding the `count` elements at `data`, as a new block.
array_block :: (info: *Type_Info_Array, count: s64, data: *void) -> *void {
    block := talloc(max(info.runtime_size, 1));

    if info.array_type == .VIEW {
        (cast(*[] u8) block).* = .{ count, data };
    } else {
        resizable := cast(*[..] u8) block;
        resizable.count = count;
        resizable.data = data;
        resizable.allocated = count;
        resizable.allocator = temp;
    }

    return block;
}

smaller_arrays :: (blocks: *Blocks, info: *Type_Info_Array, value: *void, custom: [] Custom_Generator) {
    element_type, count, data := array_view(Any.{ info, value });
    stride := element_type.runtime_size;

    if info.array_type != .FIXED && count > 0 {
        array_add(blocks, array_block(info, 0, talloc(1)));

        if count >= 2 {
            middle := count / 2;
            array_add(blocks, array_block(info, middle, copy_elements(data, 0, middle, stride)));
            array_add(blocks, array_block(info, count - middle, copy_elements(data, middle, count, stride)));
        }

        for position: position_list(count) {
            kept := talloc(max((count - 1) * stride, 1));
            copy_bytes(kept, data, position * stride);
            copy_bytes(kept + position * stride, data + (position + 1) * stride, (count - position - 1) * stride);
            array_add(blocks, array_block(info, count - 1, kept));
        }
    }

    // One element made smaller.
    for position: position_list(count) {
        element_value := data + position * stride;

        for smaller: smaller_values(element_type, element_value, custom) {
            if info.array_type == .FIXED {
                block := copy_of(info, value);
                copy_bytes(block + position * stride, smaller, stride);
                array_add(blocks, block);
            } else {
                copied := copy_elements(data, 0, count, stride);
                copy_bytes(copied + position * stride, smaller, stride);
                array_add(blocks, array_block(info, count, copied));
            }
        }
    }
}

// The elements `from` up to `to` of the array at `data`, as new memory.
copy_elements :: (data: *void, from: s64, to: s64, stride: s64) -> *void {
    block := talloc(max((to - from) * stride, 1));
    copy_bytes(block, data + from * stride, (to - from) * stride);

    return block;
}
