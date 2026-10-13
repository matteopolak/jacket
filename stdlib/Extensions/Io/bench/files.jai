// File reads through Io on the readiness backends (epoll, kqueue): every size once with the read
// run inline on the scheduler's thread and once on the worker pool, from one fiber (latency of a
// single call) and from eight fibers (what overlapping buys). The file is in the page cache, which
// is the case the inline threshold (`JAI_IO_FILE_INLINE`) is chosen for.
//
//     jaic -O2 stdlib/Extensions/Io/bench/files.jai -exe /tmp/io-files-bench
//     /tmp/io-files-bench
#import "Basic";
Io :: #import "Extensions/Io";

libc :: #system_library "libc";
setenv :: (name: *u8, value: *u8, overwrite: s32) -> s32 #foreign libc;
mkdir :: (path: *u8, mode: u32) -> s32 #foreign libc;

FILE_BYTES :: 64 * 1024 * 1024;
PATH :: ".scratch/files-bench.bin";

Job :: struct {
    path: string;
    size: int;
    count: int;
    group: *Io.Wait_Group;
    seed: int;
}

reader :: (job: Job) {
    defer Io.done(job.group);

    file, error := Io.open_file(job.path, .READ);

    assert(error == .NONE, "open: %", error);

    defer Io.close(*file);

    buffer := NewArray(job.size, u8, false);

    defer free(buffer.data);

    position := job.seed;

    for 1..job.count {
        position = (position * 1103515245 + 12345) & 0x7fffffff;

        offset := (position % (FILE_BYTES / job.size)) * job.size;
        got, read_error := Io.read_at(*file, buffer, offset);

        assert(read_error == .NONE && got == job.size, "read: % %", got, read_error);
    }
}

run_case :: (size: int, fibers: int) -> float64, float64 {
    total := min(20000, 512 * 1024 * 1024 / size);

    started := current_time_monotonic();
    group: Io.Wait_Group;

    Io.add(*group, fibers);
    for 1..fibers Io.spawn(reader, Job.{ PATH, size, total / fibers, *group, it });

    Io.wait(*group);

    seconds := to_float64_seconds(current_time_monotonic() - started);

    return seconds / total * 1_000_000, (total * size) / seconds / (1024 * 1024);
}

Case :: struct {
    size: int;
    fibers: int;
    microseconds: float64;
    megabytes: float64;
}

results: [..] Case;
current_size: int;
current_fibers: int;

measure :: () {
    microseconds, megabytes := run_case(current_size, current_fibers);

    array_add(*results, .{ current_size, current_fibers, microseconds, megabytes });
}

prepare :: () {
    mkdir(".scratch", 0x1ed);

    data := NewArray(FILE_BYTES, u8, false);

    defer free(data.data);

    for *data it.* = cast(u8) (it_index & 0xff);

    assert(Io.write_file(PATH, data) == .NONE);

    print("backend %\n", Io.backend_name());
}

main :: () {
    setenv("JAI_IO_FILE_INLINE", "0", 1);
    Io.io_run(prepare);

    sizes := int.[512, 4096, 16384, 65536, 262144, 1048576];

    for size: sizes {
        for fibers: int.[1, 8] {
            for mode: string.["inline", "pool"] {
                setenv("JAI_IO_FILE_INLINE", temp_c_string(ifx mode == "inline" then "1073741824" else "0"), 1);

                results.count = 0;
                current_size = size;
                current_fibers = fibers;

                // The best of three, to keep a warm-up out of the numbers.
                best_microseconds: float64 = 1.0e18;
                best_megabytes: float64 = 0.0;

                for 1..3 {
                    results.count = 0;
                    Io.io_run(measure);

                    if results[0].microseconds < best_microseconds {
                        best_microseconds = results[0].microseconds;
                        best_megabytes = results[0].megabytes;
                    }
                }

                print("% bytes, % fibers, %: % us per read, % MB/s\n", size, fibers, mode, formatFloat(best_microseconds, trailing_width = 2), formatFloat(best_megabytes, trailing_width = 0));
            }
        }
    }
}
