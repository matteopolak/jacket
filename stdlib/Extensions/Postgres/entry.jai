// What the describe service learns about one statement, and the JSON form it takes in the
// committed cache (`db/queries/<statement id>.json`).
//
//     {
//       "format": 1,                     ENTRY_FORMAT; an entry of another format is not read
//       "id": "9f2c...",                 full SHA-256 of "sql"; the file name is its first 16 digits
//       "sql": "select ... where u.id = $1",
//       "schema_hash": "...",            of the migrations that produced the description
//       "server_major": 17,
//       "search_path": "",
//       "deps": { "r:public.users": "a41b...", "t:public.mood": "77e0..." },
//       "deps_complete": true,
//       "params": [ { "type": "bigint", "base": "int8" } ],
//       "columns": [ { "name": "id", "type": "bigint", "base": "int8", "nullable": false,
//                      "origin": "public.users.id" } ]
//     }
//
// `deps` keys name a relation ("r:schema.name"), a type ("t:schema.name") or a function
// ("f:schema.name(args)"). A column's `jai` is the type the alias asked for ("created_at: Timestamp").

#scope_module

Json_Value :: Json_Module.Json_Value;
json_object :: Json_Module.json_object;
json_array :: Json_Module.json_array;
json_string :: Json_Module.json_string;
json_int :: Json_Module.json_int;
json_bool :: Json_Module.json_bool;
json_set :: Json_Module.json_set;
json_get :: Json_Module.json_get;
json_at :: Json_Module.json_at;
json_push :: Json_Module.json_push;
json_count :: Json_Module.json_count;
json_as_string :: Json_Module.json_as_string;
json_as_s64 :: Json_Module.json_as_s64;
json_as_bool :: Json_Module.json_as_bool;

#scope_export

// The version of what an entry holds and how its statement is rewritten. Raise it when the
// description changes shape, or when the placeholder rewrite, the alias syntax or anything else
// that decides what a statement means does: every entry written before is then a miss, described
// again when a database is there (or by the pg tool's prepare), and an error offline.
ENTRY_FORMAT :: 1;

Forced_Nullability :: enum u8 {
    NONE;
    NON_NULL; // the alias said `name!`
    NULLABLE; // the alias said `name?`
}

Query_Type :: struct {
    type: string; // how Postgres shows it: "bigint", "character varying(20)", "mood"
    base: string; // what decides the Jai type: domains resolved, arrays as "int4[]", enums as "enum"
}

Query_Column :: struct {
    name: string; // without the alias suffixes
    type: string;
    base: string;
    nullable: bool;
    origin: string; // schema.table.column, for a column read straight from a table
    jai: string; // the alias's Jai type, or empty
    forced: Forced_Nullability;
    oid: u32; // in the database that described it; not stored
}

Query_Dep :: struct {
    key: string;
    fingerprint: string;
}

Query_Entry :: struct {
    format: s64; // ENTRY_FORMAT when written by this compiler; 0 when the file has none
    sql: string;
    schema_hash: string;
    server_major: s64;
    search_path: string;
    deps: [..] Query_Dep;
    deps_complete: bool;
    params: [..] Query_Type;
    columns: [..] Query_Column;
}

entry_to_json :: (entry: Query_Entry) -> Json_Value {
    object := json_object();

    json_set(*object, "format", json_int(entry.format));
    json_set(*object, "id", json_string(statement_digest_hex(entry.sql)));
    json_set(*object, "sql", json_string(entry.sql));
    json_set(*object, "schema_hash", json_string(entry.schema_hash));
    json_set(*object, "server_major", json_int(entry.server_major));
    json_set(*object, "search_path", json_string(entry.search_path));

    deps := json_object();
    for entry.deps json_set(*deps, it.key, json_string(it.fingerprint));

    json_set(*object, "deps", deps);
    json_set(*object, "deps_complete", json_bool(entry.deps_complete));

    params := json_array();

    for entry.params {
        item := json_object();
        json_set(*item, "type", json_string(it.type));
        json_set(*item, "base", json_string(it.base));
        json_push(*params, item);
    }

    json_set(*object, "params", params);

    columns := json_array();

    for entry.columns {
        item := json_object();

        json_set(*item, "name", json_string(it.name));
        json_set(*item, "type", json_string(it.type));
        json_set(*item, "base", json_string(it.base));
        json_set(*item, "nullable", json_bool(it.nullable));
        if it.origin json_set(*item, "origin", json_string(it.origin));
        if it.jai json_set(*item, "jai", json_string(it.jai));

        json_push(*columns, item);
    }

    json_set(*object, "columns", columns);

    return object;
}

// Reads an entry back from its JSON. False when the document is not shaped like one.
entry_from_json :: (document: *Json_Value) -> Query_Entry, bool {
    entry: Query_Entry;
    if document.kind != .OBJECT return entry, false;

    format := json_get(document, "format");
    if format entry.format, _ = json_as_s64(format);

    entry.sql = copy_string(string_member(document, "sql"));
    entry.schema_hash = copy_string(string_member(document, "schema_hash"));
    entry.search_path = copy_string(string_member(document, "search_path"));

    major := json_get(document, "server_major");
    if major entry.server_major, _ = json_as_s64(major);

    complete := json_get(document, "deps_complete");
    if complete entry.deps_complete, _ = json_as_bool(complete);

    deps := json_get(document, "deps");
    if deps for *deps.members array_add(*entry.deps, .{ copy_string(it.key), copy_string(string_member_of(*it.value)) });

    params := json_get(document, "params");
    columns := json_get(document, "columns");

    if !params || !columns || params.kind != .ARRAY || columns.kind != .ARRAY return entry, false;

    for *params.items {
        array_add(*entry.params, .{ copy_string(string_member(it, "type")), copy_string(string_member(it, "base")) });
    }

    for *columns.items {
        column: Query_Column;
        column.name = copy_string(string_member(it, "name"));
        column.type = copy_string(string_member(it, "type"));
        column.base = copy_string(string_member(it, "base"));
        column.origin = copy_string(string_member(it, "origin"));
        column.jai = copy_string(string_member(it, "jai"));

        nullable := json_get(it, "nullable");

        column.nullable = true;
        if nullable column.nullable, _ = json_as_bool(nullable);

        array_add(*entry.columns, column);
    }

    return entry, entry.sql != "";
}

#scope_module

string_member_of :: (value: *Json_Value) -> string {
    text, _ := json_as_string(value);

    return text;
}
