<<<<<<< HEAD:tests/portable-stdlib/winsock-control.jai
#load "../../stdlib/Socket/tests/portable-winsock-control.jai";
=======
// Exact production pure kernels; this does not emulate or claim Winsock execution.
#import "Basic";
#load "../../internal/net/winsock-control.jai";

Descriptor_Set :: struct {
    fd_count: u32;
    fd_array: [64] u64;
}

main :: () {
    for group: 0..255 {
        for command: 0..255 {
            actual := IOW(cast(u8) group, cast(u8) command, u32);
            expected := cast(u64) (0x80040000 + group * 256 + command);
            assert(cast,no_check(u32) actual == expected);
        }
    }

    assert(cast,no_check(u32) IOW(102, 126, u32) == 0x8004667e);
    assert(cast,no_check(u32) IOW(250, 251, [300] u8) == 0x802cfafb);
    set: Descriptor_Set;
    for i: 0..63 add_descriptor(0x100000000 + cast(u64) i, *set);

    assert(set.fd_count == 64);
    for i: 0..63 add_descriptor(0x100000000 + cast(u64) i, *set);

    assert(set.fd_count == 64);
    add_descriptor(123, *set);
    assert(set.fd_count == 64 && set.fd_array[63] == 0x10000003f);
    remove_descriptor(0x10000001f, *set);
    assert(set.fd_count == 63 && set.fd_array[31] == 0x100000020);
    remove_descriptor(123, *set);
    assert(set.fd_count == 63);
    add_descriptor(0xffffffffffffffff, *set);
    assert(set.fd_count == 64 && set.fd_array[63] == 0xffffffffffffffff);
    remove_descriptor(0xffffffffffffffff, *set);
    for i: 0..63 remove_descriptor(0x100000000 + cast(u64) i, *set);

    assert(set.fd_count == 0);
    print("ok\n");
}
>>>>>>> main:stdlib/Socket/tests/winsock-control.jai
