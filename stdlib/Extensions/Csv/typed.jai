// Releases strings and optional pointers owned by a typed row; resets decoded fields. Not for a
// row read with `Csv_Options.borrow`: use the reader form below.
// Example: csv_row_free(*row).
csv_row_free :: (value: *$T) {
    #insert -> string {
        return generate_csv(T, .FREE);
    };
}

// Releases a row of `reader` the way its options allow: with `borrow` only the optional pointers
// and not the strings, which may be slices of the input; otherwise as csv_row_free(*row).
// Example: csv_row_free(*reader, *row).
csv_row_free :: (reader: *Csv_Reader($Row), value: *Row) {
    if reader.options.borrow {
        #insert -> string {
            return generate_csv(Row, .FREE_BORROWED);
        };
    } else csv_row_free(value);
}

// Writes a struct array with a header, honoring @csv(name=...) and @csv(skip).
// Example: error := csv_write_rows(*builder, users).
csv_write_rows :: (out: *String_Builder, rows: [] $T, options := Csv_Options.{}) -> Csv_Error {
    fields: [..] string;

    defer array_free(fields);

    #insert -> string {
        return generate_csv(T, .HEADER);
    };

    error := csv_write_record(out, fields, options);
    if error.kind != .NONE return error;

    for value: rows {
        fields.count = 0;

        {
            #insert -> string {
                return generate_csv(T, .ENCODE);
            };

            defer {
                #insert -> string {
                    return generate_csv(T, .CLEAN);
                };
            }

            error = csv_write_record(out, fields, options);
        }

        if error.kind != .NONE return error;
    }

    return .{};
}

#scope_module
decode_row :: (r: *Csv_Core, fields: [] string, value: *$T) -> bool {
    #insert -> string {
        return generate_csv(T, .DECODE);
    };
}

locate_error :: (r: *Csv_Core, index: int) {
    offset := r.record_offset;
    line := r.record_line;
    line_start := r.record_line_start;

    if index >= 0 && index < r.fields.count {
        if r.record_scanned {
            offset, line, line_start = field_position(r, index);
        } else {
            cell := r.cells[index];
            offset = cell.offset;
            line = cell.line;
            line_start = cell.line_start;
        }
    }

    r.error.offset = offset;
    r.error.line = line;
    r.error.column = offset - line_start + 1;
}

// Finds the document column of each member of `Row`, once per reader: by header name, or by
// declaration order without a header. A member the header lacks gets -1, which decoding reports.
bind_columns :: (r: *Csv_Reader($Row)) {
    #insert -> string {
        return generate_csv(Row, .BIND);
    };

    r.bound = true;
}

member_column :: (r: *Csv_Core, name: string, order: int) -> int {
    if !r.options.header return order;

    return Hash_Map.get(*r.header_index, name, -1);
}

read_cell :: (r: *Csv_Core, text: string, out: *$T, name: string) -> bool {
    #if type_info(T).type == .POINTER {
        if !text.count {
            out.* = null;
            return true;
        }

        out.* = New(type_of(out.*.*));

        return read_cell(r, text, out.*, name);
    } else #if T == string {
        // With `borrow` the input is a slice and outlives the row; text read through the buffer
        // (doubled quotes) is outside it and is copied.
        if r.options.borrow && text.data >= r.input.data && text.data < r.input.data + r.input.count out.* = text;
        else out.* = copy_string(text);

        return true;
    } else #if T == bool {
        if text == "true" {
            out.* = true;
            return true;
        }

        if text == "false" {
            out.* = false;
            return true;
        }
    } else #if type_info(T).type == .INTEGER {
        number, ok := parse_integer(text, T);
        if ok {
            out.* = number;
            return true;
        }
    } else #if type_info(T).type == .FLOAT {
        number, ok := parse_float(text, T);
        if ok {
            out.* = number;
            return true;
        }
    }

    fail(r, .TYPE, "invalid or empty typed column");
    r.error.column_name = name;

    return false;
}

