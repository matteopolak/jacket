// Independent SDK ownership regression; no reads through the protected pointer.
// TEST_VM names the own source-built Mach test helper supplied by the runner.
#import "Basic";
U :: #import "Unmapping_Allocator";
H :: #import "Hash_Table";

errors: s64;

capture :: (message: string, data: *void, info: Log_Info) {
    if (info.common_flags & .ERROR) != 0 errors += 1;
}

recover :: (loc: Source_Code_Location, message: string) -> bool {
    return false;
}

mark :: (calls: *s64) -> bool {
    calls.* += 1;
    return true;
}

main :: () {
    args := get_command_line_arguments();
    if args.count != 2 exit(8);

    calls: s64;

    assert(mark(*calls));

    if args[1] == "enabled" {
        if calls != 1 exit(9);
    } else if args[1] == "disabled" {
        if calls != 0 exit(10);
    } else exit(11);

    allocator := U.get_unmapping_allocator();
    owner := cast(*U.Unmapping_Allocator) allocator.data;
    memory := allocator.proc(.ALLOCATE, 16, 0, null, allocator.data);

    if !memory exit(2);
    if deny_read(memory) != 0 exit(3);

    saved := context;
    scoped := context;
    scoped.logger = capture;
    scoped.assertion_failed = recover;
    context = scoped;
    allocator.proc(.FREE, 0, 0, memory, allocator.data);
    context = saved;
    if errors == 0 exit(4);
    if !H.table_contains(*owner.table, memory) exit(5);
    if restore(memory) != 0 exit(6);

    errors = 0;
    context = scoped;
    allocator.proc(.FREE, 0, 0, memory, allocator.data);
    context = saved;
    if errors != 0 || H.table_contains(*owner.table, memory) exit(7);

    U.deinit_unmapping_allocator(allocator);
    write_string("ok\n");
}

#scope_file
deny_read :: (memory: *void) -> s32 #foreign TEST_VM "test_unmapping_deny_read";
restore :: (memory: *void) -> s32 #foreign TEST_VM "test_unmapping_restore";
