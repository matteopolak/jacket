// How `json` and `jsonb` documents are decoded: the options, and the layers they come from.
//
// Three places can say how strict decoding is, from least to most specific:
//
//   1. the connection (`Config.json_options`), copied into each result;
//   2. a `Json_Columns` entry, which rides along in the column's type as `Json(T, OPTIONS)`
//      (schema.jai);
//   3. one query (`Pg.query(db, "...", json = .{ unknown_fields = .DENY })`), kept in the result's
//      `json_query`.
//
// Every option has an INHERIT value, which is the default: it takes the layer below, and the
// bottom layer's INHERIT is the built-in default (IGNORE, ERROR, LAST). A struct's own
// `@json(deny_unknown_fields)` and `@json(default)` notes win over all of them (the Json module).
// See docs/postgres/typed-queries.md.

#scope_export

Json_Unknown_Fields :: enum u8 {
    INHERIT;
    IGNORE; // the built-in default: a key the struct has no field for is skipped
    DENY; // it is a decode error
}

Json_Missing_Fields :: enum u8 {
    INHERIT;
    ERROR; // the built-in default: a field that is not optional must be there
    DEFAULT; // it keeps its declared initializer
}

Json_Duplicate_Keys :: enum u8 {
    INHERIT;
    LAST; // the built-in default: what `jsonb` keeps, and what `json` text may contain
    ERROR;
}

Json_Options :: struct {
    unknown_fields := Json_Unknown_Fields.INHERIT;
    missing_fields := Json_Missing_Fields.INHERIT;
    duplicate_keys := Json_Duplicate_Keys.INHERIT;
    max_depth := 256; // nesting of arrays and objects
    max_size := 1 << 30; // bytes of text
}

// A `Json_Columns` entry's three options as the second parameter of `Json(T, OPTIONS)`: two bits
// each, in the order of the enums.
pack_json_options :: (options: Json_Options) -> u32 {
    unknown: u32 = xx options.unknown_fields;
    missing: u32 = xx options.missing_fields;
    duplicate: u32 = xx options.duplicate_keys;

    return unknown | (missing << 2) | (duplicate << 4);
}

unpack_json_options :: (packed: u32) -> Json_Options {
    options: Json_Options;

    options.unknown_fields = xx (packed & 3);
    options.missing_fields = xx ((packed >> 2) & 3);
    options.duplicate_keys = xx ((packed >> 4) & 3);

    return options;
}

#scope_module

// The options a decode runs under: the connection's and the query's layers, which `get` reads from
// the result, and whether strings may point into the result's data.
Json_Layers :: struct {
    connection: Json_Options;
    query: Json_Options;
    borrow := false;
}

// `over`, with every INHERIT replaced by what `under` says.
json_options_over :: (over: Json_Options, under: Json_Options) -> Json_Options {
    merged := over;

    if merged.unknown_fields == .INHERIT merged.unknown_fields = under.unknown_fields;
    if merged.missing_fields == .INHERIT merged.missing_fields = under.missing_fields;
    if merged.duplicate_keys == .INHERIT merged.duplicate_keys = under.duplicate_keys;

    return merged;
}

// What the Json module is given: the query over the column over the connection, each INHERIT
// replaced by the built-in default.
module_json_options :: (layers: Json_Layers, column: u32) -> Json_Module.Json_Options {
    merged := json_options_over(layers.query, json_options_over(unpack_json_options(column), layers.connection));

    module: Json_Module.Json_Options;
    module.max_depth = layers.connection.max_depth;
    module.max_size = layers.connection.max_size;
    module.deny_unknown_fields = merged.unknown_fields == .DENY;
    module.missing_fields_default = merged.missing_fields == .DEFAULT;
    module.duplicate_keys = ifx merged.duplicate_keys == .ERROR then Json_Module.Json_Duplicate_Keys.ERROR else Json_Module.Json_Duplicate_Keys.LAST;
    module.borrow = layers.borrow;

    return module;
}