// Reads `text` as a whole decimal number of type `T`, with the result of `string_to_int(text, 10, T)`
// and no text left over. Plain digits with an optional sign (up to 18, so the value cannot overflow
// 64 bits and needs no check per digit) are read here from the bytes; anything else, such as
// white space, a number out of range or a sign on an unsigned type, goes to `string_to_int`.
parse_integer :: (text: string, $T: Type) -> T, bool {
    data := text.data;
    count := text.count;
    at := 0;
    negative := false;

    if count > 0 && (data[0] == #char "-" || data[0] == #char "+") {
        negative = data[0] == #char "-";
        at = 1;
    }

    if count - at >= 1 && count - at <= 18 {
        value: u64;

        while at < count {
            digit := cast(u64) data[at] - 48;
            if digit > 9 break;

            value = value * 10 + digit;
            at += 1;
        }

        if at == count {
            info := cast(*Type_Info_Integer) type_info(T);
            limit: u64 = 0xffffffffffffffff;
            bits := size_of(T) * 8;

            if bits < 64 limit = ((cast(u64) 1) << cast(u64) bits) - 1;
            if info.signed limit >>= 1;

            if !negative && value <= limit return cast,trunc(T) value, true;
            if negative && info.signed && value <= limit + 1 return cast,trunc(T) (0 - value), true;
        }
    }

    number, ok, rest := string_to_int(text, 10, T);

    return number, ok && !rest.count;
}

// The powers of ten a `float64` holds exactly.
EXACT_POWERS :: float64.[1, 1e1, 1e2, 1e3, 1e4, 1e5, 1e6, 1e7, 1e8, 1e9, 1e10, 1e11, 1e12, 1e13, 1e14, 1e15, 1e16, 1e17, 1e18, 1e19, 1e20, 1e21, 1e22];

// Reads `text` as a whole number of type `T`, with the result of `string_to_float64` (`string_to_float`
// for `float32`) and no text left over. A short plain decimal (`12`, `-3.14159`) has an exact integer
// mantissa and an exact power of ten, so one division rounds it correctly: that is read here from the
// bytes. Anything else (an exponent, a long mantissa, a missing digit, white space, `inf`) goes to the
// general reader.
parse_float :: (text: string, $T: Type) -> T, bool {
    data := text.data;
    count := text.count;
    at := 0;
    negative := false;

    if count > 0 && (data[0] == #char "-" || data[0] == #char "+") {
        negative = data[0] == #char "-";
        at = 1;
    }

    // `float32` holds integers to 2^24 and powers of ten to 10^10 exactly; `float64` to 2^53 and 10^22.
    max_digits := ifx T == float32 then 7 else 15;
    max_fraction := ifx T == float32 then 10 else 22;

    mantissa: u64;
    whole := 0;
    fraction := 0;

    while at < count {
        digit := cast(u64) data[at] - 48;
        if digit > 9 break;

        mantissa = mantissa * 10 + digit;
        whole += 1;
        at += 1;

        if whole > max_digits break;
    }

    if whole >= 1 && whole <= max_digits && at < count && data[at] == #char "." {
        at += 1;

        while at < count {
            digit := cast(u64) data[at] - 48;
            if digit > 9 break;

            mantissa = mantissa * 10 + digit;
            fraction += 1;
            at += 1;

            if whole + fraction > max_digits break;
        }

        if fraction == 0 whole = 0;
    }

    if at == count && whole >= 1 && whole + fraction <= max_digits && fraction <= max_fraction {
        number: T;

        #if T == float32 number = cast(float32) mantissa / cast(float32) EXACT_POWERS[fraction];
        else number = cast(float64) mantissa / EXACT_POWERS[fraction];

        return ifx negative then -number else number, true;
    }

    number: T;
    ok: bool;
    rest: string;

    #if T == float32 number, ok, rest = string_to_float(text);
    else number, ok, rest = string_to_float64(text);

    return number, ok && !rest.count;
}

free_cell :: (value: *$T) {
    #if type_info(T).type == .POINTER {
        if value.* {
            free_cell(value.*);
            free(value.*);
            value.* = null;
        }
    } else #if T == string {
        if value.data free(value.data);
        value.* = "";
    }
}

// Releases an optional pointer but not what it points at, which a borrowing row does not own.
free_pointer :: (value: *$T) {
    if value.* {
        free(value.*);
        value.* = null;
    }
}

write_cell :: (value: $T) -> string {
    #if type_info(T).type == .POINTER {
        if !value return "";
        return write_cell(value.*);
    } else #if T == string {
        return value;
    } else #if type_info(T).type == .FLOAT {
        return sprint("%", formatFloat(value, mode = .SHORTEST));
    } else return sprint("%", value);
}
