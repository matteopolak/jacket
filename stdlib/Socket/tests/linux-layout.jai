// test: only(.LINUX)
// The native socket types have the size the target's C library gives them, including on
// 32-bit CPUs where `long` and `size_t` are four bytes.
#import "Basic";
S :: #import "Socket";

WORD :: size_of(*void);

#assert size_of(S.sockaddr_storage) == 128;
#assert size_of(S.fd_set) == 128;
#assert size_of(S.cmsghdr) == WORD + 8;
#assert size_of(S.msghdr) == 7 * WORD;
#assert size_of(S.ifmap) == 2 * WORD + 8;

main :: () {
    // Descriptor `n` is bit `n` of the array of machine words, on every CPU and byte order.
    set: S.fd_set;
    bits := 8 * size_of(*void);

    S.FD_SET(3, *set);
    S.FD_SET(65, *set);
    assert(cast(s64) set.fds_bits[0] == 8, "descriptor 3 is bit 3 of the first word: %", set.fds_bits[0]);
    assert(cast(s64) set.fds_bits[65 / bits] & ((cast(s64) 1) << (65 % bits)) != 0, "descriptor 65");
    assert(S.FD_ISSET(3, *set) == 1 && S.FD_ISSET(65, *set) == 1 && S.FD_ISSET(4, *set) == 0);
    print("ok\n");
}
