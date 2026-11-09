#scope_module

Csv_Generation :: enum {
    DECODE;
    BIND;
    FREE_BORROWED;
    FREE;
    HEADER;
    ENCODE;
    CLEAN;
}

generate_csv :: (T: Type, mode: Csv_Generation) -> string {
    info := type_info(T);
    if info.type != .STRUCT {
        compiler_report("Csv requires a struct");
        return "";
    }

    out: String_Builder;
    if mode == .FREE || mode == .FREE_BORROWED append(*out, "defaults: type_of(value.*);\n");
    if mode == .DECODE append(*out, "columns := r.columns.data;\n");

    order := 0;

    for member: (cast(*Type_Info_Struct) info).members {
        if member.flags & .CONSTANT continue;

        name := member.name;
        skip := false;

        for note: member.notes {
            if !begins_with(note, "csv(") || !ends_with(note, ")") continue;

            rest := slice(note, 4, note.count - 5);
            while rest.count {
                found, piece, after := split_note_option(rest);
                piece = trim(piece);
                if piece == "skip" skip = true;
                else if begins_with(piece, "name=") {
                    name = slice(piece, 5, piece.count - 5);
                    decoded, ok := unquote(name);
                    if !ok compiler_report("Csv: invalid quoted name");
                    else name = decoded;
                } else compiler_report(tprint("Csv: unknown note on %", member.name));

                rest = ifx found then after else "";
            }
        }

        if skip continue;

        leaf := member.type;
        if leaf.type == .POINTER leaf = (cast(*Type_Info_Pointer) leaf).pointer_to;

        if leaf.type != .INTEGER && leaf.type != .FLOAT && leaf.type != .BOOL && leaf.type != .STRING {
            compiler_report(tprint("Csv: unsupported field %", member.name));
            continue;
        }

        if leaf.type == .INTEGER && leaf.runtime_size > 8 compiler_report("Csv: integers wider than 64 bits are unsupported");
        if !name.count compiler_report("Csv: empty field name");
        if leaf.type == .FLOAT && leaf.runtime_size != 4 && leaf.runtime_size != 8 compiler_report("Csv: only float32 and float64 are supported");

        key := jai_quote(name);

        if mode == .DECODE {
            print(*out, "{\nindex := columns[%];\n", order);
            print(*out, "if index < 0 || index >= fields.count { fail(r, .HEADER, \"missing column\"); r.error.column_name = %; locate_error(r, -1); return false; }\n", key);
            print(*out, "if !read_cell(r, fields[index], *value.%, %) { locate_error(r, index); return false; }\n}\n", member.name, key);
        } else if mode == .BIND {
            print(*out, "array_add(*r.columns, member_column(*r.core, %, %));\n", key, order);
        } else if mode == .FREE_BORROWED {
            // Strings may be slices of the input, so only the pointers are released.
            if member.type.type == .POINTER print(*out, "if value.% != defaults.% free_pointer(*value.%);\n", member.name, member.name, member.name);

            print(*out, "value.% = defaults.%;\n", member.name, member.name);
        } else if mode == .FREE {
            if member.type.type == .STRING {
                print(*out, "if value.%.data != defaults.%.data free_cell(*value.%);\n", member.name, member.name, member.name);
            } else if member.type.type == .POINTER {
                print(*out, "if value.% != defaults.% free_cell(*value.%);\n", member.name, member.name, member.name);
            }

            print(*out, "value.% = defaults.%;\n", member.name, member.name);
        } else if mode == .HEADER {
            print(*out, "array_add(*fields, %);\n", key);
        } else if mode == .CLEAN {
            if leaf.type != .STRING print(*out, "if fields[%].count free(fields[%].data);\n", order, order);
        } else {
            print(*out, "array_add(*fields, write_cell(value.%));\n", member.name);
        }

        order += 1;
    }

    if mode == .DECODE append(*out, "return true;\n");

    return builder_to_string(*out,, temp);
}
