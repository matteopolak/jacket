// Official Apple tcp.h ABI; reviewed public accessor signatures.
#assert OS == .MACOS;

get_th_off :: (fields: ABI.tcphdr) -> u32 {
    return cast(u32) ((fields.__bitfield >> 4) & 15);
}

set_th_off :: (fields: *ABI.tcphdr, value: u32) {
    fields.__bitfield = (fields.__bitfield & 15) | cast(u8) ((value & 15) << 4);
}

get_th_x2 :: (fields: ABI.tcphdr) -> u32 {
    return cast(u32) ((fields.__bitfield >> 0) & 15);
}

set_th_x2 :: (fields: *ABI.tcphdr, value: u32) {
    fields.__bitfield = (fields.__bitfield & 240) | cast(u8) ((value & 15) << 0);
}

get_tcpi_tfo_cookie_invalid :: (fields: ABI.tcp_connection_info) -> bool {
    return (fields.__bitfield & 256) != 0;
}

set_tcpi_tfo_cookie_invalid :: (fields: *ABI.tcp_connection_info, value: bool) {
    if value fields.__bitfield |= 256;
    else fields.__bitfield &= ~cast(u32) 256;
}

get_tcpi_tfo_cookie_rcv :: (fields: ABI.tcp_connection_info) -> bool {
    return (fields.__bitfield & 2) != 0;
}

set_tcpi_tfo_cookie_rcv :: (fields: *ABI.tcp_connection_info, value: bool) {
    if value fields.__bitfield |= 2;
    else fields.__bitfield &= ~cast(u32) 2;
}

get_tcpi_tfo_cookie_req :: (fields: ABI.tcp_connection_info) -> bool {
    return (fields.__bitfield & 1) != 0;
}

set_tcpi_tfo_cookie_req :: (fields: *ABI.tcp_connection_info, value: bool) {
    if value fields.__bitfield |= 1;
    else fields.__bitfield &= ~cast(u32) 1;
}

get_tcpi_tfo_cookie_req_rcv :: (fields: ABI.tcp_connection_info) -> bool {
    return (fields.__bitfield & 64) != 0;
}

set_tcpi_tfo_cookie_req_rcv :: (fields: *ABI.tcp_connection_info, value: bool) {
    if value fields.__bitfield |= 64;
    else fields.__bitfield &= ~cast(u32) 64;
}

get_tcpi_tfo_cookie_sent :: (fields: ABI.tcp_connection_info) -> bool {
    return (fields.__bitfield & 128) != 0;
}

set_tcpi_tfo_cookie_sent :: (fields: *ABI.tcp_connection_info, value: bool) {
    if value fields.__bitfield |= 128;
    else fields.__bitfield &= ~cast(u32) 128;
}

get_tcpi_tfo_cookie_wrong :: (fields: ABI.tcp_connection_info) -> bool {
    return (fields.__bitfield & 512) != 0;
}

set_tcpi_tfo_cookie_wrong :: (fields: *ABI.tcp_connection_info, value: bool) {
    if value fields.__bitfield |= 512;
    else fields.__bitfield &= ~cast(u32) 512;
}

get_tcpi_tfo_heuristics_disable :: (fields: ABI.tcp_connection_info) -> bool {
    return (fields.__bitfield & 2048) != 0;
}

set_tcpi_tfo_heuristics_disable :: (fields: *ABI.tcp_connection_info, value: bool) {
    if value fields.__bitfield |= 2048;
    else fields.__bitfield &= ~cast(u32) 2048;
}

get_tcpi_tfo_no_cookie_rcv :: (fields: ABI.tcp_connection_info) -> bool {
    return (fields.__bitfield & 1024) != 0;
}

set_tcpi_tfo_no_cookie_rcv :: (fields: *ABI.tcp_connection_info, value: bool) {
    if value fields.__bitfield |= 1024;
    else fields.__bitfield &= ~cast(u32) 1024;
}

get_tcpi_tfo_onebyte_proxy :: (fields: ABI.tcp_connection_info) -> bool {
    return (fields.__bitfield & 16384) != 0;
}

set_tcpi_tfo_onebyte_proxy :: (fields: *ABI.tcp_connection_info, value: bool) {
    if value fields.__bitfield |= 16384;
    else fields.__bitfield &= ~cast(u32) 16384;
}

get_tcpi_tfo_recv_blackhole :: (fields: ABI.tcp_connection_info) -> bool {
    return (fields.__bitfield & 8192) != 0;
}

set_tcpi_tfo_recv_blackhole :: (fields: *ABI.tcp_connection_info, value: bool) {
    if value fields.__bitfield |= 8192;
    else fields.__bitfield &= ~cast(u32) 8192;
}

get_tcpi_tfo_send_blackhole :: (fields: ABI.tcp_connection_info) -> bool {
    return (fields.__bitfield & 4096) != 0;
}

set_tcpi_tfo_send_blackhole :: (fields: *ABI.tcp_connection_info, value: bool) {
    if value fields.__bitfield |= 4096;
    else fields.__bitfield &= ~cast(u32) 4096;
}

get_tcpi_tfo_syn_data_acked :: (fields: ABI.tcp_connection_info) -> bool {
    return (fields.__bitfield & 16) != 0;
}

set_tcpi_tfo_syn_data_acked :: (fields: *ABI.tcp_connection_info, value: bool) {
    if value fields.__bitfield |= 16;
    else fields.__bitfield &= ~cast(u32) 16;
}

get_tcpi_tfo_syn_data_rcv :: (fields: ABI.tcp_connection_info) -> bool {
    return (fields.__bitfield & 32) != 0;
}

set_tcpi_tfo_syn_data_rcv :: (fields: *ABI.tcp_connection_info, value: bool) {
    if value fields.__bitfield |= 32;
    else fields.__bitfield &= ~cast(u32) 32;
}

get_tcpi_tfo_syn_data_sent :: (fields: ABI.tcp_connection_info) -> bool {
    return (fields.__bitfield & 8) != 0;
}

set_tcpi_tfo_syn_data_sent :: (fields: *ABI.tcp_connection_info, value: bool) {
    if value fields.__bitfield |= 8;
    else fields.__bitfield &= ~cast(u32) 8;
}

get_tcpi_tfo_syn_loss :: (fields: ABI.tcp_connection_info) -> bool {
    return (fields.__bitfield & 4) != 0;
}

set_tcpi_tfo_syn_loss :: (fields: *ABI.tcp_connection_info, value: bool) {
    if value fields.__bitfield |= 4;
    else fields.__bitfield &= ~cast(u32) 4;
}

get___pad2 :: (fields: ABI.tcp_connection_info) -> u32 {
    return fields.__bitfield >> 15;
}

set___pad2 :: (fields: *ABI.tcp_connection_info, value: u32) {
    fields.__bitfield = (fields.__bitfield & 0x7fff) | ((value & 0x1ffff) << 15);
}

#scope_file
ABI :: #import "internal/net/socket-abi";
