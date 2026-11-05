// Fixed-batch warm measurements. Compile and run only through tools/bench Sandbox.
// Input loading and pool reservation are outside the measured interval. Both Jai libraries
// materialize general values; pooled modes reset the same allocator after every iteration.
#import "Basic";
#import "String";
#import "Flat_Pool";
#import "Extensions/Testing";
F :: #import "File";
T :: #import "Extensions/Toml";
C :: #import "toml-jai";

main :: () {
    args := get_command_line_arguments();

    assert(args.count == 4 || args.count == 5);
    mode := args[1];
    input, loaded := F.read_entire_file(args[2]);

    assert(loaded);
    defer free(input);
    iterations, valid := string_to_int(args[3]);

    assert(valid && iterations > 0);
    pool: Flat_Pool;

    init(*pool, 1 << 30);
    defer fini(*pool);
    storage: Flat_Pool;

    init(*storage, 1 << 30);
    defer fini(*storage);
    prepared: T.Toml_Document;
    community_value: C.Value;

    if mode == "own-write" || mode == "own-write-verify" || mode == "own-lookup" || mode == "community-write" || mode == "community-write-verify" {
        push_allocator(flat_pool_allocator_proc, *storage);

        if mode == "community-write" || mode == "community-write-verify" {
            ok: bool;
            ok, community_value = C.string_to_type(input, C.Value);
            assert(ok, "community prepared input rejected");
        } else {
            error: T.Toml_Error;
            prepared, error = T.parse_toml(input, .{ max_retained_bytes = 1 << 30 });
            assert(error.kind == .NONE, error.message);
        }
    }

    total_bytes: u64;
    checksum: u64;
    started := current_time_monotonic();
    first := ifx args.count == 5 && args[4] == "cold" then 0 else -3;

    for warm: first..iterations - 1 {
        if warm == 0 {
            started = current_time_monotonic();
            checksum = 0;
            total_bytes = 0;
        }

        push_allocator(flat_pool_allocator_proc, *pool);

        if mode == "own-reject" || mode == "community-reject" || mode == "own-verify-error" || mode == "community-verify-error" {
            rejected: bool;

            if mode == "own-reject" || mode == "own-verify-error" {
                doc, error := T.parse_toml(black_box(input), .{ max_retained_bytes = 1 << 30 });
                rejected = error.kind != .NONE;
                black_box(error);
                T.free_toml(*doc);
            } else {
                ok, value := C.string_to_type(black_box(input), C.Value);
                rejected = !ok;
                if ok black_box(value);
            }

            if mode == "own-verify-error" || mode == "community-verify-error" {
                print("{\"rejected\":%}\n", rejected);
                exit(0);
            }

            assert(rejected, "invalid benchmark input was accepted");
            checksum += 1;
        } else if mode == "own-lookup" {
            for prepared.root.members {
                value := T.toml_get(prepared.root, black_box(it.key));
                assert(value == it.value);
                black_box(value);
                checksum += 1;
            }
        } else if mode == "own-write" || mode == "own-write-verify" || mode == "community-write" || mode == "community-write-verify" {
            output: string;

            if mode == "own-write" || mode == "own-write-verify" {
                error: T.Toml_Error;
                output, error = T.write_toml(prepared.root);
                assert(error.kind == .NONE, error.message);
            } else {
                ok: bool;
                ok, output = C.type_to_string(community_value,, toml = .{ default_format_float = .{ mode = .SCIENTIFIC, trailing_width = 16, zero_removal = .NO } });
                assert(ok, "community prepared output rejected");
            }

            if warm == 0 && (mode == "own-write-verify" || mode == "community-write-verify") {
                write_string(output);
                exit(0);
            }

            for output checksum += it;

            total_bytes += cast(u64) output.count;
        } else if mode == "own-read" || mode == "own-read-write" || mode == "own-verify" {
            options: T.Toml_Options;
            options.max_retained_bytes = 1 << 30;
            doc, error := T.parse_toml(black_box(input), options);

            assert(error.kind == .NONE, error.message);
            black_box(doc.root);

            if mode == "own-read-write" || mode == "own-verify" {
                output, output_error := T.write_toml(doc.root);

                assert(output_error.kind == .NONE, output_error.message);

                if mode == "own-verify" || mode == "community-verify" {
                    write_string(output);
                    exit(0);
                }

                for output checksum += it;

                total_bytes += cast(u64) output.count;
                free(output);
            }

            T.free_toml(*doc);
        } else {
            assert(mode == "community-read" || mode == "community-read-write" || mode == "community-verify");
            ok, value := C.string_to_type(black_box(input), C.Value);

            assert(ok, "community input rejected");
            black_box(value);

            if mode == "community-read-write" || mode == "community-verify" {
                output_ok, output := C.type_to_string(value,, toml = .{ default_format_float = .{ mode = .SCIENTIFIC, trailing_width = 16, zero_removal = .NO } });

                assert(output_ok, "community output rejected");

                if mode == "own-verify" || mode == "community-verify" {
                    write_string(output);
                    exit(0);
                }

                for output checksum += it;

                total_bytes += cast(u64) output.count;
            }
        }

        reset(*pool);
        reset_temporary_storage();
    }

    nanoseconds, clock_ok := to_nanoseconds(current_time_monotonic() - started);

    assert(clock_ok);
    print("{\"nanoseconds\":%,\"iterations\":%,\"output_bytes\":%,\"checksum\":%}\n", nanoseconds, iterations, total_bytes, checksum);
}
