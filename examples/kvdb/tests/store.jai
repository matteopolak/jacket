// The storage engine: the log, the index, recovery after a crash, compaction.

#import "Basic";
#import "String";
#import "File";
#import "Extensions/Testing";
#import,dir "../store";

#import,dir "support";

values_survive_a_reopen :: (t: *Test) {
    path := log_path(t);
    store: Store;

    require_equal(store_open(*store, path).status, Open_Status.OK);

    expect_equal(store_put(*store, "name", "Ada"), Write_Status.OK);
    expect_equal(store_put(*store, "city", "London"), Write_Status.OK);
    expect_equal(store_put(*store, "empty", ""), Write_Status.OK);

    store_close(*store);

    opened := store_open(*store, path);

    defer store_close(*store);

    expect_equal(opened.status, Open_Status.OK);
    expect_equal(opened.records, 3);

    value, found := store_get(*store, "name");

    expect(found && value == "Ada");

    value, found = store_get(*store, "empty");
    expect(found && value == "");

    _, found = store_get(*store, "country");
    expect(!found);
} @test

the_newest_record_for_a_key_wins :: (t: *Test) {
    path := log_path(t);
    store: Store;

    store_open(*store, path);

    store_put(*store, "k", "first");
    store_put(*store, "k", "second");
    store_put(*store, "gone", "x");
    store_delete(*store, "gone");
    store_put(*store, "k", "third");
    store_close(*store);

    opened := store_open(*store, path);

    defer store_close(*store);

    expect_equal(opened.records, 5);
    expect_equal(store.index.count, 1);

    value, _ := store_get(*store, "k");

    expect_equal(value, "third");

    _, found := store_get(*store, "gone");

    expect(!found);
} @test

deleting_a_missing_key_writes_nothing :: (t: *Test) {
    store: Store;

    store_open(*store, log_path(t));
    defer store_close(*store);

    expect_equal(store_delete(*store, "nothing"), Write_Status.MISSING);
    expect_equal(store.log_size, 0);
} @test

