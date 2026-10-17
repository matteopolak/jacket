#import "Basic";
E :: #import "executable_formats";
S :: #import "String";
F :: #import "File";

release :: (documents: [] E.Text_Based_Dylib) {
    for document: documents {
        for document.targets free(it);

        array_free(document.targets);
        free(document.install_name);

        for group: document.exports {
            for group.targets free(it);
            for group.symbols free(it);
            for group.re_exports free(it);

            array_free(group.targets);
            array_free(group.symbols);
            array_free(group.re_exports);
        }

        array_free(document.exports);
    }

    array_free(documents);
}

Tracking :: struct {
    base: Allocator;
    live: s64;
}

track :: (mode: Allocator_Mode, size: s64, old_size: s64, old: *void, data: *void) -> *void {
    state := cast(*Tracking) data;
    result := state.base.proc(mode, size, old_size, old, state.base.data);

    if mode == .ALLOCATE && result state.live += 1;
    if mode == .RESIZE && !old && result state.live += 1;
    if mode == .FREE && old state.live -= 1;

    return result;
}

main :: () {
    tracking := Tracking.{ base = context.allocator };
    scoped := context;
    scoped.allocator = .{ track, *tracking };
    push_context scoped run_cases();
    assert(tracking.live == 0, "TBD leaked % allocations", tracking.live);
    print("ok\n");
}

run_cases :: () {
    root := sprint("%/../../../target/executable-text-fixtures/", S.path_strip_filename(#file));

    defer free(root);

    for label: string.["original-v4", "original-v3", "sdk-objc", "sdk-system"] {
        path := sprint("%%.tbd", root, label);

        defer free(path);
        ok, documents := E.parse_tbd_file(path);

        assert(ok && documents.count > 0, "TBD fixture % failed", label);
        first := documents[0];

        assert(first.targets.count >= 2 && first.exports.count > 0);

        if label == "original-v3" {
            assert(first.tbd_version == 3 && first.targets[0] == "x86_64-macos" && first.targets[1] == "arm64-macos");
            assert(first.install_name == "/usr/lib/libIndependent3.dylib");
        } else assert(first.tbd_version == 4);

        if label == "original-v4" {
            assert(documents.count == 2 && first.targets[0] == "arm64-macos");
            assert(first.install_name == "/usr/lib/libIndependent4.dylib");
            assert(first.exports[0].symbols[0] == "_one" && first.exports[0].symbols[1] == "_two");
            found_class := false;
            found_reexport := false;

            for group: first.exports {
                for group.symbols if it == "_OBJC_CLASS_$_Widget" found_class = true;
                for group.re_exports if it == "/usr/lib/libSystem.B.dylib" found_reexport = true;
            }

            assert(found_class && found_reexport);
            assert(documents[1].install_name == "/usr/lib/lib#Child.dylib");
            assert(first.exports[0].symbols[2] == "_hash#Symbol" && first.exports[0].symbols[3] == "_é");
        }

        if label == "sdk-objc" {
            assert(first.install_name == "/usr/lib/libobjc.A.dylib");
            found := false;
            for group: first.exports for group.symbols if it == "_objc_msgSend" found = true;
            assert(found);
        }

        if label == "sdk-system" assert(documents.count > 10 && first.install_name == "/usr/lib/libSystem.B.dylib");

        builder: String_Builder;

        defer free_buffers(*builder);
        for group: first.exports for target : group.targets for symbol : group.symbols print_to_builder(*builder, "%|%\n", target, symbol);

        dump_path := sprint("%%-parsed-symbols.txt", root, label);

        defer free(dump_path);
        assert(F.write_entire_file(dump_path, *builder));
        release(documents);
    }

    input := "--- !tapi-tbd\ntbd-version: 4\ntargets: [arm64-macos]\ninstall-name: '/owned'\nexports:\n - targets: [arm64-macos]\n   symbols: [_borrow_test]\n...\n";
    bytes := copy_string(input);

    assert(E.is_tbd(bytes) && !E.is_tbd("not a stub"));
    valid, owned := E.parse_tbd(bytes);

    assert(valid && owned.count == 1 && owned[0].exports[0].symbols[0] == "_borrow_test");
    free(bytes);
    assert(owned[0].install_name == "/owned" && owned[0].exports[0].symbols[0] == "_borrow_test");
    release(owned);
    block_input := "--- !tapi-tbd # header\ntbd-version: 4\ntargets:\n - arm64-macos\ninstall-name: '/block' # inline\nexports:\n - targets: [arm64-macos,]\n   symbols:\n     - _block_one\n     - 'two#quoted'\n... # end\n";
    block_ok, block := E.parse_tbd(block_input);

    assert(block_ok && block[0].exports[0].symbols.count == 2 && block[0].exports[0].symbols[1] == "two#quoted");
    release(block);
}
