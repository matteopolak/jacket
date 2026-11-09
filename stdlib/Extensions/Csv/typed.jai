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
        number, ok, rest := string_to_int(text, 10, T);
        if ok && !rest.count {
            out.* = number;
            return true;
        }
    } else #if type_info(T).type == .FLOAT {
        number: T;
        ok: bool;
        rest: string;

        #if T == float32 number, ok, rest = string_to_float(text);
        else number, ok, rest = string_to_float64(text);

        if ok && !rest.count {
            out.* = cast(T) number;
            return true;
        }
    }

    fail(r, .TYPE, "invalid or empty typed column");
    r.error.column_name = name;

    return false;
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
