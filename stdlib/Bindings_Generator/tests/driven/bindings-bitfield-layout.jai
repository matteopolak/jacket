// Generates bindings for bindings-bitfield-layout.h on the host and for other targets, checks the
// printed bitfield storage and accessors, and writes the files the round trip programs import.
#import "Basic";
#import "File";
#import "String";
B :: #import "Bindings_Generator";

Target :: struct {
    name: string;
    os: type_of(OS);
    cpu: type_of(CPU);
}

generate :: (options: B.Generate_Bindings_Options) -> string {
    builder, success := B.generate_bindings(options);

    defer free_buffers(*builder);
    assert(success && !builder.failed);

    return builder_to_string(*builder);
}

main :: () {
    args := get_command_line_arguments();

    defer array_free(args);
    assert(args.count == 2);
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%bindings-bitfield-layout.h", #filepath));
    defer array_free(options.source_files);
    options.add_generator_command = false;
    options.log_stripped_declarations = false;
    text := generate(options);

    defer free(text);

    // The reported header: one storage field, then its members in a comment.
    assert(contains(text, "__bitfield: u32;\n /*\n  type: u32; /* 8 bits */ // type of the operand data.\n  size: u32; /* 24 bits */\n */\n offset: u32;"));
    assert(!contains(text, "type: u32 /* 8 bits */"));

    // Several units are numbered after the first; unnamed fields keep their place in the comment.
    assert(contains(text, "__bitfield_1: u32;") && contains(text, "__bitfield_2: u16;"));
    assert(contains(text, "  _: u32; /* 5 bits */") && !contains(text, "__bitfield_3: u32;\n /*\n  c: u32;"));

    // Source comments are carried into the member list and cannot end it early.
    assert(contains(text, "// prefix line one\n") && contains(text, "// postfix on count"));
    assert(contains(text, "/ * an opener / * inside a comment * /"));

    // Packed and padded records are placed field by field.
    assert(contains(text, "BfPack1 :: struct #no_padding {") && contains(text, "__padding_1: [1] u8;"));

    // Accessors follow the record, as `set_` and `get_` procedures overloaded on the record, with the numeric suffix a
    // clashing name gets.
    assert(contains(text, "set_type :: (fields: *BfIssue, value: u32) {\n    MASK0: u32 : 0b1111_1111;"));
    assert(contains(text, "get_type :: (fields: BfIssue) -> u32 {\n    return (fields.__bitfield & 0b1111_1111);"));
    assert(contains(text, "get_size :: (fields: BfIssue) -> u32 {\n    return ((fields.__bitfield & (0b1111_1111_1111_1111_1111_1111 << 8)) >> 8);"));
    assert(contains(text, "get_a :: (fields: BfSigned) -> s8 {") && contains(text, "get_flag :: (fields: BfBool) -> bool {"));
    assert(contains(text, "get_color :: (fields: BfEnum) -> BfColor {"));
    assert(contains(text, "get_x_1 :: (fields: BfClash) -> s32 {") && contains(text, "set_x_1 :: (fields: *BfClash, value: s32) {"));
    assert(contains(text, "get_y :: (fields: BfClash)") && !contains(text, "get_x :: (fields: BfClash)"));
    assert(contains(text, "get_parts_lo :: (fields: BfUnion)") && !contains(text, "BfUnnamed__"));
    assert(!contains(text, "inline (s: "));

    path := tprint("%/developer-tools-bitfield-layout-generated.jai", args[1]);

    assert(write_entire_file(path, text));

    // With the helpers turned off, the storage and its comment stay and no procedures follow.
    plain := options;
    plain.generate_bitfield_helpers = false;
    without := generate(plain);

    defer free(without);
    assert(contains(without, "__bitfield: u32;\n /*\n  type: u32; /* 8 bits */") && !contains(without, "set_type") && !contains(without, "get_type"));
    assert(write_entire_file(tprint("%/developer-tools-bitfield-layout-plain.jai", args[1]), without));

    targets := Target.[
        .{ "windows", .WINDOWS, .X64 },
        .{ "linux", .LINUX, .X64 },
        .{ "linux-arm", .LINUX, .ARM64 },
        .{ "macos-arm", .MACOS, .ARM64 },
    ];

    // The header needs no system headers or libraries, so no target SDK has to be installed: the
    // output directory stands in for both search paths.
    stand_in: [..] string;

    defer array_free(stand_in);
    array_add(*stand_in, args[1]);

    for target: targets {
        cross := options;
        cross.os = target.os;
        cross.cpu = target.cpu;
        cross.system_include_paths = stand_in;
        cross.system_library_search_paths = stand_in;
        output := generate(cross);

        defer free(output);
        assert(contains(output, "__bitfield") && contains(output, "get_type :: (fields: BfIssue)"));
        assert(write_entire_file(tprint("%/developer-tools-bitfield-layout-%.jai", args[1], target.name), output));
    }

    print("ok\n");
}
