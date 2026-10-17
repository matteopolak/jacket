#import "Basic";
E :: #import "executable_formats";
S :: #import "String";

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

release :: (archive: E.Archive) {
    array_free(archive.files);
    array_free(archive.symbol_names);
    array_free(archive.symbol_file_offsets);
}

main :: () {
    state := Tracking.{ base = context.allocator };
    scoped := context;
    scoped.allocator = .{ track, *state };
    push_context scoped run_cases();
    assert(state.live == 0, "archive parser/test leaked storage: %", state.live);
    print("ok\n");
}

run_cases :: () {
    #assert !#exists(E.part);
    #assert !#exists(E.parse_bsd_index);
    magic: [4] u8 = .[0xca, 0xfe, 0xba, 0xbe];

    assert(E.is_apple_universal(.{ 4, magic.data }));
    magic = .[0xbe, 0xba, 0xfe, 0xca];
    assert(E.is_apple_universal(.{ 4, magic.data }));
    magic = .[0xca, 0xfe, 0xba, 0xbf];
    assert(E.is_apple_universal(.{ 4, magic.data }));
    magic = .[0xbf, 0xba, 0xfe, 0xca];
    assert(E.is_apple_universal(.{ 4, magic.data }));
    assert(!E.is_apple_universal(.{ 3, magic.data }));
    assert(!E.is_apple_universal("test"));
    assert(E.is_ar("!<arch>\n"));
    assert(!E.is_ar("!<thin>\n") && !E.is_ar("!<arch>"));
    ok, empty := E.parse_ar("!<arch>\n", "empty.a");

    assert(ok && empty.files.count == 0 && empty.symbol_names.count == 0);
    release(empty);
    content := "!<arch>\nshort.o/        0           0     0     100644  3         `\nabc\n";
    parsed, archive := E.parse_ar(content, "simple.a");

    assert(parsed && archive.content.data == content.data && archive.files.count == 1);
    assert(archive.files[0].name == "short.o" && archive.files[0].header_start_offset == 8);
    assert(archive.files[0].data.count == 3 && archive.files[0].data.data == content.data + 68);
    assert(archive.files[0].data[0] == #char "a" && archive.files[0].data[2] == #char "c");
    release(archive);

    for 9..content.count - 1 {
        truncated: string = .{ it, content.data };
        ok, bad := E.parse_ar(truncated, "truncated.a");
        assert(!ok && bad.files.count == 0 && bad.content.count == 0);
    }

    assert((cast(*Type_Info_Struct) type_info(E.Archive)).name == "Archive");
    assert((cast(*Type_Info_Struct) type_info(E.Archive_File)).name == "Archive_File");
    root := sprint("%/../../../target/portable-tests/archive-fixtures/", S.path_strip_filename(#file));

    defer free(root);

    for label: string.["classic", "bsd-big", "bsd-little", "gnu", "coff", "native"] {
        path := sprint("%%.a", root, label);

        defer free(path);
        valid, value := E.parse_ar_file(path);

        assert(valid && value.files.count == 1, "archive fixture % failed", label);
        assert(value.files[0].data.count > 0);

        if label == "native" {
            found := false;
            for name: value.symbol_names if name == "_portable_archive_answer" || name == "portable_archive_answer" found = true;
            assert(found, "native archive is missing its real C symbol");
        } else if label != "classic" {
            assert(value.files[0].name == "long member name.o");
            assert(value.symbol_names[0] == "answer");
            assert(value.symbol_file_offsets[0] == value.files[0].header_start_offset);
            assert(value.files[0].data.count == 3 && value.files[0].data[1] == #char "b");
        }

        release(value);
        free(value.content);
    }

    for label: string.["bad-gnu-offset", "bad-coff-offset", "bad-gnu-count", "bad-coff-index", "bad-symbol-termination", "bad-bsd-string", "bad-size", "bad-padding", "bad-trailer", "bad-name", "unsupported64"] {
        path := sprint("%%.a", root, label);
        defer free(path);
        valid, value := E.parse_ar_file(path);
        assert(!valid && value.files.count == 0 && value.content.count == 0, "malformed archive % accepted", label);
    }
}
