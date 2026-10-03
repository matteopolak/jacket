// Portable checked wire/version control. Real image resizing and file writes
// remain required in portable-data-resource with its caller-owned directory.
#import "Basic";
R :: #import "internal/resource-wire/icon-directory";
W :: #import "internal/resource-wire/integers";
Versions :: #import "internal/resource-wire/tool-versions";

main :: () {
    ok, version := Versions.parse_version("10.0.22621.0", 4);

    assert(ok && version[2] == 22621);
    old_ok, old := Versions.parse_version("10.0.9999.0", 4);

    assert(old_ok && Versions.is_later(version, old) && !Versions.is_later(old, version));
    bad_ok := Versions.parse_version("10.0.4294967296.0", 4);

    assert(!bad_ok);
    prefix_ok, prefixed := Versions.parse_version("winv6.3", 2, "winv");

    assert(prefix_ok && prefixed[0] == 6 && prefixed[1] == 3);

    // ICO directory with two opaque four-byte payloads. The wire reader checks
    // extents and resource IDs; image decoding belongs to the native control.
    data: [46] u8;
    wire := string.{ data.count, data.data };

    W.store_u16(data.data, 2, 1);
    W.store_u16(data.data, 4, 2);

    for index: 0..1 {
        entry := 6 + index * 16;
        data[entry] = 1;
        data[entry + 1] = 1;
        W.store_u16(data.data, entry + 4, 1);
        W.store_u16(data.data, entry + 6, 32);
        W.store_u32(data.data, entry + 8, 4);
        W.store_u32(data.data, entry + 12, cast(u32) (38 + index * 4));
        data[38 + index * 4] = cast(u8) (71 + index);
    }

    valid, directory := R.inspect(wire);

    assert(valid && directory.images.count == 2);
    assert(directory.images[0].bytes.data == data.data + 38);
    assert(directory.images[1].bytes[0] == 72);
    group_ok, group := R.make_group(directory, 65534);

    assert(group_ok && group.count == 34);
    assert(W.load_u16(group, 18) == 65534 && W.load_u16(group, 32) == 65535);
    free(group);
    overflow_ok, overflow := R.make_group(directory, 65535);

    assert(!overflow_ok && overflow.data == null);
    zero_ok, zero := R.make_group(directory, 0);

    assert(zero_ok && W.load_u16(zero, 18) == 0 && W.load_u16(zero, 32) == 1);
    free(zero);
    R.release(*directory);
    assert(directory.images.count == 0 && directory.images.data == null);
    assert(data[38] == 71); // Releasing the views does not own the input bytes.

    for prefix: 0..45 {
        truncated_ok, truncated := R.inspect(.{ prefix, data.data });
        assert(!truncated_ok && truncated.images.count == 0);
    }

    W.store_u32(data.data, 18, 0xffffffff);
    invalid_ok, invalid := R.inspect(wire);

    assert(!invalid_ok && invalid.images.count == 0);
    W.store_u32(data.data, 18, 6);
    invalid_ok, invalid = R.inspect(wire);
    assert(!invalid_ok && invalid.images.count == 0);
    print("ok\n");
}
