#if OS == .MACOS {
    // An IPv4 address: `s_addr` holds the four octets in network byte order, the first octet at the lowest address.
    in_addr :: struct {
        s_addr: in_addr_t;
    }
}

// Android records, laid out as in the bionic headers.
#if OS == .ANDROID {
    // The directions `shutdown` can close: `RD` stops receiving, `WR` stops sending (the peer then reads end of stream) and
    // `RDWR` closes both. The `SHUT_`-prefixed names are the same values.
    SHUT :: enum u32 {
        RD :: 0;
        WR :: 1;
        RDWR :: 2;
        SHUT_RD :: RD;
        SHUT_WR :: WR;
        SHUT_RDWR :: RDWR;
    }

    // The generic socket address header: a family and 14 bytes of family-specific data. Cast a `sockaddr_in` or `sockaddr_in6`
    // to `*sockaddr` for calls that take any address.
    sockaddr :: struct {
        sa_family: sa_family_t;
        sa_data: [14] u8;
    }

    // The `SO_LINGER` option value: `l_onoff` non-zero makes `close` wait up to `l_linger` seconds for queued data to be sent.
    linger :: struct {
        l_onoff: s32;
        l_linger: s32;
    }

    // An IPv4 address: `s_addr` holds the four octets in network byte order, the first octet at the lowest address.
    in_addr :: struct {
        s_addr: in_addr_t;
    }

    // The `IP_ADD_MEMBERSHIP` option value that names the interface by index: the multicast group, the local address and the
    // interface number.
    ip_mreqn :: struct {
        imr_multiaddr: in_addr;
        imr_address: in_addr;
        imr_ifindex: s32;
    }

    // The ancillary data `IP_PKTINFO` reports for a received datagram: the interface it arrived on, the local address it was
    // sent to and the header's destination address.
    in_pktinfo :: struct {
        ipi_ifindex: s32;
        ipi_spec_dst: in_addr;
        ipi_addr: in_addr;
    }
}
