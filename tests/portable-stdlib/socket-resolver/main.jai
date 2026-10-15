// Reviewed macOS addrinfo shape, canonical metadata and real numeric resolver.
#import "Basic";
S :: #import "Socket";

main :: () {
    // Public mutable SDK data must remain the original storage, never a value copy.
    assert(*S.h_errno == *sdk_h_errno);
    assert(*S.in6addr_any == *sdk_in6addr_any);
    original_error := S.h_errno;

    defer S.h_errno = original_error;
    S.h_errno = 123;
    assert(sdk_h_errno == 123);
    sdk_h_errno = original_error;
    assert(S.h_errno == original_error);
    assert(type_info(S.addrinfo).name == "addrinfo");
    info: S.addrinfo;

    assert(type_of(info.ai_flags) == S.AI && type_of(info.ai_family) == s32);
    assert(type_of(info.ai_socktype) == S.SOCK && type_of(info.ai_protocol) == S.IPPROTO);
    assert(type_of(info.ai_addrlen) == S.socklen_t);
    assert(size_of(S.addrinfo) == 48);
    assert(cast(u64) *info.ai_canonname - cast(u64) *info == 24);
    assert(cast(u64) *info.ai_addr - cast(u64) *info == 32);
    hints: S.addrinfo;
    hints.ai_flags = .NUMERICHOST | .NUMERICSERV;
    hints.ai_family = cast(s32) S.AF_INET;
    hints.ai_socktype = .STREAM;
    hints.ai_protocol = .TCP;
    host := to_c_string("127.0.0.1");
    service := to_c_string("32123");

    defer free(host);
    defer free(service);
    result: *S.addrinfo;
    status := S.getaddrinfo(host, service, *hints, *result);

    assert(status == 0 && result != null);
    defer S.freeaddrinfo(result);
    assert(result.ai_family == cast(s32) S.AF_INET && result.ai_socktype == .STREAM && result.ai_protocol == .TCP);
    assert(result.ai_addrlen == size_of(S.sockaddr_in) && result.ai_addr != null);
    address := cast(*S.sockaddr_in) result.ai_addr;

    assert(S.ntohs(address.sin_port) == 32123 && S.ntohl(address.sin_addr.s_addr) == 0x7f000001);
    print("ok\n");
}

#scope_file
libc :: #library,system "libc";
sdk_h_errno: s32 #elsewhere libc "h_errno";
sdk_in6addr_any: S.in6_addr #elsewhere libc "in6addr_any";
