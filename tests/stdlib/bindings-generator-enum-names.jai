// Bindings_Generator enums as upstream's pinned bindings spell them (jai-tracy, sgpu's Slang and
// Vulkan_With_VMA, Vk-Engine's ImGui): a prefix shared by camel-case value names is dropped and
// kept as an alias, values lose their comments but keep the blank lines around them, an
// empty struct closes on its own line, and a Windows target types an enum `int` like MSVC.
#import "Basic";
#import "String";
#import "File";
#import "Bindings_Generator";
#import "Scratch_Directory";

HEADER :: #string C
enum PlotFormatEnum
{
    PlotFormatNumber,
    PlotFormatMemory,
    PlotFormatWatt
};

enum Severity
{
    SeverityTrace,   // Broadly track variable states.
    SeverityDebug,   // Describes specific events.
    SeverityFatal,   // Describes a failure.
};

enum Digits { Digit1 = 1, Digit2 = 2 };

enum Single { SingleOnly };

enum Signed { SIGNED_LOW = -1, SIGNED_HIGH = 1 };

enum Spaced
{
    SPACED_ONE = 1,

    // A note.
    SPACED_TWO = 2,
    SPACED_THREE = 3
};

struct Opaque;
struct Opaque *make_opaque(void);
C;

generate :: (name: string, windows: bool) -> string {
    dir := scratch_directory(name);

    hide_scratch_directory_in_log(dir);
    defer delete_directory(dir);
    header_path := tprint("%/test.h", dir);
    output_path := tprint("%/test.jai", dir);

    assert(write_entire_file(header_path, HEADER));

    opts: Generate_Bindings_Options;

    array_add(*opts.source_files, header_path);
    opts.generate_library_declarations = false;
    array_add(*opts.libraries, .{ filename = "does_not_exist", identifier = "lib" });
    if windows opts.os = .WINDOWS;
    else opts.os = .LINUX;

    assert(generate_bindings(opts, output_path));
    text, ok := read_entire_file(output_path);

    assert(ok);

    return text;
}

has :: (text: string, needle: string) -> bool {
    return contains(text, needle);
}

#run check_enum_names();

check_enum_names :: () {
    text := generate("bindgen-enum-names", false);

    // Names that share a camel-case word prefix lose it; the C names stay as aliases.
    assert(has(text, "PlotFormatEnum :: enum u32 {\n    Number :: 0;\n    Memory :: 1;\n    Watt   :: 2;\n"));
    assert(has(text, "    PlotFormatNumber :: Number;\n"));
    assert(has(text, "Severity :: enum u32 {\n    Trace :: 0;\n    Debug :: 1;\n    Fatal :: 2;\n"));
    assert(has(text, "    SeverityFatal :: Fatal;\n"));

    // Comments on values are dropped.
    assert(!has(text, "Broadly track"));
    assert(!has(text, "Describes"));

    // A prefix must end a word in every name (`Digit` + `1` does not), and a lone value still loses
    // the enum's name.
    assert(has(text, "    Digit1 :: 1;\n"));
    assert(has(text, "    Only :: 0;\n"));

    // Values keep the blank line the header had between them, and so do their aliases.
    assert(has(text, "    ONE   :: 1;\n\n    TWO   :: 2;\n    THREE :: 3;\n"));
    assert(has(text, "    SPACED_ONE   :: ONE;\n\n    SPACED_TWO   :: TWO;\n    SPACED_THREE :: THREE;\n"));

    // A negative value in a signed enum stays signed.
    assert(has(text, "Signed :: enum s32 {"));

    // A struct without members is `struct {}`.
    assert(has(text, "Opaque :: struct {}"));

    // MSVC types an enum without a stated type `int`; a host with another default is told by `os`.
    windows := generate("bindgen-enum-names-windows", true);

    assert(has(windows, "PlotFormatEnum :: enum s32 {"));
    assert(has(windows, "Severity :: enum s32 {"));
    assert(has(windows, "Signed :: enum s32 {"));
}
