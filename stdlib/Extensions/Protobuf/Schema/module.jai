// Compile-time protobuf schema graphs, or call protobuf_generate to obtain Jai source.
//
// `Api :: #import "Extensions/Protobuf/Schema"(TEXT = "syntax = 'proto3'; message Ping {}");`

#module_parameters(
    // A single schema path, relative to the compiler working directory or absolute.
    PATH := "",
    // Inline schema text. Omit PATH when using TEXT.
    TEXT := "",
    // Recursively generate every .proto under one directory into a shared module.
    DIR := "",
    // Semicolon-separated import search paths, after the importing file directory.
    IMPORT_PATHS := "",
    // Map container, defaulting to Extensions/Hash_Map.
    MAP := Map_Mode.HASH_MAP,
    // Module providing a custom map type and iteration.
    MAP_MODULE := "",
    // Polymorphic custom map type name.
    MAP_TYPE := "",
    // Custom insert procedure name.
    MAP_INSERT := "",
    // Custom deinitialization procedure name.
    MAP_DEINIT := "",
    // Strip ENUM_NAME_ from enum values where a valid identifier remains.
    STRIP_ENUM_PREFIX := false
);

// The generated map representation; ARRAY preserves duplicate entries.
Map_Mode :: enum {
    HASH_MAP;
    TABLE;
    ARRAY;
    CUSTOM;
}

// Configuration for the standalone source generator. path is used in diagnostics.
Generate_Options :: struct {
    path: string = "<text.proto>";
    // Additional import roots, separated by semicolons.
    import_paths: string;
    // Generate a complete directory tree instead of the supplied text.
    dir: string;
    map: Map_Mode = .HASH_MAP;
    map_module: string;
    map_type: string;
    map_insert: string;
    map_deinit: string;
    strip_enum_prefix: bool;
}

// Schema failure at a one-based line and column. Free message with Basic.free;
// path shares its allocation and remains valid until message is freed.
Schema_Error :: struct {
    message: string;
    path: string;
    line: s64;
    column: s64;
}

// Generates one module for a protobuf schema and its imports, or options.dir.
// Free the source or error.message with Basic.free.
// Parsing scratch uses temp; returned text and error messages use context.allocator.
protobuf_generate :: (text: string, options := Generate_Options.{}) -> string, Schema_Error {
    allocator := context.allocator;

    push_allocator(temp);
    s := proto_Schema.{ text = text, options = options };

    proto_load_graph(*s);
    if !s.error.message.count proto_resolve_schema(*s);

    if s.error.message.count {
        error := s.error;
        combined := proto_schema_format("%\0%", error.message, error.path);

        {
            push_allocator(allocator);
            // One owned allocation keeps imported paths alive alongside the message.
            storage := copy_string(combined);

            error.path = slice(storage, error.message.count + 1, error.path.count);
            error.message = slice(storage, 0, error.message.count);
        }

        return "", error;
    }

    builder: String_Builder;

    proto_emit_schema(*s, *builder);

    return builder_to_string(*builder,, allocator), .{};
}

#insert #run proto_generate_schema();

#load "lexer.jai";
#load "parser.jai";
#load "features.jai";
#load "resolve.jai";
#load "emit.jai";
#load "imports.jai";
#load "well_known/schemas.jai";

#scope_module

proto_generate_schema :: () -> string {
    options := Generate_Options.{ path = PATH, map = MAP, map_module = MAP_MODULE,
        map_type = MAP_TYPE, map_insert = MAP_INSERT, map_deinit = MAP_DEINIT,
        strip_enum_prefix = STRIP_ENUM_PREFIX, import_paths = IMPORT_PATHS, dir = DIR };
    if !options.path.count options.path = "<text.proto>";

    count := cast(s64) (PATH.count != 0) + cast(s64) (TEXT.count != 0) + cast(s64) (DIR.count != 0);
    if count > 1 {
        compiler_report(options.path, 1, 1, "choose exactly one of PATH, TEXT or DIR");
        return "";
    }

    // Without a source this module exposes the standalone generator API.
    if !count return "";

    text := TEXT;

    if PATH.count {
        ok: bool;
        text, ok = read_entire_file(PATH);
        if !ok {
            compiler_report(PATH, 1, 1, "cannot read schema");
            return "";
        }
    }

    output, error := protobuf_generate(text, options);
    if error.message.count compiler_report(error.path, error.line, error.column, error.message);

    return output;
}

#import "Basic";
#import "String";
#import "Compiler";
#import "File";
#import "File_Utilities";
Sorting :: #import "Sort";

proto_symbols :: #import "Extensions/Hash_Map";
