// Reviewed mutable native declarations must forward entities, not values.
using #import "Basic";
S :: #import "Socket";
B :: #import "Socket_Storage_Bridge";

main :: () {
    #assert type_of(S.h_errno) == s32;
    #assert type_of(S.in6addr_any) == S.in6_addr;
    #assert type_of(S.in6addr_loopback) == S.in6_addr;
    assert(cast(*void) *S.h_errno == B.h_errno_address());
    assert(cast(*void) *S.in6addr_any == B.any_address());
    assert(cast(*void) *S.in6addr_loopback == B.loopback_address());
    old := S.h_errno;

    S.h_errno = 41;
    assert(S.h_errno == 41);
    assert(B.h_errno_value() == 41);
    S.h_errno = old;
    assert(S.h_errno == old);
    print("ok\n");
}
