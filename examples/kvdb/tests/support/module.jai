// A module of the tests (imported with `#import,dir "support"`): what they share: a log path inside the test's own folder, and ways to damage a log file the
// way a crash or a bad disk would.

#import "Basic";
#import "String";
#import "File";
#import "Extensions/Testing";
#import,dir "../../store";

log_path :: (t: *Test, name := "kvdb.log") -> string {
    return tprint("%/%", temp_dir(t), name);
} @TempAllocates

// The log's bytes; they are in temporary storage.
log_bytes :: (path: string) -> string {
    bytes, ok := read_entire_file(path,, temp);

    expect(ok, "the log can be read");

    return bytes;
}

write_bytes :: (path: string, bytes: string) {
    expect(write_entire_file(path, bytes), "the log can be written");
}

// Simulates a crash in the middle of an append: the log keeps only its first `length` bytes.
cut_to :: (path: string, length: s64) {
    bytes := log_bytes(path);

    write_bytes(path, slice(bytes, 0, length));
}

// Simulates damage: flips one bit of the byte at `offset`.
flip_bit :: (path: string, offset: s64) {
    bytes := copy_string(log_bytes(path),, temp);

    bytes[offset] ^= 1;

    write_bytes(path, bytes);
}

// Writes `count` records "key0=value0", "key1=value1", ... and returns the log's length after each
// one, so a test can cut the log inside any record.
fill :: (store: *Store, count: s64) -> [] s64 {
    ends: [..] s64;
    ends.allocator = temp;

    for 0..count - 1 {
        expect_equal(store_put(store, tprint("key%", it), tprint("value%", it)), Write_Status.OK);

        array_add(*ends, store.log_size);
    }

    return ends;
}
