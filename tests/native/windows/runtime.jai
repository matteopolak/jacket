// Self-checking program for native Windows builds (also runs on macOS and Linux): the
// runtime start-up, Basic, String, Hash_Table, File, Thread and the C ABI for small structs.
// Prints "ok" and exits 0 when every check holds. See docs/native/windows.md.
#import "Basic";
#import "Math";
#import "String";
#import "Hash_Table";
#import "File";
#import "Thread";

failures := 0;
check :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    failures += 1;
    print("FAILED: % (line %)\n", what, loc.line_number);
}

Pair :: struct { a: s32; b: s32; }
Big :: struct { x, y, z: s64; }
Floats :: struct { f: float; g: float; }

add_pair :: (p: Pair) -> Pair #c_call { return .{p.a + 1, p.b + 2}; }
sum_big :: (b: Big) -> Big #c_call { return .{b.x + b.y + b.z, b.y, b.z}; }
scale_floats :: (f: Floats, k: float) -> Floats #c_call { return .{f.f * k, f.g * k}; }
many :: (a: s64, b: float64, c: s64, d: float, e: s64, f: float64, g: Pair) -> float64 #c_call {
    return cast(float64) (a + c + e + g.a + g.b) + b + d + f;
}

counter: s64;
counter_mutex: Mutex;

worker :: (thread: *Thread) -> s64 {
    for 1..1000 {
        lock(*counter_mutex);
        counter += 1;
        unlock(*counter_mutex);
    }
    // Each thread has its own context and temporary storage.
    s := tprint("thread %", thread.index);
    return s.count;
}

main :: () {
    // Basic: formatting, dynamic arrays, temporary storage, the default allocator.
    check(tprint("% % %", 42, "x", 1.5) == "42 x 1.5", "tprint");
    numbers: [..] int;
    for 0..9999 array_add(*numbers, it * 2);
    total := 0;
    for numbers total += it;
    check(total == 99990000, "array sum");
    mark := get_temporary_storage_mark();
    for 1..1000 { s := tprint("% and %", it, it * 3); check(s.count > 0, "tprint"); }
    set_temporary_storage_mark(mark);
    big := alloc(1 << 24);
    memset(big, 7, 1 << 24);
    check((cast(*u8) big)[(1 << 24) - 1] == 7, "large allocation");
    free(big);
    check(abs(sqrt(2.0) - 1.41421356) < 0.00001, "sqrt");
    args := get_command_line_arguments();
    for args check(it.count > 0, "command line argument");  // `jaic run` passes none.
    #if OS == .WINDOWS check(args.count == 1 && ends_with(args[0], ".exe"), "argv[0] from the wide command line");

    // String.
    parts := split("alpha,beta,gamma", ",");
    check(parts.count == 3 && parts[2] == "gamma", "split");
    check(join(..parts, "+") == "alpha+beta+gamma", "join");
    check(to_upper_copy("windows") == "WINDOWS", "to_upper_copy");
    found, before, after := split_from_left("key=value", #char "=");
    check(found && before == "key" && after == "value", "split_from_left");

    // Hash_Table.
    table: Table(string, int);
    for 0..499 table_add(*table, tprint("k%", it), it);
    was_found, found_value := table_find(*table, "k321");
    check(was_found && found_value == 321, "table_find");
    check(table.count == 500, "table count");

    // File: write, read back, delete (in the working directory).
    name := "jaic-windows-runtime-test.txt";
    check(write_entire_file(name, "line one\nline two\n"), "write_entire_file");
    text, read_ok := read_entire_file(name);
    check(read_ok && text == "line one\nline two\n", "read_entire_file");
    check(file_exists(name), "file_exists");
    check(file_delete(name), "file_delete");
    check(!file_exists(name), "file deleted");

    // Thread: four workers contending on a mutex.
    init(*counter_mutex);
    threads: [4] Thread;
    for *threads {
        check(thread_init(it, worker), "thread_init");
        check(thread_start(it), "thread_start");
    }
    for *threads {
        while !thread_is_done(it, 1000) {}
        thread_deinit(it);
    }
    check(counter == 4000, "threads counted");

    // C ABI for #c_call procedures called through pointers: small structs travel in an
    // integer register on Win64, others by reference to a copy.
    pair_proc := add_pair;
    p := pair_proc(.{10, 20});
    check(p.a == 11 && p.b == 22, "8-byte struct");
    big_proc := sum_big;
    b := big_proc(.{1, 2, 3});
    check(b.x == 6 && b.y == 2 && b.z == 3, "24-byte struct");
    scale_proc := scale_floats;
    f := scale_proc(.{1.5, 2.5}, 2);
    check(f.f == 3 && f.g == 5, "two-float struct");
    many_proc := many;
    check(many_proc(1, 0.5, 2, 0.25, 3, 0.125, .{4, 5}) == 15.875, "stack arguments");

    if failures == 0 print("ok\n");
}
