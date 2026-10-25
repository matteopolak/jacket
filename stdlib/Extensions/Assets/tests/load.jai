// load_asset reads a file by a relative path and refuses paths that leave the asset folder.
//
//     jaic stdlib/Extensions/Assets/tests/load.jai -interp

#import "Basic";
#import "File";
#import "Extensions/Assets";

main :: () {
    failures := 0;

    check :: (condition: bool, what: string) #expand {
        if condition return;
        print("FAIL: %\n", what);
        failures += 1;
    }

    for string.["a.png", "assets/a.png", "a/b/c.bin", "a..b", ".hidden/x"] {
        check(asset_path_is_relative(it), tprint("% is a relative asset path", it));
    }

    for string.["", "/etc/passwd", "../a", "a/../../b", "a/..", "a//b", "a/", "C:/x", "http://x/y", "a\\b", "a\0b"] {
        check(!asset_path_is_relative(it), tprint("% is refused", it));
        _, ok := load_asset(it);
        check(!ok, tprint("load_asset refuses %", it));
    }

    name :: "load-asset-test.bin";
    contents := "\x00\x01\x02asset\xff";

    if write_entire_file(name, contents) {
        defer file_delete(name);
        bytes, ok := load_asset(name);

        check(ok, "a written file loads");
        check(cast(string) bytes == contents, "with the same bytes");
        array_free(bytes);
    } else {
        print("This mode cannot write files; skipped the read check.\n");
    }

    _, missing := load_asset("no-such-asset.bin");

    check(!missing, "a missing file fails");

    if failures exit(1);

    print("ok\n");
}
