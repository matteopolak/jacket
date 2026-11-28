// Accessors for the bitfields of glibc and bionic TCP structures.
//
// A TCP header has the same bytes on every CPU, so its one-byte fields read alike. A big-endian CPU
// reads the 16-bit flags word with its bytes swapped, and numbers the bits of `tcp_info`'s window
// scale byte from the other end.
#assert OS == .LINUX || OS == .ANDROID;
#assert CPU == .X64 || CPU == .ARM64 || ARCHITECTURE == .X86 || ARCHITECTURE == .ARM || ARCHITECTURE == .S390X || ARCHITECTURE == .POWERPC64, "The bit order of this CPU is not known";

#scope_file

BIG_ENDIAN :: ARCHITECTURE == .S390X || ARCHITECTURE == .POWERPC64;

// The 16-bit flags word of a TCP header with its bytes in the order a little-endian CPU reads
// them, which the masks below assume. Swapping twice gives the word back.
header_word :: (word: u16) -> u16 {
    #if BIG_ENDIAN return (word << 8) | (word >> 8);
    else return word;
}

// The bit positions of the two 4-bit window scales in the byte they share: the one declared first is the
// low half on a little-endian CPU and the high half on a big-endian one.
#if BIG_ENDIAN {
    SEND_SHIFT :: 4;
    RECEIVE_SHIFT :: 0;
} else {
    SEND_SHIFT :: 0;
    RECEIVE_SHIFT :: 4;
}

#scope_export

// Reads the ACK flag of a TCP header from `fields`.
get_anon_struct_1_ack :: (fields: tcphdr) -> bool {
    return (header_word(fields.__bitfield_1) & 4096) != 0;
}

// Sets the ACK flag of a TCP header in `fields`, keeping the other bits.
set_anon_struct_1_ack :: (fields: *tcphdr, value: bool) {
    if value fields.__bitfield_1 = header_word(header_word(fields.__bitfield_1) | 4096);
    else fields.__bitfield_1 = header_word(header_word(fields.__bitfield_1) & ~cast(u16) 4096);
}

// Reads the header length of a TCP header, in 32-bit words (4 bits) from `fields`.
get_anon_struct_1_doff :: (fields: tcphdr) -> u16 {
    return (header_word(fields.__bitfield_1) >> 4) & 15;
}

// Sets the header length of a TCP header, in 32-bit words (4 bits) in `fields`, keeping the other bits.
set_anon_struct_1_doff :: (fields: *tcphdr, value: u16) {
    fields.__bitfield_1 = header_word((header_word(fields.__bitfield_1) & ~cast(u16) 240) | ((value & 15) << 4));
}

// Reads the FIN flag of a TCP header from `fields`.
get_anon_struct_1_fin :: (fields: tcphdr) -> bool {
    return (header_word(fields.__bitfield_1) & 256) != 0;
}

// Sets the FIN flag of a TCP header in `fields`, keeping the other bits.
set_anon_struct_1_fin :: (fields: *tcphdr, value: bool) {
    if value fields.__bitfield_1 = header_word(header_word(fields.__bitfield_1) | 256);
    else fields.__bitfield_1 = header_word(header_word(fields.__bitfield_1) & ~cast(u16) 256);
}

// Reads the PSH flag of a TCP header from `fields`.
get_anon_struct_1_psh :: (fields: tcphdr) -> bool {
    return (header_word(fields.__bitfield_1) & 2048) != 0;
}

// Sets the PSH flag of a TCP header in `fields`, keeping the other bits.
set_anon_struct_1_psh :: (fields: *tcphdr, value: bool) {
    if value fields.__bitfield_1 = header_word(header_word(fields.__bitfield_1) | 2048);
    else fields.__bitfield_1 = header_word(header_word(fields.__bitfield_1) & ~cast(u16) 2048);
}

