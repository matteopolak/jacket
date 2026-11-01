// The store: a log file on disk and an index of the live keys in memory.
//
// The log is the only durable state. Opening a store replays it from the start, so the index is
// always rebuilt from what is on disk, and the last record for a key wins. Deleted keys leave
// their records behind until `store_compact` rewrites the log.
//
// A store is not thread safe. The server runs on one scheduler thread, so it never needs to be.

// A live key and its value. The index owns both strings.
Item :: struct {
    key: string;
    value: string;
}

Store :: struct {
    path: string;
    log: File;
    index: Table(string, Item);

    // Bytes in the log, and how many records they hold, live or not.
    log_size: s64;
    log_records: s64;
    // What the live items alone would take: the size of the log after `store_compact`.
    live_bytes: s64;
}

Open_Status :: enum {
    // The log was read to its end.
    OK;
    // The log ended in a record that was never finished. It was cut off and the store is usable.
    RECOVERED;
    // A record before the end of the log fails its checksum. Nothing was changed on disk.
    CORRUPT;
    IO_ERROR;
}

Opened :: struct {
    status: Open_Status;
    // Records replayed.
    records: s64;
    // For `.RECOVERED`: how many bytes were cut off.
    dropped_bytes: s64;
    // For `.CORRUPT`: where the first bad record starts.
    bad_offset: s64;
}

Write_Status :: enum {
    OK;
    // `store_delete` of a key that is not there. Nothing is written.
    MISSING;
    // A key that `valid_key` refuses, or an oversized value.
    INVALID;
    IO_ERROR;
}

Compaction :: struct {
    ok: bool;
    bytes_before: s64;
    bytes_after: s64;
}

// Opens the log at `path`, creating it when it does not exist, and rebuilds the index from it.
// On `.CORRUPT` or `.IO_ERROR` the store is already closed again.
//
// A crash can leave half a record at the end of the log. Because every record is checksummed, the
// replay stops at the first one that is incomplete (or complete but wrong, when it is the last
// thing in the file: its bytes were only partly written) and truncates the log there, so the next
// append starts on a record boundary. A bad record with more data after it cannot come from a
// crash, so it is reported instead of silently dropping everything behind it.
store_open :: (store: *Store, path: string) -> Opened {
    store.* = .{};
    store.path = copy_string(path);

    opened: Opened;
    log, exists := read_entire_file(path);

    if !exists log = "";

    defer free(log);

    at := 0;
    status := Decode_Status.OK;
    record: Record;

    while at < log.count {
        record, status = decode_record(log, at);

        if status != .OK break;

        replay(store, record);
        store.log_records += 1;
        at += record.size;
    }

    opened.records = store.log_records;

    torn := status == .TORN || (status == .BAD_CHECKSUM && at + record.size == log.count);

    if status == .BAD_HEADER || (status == .BAD_CHECKSUM && !torn) {
        opened.status = .CORRUPT;
        opened.bad_offset = at;
        store_close(store);

        return opened;
    }

    if torn {
        opened.status = .RECOVERED;
        opened.dropped_bytes = log.count - at;

        if !cut_log(path, slice(log, 0, at)) {
            opened.status = .IO_ERROR;
            store_close(store);

            return opened;
        }
    }

    store.log_size = at;

    if !open_log(store) {
        opened.status = .IO_ERROR;
        store_close(store);
    }

    return opened;
}

// Frees the index and closes the log. The store can be opened again afterwards.
store_close :: (store: *Store) {
    for store.index {
        free(it.key);
        free(it.value);
    }

    deinit(*store.index);

    if is_valid(store.log) file_close(*store.log);

    free(store.path);
    store.* = .{};
}

// The value of `key`, owned by the store: copy it before the next write.
store_get :: (store: *Store, key: string) -> string, bool {
    item := table_find_pointer(*store.index, key);

    if !item return "", false;

    return item.value, true;
}

