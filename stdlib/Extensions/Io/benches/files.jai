#scope_file
// Run with jacket bench --path stdlib/Extensions/Io.
#import "Extensions/Testing";
#import "Basic";
#import "File";
#import "Extensions/Environment";
Io :: #import "Extensions/Io";

libc :: #system_library "libc"; @native(external)
setenv :: (name: *u8, value: *u8, overwrite: s32) -> s32 #foreign libc;
FILE_BYTES :: 64 * 1024 * 1024;

File_Read :: struct {
    file: *Io.Io_File;
    buffer: [] u8;
    seed: int;
    iterations: int;
    group: *Io.Wait_Group;
}

reader :: (job: File_Read) {
    defer Io.done(job.group);
    position := job.seed;

    for 0..job.iterations - 1 {
        position = (position * 1103515245 + 12345) & 0x7fffffff;
        offset := (position % (FILE_BYTES / job.buffer.count)) * job.buffer.count;
        got, error := Io.read_at(job.file, job.buffer, black_box(offset));

        assert(error == .NONE && got == job.buffer.count);
        black_box(job.buffer);
    }
}

file_ready: bool;

file_case :: (b: *Bench, size: int, fibers: int, inline_reads: bool) {
    base := environment_variable("JACKET_OUTPUT_DIR");
    if !base.count base = tprint("%../../../../target", #filepath);

    make_directory_if_it_does_not_exist(base);
    path := tprint("%/io-bench.bin", base);

    if !file_ready {
        data := NewArray(FILE_BYTES, u8);
        for *data it.* = cast(u8) (it_index & 0xff);

        assert(write_entire_file(path, cast(string) data));
        array_free(data);
        file_ready = true;
    }

    previous := environment_variable("JAI_IO_FILE_INLINE");

    setenv("JAI_IO_FILE_INLINE", temp_c_string(ifx inline_reads then "1073741824" else "0"), 1);
    defer setenv("JAI_IO_FILE_INLINE", temp_c_string(previous), 1);
    state := File_Bench.{ b, path, size, fibers };

    Io.io_run((state: *File_Bench) {
        b := state.b;

        files: [8] Io.Io_File;
        buffers: [8][] u8;

        for index: 0..state.fibers - 1 {
            error: Io.Io_Error;
            files[index], error = Io.open_file(state.path, .READ);
            assert(error == .NONE);
            buffers[index] = NewArray(state.size, u8);
        }

        defer for index: 0..state.fibers - 1 {
            Io.close(*files[index]);
            free(buffers[index].data);
        }

        group: Io.Wait_Group;

        Io.add(*group, state.fibers);

        for index: 0..state.fibers - 1 {
            Io.spawn(reader, File_Read.{ *files[index], buffers[index], index + 1, b.iterations, *group });
        }

        set_bytes(b, state.size * state.fibers);
        reset_timer(b);
        Io.wait(*group);
    }, *state);
}

File_Bench :: struct {
    b: *Bench;
    path: string;
    size: int;
    fibers: int;
}

file_benches :: () -> string {
    out: String_Builder;

    for size: int.[512, 4096, 16384, 65536, 262144, 1048576] for fibers : int.[1, 8] for mode : 0..1 {
        print(*out, "file_%_%_% :: (b: *Bench) { file_case(b, %, %, %); } @bench\n", size, fibers, ifx mode == 0 then "inline" else "pool", size, fibers, mode == 0);
    }

    return builder_to_string(*out);
}

#scope_export
#insert #run file_benches();
