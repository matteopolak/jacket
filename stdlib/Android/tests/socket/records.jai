// Exact reviewed public Android shapes; compile with the actual Android target.
#import "Basic";
S :: #import "Socket";

#assert OS == .ANDROID;
#assert size_of(S.sockaddr) == 16;
#assert size_of(S.linger) == 8;
#assert size_of(S.in_addr) == 4;
#assert size_of(S.ip_mreqn) == 12;
#assert size_of(S.in_pktinfo) == 12;
#assert size_of(S.SHUT) == 4;
#assert S.SHUT.RD == S.SHUT.SHUT_RD && S.SHUT.WR == S.SHUT.SHUT_WR && S.SHUT.RDWR == S.SHUT.SHUT_RDWR;
#assert cast(u32) S.SHUT.RD == 0 && cast(u32) S.SHUT.WR == 1 && cast(u32) S.SHUT.RDWR == 2;
#assert #run verify();

verify :: () -> bool {
    address: S.sockaddr;
    linger: S.linger;
    membership: S.ip_mreqn;
    packet: S.in_pktinfo;

    #assert type_of(address.sa_family) == S.sa_family_t;
    #assert type_of(address.sa_data) == [14] u8;
    #assert type_of(linger.l_onoff) == s32 && type_of(linger.l_linger) == s32;
    #assert type_of(membership.imr_ifindex) == s32 && type_of(packet.ipi_ifindex) == s32;
    #assert type_of(packet.ipi_addr.s_addr) == S.in_addr_t;

    if cast(u64) *address.sa_data - cast(u64) *address != 2 return false;
    if cast(u64) *linger.l_linger - cast(u64) *linger != 4 return false;
    if cast(u64) *membership.imr_address - cast(u64) *membership != 4 return false;
    if cast(u64) *membership.imr_ifindex - cast(u64) *membership != 8 return false;
    if cast(u64) *packet.ipi_spec_dst - cast(u64) *packet != 4 return false;
    if cast(u64) *packet.ipi_addr - cast(u64) *packet != 8 return false;
    if S.CMSG_SIZE(0) != 16 || S.CMSG_SIZE(1) != 24 || S.CMSG_SIZE(8) != 24 || S.CMSG_SIZE(9) != 32 return false;

    return true;
}

main :: () {
    assert(verify());
    print("ok\n");
}
