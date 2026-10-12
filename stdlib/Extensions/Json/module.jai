// Json: typed JSON for jaic programs, in the spirit of Rust's serde_json. Encoders and decoders
// are generated from your types at compile time, so there is no reflection walk at run time.
// A jaic extension, not official Jai: other Jai compilers have no such module, and it may change
// between jaic releases. See docs/stdlib/json.md.
//
//     #import "Extensions/Json";
//
//     User :: struct @json(rename_all=camelCase) {
//         id: int;
//         display_name: string;                 // "displayName"
//         email: string;      @json(skip_serializing_if=empty)
//         tags: [..] string;
//     }
//
//     text := to_json(user);                    // pretty: to_json(user, pretty = true)
//     user, error := from_json(User, text);     // error.kind == .NONE on success
//     value, error := parse_json(text);         // a Json_Value, when the shape is unknown

Json_Error_Kind :: enum u8 {
    NONE;
    SYNTAX; // not JSON
    TYPE; // valid JSON of the wrong shape: a string where a number belongs
    OUT_OF_RANGE; // a number that does not fit its type
    INVALID_VALUE; // an enum name or value the type does not have
    MISSING_FIELD;
    UNKNOWN_FIELD; // with deny_unknown_fields
    DUPLICATE_FIELD;
    LIMIT; // too deep, or too large
}

// Why decoding failed and where. `path` names the value inside the document (`tags[2]`,
// `address.zip`); `line` and `column` count from 1, the column in characters. `message` and
// `path` are allocated with context.allocator: free_json_error releases them.
Json_Error :: struct {
    kind: Json_Error_Kind;
    message: string;
    offset: int; // byte offset in the text
    line: int;
    column: int;
    path: string;
}

Json_Options :: struct {
    max_depth := 256; // nesting of arrays and objects
    max_size := 1 << 30; // bytes of text
}

// Switches for the vector code paths, which give the same results as the scalar ones. Set
// json_use_simd to false to force the scalar paths (the tests run both). A text of at least
// json_index_min_size bytes gets a stage-1 structural index before it is parsed.
json_use_simd := true;
json_index_min_size := 256;

// How the structural index fared since the program began, for tests and curiosity.
Json_Index_Stats :: struct {
    parsed: int; // texts decoded with an index
    unindexable: int; // texts stage 1 declined (control characters in strings, bad UTF-8...)
    sparse: int; // large texts whose strings are too short or too escaped for the index to pay
    reparsed: int; // texts that failed with an index and were parsed again to get the error
}

json_index_stats: Json_Index_Stats;

// `message at line 3, column 5 (tags[2])`
json_error_text :: (error: Json_Error) -> string {
    if error.kind == .NONE return "no error";

    if error.path.count return tprint("% at line %, column % (%)", error.message, error.line, error.column, error.path);

    return tprint("% at line %, column %", error.message, error.line, error.column);
}

free_json_error :: (error: *Json_Error) {
    if error.message.count free(error.message);
    if error.path.count free(error.path);

    error.* = .{};
}

// Encodes `value`. Output is compact unless `pretty` (two-space indent). The result comes from
// context.allocator.
to_json :: (value: $T, pretty := false) -> string {
    builder: String_Builder;
    write_json(*builder, value, pretty);

    return builder_to_string(*builder);
}

write_json :: (builder: *String_Builder, value: $T, pretty := false) {
    w: Json_Writer;
    w.builder = builder;
    w.pretty = pretty;

    json_encode(*w, *value);
}

// Decodes `text` into a `T`. Strings, arrays, pointers and tables in the result are allocated
// from context.allocator; free_json releases them. After a failure everything the decoder
// allocated has been freed already and the returned value is a default `T`.
from_json :: ($T: Type, text: string, options := Json_Options.{}) -> T, Json_Error {
    empty: T;

    if text.count > options.max_size {
        r: Json_Reader;

        reader_init(*r, text, options);
        json_fail_at(*r, .LIMIT, tprint("the input is larger than the limit of % bytes", options.max_size), 0);
        defer reader_free(*r);

        return empty, finish_error(*r);
    }

    if simd_enabled() && text.count >= json_index_min_size {
        // The fast attempt. If it fails for any reason the text is parsed again without the
        // index, so errors always come from the scalar reader.
        index: [..] u32;

        defer array_free(index);

        sparse_before := json_index_stats.sparse;
        indexed := build_index(text, *index);

        if !indexed && json_index_stats.sparse == sparse_before json_index_stats.unindexable += 1;

        if indexed {
            r: Json_Reader;

            reader_init(*r, text, options);
            defer reader_free(*r);

            r.index = index;
            r.indexed = true;

            result: T;

            if decode_document(*r, *result) {
                json_index_stats.parsed += 1;

                return result, .{};
            }

            // The scalar pass below reports the error again; this one's message is not kept.
            free_json_error(*r.error);
            json_index_stats.reparsed += 1;
        }
    }

    r: Json_Reader;

    reader_init(*r, text, options);
    defer reader_free(*r);

    result: T;

    if !decode_document(*r, *result) return empty, finish_error(*r);

    return result, .{};
}

decode_document :: (r: *Json_Reader, out: *$T) -> bool {
    if !json_decode(r, out) return false;

    skip_whitespace(r);

    if r.pos < r.text.count {
        free_json(out);

        return json_fail(r, .SYNTAX, "unexpected data after the value");
    }

    return true;
}

// Releases what from_json allocated for `value`: its strings, arrays, pointers (and what they
// point to), tables and nested values, all with `free`. The fields are reset, so a second call
// does nothing. A field that still holds its declared default (a document that omitted it) is
// not freed, so defaults such as `name := "guest"` are safe. Fields with a `with=` handler are
// left to the handler. Do not use it on values that were not decoded, as their strings may not
// have come from the allocator.
free_json :: (value: *$T) {
    defaults: T;

    json_free_value(value, *defaults);
}

json_free_value :: (value: *$T, defaults: *T) {
    #insert -> string {
        return generate_freer(T);
    };
}

// Parses `text` without a target type.
parse_json :: (text: string, options := Json_Options.{}) -> Json_Value, Json_Error {
    value, error := from_json(Json_Value, text, options);

    return value, error;
}

// Writes the JSON of the value `value` points to. This and json_decode are what `with=`
// handlers call for the parts they do not encode themselves.
json_encode :: (w: *Json_Writer, value: *$T) {
    #insert -> string {
        return generate_encoder(T);
    };
}

// Reads a `T` into `out`; false after recording an error.
json_decode :: (r: *Json_Reader, out: *$T) -> bool {
    #insert -> string {
        return generate_decoder(T);
    };
}

#load "value.jai";
#load "number.jai";
#load "writer.jai";
#load "reader.jai";
#load "index.jai";
#load "generate.jai";
#load "pairs.jai";

#scope_module

simd_enabled :: inline () -> bool {
    #if OS == .WASM return false;
    else return json_use_simd;
}

#import "Basic";
#import "String";
#import "Compiler";
#import "Hash_Table";
#import "Extensions/SIMD";