// Sets `key` to `value`. The record is on disk (handed to the operating system) before the index
// changes, so the index never holds a value the log would lose.
store_put :: (store: *Store, key: string, value: string) -> Write_Status {
    if !valid_key(key) || value.count > MAX_VALUE_SIZE return .INVALID;
    if !append(store, .PUT, key, value) return .IO_ERROR;

    index_put(store, key, value);

    return .OK;
}

// Removes `key`. Deleting a key that is not there writes nothing.
store_delete :: (store: *Store, key: string) -> Write_Status {
    if !table_contains(*store.index, key) return .MISSING;
    if !append(store, .DELETE, key, "") return .IO_ERROR;

    index_remove(store, key);

    return .OK;
}

// The items whose key starts with `prefix`, in key order. The array is in temporary storage; the
// strings belong to the store, as with `store_get`.
store_scan :: (store: *Store, prefix: string) -> [] Item {
    items: [..] Item;
    items.allocator = temp;

    for store.index {
        if begins_with(it.key, prefix) array_add(*items, it);
    }

    quick_sort(items, (a: Item, b: Item) -> int {
        return compare_strings(a.key, b.key);
    });

    return items;
}

// Rewrites the log with one record for each live key, in key order, and swaps it in. The new log
// is built beside the old one and renamed over it, so a crash during compaction leaves either the
// old log or the new one, never a mixture. A leftover `.compact` file from a crash is overwritten
// by the next compaction.
store_compact :: (store: *Store) -> Compaction {
    result: Compaction;
    result.bytes_before = store.log_size;

    items := store_scan(store, "");
    rewritten := tprint("%.compact", store.path);
    out, opened := file_open(rewritten, for_writing = true);

    if !opened return result;

    size := 0;
    written := true;

    for items {
        record := encode_record(.PUT, it.key, it.value);
        defer free(record);

        written = written && file_write(*out, record);
        size += record.count;
    }

    written = file_flush(out) && written;
    file_close(*out);

    if !written {
        file_delete(rewritten);

        return result;
    }

    file_close(*store.log);

    swapped := replace_file(rewritten, store.path);

    if !swapped file_delete(rewritten);

    if swapped {
        store.log_size = size;
        store.log_records = items.count;
    }

    // The log is reopened either way: after a failed swap the old file is still the log.
    if !open_log(store) return result;

    result.ok = swapped;
    result.bytes_after = store.log_size;

    return result;
}

#scope_file

replace_file :: atomic_file_replace_or_just_rename_if_destination_file_does_not_exist;

open_log :: (store: *Store) -> bool {
    ok: bool;

    store.log, ok = file_open(store.path, for_writing = true, keep_existing_content = true);

    return ok && file_seek(store.log, 0, .END);
}

// Replaces the log with `kept`, its valid prefix. There is no way to shorten a file in place, so
// the prefix is written beside it and renamed over it, which is no less safe: a crash during
// recovery leaves the old log, torn tail and all, for the next open to recover again.
cut_log :: (path: string, kept: string) -> bool {
    beside := tprint("%.recover", path);

    if !write_entire_file(beside, kept) return false;

    if !replace_file(beside, path) {
        file_delete(beside);

        return false;
    }

    return true;
}

append :: (store: *Store, kind: Record_Kind, key: string, value: string) -> bool {
    record := encode_record(kind, key, value);

    defer free(record);

    if !file_write(*store.log, record) || !file_flush(store.log) return false;

    store.log_size += record.count;
    store.log_records += 1;

    return true;
}

replay :: (store: *Store, record: Record) {
    if record.kind == .PUT index_put(store, record.key, record.value);
    else index_remove(store, record.key);
}

index_put :: (store: *Store, key: string, value: string) {
    item := table_find_pointer(*store.index, key);

    if item {
        store.live_bytes -= record_size(item.key, item.value);
        free(item.value);

        item.value = copy_string(value);
    } else {
        owned := copy_string(key);

        table_add(*store.index, owned, .{ owned, copy_string(value) });
    }

    store.live_bytes += record_size(key, value);
}

index_remove :: (store: *Store, key: string) {
    found, item := table_remove(*store.index, key);

    if !found return;

    store.live_bytes -= record_size(item.key, item.value);
    free(item.key);
    free(item.value);
}
