NET_Address :: struct {
    port: s32;
}

NET_port_of :: (address: *NET_Address) -> s32 {
    return address.port;
}
