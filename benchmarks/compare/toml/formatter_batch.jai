// The actual formatter configuration consumer, linked against baseline or candidate stdlib.
// Input loading/pool reservation are excluded; parse, schema application and reset are included.
// Example: formatter-batch jacket.toml 1000
#import "Basic";
#import "Flat_Pool";
#import "Extensions/Testing";
F :: #import "File";
J :: #import "Extensions/Jai_Format";
#load "settings_projection.jai";

main :: () {
    args := get_command_line_arguments();

    assert(args.count == 3 || args.count == 4);
    input, loaded := F.read_entire_file(args[1]);

    assert(loaded);
    defer free(input);
    iterations, valid := string_to_int(args[2]);

    assert(valid && iterations > 0);

    if args.count == 4 && args[3] == "verify" {
        config, ok := J.parse_config(input);
        if !ok exit(1);

        value := project_settings(config);
        output := Json.to_json(value);

        write_string(output);
        exit(0);
    }

    pool: Flat_Pool;

    init(*pool, 1 << 30);
    defer fini(*pool);
    checksum: u64;
    started := current_time_monotonic();
    first := ifx args.count == 4 && args[3] == "cold" then 0 else -3;

    for warm: first..iterations - 1 {
        if warm == 0 {
            started = current_time_monotonic();
            checksum = 0;
        }

        push_allocator(flat_pool_allocator_proc, *pool);
        config, ok, error := J.parse_config(black_box(input));

        assert(ok, error);
        black_box(config);
        checksum += cast(u64) (config.indent_width + config.max_width + config.ignore.count + config.doc_format.count + config.unknown_keys.count);
        reset(*pool);
        reset_temporary_storage();
    }

    nanoseconds, clock_ok := to_nanoseconds(current_time_monotonic() - started);

    assert(clock_ok);
    print("{\"nanoseconds\":%,\"iterations\":%,\"checksum\":%}\n", nanoseconds, iterations, checksum);
}
