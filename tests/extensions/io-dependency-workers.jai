// Worker threads must finish before their contexts, pools and stacks are freed.
#import "Basic";
Atomics :: #import "Atomics";
Io :: #import "Extensions/Io";

completed: s32;

worker :: () {
    Io.sleep(5);
    for 1..10 Io.yield();
    Atomics.atomic_add(*completed, 1);
}

files :: () {
    argv := get_command_line_arguments();

    assert(argv.count == 2, "pass a scratch file path");

    file, error := Io.open_file(argv[1], .WRITE);

    assert(error == .NONE);
    data: [32768] u8;
    for *data it.* = cast(u8) (it_index % 251);

    count, write_error := Io.write(*file, data);

    assert(count == data.count && write_error == .NONE);
    assert(Io.close(*file) == .NONE);

    file, error = Io.open_file(argv[1]);
    assert(error == .NONE);
    read_back: [32768] u8;
    read_count, read_error := Io.read_full(*file, read_back);

    assert(read_count == read_back.count && read_error == .NONE);
    for read_back assert(it == data[it_index]);

    assert(Io.close(*file) == .NONE);
}

main :: () {
    Io.io_run_threads(4, worker);
    assert(atomic_load(*completed) == 4, "io_run_threads must wait for every worker");
    Io.io_run(files);
    print("ok\n");
}
