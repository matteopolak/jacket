#import "Basic";
#import "Extensions/Toml";

// A table of many keys the type does not know is still checked for duplicates, at any size.
Config :: struct {
    name: string; @toml(default)
}

wide :: (count: int, repeat: int) -> string {
    builder: String_Builder;

    for 0..count - 1 print(*builder, "k% = %\n", it, it);

    if repeat >= 0 print(*builder, "k% = 0\n", repeat);

    return builder_to_string(*builder);
}

main :: () {
    for size: int.[3, 8, 9, 40, 3000] {
        text := wide(size, -1);
        config, error := from_toml(Config, text);

        assert(error.kind == .NONE, tprint("% keys: %", size, error.message));
        free_toml_value(*config);
        free(text);

        // Every key is found again, the first, the last and one in the middle.
        for repeat: int.[0, size / 2, size - 1] {
            text = wide(size, repeat);
            config, error = from_toml(Config, text);

            assert(error.kind == .DUPLICATE, tprint("% keys, repeat %: %", size, repeat, error.kind));
            assert(toml_error_path(*error) == tprint("$[\"k%\"]", repeat), toml_error_path(*error));
            free_toml_value(*config);
            free(text);
        }
    }

    print("ok\n");
}
