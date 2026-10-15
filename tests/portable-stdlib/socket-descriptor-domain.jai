// Reviewed descriptor domain. Cross-target #run uses no native Winsock calls.
#import "Basic";
S :: #import "Socket";

#assert !#exists(S.FD_ZERO);
#assert #run verify();

same_set :: (a: S.fd_set, b: S.fd_set) -> bool {
    #if OS == .WINDOWS {
        if a.fd_count != b.fd_count return false;
        for index: 0..a.fd_array.count - 1 if a.fd_array[index] != b.fd_array[index] return false;
    } else {
        for index: 0..a.fds_bits.count - 1 if a.fds_bits[index] != b.fds_bits[index] return false;
    }

    return true;
}

verify :: () -> bool {
    #if OS == .WINDOWS {
        set: S.fd_set;
        opaque: [4] S.Socket = .[0, 0xffffffffffffffff, 0x8000000000000000, 1024];

        for value: opaque {
            S.FD_SET(value, *set);
            S.FD_SET(value, *set);
        }

        if set.fd_count != 4 return false;
        for index: 0..3 if set.fd_array[index] != opaque[index] return false;

        before := set;

        S.FD_CLR(12345, *set);
        if !same_set(before, set) return false;

        S.FD_CLR(opaque[1], *set);
        if set.fd_count != 3 || set.fd_array[0] != opaque[0] || set.fd_array[1] != opaque[2] || set.fd_array[2] != opaque[3] return false;

        full: S.fd_set;
        for index: 0..full.fd_array.count - 1 S.FD_SET(cast(S.Socket) index, *full);

        if full.fd_count != full.fd_array.count return false;

        before = full;
        S.FD_SET(0xffffffffffffffff, *full);
        S.FD_SET(0, *full);
        S.FD_CLR(0xffffffffffffffff, *full);
        if !same_set(before, full) return false;

        S.FD_CLR(31, *full);
        if full.fd_count != 63 || full.fd_array[31] != 32 || full.fd_array[62] != 63 return false;

        S.FD_SET(0xffffffffffffffff, *full);
        if full.fd_count != 64 || full.fd_array[63] != 0xffffffffffffffff return false;
    } else {
        guarded: struct {
            prefix: u64 = 0x123456789abcdef0;
            set: S.fd_set;
            suffix: u64 = 0xfedcba9876543210;
        };

        capacity :: size_of(S.fd_set) * 8;

        for fd: 0..capacity - 1 {
            S.FD_SET(cast(s32) fd, *guarded.set);
            if S.FD_ISSET(cast(s32) fd, *guarded.set) == 0 return false;
        }

        bytes := cast(*u8) *guarded.set;
        for index: 0..size_of(S.fd_set) - 1 if bytes[index] != 255 return false;
        for fd: 0..capacity - 1 if (fd & 1) != 0 S.FD_CLR(cast(s32) fd, *guarded.set);
        for index: 0..size_of(S.fd_set) - 1 if bytes[index] != 0x55 return false;

        S.FD_SET(cast(s32) (capacity - 1), *guarded.set);
        if S.FD_ISSET(cast(s32) (capacity - 1), *guarded.set) == 0 return false;

        before := guarded.set;

        for fd: s32.[cast(s32) capacity, cast(s32) (capacity + 1), 0x7fffffff] {
            S.FD_SET(fd, *guarded.set);
            S.FD_CLR(fd, *guarded.set);
            if S.FD_ISSET(fd, *guarded.set) != 0 || !same_set(before, guarded.set) return false;
        }

        if guarded.prefix != 0x123456789abcdef0 || guarded.suffix != 0xfedcba9876543210 return false;
    }

    return true;
}

main :: () {
    assert(verify());
    print("ok\n");
}
