// What the typed query macros ask at compile time: the row struct's text, the statement to send,
// and the checks of arguments and hand-written row types against the described statement.

#scope_module

RESERVED_WORDS :: string.[
    "if", "else", "then", "for", "while", "return", "struct", "enum", "union", "defer", "using",
    "cast", "xx", "null", "true", "false", "case", "break", "continue", "it", "context", "inline",
    "push_context", "remove", "size_of", "type_of", "string", "bool", "int", "float", "void",
];

// The Jai type a Postgres type maps to, by the `base` the describe step resolved.
jai_type_of :: (base: string) -> string {
    if ends_with(base, "[]") return tprint("[] %", jai_type_of(slice(base, 0, base.count - 2)));

    if base == "bool" return "bool";
    if base == "int2" return "s16";
    if base == "int4" return "s32";
    if base == "int8" return "s64";
    if base == "oid" return "u32";
    if base == "float4" return "float32";
    if base == "float8" return "float64";
    if base == "bytea" return "[] u8";
    if base == "uuid" return "Uuid";
    if base == "timestamptz" return "Timestamp";
    if base == "timestamp" return "Local_Timestamp";
    if base == "date" return "Date";
    if base == "time" return "Time";
    if base == "interval" return "Interval";
    if base == "numeric" return "Numeric";
    if base == "json" || base == "jsonb" return "Json_Raw";

    // text, varchar, bpchar, name, enums, citext and every other type read as their text form.
    return "string";
}

// Types whose binary form `get` decodes; a statement with any other column is fetched as text.
binary_decodable :: (base: string) -> bool {
    if ends_with(base, "[]") return binary_decodable(slice(base, 0, base.count - 2));

    names :: string.["bool", "int2", "int4", "int8", "oid", "float4", "float8", "text", "varchar", "bpchar", "name", "bytea", "uuid", "timestamptz", "timestamp", "date", "time", "interval", "numeric", "json", "jsonb"];
    for names if it == base return true;

    return false;
}

// The type of a column's field: the alias's choice, else the mapped type, `Nullable` when the
// column can be NULL.
field_type_text :: (column: Query_Column) -> string {
    text := jai_type_of(column.base);

    if column.jai {
        text = column.jai;
        if begins_with(text, "Pg.") text = advance(text, 3);
    }

    if column.nullable && !begins_with(text, "Nullable(") return tprint("Nullable(%)", text);

    return text;
}

