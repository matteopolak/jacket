// Accessors for the little-endian bitfields of glibc and bionic TCP structures.
#assert OS == .LINUX || OS == .ANDROID;
#assert CPU == .X64 || CPU == .ARM64;

// Reads the ACK flag of a TCP header from `fields`.
get_anon_struct_1_ack :: (fields: tcphdr) -> bool {
    return (fields.__bitfield_1 & 4096) != 0;
}

// Sets the ACK flag of a TCP header in `fields`, keeping the other bits.
set_anon_struct_1_ack :: (fields: *tcphdr, value: bool) {
    if value fields.__bitfield_1 |= 4096;
    else fields.__bitfield_1 &= ~cast(u16) 4096;
}

// Reads the header length of a TCP header, in 32-bit words (4 bits) from `fields`.
get_anon_struct_1_doff :: (fields: tcphdr) -> u16 {
    return (fields.__bitfield_1 >> 4) & 15;
}

// Sets the header length of a TCP header, in 32-bit words (4 bits) in `fields`, keeping the other bits.
set_anon_struct_1_doff :: (fields: *tcphdr, value: u16) {
    fields.__bitfield_1 = (fields.__bitfield_1 & ~cast(u16) 240) | ((value & 15) << 4);
}

// Reads the FIN flag of a TCP header from `fields`.
get_anon_struct_1_fin :: (fields: tcphdr) -> bool {
    return (fields.__bitfield_1 & 256) != 0;
}

// Sets the FIN flag of a TCP header in `fields`, keeping the other bits.
set_anon_struct_1_fin :: (fields: *tcphdr, value: bool) {
    if value fields.__bitfield_1 |= 256;
    else fields.__bitfield_1 &= ~cast(u16) 256;
}

// Reads the PSH flag of a TCP header from `fields`.
get_anon_struct_1_psh :: (fields: tcphdr) -> bool {
    return (fields.__bitfield_1 & 2048) != 0;
}

// Sets the PSH flag of a TCP header in `fields`, keeping the other bits.
set_anon_struct_1_psh :: (fields: *tcphdr, value: bool) {
    if value fields.__bitfield_1 |= 2048;
    else fields.__bitfield_1 &= ~cast(u16) 2048;
}

// Reads the first group of reserved bits of a TCP header, which are zero in valid segments from `fields`.
get_anon_struct_1_res1 :: (fields: tcphdr) -> u16 {
    return (fields.__bitfield_1 >> 0) & 15;
}

// Sets the first group of reserved bits of a TCP header, which are zero in valid segments in `fields`, keeping the other bits.
set_anon_struct_1_res1 :: (fields: *tcphdr, value: u16) {
    fields.__bitfield_1 = (fields.__bitfield_1 & ~cast(u16) 15) | ((value & 15) << 0);
}

// Reads the second group of reserved bits of a TCP header, which are zero in valid segments from `fields`.
get_anon_struct_1_res2 :: (fields: tcphdr) -> u16 {
    return (fields.__bitfield_1 >> 14) & 3;
}

// Sets the second group of reserved bits of a TCP header, which are zero in valid segments in `fields`, keeping the other bits.
set_anon_struct_1_res2 :: (fields: *tcphdr, value: u16) {
    fields.__bitfield_1 = (fields.__bitfield_1 & ~cast(u16) 49152) | ((value & 3) << 14);
}

// Reads the RST flag of a TCP header from `fields`.
get_anon_struct_1_rst :: (fields: tcphdr) -> bool {
    return (fields.__bitfield_1 & 1024) != 0;
}

// Sets the RST flag of a TCP header in `fields`, keeping the other bits.
set_anon_struct_1_rst :: (fields: *tcphdr, value: bool) {
    if value fields.__bitfield_1 |= 1024;
    else fields.__bitfield_1 &= ~cast(u16) 1024;
}

// Reads the SYN flag of a TCP header from `fields`.
get_anon_struct_1_syn :: (fields: tcphdr) -> bool {
    return (fields.__bitfield_1 & 512) != 0;
}

// Sets the SYN flag of a TCP header in `fields`, keeping the other bits.
set_anon_struct_1_syn :: (fields: *tcphdr, value: bool) {
    if value fields.__bitfield_1 |= 512;
    else fields.__bitfield_1 &= ~cast(u16) 512;
}

// Reads the URG flag of a TCP header from `fields`.
get_anon_struct_1_urg :: (fields: tcphdr) -> bool {
    return (fields.__bitfield_1 & 8192) != 0;
}

// Sets the URG flag of a TCP header in `fields`, keeping the other bits.
set_anon_struct_1_urg :: (fields: *tcphdr, value: bool) {
    if value fields.__bitfield_1 |= 8192;
    else fields.__bitfield_1 &= ~cast(u16) 8192;
}

// Reads the header length of a TCP header, in 32-bit words (4 bits) from `fields`.
get_anon_struct_th_off :: (fields: tcphdr) -> u8 {
    return (fields.__bitfield >> 4) & 15;
}

// Sets the header length of a TCP header, in 32-bit words (4 bits) in `fields`, keeping the other bits.
set_anon_struct_th_off :: (fields: *tcphdr, value: u8) {
    fields.__bitfield = (fields.__bitfield & ~cast(u8) 240) | ((value & 15) << 4);
}

// Reads the 4 reserved bits of a TCP header, which are zero in valid segments from `fields`.
get_anon_struct_th_x2 :: (fields: tcphdr) -> u8 {
    return (fields.__bitfield >> 0) & 15;
}

// Sets the 4 reserved bits of a TCP header, which are zero in valid segments in `fields`, keeping the other bits.
set_anon_struct_th_x2 :: (fields: *tcphdr, value: u8) {
    fields.__bitfield = (fields.__bitfield & ~cast(u8) 15) | ((value & 15) << 0);
}

#if OS == .LINUX {
    // Reads the window scale shift the connection uses for what it receives (4 bits) from `fields`.
    get_tcpi_rcv_wscale :: (fields: tcp_info) -> u8 {
        return (fields.__bitfield >> 4) & 15;
    }

    // Sets the window scale shift the connection uses for what it receives in `fields`, keeping the other bits.
    set_tcpi_rcv_wscale :: (fields: *tcp_info, value: u8) {
        fields.__bitfield = (fields.__bitfield & ~cast(u8) 240) | ((value & 15) << 4);
    }
}

#if OS == .LINUX {
    // Reads the window scale shift the connection uses for what it sends (4 bits) from `fields`.
    get_tcpi_snd_wscale :: (fields: tcp_info) -> u8 {
        return (fields.__bitfield >> 0) & 15;
    }

    // Sets the window scale shift the connection uses for what it sends in `fields`, keeping the other bits.
    set_tcpi_snd_wscale :: (fields: *tcp_info, value: u8) {
        fields.__bitfield = (fields.__bitfield & ~cast(u8) 15) | ((value & 15) << 0);
    }
}
