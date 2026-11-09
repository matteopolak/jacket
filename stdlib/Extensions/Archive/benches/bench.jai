// Run with jacket bench --path stdlib/Extensions/Archive.
#import "Basic";
#import "File";
#import "File_Utilities";
#import "Extensions/Archive";
#import "Extensions/Testing";
#import "Extensions/Environment";

archive_case :: (b: *Bench, name: string, format: Archive_Format, extracting: bool) {
    base := environment_variable("JACKET_OUTPUT_DIR");
    if !base.count base = tprint("%../../../../target", #filepath);

    bytes, ok := read_entire_file(tprint("%/archive-bench/%", base, name));

    assert(ok, "run `PYTHONPATH=tools python3 tools/portable/build_portable.py bench` first");
    defer free(bytes);
    a, error := read_archive(cast([] u8) bytes, format);

    assert(error.kind == .NONE, "%", error.message);
    defer free_archive(*a);
    destinations: [..] string;

    if extracting {
        for 0..b.iterations - 1 {
            dest := tprint("%/archive-bench/extract-%", base, it);
            assert(make_directory_if_it_does_not_exist(dest));
            array_add(*destinations, copy_string(dest));
        }

        set_bytes(b, 32 * 1024 * 1024);
    }

    defer {
        for destinations {
            assert(delete_directory_tree(it));
            free(it);
        }

        array_free(destinations);
    }

    reset_timer(b);

    for 0..b.iterations - 1 {
        if extracting {
            failure := extract(black_box(*a), destinations[it]);
            assert(failure.kind == .NONE, "%", failure.message);
            black_box(failure);
        } else {
            indexed, failure := read_archive(cast([] u8) black_box(bytes), format);
            assert(failure.kind == .NONE, "%", failure.message);
            black_box(indexed);
            free_archive(*indexed);
        }
    }
}

stored_index :: (b: *Bench) {
    archive_case(b, "stored.zip", .ZIP, false);
} @bench

stored_extract :: (b: *Bench) {
    archive_case(b, "stored.zip", .ZIP, true);
} @bench

deflate_index :: (b: *Bench) {
    archive_case(b, "deflate.zip", .ZIP, false);
} @bench

deflate_extract :: (b: *Bench) {
    archive_case(b, "deflate.zip", .ZIP, true);
} @bench

tar_index :: (b: *Bench) {
    archive_case(b, "plain.tar", .TAR, false);
} @bench

tar_extract :: (b: *Bench) {
    archive_case(b, "plain.tar", .TAR, true);
} @bench
