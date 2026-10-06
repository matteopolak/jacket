// libc structures whose layout depends on the target CPU, used through the stdlib: stat()
// results (directory bit, file size) and a mutex next to a guard value that glibc must not
// overwrite. AArch64 Linux lays these out differently from x86-64, so a wrong binding reads
// garbage or corrupts the neighbour. Prints "ok"; also built natively by
// crates/jaic-cli/tests/native.rs, which is what covers each CI host's real libc.
#import "Basic";
#import "File";
#import "File_Utilities";
#import "Thread";

Guarded :: struct {
    mutex: Mutex;
    guard: u64;
}

check :: () -> string {
    is_dir, ok := is_directory(".");
    if !ok || !is_dir return "is_directory(.)";

    name := "posix-stat-and-mutex.tmp";
    if !write_entire_file(name, "twelve bytes") return "write_entire_file";
    defer file_delete(name);
    is_dir, ok = is_directory(name);
    if !ok || is_dir return "is_directory(file)";
    modtime, size, found := file_modtime_and_size(name);
    if !found || size != 12 return tprint("file size %", size);

    guarded: Guarded;
    guarded.guard = 0x0123_4567_89ab_cdef;
    init(*guarded.mutex);
    lock(*guarded.mutex);
    unlock(*guarded.mutex);
    destroy(*guarded.mutex);
    if guarded.guard != 0x0123_4567_89ab_cdef return "mutex overwrote its neighbour";
    return "";
}

main :: () {
    failure := check();
    if failure {
        print("failure: %\n", failure);
        exit(1);
    }
    print("ok\n");
}
