#import "Basic";
F :: #import "File";
I :: #import "Ico_File";
R :: #import "internal/resource-wire/icon-directory";
W :: #import "internal/resource-wire/integers";

#if OS == .WINDOWS {
    Public :: #import "Windows_Resources";

    #assert size_of(Public.ICONDIR) == 6;
    #assert size_of(Public.ICONDIRENTRY) == 16;
}

Versions :: #import "internal/resource-wire/tool-versions";

main :: () {
    version_ok, version := Versions.parse_version("10.0.22621.0", 4);

    assert(version_ok && version[2] == 22621);
    old_ok, old := Versions.parse_version("10.0.9999.0", 4);

    assert(old_ok && Versions.is_later(version, old) && !Versions.is_later(old, version));
    bad_ok, _bad := Versions.parse_version("10.0.4294967296.0", 4);

    assert(!bad_ok);
    eight_ok, eight := Versions.parse_version("winv6.3", 2, "winv");

    assert(eight_ok && eight[0] == 6 && eight[1] == 3);

    args := get_command_line_arguments();

    assert(args.count == 2);
    pixels: [16 * 16 * 4] u8;

    for index: 0..16 * 16 - 1 {
        pixels[index * 4 + 0] = 50;
        pixels[index * 4 + 1] = 100;
        pixels[index * 4 + 2] = 150;
        pixels[index * 4 + 3] = 255;
    }

    icon := I.create_ico_file(16, 16, 4, pixels.data);

    defer free(icon);
    valid, directory := R.inspect(icon);

    assert(valid && directory.images.count == 5);
    defer R.release(*directory);
    assert(directory.images[0].bytes.data == icon.data + 86 && directory.images[0].bytes.count == I.get_total_bytes_for_size(4, 256));
    group_ok, group := R.make_group(directory, 1);

    assert(group_ok && group.count == 76);
    defer free(group);
    assert(W.load_u16(group, 4) == 5 && group[6] == 0 && group[7] == 0);
    assert(W.load_u32(group, 14) == cast(u32) I.get_total_bytes_for_size(4, 256) && W.load_u16(group, 18) == 1);
    path := sprint("%/group.bin", args[1]);

    defer free(path);
    assert(F.write_entire_file(path, group));
    path_icon := sprint("%/resource.ico", args[1]);

    defer free(path_icon);
    assert(F.write_entire_file(path_icon, icon));
    // Two real image payloads, independently assembled directory.
    combined := alloc_string(38 + 2 * (directory.images[0].bytes.count));

    defer free(combined);
    memset(combined.data, 0, combined.count);
    W.store_u16(combined.data, 2, 1);
    W.store_u16(combined.data, 4, 2);

    for index: 0..1 {
        memcpy(combined.data + 6 + 16 * index, icon.data + 6, 12);
        W.store_u32(combined.data, 18 + 16 * index, cast(u32) (38 + index * (directory.images[0].bytes.count)));
        memcpy(combined.data + 38 + index * (directory.images[0].bytes.count), icon.data + 86, directory.images[0].bytes.count);
    }

    two_ok, two := R.inspect(combined);

    assert(two_ok && two.images.count == 2);
    defer R.release(*two);
    last_ok, last := R.make_group(two, 65534);

    assert(last_ok && W.load_u16(last, 18) == 65534 && W.load_u16(last, 32) == 65535);
    free(last);
    overflow_ok, overflow := R.make_group(two, 65535);

    assert(!overflow_ok && overflow.data == null);
    zero_ok, zero := R.make_group(two, 0);

    assert(zero_ok && W.load_u16(zero, 18) == 0 && W.load_u16(zero, 32) == 1);
    free(zero);

    for prefix: 0..38 {
        truncated_ok, truncated := R.inspect(.{ prefix, combined.data });
        assert(!truncated_ok && truncated.images.count == 0);
    }

    tail_ok, tail := R.inspect(.{ combined.count - 1, combined.data });

    assert(!tail_ok && tail.images.count == 0);
    W.store_u32(combined.data, 18, 0xffffffff);
    invalid_ok, invalid := R.inspect(combined);

    assert(!invalid_ok && invalid.images.count == 0);
    W.store_u32(combined.data, 18, 6);
    invalid_ok, invalid = R.inspect(combined);
    assert(!invalid_ok && invalid.images.count == 0);
    print("ok\n");
}