valid_field_name :: (name: string) -> bool {
    if !name.count return false;
    if scan_is_digit(name[0]) return false;

    for name if !(scan_is_word(it) && it != #char "$" && it < 0x80) return false;
    for RESERVED_WORDS if it == name return false;

    return true;
}

// The body of the row struct, with a doc comment per column for hover; or a problem to report.
row_shape_text :: (entry: Query_Entry) -> string, problem: string {
    builder: String_Builder;
    builder.allocator = temp;

    for column, index: entry.columns {
        if !valid_field_name(column.name) {
            return "", tprint("column % (\"%\") cannot be a field name; give it an alias with `as name`", index + 1, column.name);
        }

        for 0..index - 1 if entry.columns[it].name == column.name {
            return "", tprint("two columns are named \"%\"; give one an alias with `as name`", column.name);
        }

        where := ifx column.origin then column.origin else "computed";
        if begins_with(where, "public.") where = advance(where, 7);

        print_to_builder(*builder, "/// %: %, %\n%: %;\n", where, column.type, ifx column.nullable then "nullable" else "not null", column.name, field_type_text(column));
    }

    return builder_to_string(*builder), "";
}

// The entry points the macros call inside #run.

row_shape :: (sql: string, call: Code, sql_index: s64) -> string {
    prepared := prepare_query(sql, call, sql_index);
    shape, problem := row_shape_text(prepared.entry);

    if problem report_at(prepared.sql_node, problem);

    return shape;
}

scalar_type :: (sql: string, call: Code, sql_index: s64) -> string {
    prepared := prepare_query(sql, call, sql_index);

    if !prepared.entry.columns.count {
        report_at(prepared.sql_node, "this statement returns no column");
        return "s64";
    }

    return field_type_text(prepared.entry.columns[0]);
}

rewritten_sql :: (sql: string, call: Code, sql_index: s64) -> string {
    return prepare_query(sql, call, sql_index).rewrite.sql;
}

binary_results :: (sql: string, call: Code, sql_index: s64) -> bool {
    prepared := prepare_query(sql, call, sql_index);

    for prepared.entry.columns if !binary_decodable(it.base) return false;

    return true;
}

// Argument and shape checks for `query`-like (`returns_rows`) and `execute` calls.
check_statement :: (sql: string, call: Code, sql_index: s64, returns_rows: bool) {
    prepared := prepare_query(sql, call, sql_index);

    if returns_rows && !prepared.entry.columns.count {
        report_at(prepared.sql_node, "this statement returns no rows: use Pg.execute, or add a RETURNING clause");
        return;
    }

    check_arguments(prepared, sql_index + 1);
}

check_arguments :: (prepared: *Prepared, first: s64) {
    call_node := prepared.call_node;
    count := call_node.arguments_sorted.count - first;

    if has_spread(call_node, first) return;

    params := prepared.entry.params;

    if count != params.count {
        report_at(prepared.sql_node, tprint("the statement takes % parameter% but % argument% given", params.count, ifx params.count == 1 then "" else "s", count, ifx count == 1 then " was" else "s were"));
        return;
    }

    for 0..count - 1 {
        node := call_node.arguments_sorted[first + it];
        if !node.type continue;

        if !parameter_accepts(params[it].base, node.type, node.kind == .LITERAL) {
            report_at(node, tprint("argument % is %, but parameter $% is %", it + 1, cast(Type) node.type, it + 1, params[it].type));
            return;
        }
    }
}

// Can a value of this Jai type be sent for a parameter of this Postgres base type?
parameter_accepts :: (base: string, info: *Type_Info, literal: bool) -> bool {
    if is_nullable_info(info) {
        inner := nullable_value_type(info);
        if inner == type_info(Null_Marker) return true;

        return parameter_accepts(base, inner, literal);
    }

    if is_json_info(info) || info == type_info(Json_Raw) return base == "json" || base == "jsonb";

    if info.type == .STRING {
        // Sent as text with no type, so the server reads it as whatever the statement wants.
        return !(base == "bool" || base == "int2" || base == "int4" || base == "int8" || base == "oid" || base == "float4" || base == "float8" || base == "numeric" || base == "uuid" || base == "bytea" || base == "date" || base == "time" || base == "timestamp" || base == "timestamptz" || base == "interval" || ends_with(base, "[]"));
    }

    if info.type == .BOOL return base == "bool";

    if info.type == .INTEGER {
        if base == "numeric" || literal && (base == "int2" || base == "int4" || base == "int8" || base == "oid") return true;

        integer := cast(*Type_Info_Integer) info;
        bits := info.runtime_size * 8;

        if base == "int2" return ifx integer.signed then bits <= 16 else bits <= 8;
        if base == "int4" return ifx integer.signed then bits <= 32 else bits <= 16;
        if base == "int8" return ifx integer.signed then bits <= 64 else bits <= 32;
        if base == "oid" return !integer.signed && bits <= 32;

        return false;
    }

    // Sent as its label, so only a Postgres enum takes it.
    if info.type == .ENUM return base == "enum";

    if info.type == .FLOAT {
        if info.runtime_size == 4 || literal return base == "float4" || base == "float8" || (literal && base == "numeric");

        return base == "float8";
    }

    if info == type_info(Uuid) return base == "uuid";
    if info == type_info(Timestamp) return base == "timestamptz";
    if info == type_info(Local_Timestamp) return base == "timestamp";
    if info == type_info(Date) return base == "date";
    if info == type_info(Time) return base == "time";
    if info == type_info(Interval) return base == "interval";
    if info == type_info(Numeric) return base == "numeric";

    if info.type == .ARRAY {
        array := cast(*Type_Info_Array) info;
        if array.element_type == type_info(u8) return base == "bytea";
        if !ends_with(base, "[]") return false;

        return parameter_accepts(slice(base, 0, base.count - 2), array.element_type, false);
    }

    return false;
}

nullable_value_type :: (info: *Type_Info) -> *Type_Info {
    for (cast(*Type_Info_Struct) info).members if it.name == "value" return it.type;
    return info;
}

// The Jai type of a column as `get` would produce it, for comparing with a field.
field_accepts :: (column: Query_Column, info: *Type_Info) -> string {
    nullable_field := is_nullable_info(info);
    if nullable_field info = nullable_value_type(info);

    if column.nullable && !nullable_field {
        return tprint("column \"%\" can be NULL: make the field Nullable(%)", column.name, cast(Type) info);
    }

    if !value_fits(column.base, info) {
        return tprint("column \"%\" is % (%), which a field of type % cannot hold", column.name, column.type, column.base, cast(Type) info);
    }

    return "";
}

value_fits :: (base: string, info: *Type_Info) -> bool {
    if is_json_info(info) || info == type_info(Json_Raw) return base == "json" || base == "jsonb";

    if ends_with(base, "[]") {
        if info.type != .ARRAY return false;

        element := (cast(*Type_Info_Array) info).element_type;
        if is_nullable_info(element) element = nullable_value_type(element);

        return value_fits(slice(base, 0, base.count - 2), element);
    }

    if info.type == .STRING return jai_type_of(base) == "string" || base == "json" || base == "jsonb";
    if info.type == .ENUM return base == "enum";
    if info.type == .BOOL return base == "bool";

    if info.type == .INTEGER {
        integer := cast(*Type_Info_Integer) info;
        bits := info.runtime_size * 8;

        if base == "int2" return bits >= 16 && integer.signed;
        if base == "int4" return bits >= 32 && integer.signed;
        if base == "int8" return bits >= 64 && integer.signed;
        if base == "oid" return bits >= 32;

        return false;
    }

    if info.type == .FLOAT {
        if base == "float4" return true;
        if base == "float8" return info.runtime_size == 8;

        return false;
    }

    if info.type == .ARRAY return base == "bytea" && (cast(*Type_Info_Array) info).element_type == type_info(u8);

    if info == type_info(Uuid) return base == "uuid";
    if info == type_info(Timestamp) return base == "timestamptz";
    if info == type_info(Local_Timestamp) return base == "timestamp";
    if info == type_info(Date) return base == "date";
    if info == type_info(Time) return base == "time";
    if info == type_info(Interval) return base == "interval";
    if info == type_info(Numeric) return base == "numeric";

    return false;
}

// Checks every field of `T` against the statement's columns; reports the first mismatch.
check_statement_as :: (T: Type, sql: string, call: Code, sql_index: s64) {
    prepared := prepare_query(sql, call, sql_index);

    if !prepared.entry.columns.count {
        report_at(prepared.sql_node, "this statement returns no rows: use Pg.execute, or add a RETURNING clause");
        return;
    }

    info := cast(*Type_Info_Struct) type_info(T);

    for member: info.members {
        if (member.flags & .CONSTANT) || (member.flags & .USING) continue;

        column: *Query_Column;
        for *prepared.entry.columns if it.name == member.name column = it;

        if !column {
            names := join_names(prepared.entry);
            report_at(prepared.sql_node, tprint("%.% has no column of that name; the columns are %", T, member.name, names));
            return;
        }

        why := field_accepts(column.*, member.type);
        if why {
            report_at(prepared.sql_node, tprint("%.%: %", T, member.name, why));
            return;
        }
    }

    check_arguments(prepared, sql_index + 1);
}

join_names :: (entry: Query_Entry) -> string {
    builder: String_Builder;
    builder.allocator = temp;

    for entry.columns {
        if it_index append(*builder, ", ");
        append(*builder, it.name);
    }

    return builder_to_string(*builder);
}

// `rows.map = s64.[...]`: for each field of T, its column.
field_map_code :: (T: Type, sql: string, call: Code, sql_index: s64) -> string {
    prepared := prepare_query(sql, call, sql_index);
    info := cast(*Type_Info_Struct) type_info(T);

    builder: String_Builder;
    builder.allocator = temp;

    append(*builder, "rows.map = s64.[");
    first := true;

    for member: info.members {
        if (member.flags & .CONSTANT) || (member.flags & .USING) continue;

        index := 0;
        for prepared.entry.columns if it.name == member.name index = it_index;

        print_to_builder(*builder, "%%", ifx first then "" else ", ", index);
        first = false;
    }

    append(*builder, "];");

    return builder_to_string(*builder);
}
