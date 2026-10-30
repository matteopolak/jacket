// The run-time and compile-time pieces that `db/schema.jai` (written by `generate`, see
// generate.jai) relies on. docs/postgres/schema.md has the overview.
//
//     #run Pg.check_schema(#file);                       // stale file: compile error
//     #run Pg.check_json_columns(Json_Columns, string.[...]);
//     settings: Pg.Json(Pg.json_column_type("users.settings"), Pg.json_column_options("users.settings"));

#scope_export

// One line of the hand-written `db/json_columns.jai`:
//
// ```jai
// Json_Columns :: Pg.Json_Column.[ .{ "users.settings", Settings, .{ unknown_fields = .DENY } } ];
// ```
//
// `column` is "table.column", with the schema in front when it is not `public`. `options` says how
// strictly that column's documents are decoded; it sits between the connection's options and a
// query's own (json_options.jai).
Json_Column :: struct {
    column: string;
    type: Type;
    options: Json_Options;
}

// The Jai type a json or jsonb column holds, from the `Json_Columns` visible where this is
// called (a macro, so the generated file sees the user's list), else `Json_Raw`. A mapping can
// change without generating the schema again.
json_column_type :: (column: string) -> Type #expand {
    for `Json_Columns if it.column == column return it.type;

    return Json_Raw;
}

// The options of the `Json_Columns` entry for `column`, packed as the second parameter of
// `Json(T, OPTIONS)`; 0 when it has none.
json_column_options :: (column: string) -> u32 #expand {
    for `Json_Columns if it.column == column return pack_json_options(it.options);

    return 0;
}

// Does a file called `name` sit in the folder of `file`? For the generated file to load
// `json_columns.jai` when there is one.
file_exists_beside :: (file: string, name: string) -> bool {
    return Schema_Files.file_exists(join(path_strip_filename(file), name,, temp));
}

SCHEMA_HASH_MARKER :: "// schema hash: ";

// Stops the compile when `file` (the generated schema) was made from other migrations than the
// ones in the project now. Needs no server: it compares hashes of `db/migrations/*.sql`.
check_schema :: (file: string, location := #caller_location) {
    problem := schema_problem(file);
    if !problem return;

    if #compile_time Schema_Compiler.compiler_report(problem, location);
    else log_error("%", problem);
}

// Why `file` no longer matches the migrations, or an empty string when it does.
schema_problem :: (file: string) -> string {
    text, read := read_entire_file(file,, temp);
    if !read return tprint("cannot read %", file);

    recorded := recorded_schema_hash(text);
    if !recorded return tprint("% has no schema hash: run the pg tool's generate", SCHEMA);

    _, current := migrations_of(find_project_root(file));
    if current != recorded return tprint("% is stale: run the pg tool's generate", SCHEMA);

    return "";
}

// The hash a generated file records in its header, or an empty string.
recorded_schema_hash :: (text: string) -> string {
    for line: split(text, "\n") {
        if begins_with(line, SCHEMA_HASH_MARKER) return trim(advance(line, SCHEMA_HASH_MARKER.count));
    }

    return "";
}

// Stops the compile when a `Json_Columns` entry names something that is not a json or jsonb
// column. `known` is every json column of the schema, as the generated file lists them.
check_json_columns :: (mappings: [] Json_Column, known: [] string, location := #caller_location) {
    for mapping: mappings {
        if array_find(known, mapping.column) continue;

        problem := tprint("Json_Columns maps \"%\", which is not a json or jsonb column of % (the json columns are: %)", mapping.column, SCHEMA, join(..known, separator = ", ",, temp));

        if #compile_time Schema_Compiler.compiler_report(problem, location);
        else log_error("%", problem);
    }
}

// How a Postgres enum label is spelled as a Jai enum member, and back. A label that is a plain
// name is used as it is, so `'pending'` is `.pending`. Any other (spaces, dashes, a leading
// digit, a Jai keyword, a double underscore) becomes `__` followed by its letters and digits,
// with every other byte as `_` and two hex digits: `'in progress'` is `__in_20progress`.
enum_member_name :: (label: string) -> string {
    if plain_label(label) return label;

    builder: String_Builder;
    builder.allocator = temp;
    append(*builder, "__");

    for label {
        if (is_alpha(it) && it < 0x80) || is_digit(it) {
            print_to_builder(*builder, "%", slice(label, it_index, 1));
        } else {
            print_to_builder(*builder, "_%", formatInt(it, base = 16, minimum_digits = 2));
        }
    }

    return builder_to_string(*builder,, temp);
}

// The label an enum member name stands for: the inverse of `enum_member_name`.
enum_label :: (name: string) -> string {
    if !begins_with(name, "__") return name;

    bytes: [..] u8;
    bytes.allocator = temp;
    i := 2;

    while i < name.count {
        if name[i] == #char "_" && i + 2 < name.count {
            high := hex_value(name[i + 1]);
            low := hex_value(name[i + 2]);

            if high >= 0 && low >= 0 {
                array_add(*bytes, cast(u8) (high * 16 + low));
                i += 3;
                continue;
            }
        }

        array_add(*bytes, name[i]);
        i += 1;
    }

    return cast(string) bytes;
}

#scope_file

Schema_Files :: #import "File_Utilities";
Schema_Compiler :: #import "Compiler";

plain_label :: (label: string) -> bool {
    if !label.count || is_digit(label[0]) return false;
    if find_index_from_left(label, "__") >= 0 return false;

    for label if !(is_alnum(it) && it < 0x80) && it != #char "_" return false;
    for RESERVED_WORDS if it == label return false;

    return true;
}
