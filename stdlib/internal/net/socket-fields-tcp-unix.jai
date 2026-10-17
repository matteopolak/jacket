// Accessors for the little-endian bitfields of glibc and bionic TCP structures.
#assert OS == .LINUX || OS == .ANDROID;
#assert CPU == .X64 || CPU == .ARM64;

get_anon_struct_1_ack :: (fields: tcphdr) -> bool {
    return (fields.__bitfield_1 & 4096) != 0;
}

set_anon_struct_1_ack :: (fields: *tcphdr, value: bool) {
    if value fields.__bitfield_1 |= 4096;
    else fields.__bitfield_1 &= ~cast(u16) 4096;
}

get_anon_struct_1_doff :: (fields: tcphdr) -> u16 {
    return (fields.__bitfield_1 >> 4) & 15;
}

set_anon_struct_1_doff :: (fields: *tcphdr, value: u16) {
    fields.__bitfield_1 = (fields.__bitfield_1 & ~cast(u16) 240) | ((value & 15) << 4);
}

get_anon_struct_1_fin :: (fields: tcphdr) -> bool {
    return (fields.__bitfield_1 & 256) != 0;
}

set_anon_struct_1_fin :: (fields: *tcphdr, value: bool) {
    if value fields.__bitfield_1 |= 256;
    else fields.__bitfield_1 &= ~cast(u16) 256;
}

get_anon_struct_1_psh :: (fields: tcphdr) -> bool {
    return (fields.__bitfield_1 & 2048) != 0;
}

set_anon_struct_1_psh :: (fields: *tcphdr, value: bool) {
    if value fields.__bitfield_1 |= 2048;
    else fields.__bitfield_1 &= ~cast(u16) 2048;
}

get_anon_struct_1_res1 :: (fields: tcphdr) -> u16 {
    return (fields.__bitfield_1 >> 0) & 15;
}

set_anon_struct_1_res1 :: (fields: *tcphdr, value: u16) {
    fields.__bitfield_1 = (fields.__bitfield_1 & ~cast(u16) 15) | ((value & 15) << 0);
}

get_anon_struct_1_res2 :: (fields: tcphdr) -> u16 {
    return (fields.__bitfield_1 >> 14) & 3;
}

set_anon_struct_1_res2 :: (fields: *tcphdr, value: u16) {
    fields.__bitfield_1 = (fields.__bitfield_1 & ~cast(u16) 49152) | ((value & 3) << 14);
}

get_anon_struct_1_rst :: (fields: tcphdr) -> bool {
    return (fields.__bitfield_1 & 1024) != 0;
}

set_anon_struct_1_rst :: (fields: *tcphdr, value: bool) {
    if value fields.__bitfield_1 |= 1024;
    else fields.__bitfield_1 &= ~cast(u16) 1024;
}

get_anon_struct_1_syn :: (fields: tcphdr) -> bool {
    return (fields.__bitfield_1 & 512) != 0;
}

set_anon_struct_1_syn :: (fields: *tcphdr, value: bool) {
    if value fields.__bitfield_1 |= 512;
    else fields.__bitfield_1 &= ~cast(u16) 512;
}

get_anon_struct_1_urg :: (fields: tcphdr) -> bool {
    return (fields.__bitfield_1 & 8192) != 0;
}

set_anon_struct_1_urg :: (fields: *tcphdr, value: bool) {
    if value fields.__bitfield_1 |= 8192;
    else fields.__bitfield_1 &= ~cast(u16) 8192;
}

get_anon_struct_th_off :: (fields: tcphdr) -> u8 {
    return (fields.__bitfield >> 4) & 15;
}

set_anon_struct_th_off :: (fields: *tcphdr, value: u8) {
    fields.__bitfield = (fields.__bitfield & ~cast(u8) 240) | ((value & 15) << 4);
}

get_anon_struct_th_x2 :: (fields: tcphdr) -> u8 {
    return (fields.__bitfield >> 0) & 15;
}

set_anon_struct_th_x2 :: (fields: *tcphdr, value: u8) {
    fields.__bitfield = (fields.__bitfield & ~cast(u8) 15) | ((value & 15) << 0);
}

#if OS == .LINUX {
    get_tcpi_rcv_wscale :: (fields: tcp_info) -> u8 {
        return (fields.__bitfield >> 4) & 15;
    }

    set_tcpi_rcv_wscale :: (fields: *tcp_info, value: u8) {
        fields.__bitfield = (fields.__bitfield & ~cast(u8) 240) | ((value & 15) << 4);
    }
}

#if OS == .LINUX {
    get_tcpi_snd_wscale :: (fields: tcp_info) -> u8 {
        return (fields.__bitfield >> 0) & 15;
    }

    set_tcpi_snd_wscale :: (fields: *tcp_info, value: u8) {
        fields.__bitfield = (fields.__bitfield & ~cast(u8) 15) | ((value & 15) << 0);
    }
}
