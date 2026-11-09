// test: skip_on(.WASM, reason = "Extensions/Io needs fibers, which wasm cannot run (no stack switching)")

#if OS != .WASM {
    #import "Basic";
    #import "File";
    #import "Extensions/Archive";
    #import "Extensions/Testing";
    Io :: #import "Extensions/Io";

    // The adapter shown in the `Archive_Writer` docs.
    forward :: (data: *void, bytes: [] u8) -> bool {
        _, error := Io.write((cast(*Io.Writer) data).*, bytes);
        return error == .NONE;
    }

    out_path: string;

    // An `Extensions/Io` writer receives archive output through `Archive_Writer`.
    io_writer_adapter :: (t: *Test) {
        out_path = copy_string(tprint("%/out.tar", temp_dir(t)));
        defer free(out_path);

        Io.io_run(() {
            file, open_error := Io.open_file(out_path, .WRITE);

            assert(open_error == .NONE);

            sink := Io.writer_of(*file);
            error := write_archive(Archive_Writer.{ data = *sink, write_proc = forward }, Archive_Item.[.{ name = "a.txt", data = cast([] u8) "hi" }], .TAR);

            assert(error.kind == .NONE);
            Io.close(*file);
        });

        bytes, ok := read_entire_file(out_path);

        require(ok);
        defer free(bytes);
        archive, failure := read_archive(cast([] u8) bytes, .TAR);

        require(failure.kind == .NONE && archive.entries.count == 1);
        expect(archive.entries[0].name == "a.txt");
        free_archive(*archive);
    } @test
}
