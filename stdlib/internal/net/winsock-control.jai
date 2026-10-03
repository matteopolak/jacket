IOW :: (x: u8, y: u8, $T: Type) -> s32 {
    group: u32 = x;
    command: u32 = y;
    argument_bytes := cast(u32) (size_of(T) & 127);
    word: u32 = 0x80000000 | (argument_bytes << 16) | (group << 8) | command;

    return cast,no_check(s32) word;
}

add_descriptor :: (fd: u64, set: *$T) {
    assert(set && set.fd_count <= set.fd_array.count, "invalid Winsock descriptor set");
    for i: 0..cast(s64) set.fd_count - 1 if set.fd_array[i] == fd return;

    if set.fd_count < set.fd_array.count {
        set.fd_array[set.fd_count] = fd;
        set.fd_count += 1;
    }
}

remove_descriptor :: (fd: u64, set: *$T) {
    assert(set && set.fd_count <= set.fd_array.count, "invalid Winsock descriptor set");

    for i: 0..cast(s64) set.fd_count - 1 {
        if set.fd_array[i] != fd continue;
        for next: i + 1..cast(s64) set.fd_count - 1 set.fd_array[next - 1] = set.fd_array[next];
        set.fd_count -= 1;
        return;
    }
}

#scope_file
#import "Basic";
