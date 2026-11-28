// test: only(.LINUX)
// The TCP header accessors read and write the bytes the kernel sends, whatever the CPU's byte order.
#import "Basic";
S :: #import "Socket";

// The header length sits in the high nibble of byte 12, the flags in byte 13.
#assert #run verify();

verify :: () -> bool {
    packet: S.tcphdr;
    bytes := cast(*u8) *packet;

    S.set_anon_struct_1_doff(*packet, 5);
    S.set_anon_struct_1_res1(*packet, 3);
    S.set_anon_struct_1_fin(*packet, true);
    S.set_anon_struct_1_ack(*packet, true);
    S.set_anon_struct_1_urg(*packet, true);
    S.set_anon_struct_1_res2(*packet, 2);

    if bytes[12] != 0x53 || bytes[13] != 0xb1 return false;
    if S.get_anon_struct_1_doff(packet) != 5 || S.get_anon_struct_1_res1(packet) != 3 return false;
    if !S.get_anon_struct_1_fin(packet) || !S.get_anon_struct_1_ack(packet) || !S.get_anon_struct_1_urg(packet) return false;
    if S.get_anon_struct_1_syn(packet) || S.get_anon_struct_1_rst(packet) || S.get_anon_struct_1_psh(packet) return false;
    if S.get_anon_struct_1_res2(packet) != 2 return false;

    info: S.tcp_info;

    S.set_tcpi_snd_wscale(*info, 0xA);
    S.set_tcpi_rcv_wscale(*info, 0x5);

    // The kernel declares the send scale first: the low nibble on little-endian CPUs, the high one on big-endian.
    #if ARCHITECTURE == .S390X || ARCHITECTURE == .POWERPC64 expected: u8 = 0xa5;
    else expected: u8 = 0x5a;

    if (cast(*u8) *info)[6] != expected return false;
    if S.get_tcpi_snd_wscale(info) != 0xA || S.get_tcpi_rcv_wscale(info) != 0x5 return false;

    return true;
}

main :: () {
    assert(verify());
}
