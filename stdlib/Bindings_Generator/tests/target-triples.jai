// test: only(.LINUX,.MACOS)
// The triple in extra_clang_arguments reaches clang, which decides pointer width and layout.
#import "Basic";
#import "String";
B :: #import "Bindings_Generator";

errors: [..] string;

capture :: (text: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR array_add(*errors, copy_string(text));
}

// The generated bindings of targets/target-layout.h for `arguments`, or "" when generation fails.
generate :: (arguments: [] string) -> string {
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%targets/target-layout.h", #filepath));
    array_add(*options.system_include_paths, "unused-root");
    array_add(*options.extra_clang_arguments, ..arguments);
    defer array_free(options.source_files);
    defer array_free(options.system_include_paths);
    defer array_free(options.extra_clang_arguments);
    options.add_generator_command = false;
    options.log_stripped_declarations = false;
    builder, success := B.generate_bindings(options);

    defer free_buffers(*builder);
    if !success return "";

    return builder_to_string(*builder);
}

main :: () {
    push_context {
        context.logger = capture;

        // 32-bit x86: `size_t` is 4 bytes, `long long` is 4-aligned, so the struct is 24 bytes.
        i686 := generate(.["--target=i686-linux-gnu"]);

        assert(i686 != "" && errors.count == 0);
        assert(contains(i686, "count: u32;"));
        assert(contains(i686, "#assert(size_of(Mixed) == 24);"));
        assert(contains(i686, "#assert(offset_of(Mixed, \"wide\") == 4);"));
        assert(contains(i686, "#assert(offset_of(Mixed, \"pointer\") == 12);"));
        assert(contains(i686, "#assert(size_of(Names) == 20);"));

        // `-target <triple>` spelled as two arguments is the same request.
        spaced := generate(.["-target", "i686-linux-gnu"]);

        assert(spaced == i686);

        // A freestanding 32-bit target: Cortex-M has an 8-aligned `long long`.
        thumb := generate(.["--target=thumbv7em-none-eabihf"]);

        assert(thumb != "" && contains(thumb, "count: u32;"));
        assert(contains(thumb, "#assert(size_of(Mixed) == 32);"));

        // 16-bit pointers: AVR packs everything and `size_t` is 2 bytes.
        avr := generate(.["--target=avr", "-mmcu=atmega328p"]);

        assert(avr != "" && contains(avr, "count: u16;"));
        assert(contains(avr, "#assert(size_of(Mixed) == 15);"));
        assert(contains(avr, "#assert(offset_of(Mixed, \"pointer\") == 9);"));
        assert(contains(avr, "#assert(size_of(Names) == 16);"));

        // The default follows `os` and `cpu`: 64-bit here.
        native := generate(.[]);

        assert(native != "" && contains(native, "count: u64;"));
        assert(contains(native, "#assert(size_of(Mixed) == 40);"));

        // A triple clang rejects is clang's error, not ours.
        errors.count = 0;
        rejected := generate(.["--target=bogus-triple"]);
        found := false;

        assert(rejected == "");
        for message: errors if contains(message, "unknown target triple 'bogus-triple'") found = true;

        assert(found);
    }

    print("target triples passed\n");
}
