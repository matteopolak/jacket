#import "Basic";
E :: #import "executable_formats";

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

main :: () {
    tracking := Tracking.{ base = context.allocator };
    scoped := context;
    scoped.allocator = .{ track, *tracking };
    push_context scoped run_cases();
    assert(tracking.live == 0, "TBD failure leaked % allocations", tracking.live);
    print("ok\n");
}

run_cases :: () {
    for input: string.["", "not yaml", "--- !tapi-tbd\n", "--- !tapi-tbd\ntbd-version: 5\n",
        "--- !tapi-tbd\ntbd-version: 4\ntargets: [arm64-macos\n",
        "--- !tapi-tbd\ntbd-version: 4\ntargets: [arm64-macos]\ninstall-name: '/one'\ninstall-name: '/two'\n",
        "--- !tapi-tbd\ntbd-version: 4\ntargets: [arm64-macos]\ninstall-name: '/one'\nexports:\n - targets: [x86_64-macos]\n   symbols: [_x]\n",
        "--- !tapi-tbd\ntbd-version: 4\ntargets: [arm64-macos]\ninstall-name: *alias\n",
        "--- !tapi-tbd\ntbd-version: 4\ntargets: [arm64-macos]\ninstall-name: '/one'\n--- !tapi-tbd-v5\n",
        "--- !tapi-tbd-v3\narchs: [arm64]\nplatform: unknown\ninstall-name: '/one'\n"] {
        ok, failed := E.parse_tbd(input, "negative");
        assert(!ok && failed.count == 0 && failed.data == null);
    }

    complete := "--- !tapi-tbd\ntbd-version: 4\ntargets: [arm64-macos]\ninstall-name: '/prefix'\nexports:\n - targets: [arm64-macos]\n   symbols: [_one, '_two']\n...\n";
    corrupt := copy_string(complete);
    corrupt[70] = 0xff;
    corrupt_ok, corrupt_output := E.parse_tbd(corrupt, "negative");

    assert(!corrupt_ok && corrupt_output.count == 0 && corrupt_output.data == null);
    free(corrupt);
    failures: s64;
    successes: s64;

    for 0..complete.count {
        prefix: string = .{ it, complete.data };
        ok, document := E.parse_tbd(prefix, "negative");

        if !ok {
            failures += 1;
            assert(document.count == 0 && document.data == null);
        } else {
            successes += 1;
            assert(document.count == 1 && document[0].tbd_version == 4 && document[0].targets.count == 1);
            assert(document[0].install_name == "/prefix");
            release(document);
        }
    }

    assert(failures > 50 && successes > 2); // Complete document prefixes can validly omit optional exports/end marker.
}
