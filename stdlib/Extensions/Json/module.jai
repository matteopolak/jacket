// Typed JSON for jaic programs, in the spirit of Rust's serde_json. Encoders and decoders
// are generated from your types at compile time, so there is no reflection walk at run time.
// A jaic extension, not official Jai: other Jai compilers have no such module, and it may change
// between jaic releases. See docs/extensions/json.md.
//
// ```jai
// #import "Extensions/Json";
//
// User :: struct @json(rename_all=camelCase) {
//     id: int;
//     display_name: string;                 // "displayName"
//     email: string;      @json(skip_serializing_if=empty)
//     tags: [..] string;
// }
//
// ada := User.{id = 7, display_name = "Ada"};
// text := to_json(ada);                     // pretty: to_json(ada, pretty = true)
// user, error := from_json(User, text);     // error.kind == .NONE on success
// value, value_error := from_json(Json_Value, text);    // when the shape is unknown
// free_json(*user);                         // also for a Json_Value and a Json_Error
// ```

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
    READ; // the source of a stream failed before the document ended
    UNKNOWN_VARIANT; // a tag that names none of a Variant's cases
}

// Why decoding failed and where. `path` names the value inside the document (`tags[2]`,
// `address.zip`); `line` and `column` count from 1, the column in characters. `message` and
// `path` are allocated with context.allocator: free_json releases them.
Json_Error :: struct {
    kind: Json_Error_Kind;
    message: string;
    offset: int; // byte offset in the text
    line: int;
    column: int;
    path: string;
}

Json_Duplicate_Keys :: enum u8 {
    ERROR; // a repeated key in an object is a .DUPLICATE_FIELD error
    LAST; // the last one wins (what Postgres jsonb keeps); the earlier value is released
}

Json_Options :: struct {
    max_depth := 256; // nesting of arrays and objects
    max_size := 1 << 30; // bytes of text

    // Every struct denies unknown keys, as if it had `@json(deny_unknown_fields)`. A struct's own
    // note still denies when this is false.
    deny_unknown_fields := false;

    // A missing field that is not optional keeps its declared initializer instead of failing
    // with .MISSING_FIELD.
    missing_fields_default := false;

    // What a repeated key in an object does. A Hash_Table already keeps the last value.
    duplicate_keys := Json_Duplicate_Keys.ERROR;

    // Strings without escapes (and the keys and numbers of a Json_Value) are slices of the input
    // text instead of copies; strings with escapes are still allocated from context.allocator.
    // The input must outlive the result, and the result must never be released with free_json
    // (nor may a failed decode free through a heap allocator): decode with context.allocator
    // set to an arena or `temp`, whose free does nothing, and reset that instead. Ignored by the
    // streaming from_json and by from_query/from_param, whose text does not outlive the call.
    borrow := false;
}

// A field that distinguishes a key that is missing, a `null` and a value. As a struct member a
// missing key leaves `.MISSING` (never a missing-field error), `null` gives `.NULL` and any other
// value decodes into `value` and gives `.SET`. Encoding omits the key for `.MISSING`, writes
// `null` for `.NULL` and the value for `.SET`. Outside a struct member (the document, an array
// element) `.MISSING` encodes as `null`. free_json releases `value` when it is `.SET`.
Json_Presence :: enum u8 {
    MISSING;
    NULL;
    SET;
}

Json_Maybe :: struct(T: Type) {
    state: Json_Presence;
    value: T;
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

// The error's path in jsonpath form, as `jsonb_path_query` takes it: `$` for the document itself,
// `$.address.tags[2]`, and a key that is not a plain identifier quoted: `$."we ird"`. Allocated
// with the temporary allocator.
json_path_text :: (error: Json_Error) -> string {
    builder: String_Builder;
    builder.allocator = temp;

    append(*builder, "$");

    path := error.path;
    i := 0;

    while i < path.count {
        c := path[i];

        if c == #char "[" {
            end := i;
            while end < path.count && path[end] != #char "]" end += 1;
            if end < path.count end += 1;

            append(*builder, slice(path, i, end - i));
            i = end;
        } else if c == #char "\"" {
            // A quoted key, already in JSON string form.
            end := i + 1;
            while end < path.count && path[end] != #char "\"" {
                if path[end] == #char "\\" end += 1;
                end += 1;
            }

            if end < path.count end += 1;

            append(*builder, ".");
            append(*builder, slice(path, i, end - i));
            i = end;
        } else if c == #char "." {
            i += 1;
        } else {
            end := i;
            while end < path.count && path[end] != #char "." && path[end] != #char "[" end += 1;

            key := slice(path, i, end - i);

            append(*builder, ".");

            if is_path_identifier(key) append(*builder, key);
            else append_json_string(*builder, key);

            i = end;
        }
    }

    return builder_to_string(*builder,, temp);
}

