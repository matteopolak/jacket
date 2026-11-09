#import "Basic";
#import "Extensions/Archive";
#import "Extensions/Testing";

// Writes `value` as zero-padded octal digits and a NUL, `width` bytes in all.
//
// Extended-header semantics under test: an empty keyword value deletes the keyword. In a global
// header it forgets the earlier global value; in a local header it also hides the global one.
octal_field :: (out: *[..] u8, value: s64, width: s64) {
    digits: [16] u8;

    for 0..width - 2 {
        digits[width - 2 - it] = cast(u8) (#char "0" + value % 8);
        value /= 8;
    }

    for 0..width - 2 array_add(out, digits[it]);

    array_add(out, 0);
}

// Appends a header block, then the payload padded to a whole block.
pax_test_block :: (out: *[..] u8, name: string, size: s64, mtime: s64, kind: u8, payload: string) {
    start := out.count;

    for 0..511 array_add(out, 0);
    for 0..name.count - 1 out.*[start + it] = name[it];

    mode := "0000644";

    for 0..6 out.*[start + 100 + it] = mode[it];

    fields: [..] u8;

    defer array_free(fields);
    octal_field(*fields, size, 12);
    for fields out.*[start + 124 + it_index] = it;

    array_reset(*fields);
    octal_field(*fields, mtime, 12);
    for fields out.*[start + 136 + it_index] = it;

    out.*[start + 156] = kind;

    magic := "ustar\00000";

    for 0..7 out.*[start + 257 + it] = magic[it];

    for 148..155 out.*[start + it] = 32;

    sum: s64;

    for 0..511 sum += out.*[start + it];

    array_reset(*fields);
    octal_field(*fields, sum, 7);
    for fields out.*[start + 148 + it_index] = it;

    out.*[start + 155] = 32;

    for payload array_add(out, it);
    for 1..(512 - payload.count % 512) % 512 array_add(out, 0);
}

// "<length> <key>=<value>\n", where the length counts its own digits.
pax_test_record :: (key: string, value: string) -> string {
    base := key.count + value.count + 3;
    length := base + 1;

    while tprint("%", length).count + base != length length += 1;

    return tprint("% %=%\n", length, key, value);
}

local_deletion_hides_global_path :: (t: *Test) {
    global := pax_test_record("path", "global/name");
    local := pax_test_record("path", "");
    out: [..] u8;

    defer array_free(out);
    pax_test_block(*out, "pax", global.count, 0, #char "g", global);
    pax_test_block(*out, "pax", local.count, 0, #char "x", local);
    pax_test_block(*out, "plain", 0, 7, #char "0", "");
    pax_test_block(*out, "other", 0, 7, #char "0", "");
    for 1..1024 array_add(*out, 0);

    a, error := read_archive(out, .TAR);

    require(error.kind == .NONE);

    defer free_archive(*a);
    require(a.entries.count == 2);
    expect(a.entries[0].name == "plain");
    expect(a.entries[1].name == "global/name");
} @test

global_deletion_forgets_global_path :: (t: *Test) {
    first := pax_test_record("path", "global/name");
    second := pax_test_record("path", "");
    out: [..] u8;

    defer array_free(out);
    pax_test_block(*out, "pax", first.count, 0, #char "g", first);
    pax_test_block(*out, "one", 0, 7, #char "0", "");
    pax_test_block(*out, "pax", second.count, 0, #char "g", second);
    pax_test_block(*out, "two", 0, 7, #char "0", "");
    for 1..1024 array_add(*out, 0);

    a, error := read_archive(out, .TAR);

    require(error.kind == .NONE);

    defer free_archive(*a);
    require(a.entries.count == 2);
    expect(a.entries[0].name == "global/name");
    expect(a.entries[1].name == "two");
} @test

local_records_in_order :: (t: *Test) {
    // A later record in the same header replaces an earlier one, including a deletion.
    records := tprint("%%", pax_test_record("path", "set/here"), pax_test_record("path", ""));
    again := tprint("%%", pax_test_record("path", ""), pax_test_record("path", "set/later"));
    out: [..] u8;

    defer array_free(out);
    pax_test_block(*out, "pax", records.count, 0, #char "x", records);
    pax_test_block(*out, "plain", 0, 7, #char "0", "");
    pax_test_block(*out, "pax", again.count, 0, #char "x", again);
    pax_test_block(*out, "plain2", 0, 7, #char "0", "");
    for 1..1024 array_add(*out, 0);

    a, error := read_archive(out, .TAR);

    require(error.kind == .NONE);

    defer free_archive(*a);
    require(a.entries.count == 2);
    expect(a.entries[0].name == "plain");
    expect(a.entries[1].name == "set/later");
} @test

local_value_overrides_global :: (t: *Test) {
    global := pax_test_record("path", "global/name");
    local := pax_test_record("path", "local/name");
    out: [..] u8;

    defer array_free(out);
    pax_test_block(*out, "pax", global.count, 0, #char "g", global);
    pax_test_block(*out, "pax", local.count, 0, #char "x", local);
    pax_test_block(*out, "plain", 0, 7, #char "0", "");
    pax_test_block(*out, "plain2", 0, 7, #char "0", "");
    for 1..1024 array_add(*out, 0);

    a, error := read_archive(out, .TAR);

    require(error.kind == .NONE);

    defer free_archive(*a);
    require(a.entries.count == 2);
    expect(a.entries[0].name == "local/name");
    expect(a.entries[1].name == "global/name");
} @test

local_deletion_of_size_and_mtime :: (t: *Test) {
    // The global size would demand a payload; the local deletion restores the header block's size.
    global := tprint("%%", pax_test_record("size", "5"), pax_test_record("mtime", "100"));
    local := tprint("%%", pax_test_record("size", ""), pax_test_record("mtime", ""));
    out: [..] u8;

    defer array_free(out);
    pax_test_block(*out, "pax", global.count, 0, #char "g", global);
    pax_test_block(*out, "pax", local.count, 0, #char "x", local);
    pax_test_block(*out, "empty", 0, 7, #char "0", "");
    pax_test_block(*out, "full", 0, 7, #char "0", "12345");
    for 1..1024 array_add(*out, 0);

    a, error := read_archive(out, .TAR);

    require(error.kind == .NONE);

    defer free_archive(*a);
    require(a.entries.count == 2);
    expect(a.entries[0].size == 0);
    expect(a.entries[0].mtime == 7);
    expect(a.entries[1].size == 5);
    expect(a.entries[1].mtime == 100);
} @test

orphan_deletion :: (t: *Test) {
    local := pax_test_record("path", "");
    out: [..] u8;

    defer array_free(out);
    pax_test_block(*out, "pax", local.count, 0, #char "x", local);
    for 1..1024 array_add(*out, 0);

    a, error := read_archive(out, .TAR);

    expect(error.kind == .INVALID);
    free_archive(*error);
    free_archive(*a);
} @test
