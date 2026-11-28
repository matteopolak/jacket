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
    print("ok\n");
}
