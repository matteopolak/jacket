// Supplemental native shape verified against Apple's public SDK netinet/in.h.
#if OS == .MACOS {
    in_addr :: struct {
        s_addr: in_addr_t;
    }
}

// Reviewed Android public records; native field ABI agrees with bionic headers.
#if OS == .ANDROID {
    SHUT :: enum u32 {
        RD :: 0;
        WR :: 1;
        RDWR :: 2;
        SHUT_RD :: RD;
        SHUT_WR :: WR;
        SHUT_RDWR :: RDWR;
    }

    sockaddr :: struct {
        sa_family: sa_family_t;
        sa_data: [14] u8;
    }

    linger :: struct {
        l_onoff: s32;
        l_linger: s32;
    }

    in_addr :: struct {
        s_addr: in_addr_t;
    }

    ip_mreqn :: struct {
        imr_multiaddr: in_addr;
        imr_address: in_addr;
        imr_ifindex: s32;
    }

    in_pktinfo :: struct {
        ipi_ifindex: s32;
        ipi_spec_dst: in_addr;
        ipi_addr: in_addr;
    }
}
