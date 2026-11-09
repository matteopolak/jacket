// Decodes the next row by header names or declaration order; pointers are optional.
// Example: row, ok, error := csv_next_row(User, *reader); if ok csv_row_free(*row).
csv_next_row :: ($T: Type, r: *Csv_Reader) -> T, bool, Csv_Error {
    value: T;
    fields, ok, error := csv_next(r);

    if !ok return value, false, error;

    if !decode_row(r, fields, *value) {
        csv_row_free(*value);
        return value, false, r.error;
    }

    return value, true, .{};
}

// Releases strings and optional pointers owned by a typed row; resets decoded fields.
// Example: csv_row_free(*row).
csv_row_free :: (value: *$T) {
    #insert -> string {
        return generate_csv(T, .FREE);
    };
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
decode_row :: (r: *Csv_Reader, fields: [] string, value: *$T) -> bool {
    #insert -> string {
        return generate_csv(T, .DECODE);
    };
}

locate_error :: (r: *Csv_Reader, index: int) {
    offset := r.record_offset;
    line := r.record_line;
    line_start := r.record_line_start;

    if index >= 0 && index < r.cells.count {
        cell := r.cells[index];
        offset = cell.offset;
        line = cell.line;
        line_start = cell.line_start;
    }

    r.error.offset = offset;
    r.error.line = line;
    r.error.column = offset - line_start + 1;
}

column_index :: (r: *Csv_Reader, name: string, order: int) -> int {
    if !r.options.header return order;

    // Rows of one type ask for the same names in the same order, so each member keeps its answer.
    // Names are literals: the same address and length mean the same text.
    if order < r.columns.count {
        known := r.columns[order];
        if known.name.data == name.data && known.name.count == name.count return known.index;
    }

    index := Hash_Map.get(*r.header_index, name, -1);

    while r.columns.count <= order array_add(*r.columns, .{});

    r.columns[order] = .{ name, index };

    return index;
}

read_cell :: (r: *Csv_Reader, text: string, out: *$T, name: string) -> bool {
    #if type_info(T).type == .POINTER {
        if !text.count {
            out.* = null;
            return true;
        }

        out.* = New(type_of(out.*.*));

        return read_cell(r, text, out.*, name);
    } else #if T == string {
        out.* = copy_string(text);
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
