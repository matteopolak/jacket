// Is the short name for [read_silly_alias].
read :: read_silly_alias;

// Reads a block that starts with a 32-bit little-endian length from the front of `s`, and advances `s` past
// it.
//
// Returns the block, which points into `s`. Returns an empty string when the prefix or the block runs past the
// end; that also logs an error and fails an assertion and leaves `s` unchanged.
read_length_prefixed_block :: (s: *string) -> string {
    if s.count < 4 {
        wire_failure("binary length prefix exceeds input");
        return "";
    }

    remaining := s.*;
    count := read(u32, *remaining);

    if cast(u64) count > cast(u64) remaining.count {
        wire_failure("length-prefixed binary block exceeds input");
        return "";
    }

    value := read_block(*remaining, cast(int) count);

    s.* = remaining;

    return value;
}

// Reads an integer of type `Int_Type` (one, two, four or eight bytes) from the front of `s`.
//
// The bytes are little-endian, or big-endian with `BIG_ENDIAN`. Advances `s`. Too little input returns zero
// with a logged error that also fails an assertion. The little-endian read stores bytes in order, so it
// matches the host only on little-endian CPUs.
read_silly_alias :: ($Int_Type: Type, s: *string, $BIG_ENDIAN := false) -> Int_Type {
    #assert type_info(Int_Type).type == .INTEGER;
    #assert size_of(Int_Type) == 1 || size_of(Int_Type) == 2 || size_of(Int_Type) == 4 || size_of(Int_Type) == 8;
    result: Int_Type;
    count := size_of(Int_Type);

    if s.count < count {
        wire_failure("binary integer read exceeds input");
        return result;
    }

    // Current supported compiler targets have little-endian storage.
    destination := cast(*u8) *result;

    for index: 0..count - 1 {
        #if BIG_ENDIAN destination[index] = s.data[count - index - 1];
        else destination[index] = s.data[index];
    }

    s.data += count;
    s.count -= count;

    return result;
}

// Takes the first `length` bytes of `s` as a string that points into `s`, and advances `s` past them.
//
// A negative or too long `length` returns an empty string with a logged error that also fails an assertion.
read_block :: (s: *string, length: int) -> string {
    if length < 0 || length > s.count {
        wire_failure("binary block read exceeds input");
        return "";
    }

    result := string.{ length, s.data };

    s.data += length;
    s.count -= length;

    return result;
}