// Reads the first group of reserved bits of a TCP header, which are zero in valid segments from `fields`.
get_anon_struct_1_res1 :: (fields: tcphdr) -> u16 {
    return (header_word(fields.__bitfield_1) >> 0) & 15;
}

// Sets the first group of reserved bits of a TCP header, which are zero in valid segments in `fields`, keeping the other bits.
set_anon_struct_1_res1 :: (fields: *tcphdr, value: u16) {
    fields.__bitfield_1 = header_word((header_word(fields.__bitfield_1) & ~cast(u16) 15) | ((value & 15) << 0));
}

// Reads the second group of reserved bits of a TCP header, which are zero in valid segments from `fields`.
get_anon_struct_1_res2 :: (fields: tcphdr) -> u16 {
    return (header_word(fields.__bitfield_1) >> 14) & 3;
}

// Sets the second group of reserved bits of a TCP header, which are zero in valid segments in `fields`, keeping the other bits.
set_anon_struct_1_res2 :: (fields: *tcphdr, value: u16) {
    fields.__bitfield_1 = header_word((header_word(fields.__bitfield_1) & ~cast(u16) 49152) | ((value & 3) << 14));
}

// Reads the RST flag of a TCP header from `fields`.
get_anon_struct_1_rst :: (fields: tcphdr) -> bool {
    return (header_word(fields.__bitfield_1) & 1024) != 0;
}

// Sets the RST flag of a TCP header in `fields`, keeping the other bits.
set_anon_struct_1_rst :: (fields: *tcphdr, value: bool) {
    if value fields.__bitfield_1 = header_word(header_word(fields.__bitfield_1) | 1024);
    else fields.__bitfield_1 = header_word(header_word(fields.__bitfield_1) & ~cast(u16) 1024);
}

// Reads the SYN flag of a TCP header from `fields`.
get_anon_struct_1_syn :: (fields: tcphdr) -> bool {
    return (header_word(fields.__bitfield_1) & 512) != 0;
}

// Sets the SYN flag of a TCP header in `fields`, keeping the other bits.
set_anon_struct_1_syn :: (fields: *tcphdr, value: bool) {
    if value fields.__bitfield_1 = header_word(header_word(fields.__bitfield_1) | 512);
    else fields.__bitfield_1 = header_word(header_word(fields.__bitfield_1) & ~cast(u16) 512);
}

// Reads the URG flag of a TCP header from `fields`.
get_anon_struct_1_urg :: (fields: tcphdr) -> bool {
    return (header_word(fields.__bitfield_1) & 8192) != 0;
}

// Sets the URG flag of a TCP header in `fields`, keeping the other bits.
set_anon_struct_1_urg :: (fields: *tcphdr, value: bool) {
    if value fields.__bitfield_1 = header_word(header_word(fields.__bitfield_1) | 8192);
    else fields.__bitfield_1 = header_word(header_word(fields.__bitfield_1) & ~cast(u16) 8192);
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
        return (fields.__bitfield >> RECEIVE_SHIFT) & 15;
    }

    // Sets the window scale shift the connection uses for what it receives in `fields`, keeping the other bits.
    set_tcpi_rcv_wscale :: (fields: *tcp_info, value: u8) {
        fields.__bitfield = (fields.__bitfield & ~cast(u8) (15 << RECEIVE_SHIFT)) | ((value & 15) << RECEIVE_SHIFT);
    }
}

#if OS == .LINUX {
    // Reads the window scale shift the connection uses for what it sends (4 bits) from `fields`.
    get_tcpi_snd_wscale :: (fields: tcp_info) -> u8 {
        return (fields.__bitfield >> SEND_SHIFT) & 15;
    }

    // Sets the window scale shift the connection uses for what it sends in `fields`, keeping the other bits.
    set_tcpi_snd_wscale :: (fields: *tcp_info, value: u8) {
        fields.__bitfield = (fields.__bitfield & ~cast(u8) (15 << SEND_SHIFT)) | ((value & 15) << SEND_SHIFT);
    }
}
