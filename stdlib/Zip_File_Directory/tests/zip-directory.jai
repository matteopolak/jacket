#import "Basic";
Z :: #import "Zip_File_Directory";
F :: #import "File";
S :: #import "String";

Tracking :: struct {
    base: Allocator;
    outstanding: s64;
}

tracking_allocator :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    state := cast(*Tracking) allocator_data;
    result := state.base.proc(mode, requested_size, old_size, old_memory, state.base.data);

    if mode == .ALLOCATE && result state.outstanding += 1;
    if mode == .RESIZE && !old_memory && result state.outstanding += 1;
    if mode == .FREE && old_memory state.outstanding -= 1;

    return result;
}

main :: () {
    #assert !#exists(Z.read_u32);
    state := Tracking.{ base = context.allocator };
    scoped := context;
    scoped.allocator = .{ tracking_allocator, *state };
    push_context scoped run_cases();
    assert(state.outstanding == 0, "ZIP loader/test leaked allocations: %", state.outstanding);
    print("ok\n");
}

run_cases :: () {
    fixture_root := sprint("%/../../../target/portable-tests/zip/", S.path_strip_filename(#file));

    defer free(fixture_root);
    filename := sprint("%valid.zip", fixture_root);

    defer free(filename);
    ok, directory := Z.load_zip_directory(filename);

    assert(ok && !F.is_valid(directory.file));
    assert(directory.name == filename && directory.entries.count == 2);
    first := directory.entries[0];
    second := directory.entries[1];

    assert(first.filename == "first.txt" && second.filename == "binary.bin");
    assert(first.compressed_len == 5 && first.chunk_len == 5 && first.crc == 0x3610a686);
    assert(second.chunk_len == 3 && second.chunk_offset > first.chunk_offset);
    assert(first.filename.data >= directory.memory_block.data);
    assert(first.filename.data + first.filename.count <= directory.memory_block.data + directory.memory_block.count);
    Z.deinit(*directory);
    assert(directory.entries.count == 0 && directory.memory_block.count == 0 && directory.name.count == 0);
    ok, directory = Z.load_zip_directory(filename, false);
    assert(ok && F.is_valid(directory.file));
    retained := directory.file;

    Z.deinit(*directory);
    assert(F.is_valid(retained));
    assert(F.file_set_position(retained, 0));
    header: [4] u8;
    read_ok, count := F.file_read(retained, header.data, 4);

    assert(read_ok && count == 4 && header[0] == 0x50 && header[1] == 0x4b);
    F.file_close(*retained);
    empty_name := sprint("%empty.zip", fixture_root);

    defer free(empty_name);
    ok, directory = Z.load_zip_directory(empty_name);
    assert(ok && directory.entries.count == 0);
    Z.deinit(*directory);

    for path: string.["truncated", "bounds", "zip64", "multidisk", "count", "late-name"] {
        name := sprint("%%.zip", fixture_root, path);
        defer free(name);
        ok, directory = Z.load_zip_directory(name);
        assert(!ok && directory.entries.count == 0 && !F.is_valid(directory.file));
    }

}