keys_and_values_are_checked :: (t: *Test) {
    store: Store;

    store_open(*store, log_path(t));
    defer store_close(*store);

    expect_equal(store_put(*store, "", "v"), Write_Status.INVALID);
    expect_equal(store_put(*store, "two words", "v"), Write_Status.INVALID);
    expect_equal(store_put(*store, "line\nbreak", "v"), Write_Status.INVALID);

    long_key := alloc_string(MAX_KEY_SIZE + 1,, temp);

    memset(long_key.data, #char "k", long_key.count);
    expect_equal(store_put(*store, long_key, "v"), Write_Status.INVALID);
    expect_equal(store_put(*store, slice(long_key, 0, MAX_KEY_SIZE), "v"), Write_Status.OK);

    big := alloc_string(MAX_VALUE_SIZE + 1,, temp);

    memset(big.data, #char "v", big.count);
    expect_equal(store_put(*store, "big", big), Write_Status.INVALID);
    expect_equal(store_put(*store, "big", slice(big, 0, MAX_VALUE_SIZE)), Write_Status.OK);
} @test

scan_lists_a_prefix_in_key_order :: (t: *Test) {
    store: Store;

    store_open(*store, log_path(t));
    defer store_close(*store);

    for string.["user:3", "user:1", "order:7", "user:2", "users"] store_put(*store, it, "x");

    items := store_scan(*store, "user:");

    require_equal(items.count, 3);
    expect_equal(items[0].key, "user:1");
    expect_equal(items[1].key, "user:2");
    expect_equal(items[2].key, "user:3");

    expect_equal(store_scan(*store, "").count, 5);
    expect_equal(store_scan(*store, "nobody").count, 0);
} @test

// A crash in the middle of an append leaves the start of a record at the end of the log. Cutting
// the last record at any byte must give back every earlier record and drop the rest.
recovery_truncates_a_torn_tail :: (t: *Test) {
    path := log_path(t);
    store: Store;

    store_open(*store, path);

    ends := fill(*store, 4);

    store_close(*store);

    last_start := ends[2];
    last_end := ends[3];

    // Inside the header, just after it, in the middle of the value, one byte short of complete.
    for cut: last_start + 1..last_end - 1 {
        cut_to(path, cut);

        opened := store_open(*store, path);

        expect_equal(opened.status, Open_Status.RECOVERED);
        expect_equal(opened.records, 3);
        expect_equal(opened.dropped_bytes, cut - last_start);
        expect_equal(store.index.count, 3);

        _, found := store_get(*store, "key3");

        expect(!found, "the unfinished record is not applied");

        value, _ := store_get(*store, "key2");

        expect_equal(value, "value2");

        store_close(*store);

        expect_equal(log_bytes(path).count, last_start);

        // Writing the record again restores the full log for the next cut.
        store_open(*store, path);
        expect_equal(store_put(*store, "key3", "value3"), Write_Status.OK);
        store_close(*store);
    }
} @test

recovery_leaves_the_log_appendable :: (t: *Test) {
    path := log_path(t);
    store: Store;

    store_open(*store, path);

    ends := fill(*store, 3);

    store_close(*store);
    cut_to(path, ends[2] - 3);

    opened := store_open(*store, path);

    expect_equal(opened.status, Open_Status.RECOVERED);

    expect_equal(store_put(*store, "after", "crash"), Write_Status.OK);
    store_close(*store);

    // The new record starts on a record boundary, so the next open reads everything cleanly.
    opened = store_open(*store, path);
    defer store_close(*store);

    expect_equal(opened.status, Open_Status.OK);
    expect_equal(opened.records, 3);

    value, found := store_get(*store, "after");

    expect(found && value == "crash");
} @test

// The last record may be whole in length but half-written in content: the bytes of a larger write
// reach the disk out of order. That is a torn write as well.
a_damaged_last_record_counts_as_torn :: (t: *Test) {
    path := log_path(t);
    store: Store;

    store_open(*store, path);

    ends := fill(*store, 3);

    store_close(*store);
    flip_bit(path, ends[2] - 1);

    opened := store_open(*store, path);

    defer store_close(*store);

    expect_equal(opened.status, Open_Status.RECOVERED);
    expect_equal(opened.dropped_bytes, ends[2] - ends[1]);
    expect_equal(store.index.count, 2);
} @test

// Damage with good records behind it did not come from a crash. Dropping the tail would lose
// acknowledged writes, so the open fails and the file is left as it is.
checksum_mismatch_in_the_middle_is_reported :: (t: *Test) {
    path := log_path(t);
    store: Store;

    store_open(*store, path);

    ends := fill(*store, 4);

    store_close(*store);

    // One bit in the value of the second record.
    flip_bit(path, ends[1] - 2);

    before := copy_string(log_bytes(path),, temp);
    opened := store_open(*store, path);

    expect_equal(opened.status, Open_Status.CORRUPT);
    expect_equal(opened.bad_offset, ends[0]);
    expect_equal(store.index.count, 0);
    expect_equal(log_bytes(path), before);
} @test

an_unknown_record_kind_is_corrupt :: (t: *Test) {
    path := log_path(t);
    store: Store;

    store_open(*store, path);

    ends := fill(*store, 2);

    store_close(*store);

    bytes := copy_string(log_bytes(path),, temp);
    bytes[ends[0] + 4] = 99;
    write_bytes(path, bytes);

    opened := store_open(*store, path);

    expect_equal(opened.status, Open_Status.CORRUPT);
    expect_equal(opened.bad_offset, ends[0]);
} @test

compaction_keeps_the_newest_values :: (t: *Test) {
    path := log_path(t);
    store: Store;

    store_open(*store, path);
    defer store_close(*store);

    store_put(*store, "a", "1");
    store_put(*store, "b", "old");
    store_put(*store, "a", "2");
    store_put(*store, "c", "3");
    store_delete(*store, "c");
    store_put(*store, "b", "new");
    store_put(*store, "a", "latest");

    before := store.log_size;
    result := store_compact(*store);

    require(result.ok);
    expect_equal(result.bytes_before, before);
    expect(result.bytes_after < before, "the log got smaller");
    expect_equal(result.bytes_after, store.live_bytes);
    expect_equal(store.log_records, 2);
    expect_equal(log_bytes(path).count, result.bytes_after);

    value, _ := store_get(*store, "a");

    expect_equal(value, "latest");

    value, _ = store_get(*store, "b");
    expect_equal(value, "new");

    _, found := store_get(*store, "c");

    expect(!found);

    // The compacted log is a log like any other: more writes append to it, and a reopen agrees.
    store_put(*store, "d", "4");
    store_close(*store);

    opened := store_open(*store, path);

    expect_equal(opened.status, Open_Status.OK);
    expect_equal(opened.records, 3);

    value, _ = store_get(*store, "a");
    expect_equal(value, "latest");

    value, _ = store_get(*store, "d");
    expect_equal(value, "4");
} @test

compaction_leaves_nothing_behind :: (t: *Test) {
    path := log_path(t);
    store: Store;

    store_open(*store, path);
    defer store_close(*store);

    for 0..9 store_put(*store, "k", tprint("%", it));

    expect(store_compact(*store).ok);

    _, leftover := file_open(tprint("%.compact", path));

    expect(!leftover, "the rewritten log was renamed over the old one");
} @test

compacting_an_empty_store_gives_an_empty_log :: (t: *Test) {
    path := log_path(t);
    store: Store;

    store_open(*store, path);
    defer store_close(*store);

    store_put(*store, "k", "v");
    store_delete(*store, "k");

    result := store_compact(*store);

    expect(result.ok);
    expect_equal(result.bytes_after, 0);
    expect_equal(log_bytes(path).count, 0);
} @test
