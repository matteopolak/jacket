// test: skip_on(.WASM, reason = "reads fixture archives from the file system")

#if OS != .WASM {
    #import "Basic";
    #import "File";
    #import "File_Utilities";
    #import "Extensions/Environment";
    #import "Extensions/Archive";
    #import "Extensions/Testing";
    Deflate :: #import "Extensions/Deflate";

    Fixture :: struct {
        name: string;
        expected: Archive_Error_Kind;
        skip_reason: string;
        count: s64 = 0;
        payload_crc: u32 = 0;
    }

    // Sparse formats and bzip2 framing are outside v1. PNG/text files are auxiliary payloads.
    ZIP_FIXTURES :: Fixture.[
        .{ "comment-truncated.zip", .INVALID, "" },
        .{ "crc32-not-streamed.zip", .NONE, "", 2, 0x98ac8382 },
        .{ "dd.zip", .NONE, "", 1, 0xa2e3d6d3 },
        .{ "dupdir.zip", .NONE, "", 4, 0x00000000 },
        .{ "go-no-datadesc-sig.zip", .NONE, "", 2, 0x98ac8382 },
        .{ "go-with-datadesc-sig.zip", .NONE, "", 2, 0x98ac8382 },
        .{ "readme.notzip", .INVALID, "" },
        .{ "readme.zip", .NONE, "", 1, 0x82080ae0 },
        .{ "subdir.zip", .NONE, "", 3, 0x00000000 },
        .{ "symlink.zip", .NONE, "", 1, 0x8e9efad1 },
        .{ "test-badbase.zip", .INVALID, "" }, // Wrong directory size: strict rejection; Go tolerates it.
        .{ "test-baddirsz.zip", .INVALID, "" },
        .{ "test-prefix.zip", .NONE, "", 2, 0x232a9080 },
        .{ "test-trailing-junk.zip", .NONE, "", 2, 0x232a9080 },
        .{ "test.zip", .NONE, "", 2, 0x232a9080 },
        .{ "time-22738.zip", .NONE, "", 1, 0x00000000 },
        .{ "time-7zip.zip", .NONE, "", 1, 0x00000000 },
        .{ "time-go.zip", .NONE, "", 1, 0x00000000 },
        .{ "time-infozip.zip", .NONE, "", 1, 0x00000000 },
        .{ "time-osx.zip", .NONE, "", 1, 0x00000000 },
        .{ "time-win7.zip", .NONE, "", 1, 0x00000000 },
        .{ "time-winrar.zip", .NONE, "", 1, 0x00000000 },
        .{ "time-winzip.zip", .NONE, "", 1, 0x00000000 },
        .{ "unix.zip", .NONE, "", 4, 0x6be6ac44 },
        .{ "utf8-7zip.zip", .NONE, "", 1, 0x00000000 },
        .{ "utf8-infozip.zip", .NONE, "", 1, 0x00000000 },
        .{ "utf8-osx.zip", .NONE, "", 1, 0x00000000 },
        .{ "utf8-winrar.zip", .NONE, "", 1, 0x00000000 },
        .{ "utf8-winzip.zip", .NONE, "", 1, 0x00000000 },
        .{ "winxp.zip", .NONE, "", 4, 0x6be6ac44 },
        .{ "zip64-2.zip", .NONE, "", 1, 0x69ffe77e },
        .{ "zip64.zip", .NONE, "", 1, 0x69ffe77e },
    ];

    TAR_FIXTURES :: Fixture.[
        .{ "file-and-dir.tar", .NONE, "", 2, 0x6cbd88fc },
        .{ "gnu-incremental.tar", .NONE, "sparse TAR is outside v1" },
        .{ "gnu-long-nul.tar", .NONE, "", 1, 0x00000000 },
        .{ "gnu-multi-hdrs.tar", .NONE, "", 1, 0x00000000 },
        .{ "gnu-nil-sparse-data.tar", .NONE, "sparse TAR is outside v1" },
        .{ "gnu-nil-sparse-hole.tar", .NONE, "sparse TAR is outside v1" },
        .{ "gnu-not-utf8.tar", .NONE, "", 1, 0x00000000 },
        .{ "gnu-sparse-big.tar", .NONE, "sparse TAR is outside v1" },
        .{ "gnu-utf8.tar", .NONE, "", 1, 0x00000000 },
        .{ "gnu.tar", .NONE, "", 2, 0x95c82ad3 },
        .{ "hardlink.tar", .NONE, "", 2, 0x7b4b976f },
        .{ "hdr-only.tar", .NONE, "", 16, 0x04e8d6ea },
        .{ "invalid-go17.tar", .NONE, "", 1, 0x00000000 },
        .{ "issue10968.tar", .INVALID, "" },
        .{ "issue11169.tar", .CHECKSUM, "" },
        .{ "issue12435.tar", .INVALID, "" },
        .{ "neg-size.tar", .INVALID, "" },
        .{ "nil-uid.tar", .NONE, "", 1, 0x3691bebc },
        .{ "pax-bad-hdr-file.tar", .INVALID, "" },
        .{ "pax-bad-hdr-large.tar.bz2", .NONE, "bzip2 wrapper is outside v1" },
        .{ "pax-bad-mtime-file.tar", .INVALID, "" },
        .{ "pax-global-records.tar", .NONE, "", 4, 0x00000000 },
        .{ "pax-multi-hdrs.tar", .NONE, "", 1, 0x00000000 },
        .{ "pax-nil-sparse-data.tar", .NONE, "sparse TAR is outside v1" },
        .{ "pax-nil-sparse-hole.tar", .NONE, "sparse TAR is outside v1" },
        .{ "pax-nul-path.tar", .INVALID, "" },
        .{ "pax-nul-xattrs.tar", .NONE, "", 1, 0x00000000 },
        .{ "pax-path-hdr.tar", .INVALID, "" },
        .{ "pax-pos-size-file.tar", .NONE, "", 1, 0x5fd7e86a },
        .{ "pax-records.tar", .NONE, "", 1, 0x00000000 },
        .{ "pax-sparse-big.tar", .NONE, "sparse TAR is outside v1" },
        .{ "pax.tar", .NONE, "", 2, 0x6820bd93 },
        .{ "sparse-formats.tar", .NONE, "sparse TAR is outside v1" },
        .{ "star.tar", .NONE, "", 2, 0x95c82ad3 },
        .{ "trailing-slash.tar", .NONE, "", 1, 0x00000000 },
        .{ "ustar-file-devs.tar", .NONE, "", 1, 0x00000000 },
        .{ "ustar-file-reg.tar", .NONE, "", 1, 0x50086c36 },
        .{ "ustar.tar", .NONE, "", 1, 0x363a3020 },
        .{ "v7.tar", .NONE, "", 2, 0x95c82ad3 },
        .{ "writer-big-long.tar", .INVALID, "" },
        .{ "writer-big.tar", .INVALID, "" },
        .{ "writer.tar", .NONE, "", 3, 0x95c82ad3 },
        .{ "xattrs.tar", .NONE, "", 2, 0x95c82ad3 },
    ];

    conformance :: (t: *Test) {
        // tools/repo_output.py: the repo's target/ unless JACKET_OUTPUT_DIR relocates it.
        base := environment_variable("JACKET_OUTPUT_DIR");
        if !base.count base = tprint("%../../../../target", #filepath);

        root := tprint("%/archive-conformance", base);

        require(file_exists(tprint("%/zip", root)), "run `PYTHONPATH=tools python3 tools/portable/build_portable.py data` first");

        for format: Archive_Format.[.ZIP, .TAR] {
            fixtures: [] Fixture = ifx format == .ZIP then cast([] Fixture) ZIP_FIXTURES else cast([] Fixture) TAR_FIXTURES;
            folder := ifx format == .ZIP then "zip" else "tar";

            for fixtures {
                if it.skip_reason.count {
                    log(t, "skip %: %", it.name, it.skip_reason);
                    continue;
                }

                bytes, ok := read_entire_file(tprint("%/%/%", root, folder, it.name));

                require(ok, it.name);
                a, error := read_archive(cast([] u8) bytes, format);
                if error.kind == .NONE {
                    expect(a.entries.count == it.count, tprint("%: entries %, expected %", it.name, a.entries.count, it.count));
                    crc: u32;

                    for e, i: a.entries {
                        d, failure := read_entry(*a, i);

                        crc = Deflate.crc32(d.bytes, crc);
                        free_archive(*d);

                        if failure.kind != .NONE {
                            error = failure;
                            break;
                        }
                    }

                    if error.kind == .NONE expect(crc == it.payload_crc, it.name);
                }

                expect(error.kind == it.expected, tprint("%: expected %, got % (% at %)", it.name, it.expected, error.kind, error.message, error.offset));
                free_archive(*error);
                free_archive(*a);
                free(bytes);
            }
        }
    } @test
}
