// Bindings_Generator behaviour that matches the reference generator: system types by name,
// extern "C" blocks, packed fields, unnamed parameters, comment placement and macro rewrites.
#import "Basic";
#import "String";
#import "File";
#import "Bindings_Generator";
#import "Scratch_Directory";

HEADER :: #string END
#include <stdio.h>
#ifdef __cplusplus
extern "C" {
#endif

#define SEPARATOR '%'
#define QUOTE '"'
#define NEWLINE '\n'
#define ALL_ONES ((handle_t)(-1))
#define SMALL ((u_int32_t)0x10)

typedef unsigned int handle_t;
typedef void write_func(void *context, void *data, int size);

enum { FIRST_ANON = 1, SECOND_ANON = 2 };
struct After { int x; };

enum Mode {
    MODE_SLOW = 0, // the careful one
    MODE_FAST = 1
};

#define PROTO_TCP 6
#define PROTO_ALIAS PROTO_TCP
#define PROTO_MAX (PROTO_TCP + 1)
#define USES_PROTO (PROTO_TCP * 2)

#pragma pack(push, 4)
struct Packed4 { unsigned int index; long long wide; };
#pragma pack(pop)

extern const int library_constant;
FILE *open_log(const char *path);
int unnamed(int, float);
int write_all(write_func *func, void *context);

#ifdef __cplusplus
}
#endif
END;

has :: (text: string, needle: string) -> bool {
    return contains(text, needle);
}

// The bindings generated from `header`, written to and read back from a scratch directory.
generate :: (name: string, header: string, cpp: bool) -> string {
    dir := scratch_directory(name);
    defer delete_directory(dir);
    header_path := tprint("%/test.h", dir);
    output_path := tprint("%/test.jai", dir);
    assert(write_entire_file(header_path, header));

    opts: Generate_Bindings_Options;
    array_add(*opts.source_files, header_path);
    if cpp array_add(*opts.extra_clang_arguments, "-x", "c++");
    opts.generate_library_declarations = false;
    array_add(*opts.libraries, .{ filename = "does_not_exist", identifier = "lib" });
    array_add(*opts.generate_enums_from_macros_with_prefixes, "PROTO_");
    assert(generate_bindings(opts, output_path));
    text, ok := read_entire_file(output_path);
    assert(ok);
    return text;
}

// libclang is a native library: the WASM target (the browser playground) cannot load it, so
// there the generator only compiles and nothing is generated or inserted.
LIBCLANG :: OS != .WASM;

// Text checks run while compiling, so the test also builds natively (libclang is compile-time only).
#if LIBCLANG #run check_generated_text();

check_generated_text :: () {
    for cpp: bool.[false, true] {
        text := generate("bindgen-parity", HEADER, cpp);

        // Types from system headers are referred to by name and not generated.
        assert(has(text, "open_log :: (path: *u8) -> *FILE #foreign lib;"));
        assert(!has(text, "__sFILE :: struct"));
        assert(!has(text, "_IO_FILE :: struct"));

        // Declarations inside `extern "C" { }` are generated in C++ mode too.
        assert(has(text, "unnamed :: (unknown0: s32, unknown1: float) -> s32 #foreign lib;"));

        // A pointer to a function typedef is the typedef (a Jai procedure type is a pointer).
        assert(has(text, "write_func :: #type (_context: *void, data: *void, size: s32) -> void #c_call;"));
        assert(has(text, "write_all :: (func: write_func, _context: *void) -> s32 #foreign lib;"));

        // `extern const` data is bound like other extern data.
        assert(has(text, "library_constant: s32 #elsewhere lib;"));

        // Character macros, and casts that need `,trunc`.
        assert(has(text, "SEPARATOR :: #char \"%\";"));
        assert(has(text, "QUOTE :: #char \"\\\"\";"));
        assert(has(text, "NEWLINE :: #char \"\\n\";"));
        assert(has(text, "ALL_ONES :: cast,trunc(handle_t) (-1);"));
        assert(has(text, "SMALL :: cast(u32) 0x10;"));

        // An anonymous enum keeps its place in the file; a same-line comment follows its value.
        anon := find_index_from_left(text, "FIRST_ANON");
        after := find_index_from_left(text, "After :: struct");
        assert(anon >= 0 && after >= 0 && anon < after);
        assert(has(text, "SLOW :: 0; // the careful one"));

        // Macros that became enum values are rewritten where other macros use them.
        assert(has(text, "ALIAS :: TCP;"));
        assert(has(text, "MAX   :: TCP + 1;"));
        assert(has(text, "USES_PROTO :: xx PROTO.TCP * 2;"));

        // A field that C packs tighter than its natural alignment gets an explicit #align.
        assert(has(text, "wide:  s64 #align 4;"));
        assert(has(text, "assert(size_of(Packed4) == 12"));
    }
}

LONG_DOUBLE_HEADER :: #string END
struct WithLongDouble { char tag; long double value; };
long double get_long_double(void);
double get_double(void);
END;

// x86-64 `long double` is 16 bytes. By default it is jaic's Long_Double extension; without it,
// fields keep their layout as bytes and functions that pass one are stripped.
check_long_double :: () {
    dir := scratch_directory("bindgen-parity-long-double");
    defer delete_directory(dir);
    header_path := tprint("%/test.h", dir);
    assert(write_entire_file(header_path, LONG_DOUBLE_HEADER));
    generate_for_x64 :: (header_path: string, extension: bool) -> string {
        opts: Generate_Bindings_Options;
        array_add(*opts.source_files, header_path);
        array_add(*opts.extra_clang_arguments, "-target", "x86_64-unknown-linux-gnu");
        opts.cpu = .X64;
        opts.generate_library_declarations = false;
        opts.use_jaic_long_double = extension;
        array_add(*opts.libraries, .{ filename = "does_not_exist", identifier = "lib" });
        builder, ok := generate_bindings(opts);
        assert(ok);
        return builder_to_string(*builder);
    }
    text := generate_for_x64(header_path, true);
    assert(has(text, "value: Long_Double;"));
    assert(has(text, "assert(size_of(WithLongDouble) == 32"));
    assert(has(text, "get_long_double :: () -> Long_Double #foreign lib;"));
    assert(has(text, "#import \"Jaic_Extensions\";"));
    text = generate_for_x64(header_path, false);
    assert(has(text, "value: [16] u8;"));
    assert(has(text, "assert(size_of(WithLongDouble) == 32"));
    assert(!has(text, "get_long_double"));
    assert(!has(text, "Jaic_Extensions"));
    assert(has(text, "get_double :: () -> float64 #foreign lib;"));
}

#if LIBCLANG #run check_long_double();

main :: () {
    // The bindings compile (with their layout checks) and match the C layout.
    #if LIBCLANG {
        p: Packed4;
        assert(size_of(Packed4) == 12);
        assert(cast(*u8) *p.wide - cast(*u8) *p == 4);
        assert(SEPARATOR == #char "%");
        assert(ALL_ONES == 0xffff_ffff);
        assert(USES_PROTO == 12);
    }
    print("ok\n");
}

// The same bindings, generated while compiling and inserted below (it ends in #scope_file).
generated_for_insert :: () -> string {
    return generate("bindgen-parity-insert", HEADER, false);
}

// stdio's `FILE`, which the bindings name but do not declare; they only point to it.
FILE :: struct {}
lib :: #library "does_not_exist";

#if LIBCLANG {
    #insert #run generated_for_insert();
}
