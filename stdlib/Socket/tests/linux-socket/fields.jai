// test: only(.LINUX)
// Reviewed Socket API; official glibc TCP bit positions.
#import "Basic";
S :: #import "Socket";

#assert OS == .LINUX;
#assert #run verify();

// The masks below describe a little-endian CPU. A big-endian CPU reads the 16-bit flags word with its
// bytes swapped and numbers the bits of the window scale byte from the other end, so the test maps what
// it reads and writes to that layout.
#if ARCHITECTURE == .S390X || ARCHITECTURE == .POWERPC64 {
    wire16 :: (word: u16) -> u16 {
        return (word << 8) | (word >> 8);
    }

    nibbles :: (byte: u8) -> u8 {
        return (byte << 4) | (byte >> 4);
    }
} else {
    wire16 :: (word: u16) -> u16 {
        return word;
    }

    nibbles :: (byte: u8) -> u8 {
        return byte;
    }
}

verify :: () -> bool {
    if size_of(S.tcphdr) != 20 || size_of(S.tcp_info) != 104 return false;

    for index: 0..255 {
        word: u16 = cast(u16) (index * 257);
        packet: S.tcphdr;
        info: S.tcp_info;

        packet.__bitfield_1 = wire16(cast(u16) (word & 65535));
        if S.get_anon_struct_1_ack(packet) != ((word & 4096) != 0) return false;

        S.set_anon_struct_1_ack(*packet, false);
        if wire16(packet.__bitfield_1) != (cast(u16) (word & 65535) & 61439) return false;

        S.set_anon_struct_1_ack(*packet, true);
        if wire16(packet.__bitfield_1) != (cast(u16) (word & 65535) | 4096) return false;

        packet.__bitfield_1 = wire16(cast(u16) (word & 65535));
        if S.get_anon_struct_1_doff(packet) != (cast(u16) ((word >> 4) & 15)) return false;

        for input: 0..255 {
            packet.__bitfield_1 = wire16(cast(u16) (word & 65535));
            value: u16 = cast(u16) input;

            S.set_anon_struct_1_doff(*packet, value);
            expected: u16 = (cast(u16) (word & 65535) & 65295) | ((value & 15) << 4);
            if wire16(packet.__bitfield_1) != expected || S.get_anon_struct_1_doff(packet) != (value & 15) return false;
        }

        packet.__bitfield_1 = wire16(cast(u16) (word & 65535));
        if S.get_anon_struct_1_fin(packet) != ((word & 256) != 0) return false;

        S.set_anon_struct_1_fin(*packet, false);
        if wire16(packet.__bitfield_1) != (cast(u16) (word & 65535) & 65279) return false;

        S.set_anon_struct_1_fin(*packet, true);
        if wire16(packet.__bitfield_1) != (cast(u16) (word & 65535) | 256) return false;

        packet.__bitfield_1 = wire16(cast(u16) (word & 65535));
        if S.get_anon_struct_1_psh(packet) != ((word & 2048) != 0) return false;

        S.set_anon_struct_1_psh(*packet, false);
        if wire16(packet.__bitfield_1) != (cast(u16) (word & 65535) & 63487) return false;

        S.set_anon_struct_1_psh(*packet, true);
        if wire16(packet.__bitfield_1) != (cast(u16) (word & 65535) | 2048) return false;

        packet.__bitfield_1 = wire16(cast(u16) (word & 65535));
        if S.get_anon_struct_1_res1(packet) != (cast(u16) ((word >> 0) & 15)) return false;

        for input: 0..255 {
            packet.__bitfield_1 = wire16(cast(u16) (word & 65535));
            value: u16 = cast(u16) input;

            S.set_anon_struct_1_res1(*packet, value);
            expected: u16 = (cast(u16) (word & 65535) & 65520) | ((value & 15) << 0);
            if wire16(packet.__bitfield_1) != expected || S.get_anon_struct_1_res1(packet) != (value & 15) return false;
        }

        packet.__bitfield_1 = wire16(cast(u16) (word & 65535));
        if S.get_anon_struct_1_res2(packet) != (cast(u16) ((word >> 14) & 3)) return false;

        for input: 0..255 {
            packet.__bitfield_1 = wire16(cast(u16) (word & 65535));
            value: u16 = cast(u16) input;

            S.set_anon_struct_1_res2(*packet, value);
            expected: u16 = (cast(u16) (word & 65535) & 16383) | ((value & 3) << 14);
            if wire16(packet.__bitfield_1) != expected || S.get_anon_struct_1_res2(packet) != (value & 3) return false;
        }

        packet.__bitfield_1 = wire16(cast(u16) (word & 65535));
        if S.get_anon_struct_1_rst(packet) != ((word & 1024) != 0) return false;

        S.set_anon_struct_1_rst(*packet, false);
        if wire16(packet.__bitfield_1) != (cast(u16) (word & 65535) & 64511) return false;

        S.set_anon_struct_1_rst(*packet, true);
        if wire16(packet.__bitfield_1) != (cast(u16) (word & 65535) | 1024) return false;

        packet.__bitfield_1 = wire16(cast(u16) (word & 65535));
        if S.get_anon_struct_1_syn(packet) != ((word & 512) != 0) return false;

        S.set_anon_struct_1_syn(*packet, false);
        if wire16(packet.__bitfield_1) != (cast(u16) (word & 65535) & 65023) return false;

        S.set_anon_struct_1_syn(*packet, true);
        if wire16(packet.__bitfield_1) != (cast(u16) (word & 65535) | 512) return false;

        packet.__bitfield_1 = wire16(cast(u16) (word & 65535));
        if S.get_anon_struct_1_urg(packet) != ((word & 8192) != 0) return false;

        S.set_anon_struct_1_urg(*packet, false);
        if wire16(packet.__bitfield_1) != (cast(u16) (word & 65535) & 57343) return false;

        S.set_anon_struct_1_urg(*packet, true);
        if wire16(packet.__bitfield_1) != (cast(u16) (word & 65535) | 8192) return false;

        packet.__bitfield = cast(u8) (word & 255);
        if S.get_anon_struct_th_off(packet) != (cast(u8) ((word >> 4) & 15)) return false;

        for input: 0..255 {
            packet.__bitfield = cast(u8) (word & 255);
            value: u8 = cast(u8) input;

            S.set_anon_struct_th_off(*packet, value);
            expected: u8 = (cast(u8) (word & 255) & 15) | ((value & 15) << 4);
            if packet.__bitfield != expected || S.get_anon_struct_th_off(packet) != (value & 15) return false;
        }

        packet.__bitfield = cast(u8) (word & 255);
        if S.get_anon_struct_th_x2(packet) != (cast(u8) ((word >> 0) & 15)) return false;

        for input: 0..255 {
            packet.__bitfield = cast(u8) (word & 255);
            value: u8 = cast(u8) input;

            S.set_anon_struct_th_x2(*packet, value);
            expected: u8 = (cast(u8) (word & 255) & 240) | ((value & 15) << 0);
            if packet.__bitfield != expected || S.get_anon_struct_th_x2(packet) != (value & 15) return false;
        }

        info.__bitfield = nibbles(cast(u8) (word & 255));
        if S.get_tcpi_rcv_wscale(info) != (cast(u8) ((word >> 4) & 15)) return false;

        for input: 0..255 {
            info.__bitfield = nibbles(cast(u8) (word & 255));
            value: u8 = cast(u8) input;

            S.set_tcpi_rcv_wscale(*info, value);
            expected: u8 = (cast(u8) (word & 255) & 15) | ((value & 15) << 4);
            if nibbles(info.__bitfield) != expected || S.get_tcpi_rcv_wscale(info) != (value & 15) return false;
        }

        info.__bitfield = nibbles(cast(u8) (word & 255));
        if S.get_tcpi_snd_wscale(info) != (cast(u8) ((word >> 0) & 15)) return false;

        for input: 0..255 {
            info.__bitfield = nibbles(cast(u8) (word & 255));
            value: u8 = cast(u8) input;

            S.set_tcpi_snd_wscale(*info, value);
            expected: u8 = (cast(u8) (word & 255) & 240) | ((value & 15) << 0);
            if nibbles(info.__bitfield) != expected || S.get_tcpi_snd_wscale(info) != (value & 15) return false;
        }
    }

    return true;
}

main :: () {
    assert(verify());
}
