// test: skip_on(.WASM, reason = "safe extraction needs POSIX directory handles and a file system")

#if OS != .WASM {
    #import "Basic";
    #import "File";
    #import "Extensions/Archive";
    #import "Extensions/Testing";

    paths_and_links :: (t: *Test) {
        for name: string.["../escape", "/absolute", "C:/drive", "\\\\host\\share", "a/../../escape", "safe\\..\\escape", "a\0suffix"] {
            // Construct hostile names as TAR header bytes; the writer deliberately rejects NUL.
            if name == "a\0suffix" continue;

            items := Archive_Item.[.{ name = name, data = cast([] u8) "x" }];
            bytes, err := encode_archive(items, .TAR);

            require(err.kind == .NONE);
            defer free(bytes.data);
            a, failure := read_archive(bytes, .TAR);

            require(failure.kind == .NONE);
            defer free_archive(*a);
            error := extract(*a, temp_dir(t));

            expect(error.kind == .PATH);
            expect(error.entry == name);
            free_archive(*error);
        }

        for kind: Archive_Kind.[.SYMLINK, .HARDLINK] {
            items := Archive_Item.[.{ name = "link", kind = kind, link = "../../outside" }];
            bytes, err := encode_archive(items, .TAR);

            require(err.kind == .NONE);
            defer free(bytes.data);
            a, failure := read_archive(bytes, .TAR);

            require(failure.kind == .NONE);
            defer free_archive(*a);
            error := extract(*a, temp_dir(t));

            expect(error.kind == .PATH && error.entry == "link");
            free_archive(*error);
        }
    } @test

    duplicate_and_parent :: (t: *Test) {
        for second: string.["a", "./a", "a/child"] {
            items := Archive_Item.[.{ name = "a" }, .{ name = second }];
            bytes, error := encode_archive(items, .ZIP);

            require(error.kind == .NONE);
            defer free(bytes.data);
            a, failure := read_archive(bytes, .ZIP);

            require(failure.kind == .NONE);
            defer free_archive(*a);
            error = extract(*a, temp_dir(t));
            expect(error.kind == .DUPLICATE || error.kind == .PATH);
            free_archive(*error);
        }
    } @test

    // Writing files is only supported where extract has directory handles; elsewhere it reports UNSUPPORTED.
    #if OS == .MACOS || OS == .LINUX {
        nested_archive_is_data :: (t: *Test) {
            // A nested ZIP is data, never an instruction to recurse during extraction.
            current, err := encode_archive(Archive_Item.[.{ name = "payload", data = cast([] u8) "content" }], .ZIP);

            require(err.kind == .NONE);

            for 0..4 {
                next, e := encode_archive(Archive_Item.[.{ name = "nested.zip", data = current, method = .DEFLATE }], .ZIP);
                require(e.kind == .NONE);
                free(current.data);
                current = next;
            }

            defer free(current.data);
            nested, ne := read_archive(current, .ZIP);

            require(ne.kind == .NONE);
            defer free_archive(*nested);
            extracted := extract(*nested, temp_dir(t));

            expect(extracted.kind == .NONE);
            free_archive(*extracted);
            file, ok := read_entire_file(tprint("%/nested.zip", temp_dir(t)));

            expect(ok && file.count == nested.entries[0].size);
            free(file);
        } @test

        extract_regular :: (t: *Test) {
            bytes, error := encode_archive(Archive_Item.[.{ name = "a", kind = .DIRECTORY }, .{ name = "a/b", data = cast([] u8) "hello" }], .ZIP);

            require(error.kind == .NONE);
            defer free(bytes.data);
            a, e := read_archive(bytes, .ZIP);

            require(e.kind == .NONE);
            defer free_archive(*a);
            failure := extract(*a, temp_dir(t));

            require(failure.kind == .NONE);
            text, ok := read_entire_file(tprint("%/a/b", temp_dir(t)));

            expect(ok && text == "hello");
            free(text);
            again := extract(*a, temp_dir(t));

            expect(again.kind == .IO);
            free_archive(*again);
        } @test
    } else {
        extract_unsupported :: (t: *Test) {
            bytes, error := encode_archive(Archive_Item.[.{ name = "a", data = cast([] u8) "hello" }], .ZIP);

            require(error.kind == .NONE);
            defer free(bytes.data);
            a, e := read_archive(bytes, .ZIP);

            require(e.kind == .NONE);
            defer free_archive(*a);
            failure := extract(*a, temp_dir(t));

            expect(failure.kind == .UNSUPPORTED);
            free_archive(*failure);
            _, written := read_entire_file(tprint("%/a", temp_dir(t)));

            expect(!written);
        } @test
    }

    existing_symlink :: (t: *Test) {
        #if OS == .MACOS || OS == .LINUX {
            P :: #import "POSIX";
            outside := tprint("%/outside", temp_dir(t));
            dest := tprint("%/dest", temp_dir(t));

            require(make_directory_if_it_does_not_exist(outside));
            require(make_directory_if_it_does_not_exist(dest));
            require(P.symlink(temp_c_string(outside), temp_c_string(tprint("%/link", dest))) == 0);
            bytes, error := encode_archive(Archive_Item.[.{ name = "link/escape", data = cast([] u8) "bad" }], .TAR);

            require(error.kind == .NONE);
            defer free(bytes.data);
            a, e := read_archive(bytes, .TAR);

            require(e.kind == .NONE);
            defer free_archive(*a);
            failure := extract(*a, dest);

            expect(failure.kind == .IO);
            free_archive(*failure);
            _, ok := read_entire_file(tprint("%/escape", outside));

            expect(!ok);
        }
    } @test
}
