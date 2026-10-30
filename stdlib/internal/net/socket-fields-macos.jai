// Accessors for the bitfields of Apple tcp.h structures.
#assert OS == .MACOS;

// Reads the header length of a TCP header, in 32-bit words (4 bits) from `fields`.
get_th_off :: (fields: tcphdr) -> u32 {
    return cast(u32) ((fields.__bitfield >> 4) & 15);
}

// Sets the header length of a TCP header, in 32-bit words (4 bits) in `fields`, keeping the other bits.
set_th_off :: (fields: *tcphdr, value: u32) {
    fields.__bitfield = (fields.__bitfield & 15) | cast(u8) ((value & 15) << 4);
}

// Reads the 4 reserved bits of a TCP header, which are zero in valid segments from `fields`.
get_th_x2 :: (fields: tcphdr) -> u32 {
    return cast(u32) ((fields.__bitfield >> 0) & 15);
}

// Sets the 4 reserved bits of a TCP header, which are zero in valid segments in `fields`, keeping the other bits.
set_th_x2 :: (fields: *tcphdr, value: u32) {
    fields.__bitfield = (fields.__bitfield & 240) | cast(u8) ((value & 15) << 0);
}

// Reads the TCP Fast Open `cookie_invalid` flag of a connection's info from `fields`.
get_tcpi_tfo_cookie_invalid :: (fields: tcp_connection_info) -> bool {
    return (fields.__bitfield & 256) != 0;
}

// Sets the TCP Fast Open `cookie_invalid` flag of a connection's info in `fields`, keeping the other bits.
set_tcpi_tfo_cookie_invalid :: (fields: *tcp_connection_info, value: bool) {
    if value fields.__bitfield |= 256;
    else fields.__bitfield &= ~cast(u32) 256;
}

// Reads the TCP Fast Open `cookie_rcv` flag of a connection's info from `fields`.
get_tcpi_tfo_cookie_rcv :: (fields: tcp_connection_info) -> bool {
    return (fields.__bitfield & 2) != 0;
}

// Sets the TCP Fast Open `cookie_rcv` flag of a connection's info in `fields`, keeping the other bits.
set_tcpi_tfo_cookie_rcv :: (fields: *tcp_connection_info, value: bool) {
    if value fields.__bitfield |= 2;
    else fields.__bitfield &= ~cast(u32) 2;
}

// Reads the TCP Fast Open `cookie_req` flag of a connection's info from `fields`.
get_tcpi_tfo_cookie_req :: (fields: tcp_connection_info) -> bool {
    return (fields.__bitfield & 1) != 0;
}

// Sets the TCP Fast Open `cookie_req` flag of a connection's info in `fields`, keeping the other bits.
set_tcpi_tfo_cookie_req :: (fields: *tcp_connection_info, value: bool) {
    if value fields.__bitfield |= 1;
    else fields.__bitfield &= ~cast(u32) 1;
}

// Reads the TCP Fast Open `cookie_req_rcv` flag of a connection's info from `fields`.
get_tcpi_tfo_cookie_req_rcv :: (fields: tcp_connection_info) -> bool {
    return (fields.__bitfield & 64) != 0;
}

// Sets the TCP Fast Open `cookie_req_rcv` flag of a connection's info in `fields`, keeping the other bits.
set_tcpi_tfo_cookie_req_rcv :: (fields: *tcp_connection_info, value: bool) {
    if value fields.__bitfield |= 64;
    else fields.__bitfield &= ~cast(u32) 64;
}

// Reads the TCP Fast Open `cookie_sent` flag of a connection's info from `fields`.
get_tcpi_tfo_cookie_sent :: (fields: tcp_connection_info) -> bool {
    return (fields.__bitfield & 128) != 0;
}

// Sets the TCP Fast Open `cookie_sent` flag of a connection's info in `fields`, keeping the other bits.
set_tcpi_tfo_cookie_sent :: (fields: *tcp_connection_info, value: bool) {
    if value fields.__bitfield |= 128;
    else fields.__bitfield &= ~cast(u32) 128;
}

// Reads the TCP Fast Open `cookie_wrong` flag of a connection's info from `fields`.
get_tcpi_tfo_cookie_wrong :: (fields: tcp_connection_info) -> bool {
    return (fields.__bitfield & 512) != 0;
}

// Sets the TCP Fast Open `cookie_wrong` flag of a connection's info in `fields`, keeping the other bits.
set_tcpi_tfo_cookie_wrong :: (fields: *tcp_connection_info, value: bool) {
    if value fields.__bitfield |= 512;
    else fields.__bitfield &= ~cast(u32) 512;
}

// Reads the TCP Fast Open `heuristics_disable` flag of a connection's info from `fields`.
get_tcpi_tfo_heuristics_disable :: (fields: tcp_connection_info) -> bool {
    return (fields.__bitfield & 2048) != 0;
}

// Sets the TCP Fast Open `heuristics_disable` flag of a connection's info in `fields`, keeping the other bits.
set_tcpi_tfo_heuristics_disable :: (fields: *tcp_connection_info, value: bool) {
    if value fields.__bitfield |= 2048;
    else fields.__bitfield &= ~cast(u32) 2048;
}