// `message at line 3, column 5 (tags[2])`
error_text :: (error: Json_Error) -> string {
    if error.kind == .NONE return "no error";

    if error.path.count return tprint("% at line %, column % (%)", error.message, error.line, error.column, error.path);

    return tprint("% at line %, column %", error.message, error.line, error.column);
}

// Releases the `message` and `path` of an error and clears it. Part of the `free_json` set.
free_json :: (error: *Json_Error) {
    if error.message.data free(error.message);
    if error.path.data free(error.path);

    error.* = .{};
} @FreesArgument

// Encodes `value`. Output is compact unless `pretty` (two-space indent). The result comes from
// context.allocator.
//
// - pretty: indents with two spaces and puts members on their own lines.
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

// Where a document comes from when it is not in memory yet: `fill` puts the next piece of text in
// `buffer` and returns how many bytes it wrote (0 at the end), and a non-empty string when the
// source failed (what went wrong). `data` is handed to it as it is. Json does not know what the
// pieces come from; `Io.read_json` makes one from an `Io.Reader`.
Json_Fill :: #type (data: *void, buffer: [] u8) -> s64, string;

Json_Source :: struct {
    data: *void;
    fill: Json_Fill;
}

// Decodes a `T` from a source in pieces. The pieces are gathered in one buffer (the decoder wants
// the whole document, and its strings are copied out of it, so the buffer is freed before this
// returns) and then decoded exactly as `from_json(T, text)` would, so a document split at any
// byte gives the same value or the same error. A source that fails gives a `.READ` error with
// the source's text; a document longer than `options.max_size` is a `.LIMIT` error and the rest
// is not read.
from_json :: ($T: Type, source: Json_Source, options := Json_Options.{}) -> T, Json_Error {
    empty: T;

    copying := options;
    copying.borrow = false;

    text: [..] u8;

    defer array_free(text);

    while true {
        if text.count == text.allocated array_reserve(*text, max(4096, text.count * 2));

        count, failure := source.fill(source.data, .{ text.allocated - text.count, text.data + text.count });

        if failure.count {
            return empty, Json_Error.{ kind = .READ, message = copy_string(failure), line = 1, column = 1 };
        }

        if count == 0 break;

        text.count += count;

        if text.count > copying.max_size {
            message := copy_string(tprint("the input is larger than the limit of % bytes", copying.max_size));

            return empty, Json_Error.{ kind = .LIMIT, message = message, offset = copying.max_size, line = 1, column = 1 };
        }
    }

    return from_json(T, string.{ text.count, text.data }, copying);
}

// Decodes `text` into a `T`. Strings, arrays, pointers and tables in the result are allocated
// from context.allocator; free_json releases them. After a failure everything the decoder
// allocated has been freed already and the returned value is a default `T`.
//
// - options: nesting and size limits, and whether unknown fields are an error.
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
            free_json(*r.error);
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

#scope_module

is_path_identifier :: (key: string) -> bool {
    if !key.count return false;

    for key {
        letter := (it >= #char "a" && it <= #char "z") || (it >= #char "A" && it <= #char "Z") || it == #char "_";

        if !letter && (it_index == 0 || it < #char "0" || it > #char "9") return false;
    }

    return true;
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

#scope_export

// The one way to release what the decoders allocated, for a decoded value of any type, a
// `Json_Value` or a `Json_Error`. For a typed value: its strings, arrays, pointers (and what they
// point to), tables and nested values, all with `free`. The fields are reset, so a second call
// does nothing. A field that still holds its declared default (a document that omitted it) is
// not freed, so defaults such as `name := "guest"` are safe. Fields with a `with=` handler are
// left to the handler. Do not use it on values that were not decoded, as their strings may not
// have come from the allocator.
free_json :: (value: *$T) {
    defaults: T;

    free_json_fields(value, *defaults);
} @FreesArgument

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
#load "pow5.jai";
#load "pow10.jai";
#load "writer.jai";
#load "reader.jai";
#load "index.jai";
#load "generate.jai";
#load "variant.jai";
#load "query.jai";

#scope_module

// An internally tagged variant's case: an object of the struct's fields with the tag first
// (encoding), or an object that has a member `tag_key` to ignore (decoding).
json_encode_tagged :: (w: *Json_Writer, value: *$T, $tag_key: string, tag: string) {
    #insert -> string {
        return generate_encoder(T, tag_key);
    };
}

json_decode_tagged :: (r: *Json_Reader, out: *$T, $tag_key: string) -> bool {
    #insert -> string {
        return generate_decoder(T, tag_key);
    };
}

free_json_fields :: (value: *$T, defaults: *T) {
    #insert -> string {
        return generate_freer(T);
    };
} @FreesArgument

simd_enabled :: inline () -> bool {
    #if OS == .WASM return false;
    else return json_use_simd;
}

#import "Basic";
#import "String";
#import "Compiler";
#import "Hash_Table";
#import "Extensions/SIMD";
