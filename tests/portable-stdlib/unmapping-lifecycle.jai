// Reviewed portable lifecycle; Windows additionally checks actual SDK page states.
#import "Basic";
U :: #import "Unmapping_Allocator";
H :: #import "Hash_Table";

errors: s64;

capture :: (message: string, data: *void, info: Log_Info) {
    if (info.common_flags & .ERROR) != 0 errors += 1;
}

main :: () {
    allocator := U.get_unmapping_allocator();
    owner := cast(*U.Unmapping_Allocator) allocator.data;
    name: string;
    caps := cast(Allocator_Caps) cast(u64) allocator.proc(.CAPS, 0, 0, *name, allocator.data);

    if name != "Unmapping_Allocator" || (caps & .MULTIPLE_THREADS) == 0 || (caps & .FREE) == 0 exit(2);

    for size: 0..17 {
        memory := cast(*u8) allocator.proc(.ALLOCATE, size, 0, null, allocator.data);
        if !memory || !H.table_contains(*owner.table, memory) exit(3);

        #if OS == .WINDOWS {
            effective := max(size, 8);
            writable: W.MEMORY_BASIC_INFORMATION;
            guard: W.MEMORY_BASIC_INFORMATION;

            if W.VirtualQuery(memory, *writable, size_of(type_of(writable))) == 0 exit(4);
            if writable.State != W.MEM_COMMIT || writable.Protect != W.PAGE_READWRITE exit(5);
            if W.VirtualQuery(memory + effective, *guard, size_of(type_of(guard))) == 0 exit(6);
            if guard.State != W.MEM_RESERVE exit(7);

            memset(memory, 37, effective);
        } else if size > 0 memset(memory, 37, size);

        if size > 0 {
            saved := context.logger;

            context.logger = capture;
            errors = 0;
            failed := allocator.proc(.RESIZE, 0x7fffffffffffffff, size, memory, allocator.data);

            context.logger = saved;
            if failed || errors == 0 || !H.table_contains(*owner.table, memory) || memory[0] != 37 exit(8);

            memory[0] = 19;
        }

        grown := cast(*u8) allocator.proc(.RESIZE, 47, size, memory, allocator.data);
        if !grown || grown == memory || H.table_contains(*owner.table, memory) exit(9);
        if !H.table_contains(*owner.table, grown) || (size > 0 && grown[0] != 19) exit(10);

        for index: 1..size - 1 if grown[index] != 37 exit(11);

        allocator.proc(.FREE, 0, 0, grown, allocator.data);
        if H.table_contains(*owner.table, grown) exit(12);

        #if OS == .WINDOWS {
            protected: W.MEMORY_BASIC_INFORMATION;
            if W.VirtualQuery(grown, *protected, size_of(type_of(protected))) == 0 exit(13);
            if protected.State != W.MEM_COMMIT || protected.Protect != W.PAGE_READONLY exit(14);
        }
    }

    if owner.table.count != 0 exit(15);

    U.deinit_unmapping_allocator(allocator);
    write_string("ok\n");
}

#scope_file
#if OS == .WINDOWS W :: #import "Windows";