// Reads the TCP Fast Open `no_cookie_rcv` flag of a connection's info from `fields`.
get_tcpi_tfo_no_cookie_rcv :: (fields: tcp_connection_info) -> bool {
    return (fields.__bitfield & 1024) != 0;
}

// Sets the TCP Fast Open `no_cookie_rcv` flag of a connection's info in `fields`, keeping the other bits.
set_tcpi_tfo_no_cookie_rcv :: (fields: *tcp_connection_info, value: bool) {
    if value fields.__bitfield |= 1024;
    else fields.__bitfield &= ~cast(u32) 1024;
}

// Reads the TCP Fast Open `onebyte_proxy` flag of a connection's info from `fields`.
get_tcpi_tfo_onebyte_proxy :: (fields: tcp_connection_info) -> bool {
    return (fields.__bitfield & 16384) != 0;
}

// Sets the TCP Fast Open `onebyte_proxy` flag of a connection's info in `fields`, keeping the other bits.
set_tcpi_tfo_onebyte_proxy :: (fields: *tcp_connection_info, value: bool) {
    if value fields.__bitfield |= 16384;
    else fields.__bitfield &= ~cast(u32) 16384;
}

// Reads the TCP Fast Open `recv_blackhole` flag of a connection's info from `fields`.
get_tcpi_tfo_recv_blackhole :: (fields: tcp_connection_info) -> bool {
    return (fields.__bitfield & 8192) != 0;
}

// Sets the TCP Fast Open `recv_blackhole` flag of a connection's info in `fields`, keeping the other bits.
set_tcpi_tfo_recv_blackhole :: (fields: *tcp_connection_info, value: bool) {
    if value fields.__bitfield |= 8192;
    else fields.__bitfield &= ~cast(u32) 8192;
}

// Reads the TCP Fast Open `send_blackhole` flag of a connection's info from `fields`.
get_tcpi_tfo_send_blackhole :: (fields: tcp_connection_info) -> bool {
    return (fields.__bitfield & 4096) != 0;
}

// Sets the TCP Fast Open `send_blackhole` flag of a connection's info in `fields`, keeping the other bits.
set_tcpi_tfo_send_blackhole :: (fields: *tcp_connection_info, value: bool) {
    if value fields.__bitfield |= 4096;
    else fields.__bitfield &= ~cast(u32) 4096;
}

// Reads the TCP Fast Open `syn_data_acked` flag of a connection's info from `fields`.
get_tcpi_tfo_syn_data_acked :: (fields: tcp_connection_info) -> bool {
    return (fields.__bitfield & 16) != 0;
}

// Sets the TCP Fast Open `syn_data_acked` flag of a connection's info in `fields`, keeping the other bits.
set_tcpi_tfo_syn_data_acked :: (fields: *tcp_connection_info, value: bool) {
    if value fields.__bitfield |= 16;
    else fields.__bitfield &= ~cast(u32) 16;
}

// Reads the TCP Fast Open `syn_data_rcv` flag of a connection's info from `fields`.
get_tcpi_tfo_syn_data_rcv :: (fields: tcp_connection_info) -> bool {
    return (fields.__bitfield & 32) != 0;
}

// Sets the TCP Fast Open `syn_data_rcv` flag of a connection's info in `fields`, keeping the other bits.
set_tcpi_tfo_syn_data_rcv :: (fields: *tcp_connection_info, value: bool) {
    if value fields.__bitfield |= 32;
    else fields.__bitfield &= ~cast(u32) 32;
}

// Reads the TCP Fast Open `syn_data_sent` flag of a connection's info from `fields`.
get_tcpi_tfo_syn_data_sent :: (fields: tcp_connection_info) -> bool {
    return (fields.__bitfield & 8) != 0;
}

// Sets the TCP Fast Open `syn_data_sent` flag of a connection's info in `fields`, keeping the other bits.
set_tcpi_tfo_syn_data_sent :: (fields: *tcp_connection_info, value: bool) {
    if value fields.__bitfield |= 8;
    else fields.__bitfield &= ~cast(u32) 8;
}

// Reads the TCP Fast Open `syn_loss` flag of a connection's info from `fields`.
get_tcpi_tfo_syn_loss :: (fields: tcp_connection_info) -> bool {
    return (fields.__bitfield & 4) != 0;
}

// Sets the TCP Fast Open `syn_loss` flag of a connection's info in `fields`, keeping the other bits.
set_tcpi_tfo_syn_loss :: (fields: *tcp_connection_info, value: bool) {
    if value fields.__bitfield |= 4;
    else fields.__bitfield &= ~cast(u32) 4;
}

// Reads the padding bits at the end of the connection info, which carry no information from `fields`.
get___pad2 :: (fields: tcp_connection_info) -> u32 {
    return fields.__bitfield >> 15;
}

// Sets the padding bits at the end of the connection info, which carry no information in `fields`, keeping the other bits.
set___pad2 :: (fields: *tcp_connection_info, value: u32) {
    fields.__bitfield = (fields.__bitfield & 0x7fff) | ((value & 0x1ffff) << 15);
}
