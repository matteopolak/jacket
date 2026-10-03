// Generated exclusively from reviewed public declarations.
#scope_file
#if OS_IS_UNIX #import "POSIX";

// Unix select uses the canonical native timeval through a private spelling.
// TIMEVAL remains a public Socket name only on Windows, as reviewed.
#if OS_IS_UNIX TIMEVAL :: timeval;

#if OS == .WINDOWS {
    Windows_Native :: #import "Windows";

    BOOL :: Windows_Native.BOOL;
    BOOLEAN :: Windows_Native.BOOLEAN;
    GUID :: Windows_Native.GUID;
    HANDLE :: Windows_Native.HANDLE;
    HWND :: Windows_Native.HWND;
    OS_Error_Code :: Windows_Native.OS_Error_Code;
    OVERLAPPED :: Windows_Native.OVERLAPPED;
    SIZE_T :: Windows_Native.SIZE_T;
    WPARAM :: Windows_Native.WPARAM;
    uid :: Windows_Native.uid;
}

libc :: #system_library "libc";
ws2_32 :: #system_library "ws2_32";

#scope_export
#load "socket-native-completions.jai";

#if OS == .MACOS {
    __msfilterreq :: struct {
        msfr_ifindex: u32;
        msfr_fmode: u32;
        msfr_nsrcs: u32;
        __msfr_align: u32;
        msfr_group: sockaddr_storage #align 4;
        msfr_srcs: *sockaddr_storage #align 4;
    }

}

#if OS == .WINDOWS {
    addrinfo :: struct {
        ai_flags: AI;
        ai_family: s32;
        ai_socktype: SOCK;
        ai_protocol: IPPROTO;
        ai_addrlen: u64;
        ai_canonname: *u8;
        ai_addr: *sockaddr;
        ai_next: *addrinfo;
    }

}

#if OS == .LINUX {
    addrinfo :: struct {
        ai_flags: AI;
        ai_family: s32;
        ai_socktype: SOCK;
        ai_protocol: IPPROTO;
        ai_addrlen: socklen_t;
        ai_addr: *sockaddr;
        ai_canonname: *u8;
        ai_next: *addrinfo;
    }

}

#if OS == .ANDROID {
    addrinfo :: struct {
        ai_flags: AI;
        ai_family: s32;
        ai_socktype: SOCK;
        ai_protocol: IPPROTO;
        ai_addrlen: socklen_t;
        ai_canonname: *u8;
        ai_addr: *sockaddr;
        ai_next: *addrinfo;
    }

}

#if OS == .MACOS {
    addrinfo :: struct {
        ai_flags: AI;
        ai_family: s32;
        ai_socktype: SOCK;
        ai_protocol: IPPROTO;
        ai_addrlen: socklen_t;
        ai_canonname: *u8;
        ai_addr: *sockaddr;
        ai_next: *addrinfo;
    }

}

#if OS == .WINDOWS {
    addrinfo_dns_server :: struct {
        ai_servertype: u32;
        ai_flags: AI #align 8;
        ai_addrlen: u32 #align 8;
        ai_addr: *sockaddr;

        struct {
            ai_template: *s16;

            #overlay (ai_template) ai_hostname: *s16;
        }
    }

}

#if OS == .WINDOWS {
    addrinfoex2A :: struct {
        ai_flags: AI;
        ai_family: s32;
        ai_socktype: SOCK;
        ai_protocol: IPPROTO;
        ai_addrlen: u64;
        ai_canonname: *u8;
        ai_addr: *sockaddr;
        ai_blob: *void;
        ai_bloblen: u64;
        ai_provider: *GUID;
        ai_next: *addrinfoex2A;
        ai_version: s32;
        ai_fqdn: *u8;
    }

}

#if OS == .WINDOWS {
    addrinfoex2W :: struct {
        ai_flags: AI;
        ai_family: s32;
        ai_socktype: SOCK;
        ai_protocol: IPPROTO;
        ai_addrlen: u64;
        ai_canonname: *s16;
        ai_addr: *sockaddr;
        ai_blob: *void;
        ai_bloblen: u64;
        ai_provider: *GUID;
        ai_next: *addrinfoex2W;
        ai_version: s32;
        ai_fqdn: *s16;
    }

}

#if OS == .WINDOWS {
    addrinfoex3 :: struct {
        ai_flags: AI;
        ai_family: s32;
        ai_socktype: SOCK;
        ai_protocol: IPPROTO;
        ai_addrlen: u64;
        ai_canonname: *s16;
        ai_addr: *sockaddr;
        ai_blob: *void;
        ai_bloblen: u64;
        ai_provider: *GUID;
        ai_next: *addrinfoex3;
        ai_version: s32;
        ai_fqdn: *s16;
        ai_interfaceindex: s32;
    }

}

#if OS == .WINDOWS {
    addrinfoex4 :: struct {
        ai_flags: AI;
        ai_family: s32;
        ai_socktype: SOCK;
        ai_protocol: IPPROTO;
        ai_addrlen: u64;
        ai_canonname: *s16;
        ai_addr: *sockaddr;
        ai_blob: *void;
        ai_bloblen: u64;
        ai_provider: *GUID;
        ai_next: *addrinfoex4;
        ai_version: s32;
        ai_fqdn: *s16;
        ai_interfaceindex: s32;
        ai_resolutionhandle: HANDLE;
    }

}

#if OS == .WINDOWS {
    addrinfoex5 :: struct {
        ai_flags: AI;
        ai_family: s32;
        ai_socktype: SOCK;
        ai_protocol: IPPROTO;
        ai_addrlen: u64;
        ai_canonname: *s16;
        ai_addr: *sockaddr;
        ai_blob: *void;
        ai_bloblen: u64;
        ai_provider: *GUID;
        ai_next: *addrinfoex5;
        ai_version: s32;
        ai_fqdn: *s16;
        ai_interfaceindex: s32;
        ai_resolutionhandle: HANDLE;
        ai_ttl: u32;
    }

}

#if OS == .WINDOWS {
    addrinfoex6 :: struct {
        ai_flags: AI;
        ai_family: s32;
        ai_socktype: SOCK;
        ai_protocol: IPPROTO;
        ai_addrlen: u64;
        ai_canonname: *s16;
        ai_addr: *sockaddr;
        ai_blob: *void;
        ai_bloblen: u64;
        ai_provider: *GUID;
        ai_next: *addrinfoex5;
        ai_version: s32;
        ai_fqdn: *s16;
        ai_interfaceindex: s32;
        ai_resolutionhandle: HANDLE;
        ai_ttl: u32;
        ai_numservers: u32;
        ai_servers: *ADDRINFO_DNS_SERVER;
        ai_responseflags: u64;
    }

}

#if OS == .WINDOWS {
    addrinfoex7 :: struct {
        ai_flags: AI;
        ai_family: s32;
        ai_socktype: SOCK;
        ai_protocol: IPPROTO;
        ai_addrlen: u64;
        ai_canonname: *s16;
        ai_addr: *sockaddr;
        ai_blob: *void;
        ai_bloblen: u64;
        ai_provider: *GUID;
        ai_next: *addrinfoex7;
        ai_version: s32;
        ai_fqdn: *s16;
        ai_interfaceindex: s32;
        ai_resolutionhandle: HANDLE;
        ai_ttl: u32;
        ai_numservers: u32;
        ai_servers: *ADDRINFO_DNS_SERVER;
        ai_responseflags: u64;
        ai_extraflags: u64;
    }

}

#if OS == .WINDOWS {
    addrinfoexA :: struct {
        ai_flags: AI;
        ai_family: s32;
        ai_socktype: SOCK;
        ai_protocol: IPPROTO;
        ai_addrlen: u64;
        ai_canonname: *u8;
        ai_addr: *sockaddr;
        ai_blob: *void;
        ai_bloblen: u64;
        ai_provider: *GUID;
        ai_next: *addrinfoexA;
    }

}

#if OS == .WINDOWS {
    addrinfoexW :: struct {
        ai_flags: AI;
        ai_family: s32;
        ai_socktype: SOCK;
        ai_protocol: IPPROTO;
        ai_addrlen: u64;
        ai_canonname: *s16;
        ai_addr: *sockaddr;
        ai_blob: *void;
        ai_bloblen: u64;
        ai_provider: *GUID;
        ai_next: *addrinfoexW;
    }

}

#if OS == .WINDOWS {
    addrinfoW :: struct {
        ai_flags: AI;
        ai_family: s32;
        ai_socktype: SOCK;
        ai_protocol: IPPROTO;
        ai_addrlen: u64;
        ai_canonname: *s16;
        ai_addr: *sockaddr;
        ai_next: *addrinfoW;
    }

}

#if OS == .WINDOWS {
    AFPROTOCOLS :: struct {
        iAddressFamily: s32;
        iProtocol: s32;
    }

}

#if OS == .WINDOWS {
    AI :: enum_flags s32 {
        PASSIVE :: 0x00000001;
        CANONNAME :: 0x00000002;
        NUMERICHOST :: 0x00000004;
        NUMERICSERV :: 0x00000008;
        DNS_ONLY :: 0x00000010;
        FORCE_CLEAR_TEXT :: 0x00000020;
        BYPASS_DNS_CACHE :: 0x00000040;
        RETURN_TTL :: 0x00000080;
        ALL :: 0x00000100;
        ADDRCONFIG :: 0x00000400;
        V4MAPPED :: 0x00000800;
        NON_AUTHORITATIVE :: 0x00004000;
        SECURE :: 0x00008000;
        RETURN_PREFERRED_NAMES :: 0x00010000;
        FQDN :: 0x00020000;
        FILESERVER :: 0x00040000;
        DISABLE_IDN_ENCODING :: 0x00080000;
        SECURE_WITH_FALLBACK :: 0x00100000;
        EXCLUSIVE_CUSTOM_SERVERS :: 0x00200000;
        RETURN_RESPONSE_FLAGS :: 0x10000000;
        REQUIRE_SECURE :: 0x20000000;
        RESOLUTION_HANDLE :: 0x40000000;
        EXTENDED :: 0x80000000;
        DNS_SERVER_TYPE_UDP :: 0x1;
        DNS_SERVER_TYPE_DOH :: 0x2;
        DNS_SERVER_TYPE_DOT :: 0x3;
        DNS_SERVER_UDP_FALLBACK :: 0x1;
        DNS_RESPONSE_SECURE :: 0x1;
        DNS_RESPONSE_HOSTFILE :: 0x2;
        EXTRA_DNSSEC_REQUIRED :: 0x0000000000000001;

        AI_PASSIVE :: PASSIVE;
        AI_CANONNAME :: CANONNAME;
        AI_NUMERICHOST :: NUMERICHOST;
        AI_NUMERICSERV :: NUMERICSERV;
        AI_DNS_ONLY :: DNS_ONLY;
        AI_FORCE_CLEAR_TEXT :: FORCE_CLEAR_TEXT;
        AI_BYPASS_DNS_CACHE :: BYPASS_DNS_CACHE;
        AI_RETURN_TTL :: RETURN_TTL;
        AI_ALL :: ALL;
        AI_ADDRCONFIG :: ADDRCONFIG;
        AI_V4MAPPED :: V4MAPPED;
        AI_NON_AUTHORITATIVE :: NON_AUTHORITATIVE;
        AI_SECURE :: SECURE;
        AI_RETURN_PREFERRED_NAMES :: RETURN_PREFERRED_NAMES;
        AI_FQDN :: FQDN;
        AI_FILESERVER :: FILESERVER;
        AI_DISABLE_IDN_ENCODING :: DISABLE_IDN_ENCODING;
        AI_SECURE_WITH_FALLBACK :: SECURE_WITH_FALLBACK;
        AI_EXCLUSIVE_CUSTOM_SERVERS :: EXCLUSIVE_CUSTOM_SERVERS;
        AI_RETURN_RESPONSE_FLAGS :: RETURN_RESPONSE_FLAGS;
        AI_REQUIRE_SECURE :: REQUIRE_SECURE;
        AI_RESOLUTION_HANDLE :: RESOLUTION_HANDLE;
        AI_EXTENDED :: EXTENDED;
        AI_DNS_SERVER_TYPE_UDP :: DNS_SERVER_TYPE_UDP;
        AI_DNS_SERVER_TYPE_DOH :: DNS_SERVER_TYPE_DOH;
        AI_DNS_SERVER_TYPE_DOT :: DNS_SERVER_TYPE_DOT;
        AI_DNS_SERVER_UDP_FALLBACK :: DNS_SERVER_UDP_FALLBACK;
        AI_DNS_RESPONSE_SECURE :: DNS_RESPONSE_SECURE;
        AI_DNS_RESPONSE_HOSTFILE :: DNS_RESPONSE_HOSTFILE;
        AI_EXTRA_DNSSEC_REQUIRED :: EXTRA_DNSSEC_REQUIRED;
    }

}

#if OS == .LINUX {
    AI :: enum_flags u32 {
        PASSIVE :: 0x0001;
        CANONNAME :: 0x0002;
        NUMERICHOST :: 0x0004;
        V4MAPPED :: 0x0008;
        ALL :: 0x0010;
        ADDRCONFIG :: 0x0020;
        IDN :: 0x0040;
        CANONIDN :: 0x0080;
        NUMERICSERV :: 0x0400;

        AI_PASSIVE :: PASSIVE;
        AI_CANONNAME :: CANONNAME;
        AI_NUMERICHOST :: NUMERICHOST;
        AI_V4MAPPED :: V4MAPPED;
        AI_ALL :: ALL;
        AI_ADDRCONFIG :: ADDRCONFIG;
        AI_IDN :: IDN;
        AI_CANONIDN :: CANONIDN;
        AI_NUMERICSERV :: NUMERICSERV;
    }

}

#if OS == .ANDROID {
    AI :: enum_flags u32 {
        PASSIVE :: 0x00000001;
        CANONNAME :: 0x00000002;
        NUMERICHOST :: 0x00000004;
        NUMERICSERV :: 0x00000008;
        MASK :: PASSIVE | CANONNAME | NUMERICHOST | NUMERICSERV | ADDRCONFIG;
        ALL :: 0x00000100;
        V4MAPPED_CFG :: 0x00000200;
        ADDRCONFIG :: 0x00000400;
        V4MAPPED :: 0x00000800;
        DEFAULT :: V4MAPPED_CFG | ADDRCONFIG;

        AI_PASSIVE :: PASSIVE;
        AI_CANONNAME :: CANONNAME;
        AI_NUMERICHOST :: NUMERICHOST;
        AI_NUMERICSERV :: NUMERICSERV;
        AI_MASK :: MASK;
        AI_ALL :: ALL;
        AI_V4MAPPED_CFG :: V4MAPPED_CFG;
        AI_ADDRCONFIG :: ADDRCONFIG;
        AI_V4MAPPED :: V4MAPPED;
        AI_DEFAULT :: DEFAULT;
    }

}

#if OS == .MACOS {
    AI :: enum_flags u32 {
        PASSIVE :: 0x00000001;
        CANONNAME :: 0x00000002;
        NUMERICHOST :: 0x00000004;
        NUMERICSERV :: 0x00001000;
        MASK :: PASSIVE | CANONNAME | NUMERICHOST | NUMERICSERV | ADDRCONFIG;
        ALL :: 0x00000100;
        V4MAPPED_CFG :: 0x00000200;
        ADDRCONFIG :: 0x00000400;
        V4MAPPED :: 0x00000800;
        DEFAULT :: V4MAPPED_CFG | ADDRCONFIG;
        UNUSABLE :: 0x10000000;

        AI_PASSIVE :: PASSIVE;
        AI_CANONNAME :: CANONNAME;
        AI_NUMERICHOST :: NUMERICHOST;
        AI_NUMERICSERV :: NUMERICSERV;
        AI_MASK :: MASK;
        AI_ALL :: ALL;
        AI_V4MAPPED_CFG :: V4MAPPED_CFG;
        AI_ADDRCONFIG :: ADDRCONFIG;
        AI_V4MAPPED :: V4MAPPED;
        AI_DEFAULT :: DEFAULT;
        AI_UNUSABLE :: UNUSABLE;
    }

}

#if OS == .WINDOWS {
    ASSOCIATE_NAMERES_CONTEXT_INPUT :: struct {
        TransportSettingId: TRANSPORT_SETTING_ID;

        Handle: u64;
    }

}

#if OS == .WINDOWS {
    BLOB :: struct {
        cbSize: u32;

        pBlobData: *u8;
    }

}

#if OS == .WINDOWS {
    cmsghdr :: struct {
        cmsg_len: SIZE_T;
        cmsg_level: s32;
        cmsg_type: s32;
    }

}

#if OS == .LINUX {
    cmsghdr :: struct {
        cmsg_len: u64;

        cmsg_level: s32;
        cmsg_type: s32;

    }

}

#if OS == .ANDROID {
    cmsghdr :: struct {
        cmsg_len: u64;
        cmsg_level: s32;
        cmsg_type: s32;
    }

}

#if OS == .MACOS {
    cmsghdr :: struct {
        cmsg_len: socklen_t;
        cmsg_level: s32;
        cmsg_type: s32;
    }

}

#if OS == .WINDOWS {
    CONTROL_CHANNEL_TRIGGER_STATUS :: enum s32 {
        INVALID :: 0;
        SOFTWARE_SLOT_ALLOCATED :: 1;
        HARDWARE_SLOT_ALLOCATED :: 2;
        POLICY_ERROR :: 3;
        SYSTEM_ERROR :: 4;
        TRANSPORT_DISCONNECTED :: 5;
        SERVICE_UNAVAILABLE :: 6;

        CONTROL_CHANNEL_TRIGGER_STATUS_INVALID :: INVALID;
        CONTROL_CHANNEL_TRIGGER_STATUS_SOFTWARE_SLOT_ALLOCATED :: SOFTWARE_SLOT_ALLOCATED;
        CONTROL_CHANNEL_TRIGGER_STATUS_HARDWARE_SLOT_ALLOCATED :: HARDWARE_SLOT_ALLOCATED;
        CONTROL_CHANNEL_TRIGGER_STATUS_POLICY_ERROR :: POLICY_ERROR;
        CONTROL_CHANNEL_TRIGGER_STATUS_SYSTEM_ERROR :: SYSTEM_ERROR;
        CONTROL_CHANNEL_TRIGGER_STATUS_TRANSPORT_DISCONNECTED :: TRANSPORT_DISCONNECTED;
        CONTROL_CHANNEL_TRIGGER_STATUS_SERVICE_UNAVAILABLE :: SERVICE_UNAVAILABLE;
    }

}

#if OS == .WINDOWS {
    CSADDR_INFO :: struct {
        LocalAddr: SOCKET_ADDRESS;
        RemoteAddr: SOCKET_ADDRESS;
        iSocketType: s32;
        iProtocol: s32;
    }

}

#if OS == .WINDOWS {
    DL_EUI48 :: struct {}

}

#if OS == .WINDOWS {
    fd_set :: struct {
        fd_count: u32;
        fd_array: [64] SOCKET;
    }

}

#if OS == .LINUX {
    fd_set :: struct {
        fds_bits: [16] __fd_mask;
    }

}

#if OS == .ANDROID {
    fd_set :: struct {
        fds_bits: [16] fd_mask;
    }

}

#if OS == .MACOS {
    fd_set :: struct {
        fds_bits: [32] s32;
    }

}

#if OS == .WINDOWS {
    flowspec :: struct {
        TokenRate: u32;
        TokenBucketSize: u32;
        PeakBandwidth: u32;
        Latency: u32;
        DelayVariation: u32;
        ServiceType: SERVICETYPE;
        MaxSduSize: u32;
        MinimumPolicedSize: u32;
    }

}

#if OS == .LINUX {
    gaicb :: struct {
        ar_name: *u8;
        ar_service: *u8;
        ar_request: *addrinfo;
        ar_result: *addrinfo;

        __return: s32;
        __glibc_reserved: [5] s32;
    }

}

#if OS == .WINDOWS {
    group_filter :: struct {
        gf_interface: u32;
        gf_group: SOCKADDR_STORAGE;
        gf_fmode: MULTICAST_MODE_TYPE;
        gf_numsrc: u32;
        gf_slist: [1] SOCKADDR_STORAGE;
    }

}

#if OS == .LINUX {
    group_filter :: struct {

        gf_interface: u32;

        gf_group: sockaddr_storage;

        gf_fmode: u32;

        gf_numsrc: u32;

        gf_slist: [1] sockaddr_storage;
    }

}

#if OS == .ANDROID {
    group_filter :: struct {
        struct {
            using anon_struct: struct {
                gf_interface_aux: u32;
                gf_group_aux: sockaddr_storage;
                gf_fmode_aux: u32;
                gf_numsrc_aux: u32;
                gf_slist: [1] sockaddr_storage;
            };

            #overlay (anon_struct) using anon_struct_1: struct {
                gf_interface: u32;
                gf_group: sockaddr_storage;
                gf_fmode: u32;
                gf_numsrc: u32;

            };
        }
    }

}

#if OS == .WINDOWS {
    group_req :: struct {
        gr_interface: u32;
        gr_group: SOCKADDR_STORAGE;
    }

}

#if OS == .LINUX {
    group_req :: struct {

        gr_interface: u32;

        gr_group: sockaddr_storage;
    }

}

#if OS == .ANDROID {
    group_req :: struct {
        gr_interface: u32;
        gr_group: sockaddr_storage;
    }

}

#if OS == .MACOS {
    group_req :: struct {
        gr_interface: u32;
        gr_group: sockaddr_storage #align 4;
    }

}

#if OS == .WINDOWS {
    group_source_req :: struct {
        gsr_interface: u32;
        gsr_group: SOCKADDR_STORAGE;
        gsr_source: SOCKADDR_STORAGE;
    }

}

#if OS == .LINUX {
    group_source_req :: struct {

        gsr_interface: u32;

        gsr_group: sockaddr_storage;

        gsr_source: sockaddr_storage;
    }

}

#if OS == .ANDROID {
    group_source_req :: struct {
        gsr_interface: u32;
        gsr_group: sockaddr_storage;
        gsr_source: sockaddr_storage;
    }

}

#if OS == .MACOS {
    group_source_req :: struct {
        gsr_interface: u32;
        gsr_group: sockaddr_storage #align 4;
        gsr_source: sockaddr_storage #align 4;
    }

}

#if OS == .WINDOWS {
    hostent :: struct {
        h_name: *u8;
        h_aliases: **u8;
        h_addrtype: s16;
        h_length: s16;
        h_addr_list: **u8;
    }

}

#if OS == .LINUX {
    hostent :: struct {
        h_name: *u8;
        h_aliases: **u8;
        h_addrtype: s32;
        h_length: s32;
        h_addr_list: **u8;
    }

}

#if OS == .ANDROID {
    hostent :: struct {
        h_name: *u8;
        h_aliases: **u8;
        h_addrtype: s32;
        h_length: s32;
        h_addr_list: **u8;
    }

}

#if OS == .MACOS {
    hostent :: struct {
        h_name: *u8;
        h_aliases: **u8;
        h_addrtype: s32;
        h_length: s32;
        h_addr_list: **u8;
    }

}

#if OS == .WINDOWS {
    icmp_error_info :: struct {
        srcaddress: SOCKADDR_INET;
        protocol: IPPROTO;
        type: u8;
        code: u8;
    }

}

#if OS == .MACOS {
    if_clonereq :: struct {
        ifcr_total: s32;
        ifcr_count: s32;
        ifcr_buffer: *u8;
    }

}

#if OS == .MACOS {
    if_data :: struct {
        ifi_type: u8;
        ifi_typelen: u8;
        ifi_physical: u8;
        ifi_addrlen: u8;
        ifi_hdrlen: u8;
        ifi_recvquota: u8;
        ifi_xmitquota: u8;
        ifi_unused1: u8;
        ifi_mtu: u32;
        ifi_metric: u32;
        ifi_baudrate: u32;

        ifi_ipackets: u32;
        ifi_ierrors: u32;
        ifi_opackets: u32;
        ifi_oerrors: u32;
        ifi_collisions: u32;
        ifi_ibytes: u32;
        ifi_obytes: u32;
        ifi_imcasts: u32;
        ifi_omcasts: u32;
        ifi_iqdrops: u32;
        ifi_noproto: u32;
        ifi_recvtiming: u32;
        ifi_xmittiming: u32;
        ifi_lastchange: timeval32;
        ifi_unused2: u32;
        ifi_hwassist: u32;
        ifi_reserved1: u32;
        ifi_reserved2: u32;
    }

}

#if OS == .MACOS {
    if_data64 :: struct {
        ifi_type: u8;
        ifi_typelen: u8;
        ifi_physical: u8;
        ifi_addrlen: u8;
        ifi_hdrlen: u8;
        ifi_recvquota: u8;
        ifi_xmitquota: u8;
        ifi_unused1: u8;
        ifi_mtu: u32;
        ifi_metric: u32;
        ifi_baudrate: u64 #align 4;

        ifi_ipackets: u64 #align 4;
        ifi_ierrors: u64 #align 4;
        ifi_opackets: u64 #align 4;
        ifi_oerrors: u64 #align 4;
        ifi_collisions: u64 #align 4;
        ifi_ibytes: u64 #align 4;
        ifi_obytes: u64 #align 4;
        ifi_imcasts: u64 #align 4;
        ifi_omcasts: u64 #align 4;
        ifi_iqdrops: u64 #align 4;
        ifi_noproto: u64 #align 4;
        ifi_recvtiming: u32;
        ifi_xmittiming: u32;
        ifi_lastchange: timeval32;
    }

}

#if OS == .MACOS {
    if_msghdr :: struct {
        ifm_msglen: u16;
        ifm_version: u8;
        ifm_type: u8;
        ifm_addrs: s32;
        ifm_flags: s32;
        ifm_index: u16;
        ifm_data: if_data;
    }

}

#if OS == .MACOS {
    if_msghdr2 :: struct {
        ifm_msglen: u16;
        ifm_version: u8;
        ifm_type: u8;
        ifm_addrs: s32;
        ifm_flags: s32;
        ifm_index: u16;
        ifm_snd_len: s32;
        ifm_snd_maxlen: s32;
        ifm_snd_drops: s32;
        ifm_timer: s32;
        ifm_data: if_data64;
    }

}

#if OS == .LINUX {
    if_nameindex :: struct {
        if_index: u32;
        if_name: *u8;
    }

}

#if OS == .ANDROID {
    if_nameindex :: struct {
        if_index: u32;
        if_name: *u8;
    }

}

#if OS == .MACOS {
    if_nameindex :: struct {
        if_index: u32;
        if_name: *u8;
    }

}

#if OS == .MACOS {
    ifa_msghdr :: struct {
        ifam_msglen: u16;
        ifam_version: u8;
        ifam_type: u8;
        ifam_addrs: s32;
        ifam_flags: s32;
        ifam_index: u16;
        ifam_metric: s32;
    }

}

#if OS == .LINUX {
    ifaddr :: struct {
        ifa_addr: sockaddr;
        ifa_ifu: struct {
            ifu_broadaddr: sockaddr;

            #overlay (ifu_broadaddr) ifu_dstaddr: sockaddr;
        };

        iface :: struct {}
        ifa_ifp: *iface;
        ifa_next: *ifaddr;
    }

}

#if OS == .LINUX {
    ifaddrs :: struct {
        ifa_next: *ifaddrs;

        ifa_name: *u8;
        ifa_flags: u32;

        ifa_addr: *sockaddr;
        ifa_netmask: *sockaddr;
        ifa_ifu: struct {
            ifu_broadaddr: *sockaddr;

            #overlay (ifu_broadaddr) ifu_dstaddr: *sockaddr;
        };

        ifa_data: *void;
    }

}

#if OS == .ANDROID {
    ifaddrs :: struct {

        ifa_next: *ifaddrs;

        ifa_name: *u8;

        ifa_flags: u32;

        ifa_addr: *sockaddr;

        ifa_netmask: *sockaddr;

        ifa_ifu: struct {

            ifu_broadaddr: *sockaddr;

            #overlay (ifu_broadaddr)
            ifu_dstaddr: *sockaddr;
        };

        ifa_data: *void;
    }

}

#if OS == .MACOS {
    ifaddrs :: struct {
        ifa_next: *ifaddrs;
        ifa_name: *u8;
        ifa_flags: u32;
        ifa_addr: *sockaddr;
        ifa_netmask: *sockaddr;
        ifa_dstaddr: *sockaddr;
        ifa_data: *void;
    }

}

#if OS == .MACOS {
    ifaliasreq :: struct {
        ifra_name: [16] u8;
        ifra_addr: sockaddr;
        ifra_broadaddr: sockaddr;
        ifra_mask: sockaddr;
    }

}

#if OS == .LINUX {
    ifconf :: struct {
        ifc_len: s32;
        ifc_ifcu: struct {
            ifcu_buf: *u8;

            #overlay (ifcu_buf) ifcu_req: *ifreq;
        };
    }

}

#if OS == .MACOS {
    ifconf :: struct {
        ifc_len: s32;
        ifc_ifcu: struct {
            ifcu_buf: *u8 #align 4;

            #overlay (ifcu_buf) ifcu_req: *ifreq #align 4;
        };
    }

}

#if OS == .MACOS {
    ifdevmtu :: struct {
        ifdm_current: s32;
        ifdm_min: s32;
        ifdm_max: s32;
    }

}

#if OS == .MACOS {
    ifdrv :: struct {
        ifd_name: [16] u8;
        ifd_cmd: u64 #align 4;
        ifd_len: u64 #align 4;
        ifd_data: *void #align 4;
    }

}

#if OS == .LINUX {
    IFF :: enum u32 {
        UP :: 1;

        BROADCAST :: 2;

        DEBUG :: 4;

        LOOPBACK :: 8;

        POINTOPOINT :: 16;

        NOTRAILERS :: 32;

        RUNNING :: 64;

        NOARP :: 128;

        PROMISC :: 256;

        ALLMULTI :: 512;

        MASTER :: 1024;

        SLAVE :: 2048;

        MULTICAST :: 4096;

        PORTSEL :: 8192;

        AUTOMEDIA :: 16384;

        DYNAMIC :: 32768;

        IFF_UP :: UP;

        IFF_BROADCAST :: BROADCAST;

        IFF_DEBUG :: DEBUG;

        IFF_LOOPBACK :: LOOPBACK;

        IFF_POINTOPOINT :: POINTOPOINT;

        IFF_NOTRAILERS :: NOTRAILERS;

        IFF_RUNNING :: RUNNING;

        IFF_NOARP :: NOARP;

        IFF_PROMISC :: PROMISC;

        IFF_ALLMULTI :: ALLMULTI;

        IFF_MASTER :: MASTER;

        IFF_SLAVE :: SLAVE;

        IFF_MULTICAST :: MULTICAST;

        IFF_PORTSEL :: PORTSEL;

        IFF_AUTOMEDIA :: AUTOMEDIA;

        IFF_DYNAMIC :: DYNAMIC;
    }

}

#if OS == .MACOS {
    IFF :: enum {
        UP :: 0x1;
        BROADCAST :: 0x2;
        DEBUG :: 0x4;
        LOOPBACK :: 0x8;
        POINTOPOINT :: 0x10;
        NOTRAILERS :: 0x20;
        RUNNING :: 0x40;
        NOARP :: 0x80;
        PROMISC :: 0x100;
        ALLMULTI :: 0x200;
        OACTIVE :: 0x400;
        SIMPLEX :: 0x800;
        LINK0 :: 0x1000;
        LINK1 :: 0x2000;
        LINK2 :: 0x4000;
        ALTPHYS :: LINK2;
        MULTICAST :: 0x8000;

        IFF_UP :: UP;
        IFF_BROADCAST :: BROADCAST;
        IFF_DEBUG :: DEBUG;
        IFF_LOOPBACK :: LOOPBACK;
        IFF_POINTOPOINT :: POINTOPOINT;
        IFF_NOTRAILERS :: NOTRAILERS;
        IFF_RUNNING :: RUNNING;
        IFF_NOARP :: NOARP;
        IFF_PROMISC :: PROMISC;
        IFF_ALLMULTI :: ALLMULTI;
        IFF_OACTIVE :: OACTIVE;
        IFF_SIMPLEX :: SIMPLEX;
        IFF_LINK0 :: LINK0;
        IFF_LINK1 :: LINK1;
        IFF_LINK2 :: LINK2;
        IFF_ALTPHYS :: ALTPHYS;
        IFF_MULTICAST :: MULTICAST;
    }

}

#if OS == .MACOS {
    ifkpi :: struct {
        ifk_module_id: u32;
        ifk_type: u32;
        ifk_data: struct {
            ifk_ptr: *void #align 4;

            #overlay (ifk_ptr) ifk_value: s32;
        };
    }

}

#if OS == .MACOS {
    ifma_msghdr :: struct {
        ifmam_msglen: u16;
        ifmam_version: u8;
        ifmam_type: u8;
        ifmam_addrs: s32;
        ifmam_flags: s32;
        ifmam_index: u16;
    }

}

#if OS == .MACOS {
    ifma_msghdr2 :: struct {
        ifmam_msglen: u16;
        ifmam_version: u8;
        ifmam_type: u8;
        ifmam_addrs: s32;
        ifmam_flags: s32;
        ifmam_index: u16;
        ifmam_refcount: s32;
    }

}

#if OS == .MACOS {
    ifmaddrs :: struct {
        ifma_next: *ifmaddrs;
        ifma_name: *sockaddr;
        ifma_addr: *sockaddr;
        ifma_lladdr: *sockaddr;
    }

}

#if OS == .LINUX {
    ifmap :: struct {
        mem_start: u64;
        mem_end: u64;
        base_addr: u16;
        irq: u8;
        dma: u8;
        port: u8;
    }

}

#if OS == .MACOS {
    ifmediareq :: struct {
        ifm_name: [16] u8;
        ifm_current: s32;
        ifm_mask: s32;
        ifm_status: s32;
        ifm_active: s32;
        ifm_count: s32;
        ifm_ulist: *s32 #align 4;
    }

}

#if OS == .MACOS {
    ifqueue :: struct {
        ifq_head: *void;
        ifq_tail: *void;
        ifq_len: s32;
        ifq_maxlen: s32;
        ifq_drops: s32;
    }

}

#if OS == .LINUX {
    ifreq :: struct {
        ifr_ifrn: struct {
            ifrn_name: [16] u8;
        };

        ifr_ifru: struct {
            ifru_map: ifmap;

            #overlay (ifru_map) ifru_addr: sockaddr;
            #overlay (ifru_map) ifru_dstaddr: sockaddr;
            #overlay (ifru_map) ifru_broadaddr: sockaddr;
            #overlay (ifru_map) ifru_netmask: sockaddr;
            #overlay (ifru_map) ifru_hwaddr: sockaddr;
            #overlay (ifru_map) ifru_flags: s16;
            #overlay (ifru_map) ifru_ivalue: s32;
            #overlay (ifru_map) ifru_mtu: s32;
            #overlay (ifru_map) ifru_slave: [16] u8;
            #overlay (ifru_map) ifru_newname: [16] u8;
            #overlay (ifru_map) ifru_data: *u8;
        };
    }

}

#if OS == .MACOS {
    ifreq :: struct {
        ifr_name: [16] u8;
        ifr_ifru: struct {
            ifru_addr: sockaddr;

            #overlay (ifru_addr) ifru_dstaddr: sockaddr;
            #overlay (ifru_addr) ifru_broadaddr: sockaddr;
            #overlay (ifru_addr) ifru_flags: s16;
            #overlay (ifru_addr) ifru_metric: s32;
            #overlay (ifru_addr) ifru_mtu: s32;
            #overlay (ifru_addr) ifru_phys: s32;
            #overlay (ifru_addr) ifru_media: s32;
            #overlay (ifru_addr) ifru_intval: s32;
            #overlay (ifru_addr) ifru_data: *u8;
            #overlay (ifru_addr) ifru_devmtu: ifdevmtu;
            #overlay (ifru_addr) ifru_kpi: ifkpi;
            #overlay (ifru_addr) ifru_wake_flags: u32;
            #overlay (ifru_addr) ifru_route_refcnt: u32;
            #overlay (ifru_addr) ifru_cap: [2] s32;
            #overlay (ifru_addr) ifru_functional_type: u32;
        };
    }

}

#if OS == .MACOS {
    ifstat :: struct {
        ifs_name: [16] u8;
        ascii: [801] u8;
    }

}

#if OS == .WINDOWS {
    in6_addr :: struct {
        u: struct {
            Byte: [16] u8;

            #overlay (Byte) Word: [8] u16;
        };
    }

}

#if OS == .LINUX {
    in6_addr :: struct {
        __in6_u: struct {
            __u6_addr8: [16] u8;

            #overlay (__u6_addr8) __u6_addr16: [8] u16;
            #overlay (__u6_addr8) __u6_addr32: [4] u32;
        };
    }

}

#if OS == .ANDROID {
    in6_addr :: struct {
        in6_u: struct {
            u6_addr8: [16] u8;

            #overlay (u6_addr8) u6_addr16: [8] u16;
            #overlay (u6_addr8) u6_addr32: [4] u32;
        };
    }

}

#if OS == .MACOS {
    in6_addr :: struct {
        __u6_addr: struct {
            __u6_addr8: [16] u8;

            #overlay (__u6_addr8) __u6_addr16: [8] u16;
            #overlay (__u6_addr8) __u6_addr32: [4] u32;
        };
    }

}

#if OS == .ANDROID {
    in6_flowlabel_req :: struct {
        flr_dst: in6_addr;
        flr_label: u32;
        flr_action: u8;
        flr_share: u8;
        flr_flags: u16;
        flr_expires: u16;
        flr_linger: u16;
        __flr_pad: u32;
    }

}

#if OS == .WINDOWS {
    in6_pktinfo :: struct {
        ipi6_addr: IN6_ADDR;
        ipi6_ifindex: u32;
    }

}

#if OS == .LINUX {
    in6_pktinfo :: struct {
        ipi6_addr: in6_addr;
        ipi6_ifindex: u32;
    }

}

#if OS == .MACOS {
    in6_pktinfo :: struct {
        ipi6_addr: in6_addr;
        ipi6_ifindex: u32;
    }

}

#if OS == .WINDOWS {
    in6_pktinfo_ex :: struct {
        pkt_info: IN6_PKTINFO;
        scope_id: SCOPE_ID;
    }

}

#if OS == .WINDOWS {
    in_addr :: struct {
        S_un: struct {
            S_un_b: struct {
                s_b1: u8;
                s_b2: u8;
                s_b3: u8;
                s_b4: u8;
            };

            #overlay (S_un_b) S_un_w: struct {
                s_w1: u16;
                s_w2: u16;
            };
            #overlay (S_un_b) S_addr: u32;
        };
    }

}

#if OS == .LINUX {
    in_addr :: struct {
        s_addr: in_addr_t;
    }

}

#if OS == .WINDOWS {
    in_pktinfo :: struct {
        ipi_addr: IN_ADDR;
        ipi_ifindex: u32;
    }

}

#if OS == .LINUX {
    in_pktinfo :: struct {
        ipi_ifindex: s32;
        ipi_spec_dst: in_addr;
        ipi_addr: in_addr;
    }

}

#if OS == .MACOS {
    in_pktinfo :: struct {
        ipi_ifindex: u32;
        ipi_spec_dst: in_addr;
        ipi_addr: in_addr;
    }

}

#if OS == .WINDOWS {
    in_pktinfo_ex :: struct {
        pkt_info: IN_PKTINFO;
        scope_id: SCOPE_ID;
    }

}

#if OS == .WINDOWS {
    in_recverr :: struct {
        protocol: IPPROTO;
        info: u32;
        type: u8;
        code: u8;
    }

}

#if OS == .WINDOWS {
    INET_PORT_RANGE :: struct {
        StartPort: u16;
        NumberOfPorts: u16;
    }

}

#if OS == .WINDOWS {
    INET_PORT_RESERVATION_INFORMATION :: struct {
        OwningPid: u32;
    }

}

#if OS == .WINDOWS {
    INET_PORT_RESERVATION_INSTANCE :: struct {
        Reservation: INET_PORT_RESERVATION;
        Token: INET_PORT_RESERVATION_TOKEN;
    }

}

#if OS == .WINDOWS {
    INET_PORT_RESERVATION_TOKEN :: struct {
        Token: u64;
    }

}

#if OS == .WINDOWS {
    INTERFACE_INFO :: struct {
        iiFlags: u32;
        iiAddress: sockaddr_gen;
        iiBroadcastAddress: sockaddr_gen;
        iiNetmask: sockaddr_gen;
    }

}

#if OS == .WINDOWS {
    INTERFACE_INFO_EX :: struct {
        iiFlags: u32;
        iiAddress: SOCKET_ADDRESS;
        iiBroadcastAddress: SOCKET_ADDRESS;
        iiNetmask: SOCKET_ADDRESS;
    }

}

#if OS == .LINUX {
    ip6_mtuinfo :: struct {
        ip6m_addr: sockaddr_in6;
        ip6m_mtu: u32;
    }

}

#if OS == .MACOS {
    ip6_mtuinfo :: struct {
        ip6m_addr: sockaddr_in6;
        ip6m_mtu: u32;
    }

}

#if OS == .WINDOWS {
    ip_mreq :: struct {
        imr_multiaddr: IN_ADDR;
        imr_interface: IN_ADDR;
    }

}

#if OS == .LINUX {
    ip_mreq :: struct {

        imr_multiaddr: in_addr;

        imr_interface: in_addr;
    }

}

#if OS == .ANDROID {
    ip_mreq :: struct {
        imr_multiaddr: in_addr;
        imr_interface: in_addr;
    }

}

#if OS == .MACOS {
    ip_mreq :: struct {
        imr_multiaddr: in_addr;
        imr_interface: in_addr;
    }

}

#if OS == .WINDOWS {
    ip_mreq_source :: struct {
        imr_multiaddr: IN_ADDR;
        imr_sourceaddr: IN_ADDR;
        imr_interface: IN_ADDR;
    }

}

#if OS == .LINUX {
    ip_mreq_source :: struct {

        imr_multiaddr: in_addr;

        imr_interface: in_addr;

        imr_sourceaddr: in_addr;
    }

}

#if OS == .MACOS {
    ip_mreq_source :: struct {
        imr_multiaddr: in_addr;
        imr_sourceaddr: in_addr;
        imr_interface: in_addr;
    }

}

#if OS == .LINUX {
    ip_mreqn :: struct {
        imr_multiaddr: in_addr;
        imr_address: in_addr;
        imr_ifindex: s32;
    }

}

#if OS == .MACOS {
    ip_mreqn :: struct {
        imr_multiaddr: in_addr;
        imr_address: in_addr;
        imr_ifindex: s32;
    }

}

#if OS == .WINDOWS {
    ip_msfilter :: struct {
        imsf_multiaddr: IN_ADDR;
        imsf_interface: IN_ADDR;
        imsf_fmode: MULTICAST_MODE_TYPE;
        imsf_numsrc: u32;
        imsf_slist: [1] IN_ADDR;
    }

}

#if OS == .LINUX {
    ip_msfilter :: struct {

        imsf_multiaddr: in_addr;

        imsf_interface: in_addr;

        imsf_fmode: u32;

        imsf_numsrc: u32;

        imsf_slist: [1] in_addr;
    }

}

#if OS == .LINUX {
    ip_opts :: struct {
        ip_dst: in_addr;
        ip_opts: [40] u8;
    }

}

#if OS == .MACOS {
    ip_opts :: struct {
        ip_dst: in_addr;
        ip_opts: [40] u8;
    }

}

#if OS == .LINUX {
    IPPORT :: enum u32 {
        ECHO :: 7;
        DISCARD :: 9;
        SYSTAT :: 11;
        DAYTIME :: 13;
        NETSTAT :: 15;
        FTP :: 21;
        TELNET :: 23;
        SMTP :: 25;
        TIMESERVER :: 37;
        NAMESERVER :: 42;
        WHOIS :: 43;
        MTP :: 57;

        TFTP :: 69;
        RJE :: 77;
        FINGER :: 79;
        TTYLINK :: 87;
        SUPDUP :: 95;

        EXECSERVER :: 512;
        LOGINSERVER :: 513;
        CMDSERVER :: 514;
        EFSSERVER :: 520;

        BIFFUDP :: 512;
        WHOSERVER :: 513;
        ROUTESERVER :: 520;

        RESERVED :: 1024;

        USERRESERVED :: 5000;

        IPPORT_ECHO :: ECHO;
        IPPORT_DISCARD :: DISCARD;
        IPPORT_SYSTAT :: SYSTAT;
        IPPORT_DAYTIME :: DAYTIME;
        IPPORT_NETSTAT :: NETSTAT;
        IPPORT_FTP :: FTP;
        IPPORT_TELNET :: TELNET;
        IPPORT_SMTP :: SMTP;
        IPPORT_TIMESERVER :: TIMESERVER;
        IPPORT_NAMESERVER :: NAMESERVER;
        IPPORT_WHOIS :: WHOIS;
        IPPORT_MTP :: MTP;

        IPPORT_TFTP :: TFTP;
        IPPORT_RJE :: RJE;
        IPPORT_FINGER :: FINGER;
        IPPORT_TTYLINK :: TTYLINK;
        IPPORT_SUPDUP :: SUPDUP;

        IPPORT_EXECSERVER :: EXECSERVER;
        IPPORT_LOGINSERVER :: LOGINSERVER;
        IPPORT_CMDSERVER :: CMDSERVER;
        IPPORT_EFSSERVER :: EFSSERVER;

        IPPORT_BIFFUDP :: BIFFUDP;
        IPPORT_WHOSERVER :: WHOSERVER;
        IPPORT_ROUTESERVER :: ROUTESERVER;

        IPPORT_RESERVED :: RESERVED;

        IPPORT_USERRESERVED :: USERRESERVED;
    }

}

#if OS == .WINDOWS {
    IPPROTO :: enum s32 {
        HOPOPTS :: 0;
        ICMP :: 1;
        IGMP :: 2;
        GGP :: 3;
        IPV4 :: 4;
        ST :: 5;
        TCP :: 6;
        CBT :: 7;
        EGP :: 8;
        IGP :: 9;
        PUP :: 12;
        UDP :: 17;
        IDP :: 22;
        RDP :: 27;
        IPV6 :: 41;
        ROUTING :: 43;
        FRAGMENT :: 44;
        ESP :: 50;
        AH :: 51;
        ICMPV6 :: 58;
        NONE :: 59;
        DSTOPTS :: 60;
        ND :: 77;
        ICLFXBM :: 78;
        PIM :: 103;
        PGM :: 113;
        L2TP :: 115;
        SCTP :: 132;
        RAW :: 255;
        MAX :: 256;
        RESERVED_RAW :: 257;
        RESERVED_IPSEC :: 258;
        RESERVED_IPSECOFFLOAD :: 259;
        RESERVED_WNV :: 260;
        RESERVED_MAX :: 261;

        IPPROTO_HOPOPTS :: HOPOPTS;
        IPPROTO_ICMP :: ICMP;
        IPPROTO_IGMP :: IGMP;
        IPPROTO_GGP :: GGP;
        IPPROTO_IPV4 :: IPV4;
        IPPROTO_ST :: ST;
        IPPROTO_TCP :: TCP;
        IPPROTO_CBT :: CBT;
        IPPROTO_EGP :: EGP;
        IPPROTO_IGP :: IGP;
        IPPROTO_PUP :: PUP;
        IPPROTO_UDP :: UDP;
        IPPROTO_IDP :: IDP;
        IPPROTO_RDP :: RDP;
        IPPROTO_IPV6 :: IPV6;
        IPPROTO_ROUTING :: ROUTING;
        IPPROTO_FRAGMENT :: FRAGMENT;
        IPPROTO_ESP :: ESP;
        IPPROTO_AH :: AH;
        IPPROTO_ICMPV6 :: ICMPV6;
        IPPROTO_NONE :: NONE;
        IPPROTO_DSTOPTS :: DSTOPTS;
        IPPROTO_ND :: ND;
        IPPROTO_ICLFXBM :: ICLFXBM;
        IPPROTO_PIM :: PIM;
        IPPROTO_PGM :: PGM;
        IPPROTO_L2TP :: L2TP;
        IPPROTO_SCTP :: SCTP;
        IPPROTO_RAW :: RAW;
        IPPROTO_MAX :: MAX;
        IPPROTO_RESERVED_RAW :: RESERVED_RAW;
        IPPROTO_RESERVED_IPSEC :: RESERVED_IPSEC;
        IPPROTO_RESERVED_IPSECOFFLOAD :: RESERVED_IPSECOFFLOAD;
        IPPROTO_RESERVED_WNV :: RESERVED_WNV;
        IPPROTO_RESERVED_MAX :: RESERVED_MAX;
    }

}

#if OS == .LINUX {
    IPPROTO :: enum u32 {
        IP :: 0;
        ICMP :: 1;
        IGMP :: 2;
        IPIP :: 4;
        TCP :: 6;
        EGP :: 8;
        PUP :: 12;
        UDP :: 17;
        IDP :: 22;
        TP :: 29;
        DCCP :: 33;
        IPV6 :: 41;
        RSVP :: 46;
        GRE :: 47;
        ESP :: 50;
        AH :: 51;
        MTP :: 92;
        BEETPH :: 94;
        ENCAP :: 98;
        PIM :: 103;
        COMP :: 108;
        SCTP :: 132;
        UDPLITE :: 136;
        MPLS :: 137;
        RAW :: 255;
        MAX :: 256;

        IPPROTO_IP :: IP;
        IPPROTO_ICMP :: ICMP;
        IPPROTO_IGMP :: IGMP;
        IPPROTO_IPIP :: IPIP;
        IPPROTO_TCP :: TCP;
        IPPROTO_EGP :: EGP;
        IPPROTO_PUP :: PUP;
        IPPROTO_UDP :: UDP;
        IPPROTO_IDP :: IDP;
        IPPROTO_TP :: TP;
        IPPROTO_DCCP :: DCCP;
        IPPROTO_IPV6 :: IPV6;
        IPPROTO_RSVP :: RSVP;
        IPPROTO_GRE :: GRE;
        IPPROTO_ESP :: ESP;
        IPPROTO_AH :: AH;
        IPPROTO_MTP :: MTP;
        IPPROTO_BEETPH :: BEETPH;
        IPPROTO_ENCAP :: ENCAP;
        IPPROTO_PIM :: PIM;
        IPPROTO_COMP :: COMP;
        IPPROTO_SCTP :: SCTP;
        IPPROTO_UDPLITE :: UDPLITE;
        IPPROTO_MPLS :: MPLS;
        IPPROTO_RAW :: RAW;
        IPPROTO_MAX :: MAX;
    }

}

#if OS == .ANDROID {
    IPPROTO :: enum u32 {
        IP :: 0;
        ICMP :: 1;
        IGMP :: 2;
        IPIP :: 4;
        TCP :: 6;
        EGP :: 8;
        PUP :: 12;
        UDP :: 17;
        IDP :: 22;
        TP :: 29;
        DCCP :: 33;
        IPV6 :: 41;
        RSVP :: 46;
        GRE :: 47;
        ESP :: 50;
        AH :: 51;
        MTP :: 92;
        BEETPH :: 94;
        ENCAP :: 98;
        PIM :: 103;
        COMP :: 108;
        L2TP :: 115;
        SCTP :: 132;
        UDPLITE :: 136;
        MPLS :: 137;
        ETHERNET :: 143;
        RAW :: 255;
        MPTCP :: 262;
        MAX :: 263;
        IPPROTO_HOPOPTS :: 0;
        IPPROTO_ROUTING :: 43;
        IPPROTO_FRAGMENT :: 44;
        IPPROTO_ICMPV6 :: 58;
        IPPROTO_NONE :: 59;
        IPPROTO_DSTOPTS :: 60;
        IPPROTO_MH :: 135;

        IPPROTO_IP :: IP;
        IPPROTO_ICMP :: ICMP;
        IPPROTO_IGMP :: IGMP;
        IPPROTO_IPIP :: IPIP;
        IPPROTO_TCP :: TCP;
        IPPROTO_EGP :: EGP;
        IPPROTO_PUP :: PUP;
        IPPROTO_UDP :: UDP;
        IPPROTO_IDP :: IDP;
        IPPROTO_TP :: TP;
        IPPROTO_DCCP :: DCCP;
        IPPROTO_IPV6 :: IPV6;
        IPPROTO_RSVP :: RSVP;
        IPPROTO_GRE :: GRE;
        IPPROTO_ESP :: ESP;
        IPPROTO_AH :: AH;
        IPPROTO_MTP :: MTP;
        IPPROTO_BEETPH :: BEETPH;
        IPPROTO_ENCAP :: ENCAP;
        IPPROTO_PIM :: PIM;
        IPPROTO_COMP :: COMP;
        IPPROTO_L2TP :: L2TP;
        IPPROTO_SCTP :: SCTP;
        IPPROTO_UDPLITE :: UDPLITE;
        IPPROTO_MPLS :: MPLS;
        IPPROTO_ETHERNET :: ETHERNET;
        IPPROTO_RAW :: RAW;
        IPPROTO_MPTCP :: MPTCP;
        IPPROTO_MAX :: MAX;
    }

}

#if OS == .MACOS {
    IPPROTO :: enum u32 {
        IP :: 0;
        HOPOPTS :: 0;
        ICMP :: 1;
        IGMP :: 2;
        GGP :: 3;
        IPV4 :: 4;
        IPIP :: IPV4;
        TCP :: 6;
        ST :: 7;
        EGP :: 8;
        PIGP :: 9;
        RCCMON :: 10;
        NVPII :: 11;
        PUP :: 12;
        ARGUS :: 13;
        EMCON :: 14;
        XNET :: 15;
        CHAOS :: 16;
        UDP :: 17;
        MUX :: 18;
        MEAS :: 19;
        HMP :: 20;
        PRM :: 21;
        IDP :: 22;
        TRUNK1 :: 23;
        TRUNK2 :: 24;
        LEAF1 :: 25;
        LEAF2 :: 26;
        RDP :: 27;
        IRTP :: 28;
        TP :: 29;
        BLT :: 30;
        NSP :: 31;
        INP :: 32;
        SEP :: 33;
        _3PC :: 34;
        IDPR :: 35;
        XTP :: 36;
        DDP :: 37;
        CMTP :: 38;
        TPXX :: 39;
        IL :: 40;
        IPV6 :: 41;
        SDRP :: 42;
        ROUTING :: 43;
        FRAGMENT :: 44;
        IDRP :: 45;
        RSVP :: 46;
        GRE :: 47;
        MHRP :: 48;
        BHA :: 49;
        ESP :: 50;
        AH :: 51;
        INLSP :: 52;
        SWIPE :: 53;
        NHRP :: 54;
        ICMPV6 :: 58;
        NONE :: 59;
        DSTOPTS :: 60;
        AHIP :: 61;
        CFTP :: 62;
        HELLO :: 63;
        SATEXPAK :: 64;
        KRYPTOLAN :: 65;
        RVD :: 66;
        IPPC :: 67;
        ADFS :: 68;
        SATMON :: 69;
        VISA :: 70;
        IPCV :: 71;
        CPNX :: 72;
        CPHB :: 73;
        WSN :: 74;
        PVP :: 75;
        BRSATMON :: 76;
        ND :: 77;
        WBMON :: 78;
        WBEXPAK :: 79;
        EON :: 80;
        VMTP :: 81;
        SVMTP :: 82;
        VINES :: 83;
        TTP :: 84;
        IGP :: 85;
        DGP :: 86;
        TCF :: 87;
        IGRP :: 88;
        OSPFIGP :: 89;
        SRPC :: 90;
        LARP :: 91;
        MTP :: 92;
        AX25 :: 93;
        IPEIP :: 94;
        MICP :: 95;
        SCCSP :: 96;
        ETHERIP :: 97;
        ENCAP :: 98;
        APES :: 99;
        GMTP :: 100;
        PIM :: 103;
        IPCOMP :: 108;
        PGM :: 113;
        SCTP :: 132;
        DIVERT :: 254;
        RAW :: 255;
        MAX :: 256;
        DONE :: 257;
        MAXID :: AH + 1;

        IPPROTO_IP :: IP;
        IPPROTO_HOPOPTS :: HOPOPTS;
        IPPROTO_ICMP :: ICMP;
        IPPROTO_IGMP :: IGMP;
        IPPROTO_GGP :: GGP;
        IPPROTO_IPV4 :: IPV4;
        IPPROTO_IPIP :: IPIP;
        IPPROTO_TCP :: TCP;
        IPPROTO_ST :: ST;
        IPPROTO_EGP :: EGP;
        IPPROTO_PIGP :: PIGP;
        IPPROTO_RCCMON :: RCCMON;
        IPPROTO_NVPII :: NVPII;
        IPPROTO_PUP :: PUP;
        IPPROTO_ARGUS :: ARGUS;
        IPPROTO_EMCON :: EMCON;
        IPPROTO_XNET :: XNET;
        IPPROTO_CHAOS :: CHAOS;
        IPPROTO_UDP :: UDP;
        IPPROTO_MUX :: MUX;
        IPPROTO_MEAS :: MEAS;
        IPPROTO_HMP :: HMP;
        IPPROTO_PRM :: PRM;
        IPPROTO_IDP :: IDP;
        IPPROTO_TRUNK1 :: TRUNK1;
        IPPROTO_TRUNK2 :: TRUNK2;
        IPPROTO_LEAF1 :: LEAF1;
        IPPROTO_LEAF2 :: LEAF2;
        IPPROTO_RDP :: RDP;
        IPPROTO_IRTP :: IRTP;
        IPPROTO_TP :: TP;
        IPPROTO_BLT :: BLT;
        IPPROTO_NSP :: NSP;
        IPPROTO_INP :: INP;
        IPPROTO_SEP :: SEP;
        IPPROTO_3PC :: _3PC;
        IPPROTO_IDPR :: IDPR;
        IPPROTO_XTP :: XTP;
        IPPROTO_DDP :: DDP;
        IPPROTO_CMTP :: CMTP;
        IPPROTO_TPXX :: TPXX;
        IPPROTO_IL :: IL;
        IPPROTO_IPV6 :: IPV6;
        IPPROTO_SDRP :: SDRP;
        IPPROTO_ROUTING :: ROUTING;
        IPPROTO_FRAGMENT :: FRAGMENT;
        IPPROTO_IDRP :: IDRP;
        IPPROTO_RSVP :: RSVP;
        IPPROTO_GRE :: GRE;
        IPPROTO_MHRP :: MHRP;
        IPPROTO_BHA :: BHA;
        IPPROTO_ESP :: ESP;
        IPPROTO_AH :: AH;
        IPPROTO_INLSP :: INLSP;
        IPPROTO_SWIPE :: SWIPE;
        IPPROTO_NHRP :: NHRP;
        IPPROTO_ICMPV6 :: ICMPV6;
        IPPROTO_NONE :: NONE;
        IPPROTO_DSTOPTS :: DSTOPTS;
        IPPROTO_AHIP :: AHIP;
        IPPROTO_CFTP :: CFTP;
        IPPROTO_HELLO :: HELLO;
        IPPROTO_SATEXPAK :: SATEXPAK;
        IPPROTO_KRYPTOLAN :: KRYPTOLAN;
        IPPROTO_RVD :: RVD;
        IPPROTO_IPPC :: IPPC;
        IPPROTO_ADFS :: ADFS;
        IPPROTO_SATMON :: SATMON;
        IPPROTO_VISA :: VISA;
        IPPROTO_IPCV :: IPCV;
        IPPROTO_CPNX :: CPNX;
        IPPROTO_CPHB :: CPHB;
        IPPROTO_WSN :: WSN;
        IPPROTO_PVP :: PVP;
        IPPROTO_BRSATMON :: BRSATMON;
        IPPROTO_ND :: ND;
        IPPROTO_WBMON :: WBMON;
        IPPROTO_WBEXPAK :: WBEXPAK;
        IPPROTO_EON :: EON;
        IPPROTO_VMTP :: VMTP;
        IPPROTO_SVMTP :: SVMTP;
        IPPROTO_VINES :: VINES;
        IPPROTO_TTP :: TTP;
        IPPROTO_IGP :: IGP;
        IPPROTO_DGP :: DGP;
        IPPROTO_TCF :: TCF;
        IPPROTO_IGRP :: IGRP;
        IPPROTO_OSPFIGP :: OSPFIGP;
        IPPROTO_SRPC :: SRPC;
        IPPROTO_LARP :: LARP;
        IPPROTO_MTP :: MTP;
        IPPROTO_AX25 :: AX25;
        IPPROTO_IPEIP :: IPEIP;
        IPPROTO_MICP :: MICP;
        IPPROTO_SCCSP :: SCCSP;
        IPPROTO_ETHERIP :: ETHERIP;
        IPPROTO_ENCAP :: ENCAP;
        IPPROTO_APES :: APES;
        IPPROTO_GMTP :: GMTP;
        IPPROTO_PIM :: PIM;
        IPPROTO_IPCOMP :: IPCOMP;
        IPPROTO_PGM :: PGM;
        IPPROTO_SCTP :: SCTP;
        IPPROTO_DIVERT :: DIVERT;
        IPPROTO_RAW :: RAW;
        IPPROTO_MAX :: MAX;
        IPPROTO_DONE :: DONE;
        IPPROTO_MAXID :: MAXID;
    }

}

#if OS == .LINUX {
    IPPROTO_1 :: enum u32 {
        HOPOPTS :: 0;

        ROUTING :: 43;

        FRAGMENT :: 44;

        ICMPV6 :: 58;

        NONE :: 59;

        DSTOPTS :: 60;

        MH :: 135;

        IPPROTO_HOPOPTS :: HOPOPTS;

        IPPROTO_ROUTING :: ROUTING;

        IPPROTO_FRAGMENT :: FRAGMENT;

        IPPROTO_ICMPV6 :: ICMPV6;

        IPPROTO_NONE :: NONE;

        IPPROTO_DSTOPTS :: DSTOPTS;

        IPPROTO_MH :: MH;
    }

}

#if OS == .WINDOWS {
    ipv6_mreq :: struct {
        ipv6mr_multiaddr: IN6_ADDR;
        ipv6mr_interface: u32;
    }

}

#if OS == .LINUX {
    ipv6_mreq :: struct {

        ipv6mr_multiaddr: in6_addr;

        ipv6mr_interface: u32;
    }

}

#if OS == .ANDROID {
    ipv6_mreq :: struct {
        ipv6mr_multiaddr: in6_addr;
        ipv6mr_ifindex: s32;
    }

}

#if OS == .MACOS {
    ipv6_mreq :: struct {
        ipv6mr_multiaddr: in6_addr;
        ipv6mr_interface: u32;
    }

}

#if OS == .MACOS {
    kev_dl_proto_data :: struct {
        link_data: net_event_data;
        proto_family: u32;
        proto_remaining_count: u32;
    }

}

#if OS == .WINDOWS {
    linger :: struct {
        l_onoff: u16;
        l_linger: u16;
    }

}

#if OS == .LINUX {
    linger :: struct {
        l_onoff: s32;
        l_linger: s32;
    }

}

#if OS == .MACOS {
    linger :: struct {
        l_onoff: s32;
        l_linger: s32;
    }

}

#if OS == .WINDOWS {
    LPCONDITIONPROC :: #type (lpCallerId: *WSABUF, lpCallerData: *WSABUF, lpSQOS: *QualityOfService, lpGQOS: *QualityOfService, lpCalleeId: *WSABUF, lpCalleeData: *WSABUF, g: *GROUP, dwCallbackData: u64) -> s32 #c_call;
}

#if OS == .WINDOWS {
    LPLOOKUPSERVICE_COMPLETION_ROUTINE :: #type (dwError: u32, dwBytes: u32, lpOverlapped: *OVERLAPPED) -> void #c_call;
}

#if OS == .WINDOWS {
    LPWSAOVERLAPPED_COMPLETION_ROUTINE :: #type (dwError: u32, cbTransferred: u32, lpOverlapped: *OVERLAPPED, dwFlags: u32) -> void #c_call;
}

#if OS == .LINUX {
    mmsghdr :: struct {
        msg_hdr: msghdr;

        msg_len: u32;
    }

}

#if OS == .ANDROID {
    mmsghdr :: struct {
        msg_hdr: msghdr;
        msg_len: u32;
    }

}

#if OS == .WINDOWS {
    MSG :: enum_flags s32 {
        TRUNC :: 0x0100;
        CTRUNC :: 0x0200;
        BCAST :: 0x0400;
        MCAST :: 0x0800;
        ERRQUEUE :: 0x1000;
        OOB :: 0x1;
        PEEK :: 0x2;
        DONTROUTE :: 0x4;
        WAITALL :: 0x8;
        PUSH_IMMEDIATE :: 0x20;
        PARTIAL :: 0x8000;
        INTERRUPT :: 0x10;
        MAXIOVLEN :: 16;

        MSG_TRUNC :: TRUNC;
        MSG_CTRUNC :: CTRUNC;
        MSG_BCAST :: BCAST;
        MSG_MCAST :: MCAST;
        MSG_ERRQUEUE :: ERRQUEUE;
        MSG_OOB :: OOB;
        MSG_PEEK :: PEEK;
        MSG_DONTROUTE :: DONTROUTE;
        MSG_WAITALL :: WAITALL;
        MSG_PUSH_IMMEDIATE :: PUSH_IMMEDIATE;
        MSG_PARTIAL :: PARTIAL;
        MSG_INTERRUPT :: INTERRUPT;
        MSG_MAXIOVLEN :: MAXIOVLEN;
    }

}

#if OS == .LINUX {
    MSG :: enum_flags s32 {
        OOB :: 0x1;

        PEEK :: 0x2;

        DONTROUTE :: 0x4;

        TRYHARD :: 0x4;

        CTRUNC :: 0x8;

        PROXY :: 0x10;

        TRUNC :: 0x20;

        DONTWAIT :: 0x40;

        EOR :: 0x80;

        WAITALL :: 0x100;

        FIN :: 0x200;

        SYN :: 0x400;

        CONFIRM :: 0x800;

        RST :: 0x1000;

        ERRQUEUE :: 0x2000;

        NOSIGNAL :: 0x4000;

        MORE :: 0x8000;

        WAITFORONE :: 0x10000;

        BATCH :: 0x40000;

        ZEROCOPY :: 0x4000000;

        FASTOPEN :: 0x20000000;

        CMSG_CLOEXEC :: 0x40000000;

        MSG_OOB :: OOB;

        MSG_PEEK :: PEEK;

        MSG_DONTROUTE :: DONTROUTE;

        MSG_TRYHARD :: TRYHARD;

        MSG_CTRUNC :: CTRUNC;

        MSG_PROXY :: PROXY;

        MSG_TRUNC :: TRUNC;

        MSG_DONTWAIT :: DONTWAIT;

        MSG_EOR :: EOR;

        MSG_WAITALL :: WAITALL;

        MSG_FIN :: FIN;

        MSG_SYN :: SYN;

        MSG_CONFIRM :: CONFIRM;

        MSG_RST :: RST;

        MSG_ERRQUEUE :: ERRQUEUE;

        MSG_NOSIGNAL :: NOSIGNAL;

        MSG_MORE :: MORE;

        MSG_WAITFORONE :: WAITFORONE;

        MSG_BATCH :: BATCH;

        MSG_ZEROCOPY :: ZEROCOPY;

        MSG_FASTOPEN :: FASTOPEN;

        MSG_CMSG_CLOEXEC :: CMSG_CLOEXEC;
    }

}

#if OS == .ANDROID {
    MSG :: enum_flags s32 {
        OOB :: 1;
        PEEK :: 2;
        DONTROUTE :: 4;
        TRYHARD :: 4;
        CTRUNC :: 8;
        PROBE :: 0x10;
        TRUNC :: 0x20;
        DONTWAIT :: 0x40;
        EOR :: 0x80;
        WAITALL :: 0x100;
        FIN :: 0x200;
        SYN :: 0x400;
        CONFIRM :: 0x800;
        RST :: 0x1000;
        ERRQUEUE :: 0x2000;
        NOSIGNAL :: 0x4000;
        MORE :: 0x8000;
        WAITFORONE :: 0x10000;
        BATCH :: 0x40000;
        FASTOPEN :: 0x20000000;
        CMSG_CLOEXEC :: 0x40000000;
        EOF :: FIN;
        CMSG_COMPAT :: 0;

        MSG_OOB :: OOB;
        MSG_PEEK :: PEEK;
        MSG_DONTROUTE :: DONTROUTE;
        MSG_TRYHARD :: TRYHARD;
        MSG_CTRUNC :: CTRUNC;
        MSG_PROBE :: PROBE;
        MSG_TRUNC :: TRUNC;
        MSG_DONTWAIT :: DONTWAIT;
        MSG_EOR :: EOR;
        MSG_WAITALL :: WAITALL;
        MSG_FIN :: FIN;
        MSG_SYN :: SYN;
        MSG_CONFIRM :: CONFIRM;
        MSG_RST :: RST;
        MSG_ERRQUEUE :: ERRQUEUE;
        MSG_NOSIGNAL :: NOSIGNAL;
        MSG_MORE :: MORE;
        MSG_WAITFORONE :: WAITFORONE;
        MSG_BATCH :: BATCH;
        MSG_FASTOPEN :: FASTOPEN;
        MSG_CMSG_CLOEXEC :: CMSG_CLOEXEC;
        MSG_EOF :: EOF;
        MSG_CMSG_COMPAT :: CMSG_COMPAT;
    }

}

#if OS == .MACOS {
    MSG :: enum_flags s32 {
        OOB :: 0x1;
        PEEK :: 0x2;
        DONTROUTE :: 0x4;
        EOR :: 0x8;
        TRUNC :: 0x10;
        CTRUNC :: 0x20;
        WAITALL :: 0x40;
        DONTWAIT :: 0x80;
        EOF :: 0x100;
        WAITSTREAM :: 0x200;
        FLUSH :: 0x400;
        HOLD :: 0x800;
        SEND :: 0x1000;
        HAVEMORE :: 0x2000;
        RCVMORE :: 0x4000;
        NEEDSA :: 0x10000;

        MSG_OOB :: OOB;
        MSG_PEEK :: PEEK;
        MSG_DONTROUTE :: DONTROUTE;
        MSG_EOR :: EOR;
        MSG_TRUNC :: TRUNC;
        MSG_CTRUNC :: CTRUNC;
        MSG_WAITALL :: WAITALL;
        MSG_DONTWAIT :: DONTWAIT;
        MSG_EOF :: EOF;
        MSG_WAITSTREAM :: WAITSTREAM;
        MSG_FLUSH :: FLUSH;
        MSG_HOLD :: HOLD;
        MSG_SEND :: SEND;
        MSG_HAVEMORE :: HAVEMORE;
        MSG_RCVMORE :: RCVMORE;
        MSG_NEEDSA :: NEEDSA;
    }

}

#if OS == .LINUX {
    msghdr :: struct {
        msg_name: *void;
        msg_namelen: socklen_t;

        msg_iov: *iovec;
        msg_iovlen: u64;

        msg_control: *void;
        msg_controllen: u64;

        msg_flags: s32;
    }

}

#if OS == .ANDROID {
    msghdr :: struct {
        msg_name: *void;
        msg_namelen: socklen_t;
        msg_iov: *iovec;
        msg_iovlen: u64;
        msg_control: *void;
        msg_controllen: u64;
        msg_flags: s32;
    }

}

#if OS == .MACOS {
    msghdr :: struct {
        msg_name: *void;
        msg_namelen: socklen_t;
        msg_iov: *iovec;
        msg_iovlen: s32;
        msg_control: *void;
        msg_controllen: socklen_t;
        msg_flags: s32;
    }

}

#if OS == .WINDOWS {
    MULTICAST_MODE_TYPE :: enum s32 {
        INCLUDE :: 0;
        EXCLUDE :: 1;

        MCAST_INCLUDE :: INCLUDE;
        MCAST_EXCLUDE :: EXCLUDE;
    }

}

#if OS == .MACOS {
    net_event_data :: struct {
        if_family: u32;
        if_unit: u32;
        if_name: [16] u8;
    }

}

#if OS == .WINDOWS {
    netent :: struct {
        n_name: *u8;
        n_aliases: **u8;
        n_addrtype: s16;
        n_net: u32;
    }

}

#if OS == .LINUX {
    netent :: struct {
        n_name: *u8;
        n_aliases: **u8;
        n_addrtype: s32;
        n_net: u32;
    }

}

#if OS == .ANDROID {
    netent :: struct {
        n_name: *u8;
        n_aliases: **u8;
        n_addrtype: s32;
        n_net: u32;
    }

}

#if OS == .MACOS {
    netent :: struct {
        n_name: *u8;
        n_aliases: **u8;
        n_addrtype: s32;
        n_net: u32;
    }

}

#if OS == .WINDOWS {
    PMTUD_STATE :: enum s32 {
        NOT_SET :: 0;
        DO :: 1;
        DONT :: 2;
        PROBE :: 3;
        MAX :: 4;

        IP_PMTUDISC_NOT_SET :: NOT_SET;
        IP_PMTUDISC_DO :: DO;
        IP_PMTUDISC_DONT :: DONT;
        IP_PMTUDISC_PROBE :: PROBE;
        IP_PMTUDISC_MAX :: MAX;
    }

}

#if OS == .WINDOWS {
    pollfd :: struct {
        fd: SOCKET;
        events: s16;
        revents: s16;
    }

}

#if OS == .WINDOWS {
    PRIORITY_STATUS :: struct {
        Sender: SOCKET_PRIORITY_HINT;
        Receiver: SOCKET_PRIORITY_HINT;
    }

}

#if OS == .WINDOWS {
    protoent :: struct {
        p_name: *u8;
        p_aliases: **u8;
        p_proto: s16;
    }

}

#if OS == .LINUX {
    protoent :: struct {
        p_name: *u8;
        p_aliases: **u8;
        p_proto: s32;
    }

}

#if OS == .ANDROID {
    protoent :: struct {
        p_name: *u8;
        p_aliases: **u8;
        p_proto: s32;
    }

}

#if OS == .MACOS {
    protoent :: struct {
        p_name: *u8;
        p_aliases: **u8;
        p_proto: s32;
    }

}

#if OS == .WINDOWS {
    QOS_OBJECT_HDR :: struct {
        ObjectType: u32;
        ObjectLength: u32;
    }

}

#if OS == .WINDOWS {
    QOS_SD_MODE :: struct {
        ObjectHdr: QOS_OBJECT_HDR;
        ShapeDiscardMode: u32;
    }

}

#if OS == .WINDOWS {
    QOS_SHAPING_RATE :: struct {
        ObjectHdr: QOS_OBJECT_HDR;
        ShapingRate: u32;
    }

}

#if OS == .WINDOWS {
    QualityOfService :: struct {
        SendingFlowspec: FLOWSPEC;
        ReceivingFlowspec: FLOWSPEC;
        ProviderSpecific: WSABUF;
    }

}

#if OS == .WINDOWS {
    RCVALL_IF :: struct {
        Mode: RCVALL_VALUE;
        Interface: u32;
    }

}

#if OS == .WINDOWS {
    RCVALL_VALUE :: enum s32 {
        OFF :: 0;
        ON :: 1;
        SOCKETLEVELONLY :: 2;
        IPLEVEL :: 3;

        RCVALL_OFF :: OFF;
        RCVALL_ON :: ON;
        RCVALL_SOCKETLEVELONLY :: SOCKETLEVELONLY;
        RCVALL_IPLEVEL :: IPLEVEL;
    }

}

#if OS == .WINDOWS {
    REAL_TIME_NOTIFICATION_SETTING_INPUT :: struct {
        TransportSettingId: TRANSPORT_SETTING_ID;
        BrokerEventGuid: GUID;
    }

}

#if OS == .WINDOWS {
    REAL_TIME_NOTIFICATION_SETTING_INPUT_EX :: struct {
        TransportSettingId: TRANSPORT_SETTING_ID;
        BrokerEventGuid: GUID;
        Unmark: BOOLEAN;
    }

}

#if OS == .WINDOWS {
    REAL_TIME_NOTIFICATION_SETTING_OUTPUT :: struct {
        ChannelStatus: CONTROL_CHANNEL_TRIGGER_STATUS;
    }

}

#if OS == .MACOS {
    rpcent :: struct {
        r_name: *u8;
        r_aliases: **u8;
        r_number: s32;
    }

}

#if OS == .MACOS {
    rslvmulti_req :: struct {
        sa: *sockaddr;
        llsa: **sockaddr;
    }

}

#if OS == .WINDOWS {
    RSS_SCALABILITY_INFO :: struct {
        RssEnabled: BOOLEAN;
    }

}

#if OS == .MACOS {
    sa_endpoints :: struct {
        sae_srcif: u32;
        sae_srcaddr: *sockaddr;
        sae_srcaddrlen: socklen_t;
        sae_dstaddr: *sockaddr;
        sae_dstaddrlen: socklen_t;
    }

}

#if OS == .LINUX {
    SCM :: enum u32 {
        RIGHTS :: 1;

        CREDENTIALS :: 2;

        SCM_WIFI_STATUS :: SO_WIFI_STATUS;

        SCM_TIMESTAMPING_OPT_STATS :: 54;

        SCM_TIMESTAMPING_PKTINFO :: 58;

        SCM_TXTIME :: SO_TXTIME;

        SCM_TIMESTAMP :: SO_TIMESTAMP;
        SCM_TIMESTAMPNS :: SO_TIMESTAMPNS;
        SCM_TIMESTAMPING :: SO_TIMESTAMPING;

        SCM_RIGHTS :: RIGHTS;

        SCM_CREDENTIALS :: CREDENTIALS;
    }

}

#if OS == .ANDROID {
    SCM :: enum u32 {
        WIFI_STATUS :: SO_WIFI_STATUS;
        TIMESTAMPING_OPT_STATS :: 54;
        TIMESTAMPING_PKTINFO :: 58;
        TXTIME :: SO_TXTIME;
        TIMESTAMP :: SO_TIMESTAMP;
        TIMESTAMPNS :: SO_TIMESTAMPNS;
        TIMESTAMPING :: SO_TIMESTAMPING;
        RIGHTS :: 0x01;
        CREDENTIALS :: 0x02;
        SECURITY :: 0x03;

        SCM_WIFI_STATUS :: WIFI_STATUS;
        SCM_TIMESTAMPING_OPT_STATS :: TIMESTAMPING_OPT_STATS;
        SCM_TIMESTAMPING_PKTINFO :: TIMESTAMPING_PKTINFO;
        SCM_TXTIME :: TXTIME;
        SCM_TIMESTAMP :: TIMESTAMP;
        SCM_TIMESTAMPNS :: TIMESTAMPNS;
        SCM_TIMESTAMPING :: TIMESTAMPING;
        SCM_RIGHTS :: RIGHTS;
        SCM_CREDENTIALS :: CREDENTIALS;
        SCM_SECURITY :: SECURITY;
    }

}

#if OS == .MACOS {
    SCM :: enum u32 {
        RIGHTS :: 0x01;
        TIMESTAMP :: 0x02;
        CREDS :: 0x03;
        TIMESTAMP_MONOTONIC :: 0x04;

        SCM_RIGHTS :: RIGHTS;
        SCM_TIMESTAMP :: TIMESTAMP;
        SCM_CREDS :: CREDS;
        SCM_TIMESTAMP_MONOTONIC :: TIMESTAMP_MONOTONIC;
    }

}

#if OS == .WINDOWS {
    SCOPE_ID :: struct {
        struct {
            using anon_struct: struct {
                __bitfield: u32;

            };

            #overlay (anon_struct) Value: u32;
        }
    }

}

#if OS == .WINDOWS {
    SCOPE_LEVEL :: enum s32 {
        Interface :: 1;
        Link :: 2;
        Subnet :: 3;
        Admin :: 4;
        Site :: 5;
        Organization :: 8;
        Global :: 14;
        Count :: 16;

        ScopeLevelInterface :: Interface;
        ScopeLevelLink :: Link;
        ScopeLevelSubnet :: Subnet;
        ScopeLevelAdmin :: Admin;
        ScopeLevelSite :: Site;
        ScopeLevelOrganization :: Organization;
        ScopeLevelGlobal :: Global;
        ScopeLevelCount :: Count;
    }

}

#if OS == .WINDOWS {
    servent :: struct {
        s_name: *u8;
        s_aliases: **u8;

        s_proto: *u8;
        s_port: s16;
    }

}

#if OS == .LINUX {
    servent :: struct {
        s_name: *u8;
        s_aliases: **u8;
        s_port: s32;
        s_proto: *u8;
    }

}

#if OS == .ANDROID {
    servent :: struct {
        s_name: *u8;
        s_aliases: **u8;
        s_port: s32;
        s_proto: *u8;
    }

}

#if OS == .MACOS {
    servent :: struct {
        s_name: *u8;
        s_aliases: **u8;
        s_port: s32;
        s_proto: *u8;
    }

}

#if OS == .MACOS {
    sf_hdtr :: struct {
        headers: *iovec;
        hdr_cnt: s32;
        trailers: *iovec;
        trl_cnt: s32;
    }

}

#if OS == .LINUX {
    SHUT :: enum u32 {
        RD :: 0;

        WR :: 1;

        RDWR :: 2;

        SHUT_RD :: RD;

        SHUT_WR :: WR;

        SHUT_RDWR :: RDWR;
    }

}

#if OS == .MACOS {
    SHUT :: enum u32 {
        RD :: 0;
        WR :: 1;
        RDWR :: 2;

        SHUT_RD :: RD;
        SHUT_WR :: WR;
        SHUT_RDWR :: RDWR;
    }

}

#if OS == .MACOS {
    so_np_extensions :: struct {
        npx_flags: u32;
        npx_mask: u32;
    }

}

#if OS == .WINDOWS {
    SOCK :: enum s32 {
        STREAM :: 1;
        DGRAM :: 2;
        RAW :: 3;
        RDM :: 4;
        SEQPACKET :: 5;

        SOCK_STREAM :: STREAM;
        SOCK_DGRAM :: DGRAM;
        SOCK_RAW :: RAW;
        SOCK_RDM :: RDM;
        SOCK_SEQPACKET :: SEQPACKET;
    }

}

#if OS == .LINUX {
    SOCK :: enum u32 {
        STREAM :: 1;
        DGRAM :: 2;
        RAW :: 3;
        RDM :: 4;
        SEQPACKET :: 5;
        DCCP :: 6;
        PACKET :: 10;
        CLOEXEC :: 524288;
        NONBLOCK :: 2048;

        SOCK_STREAM :: STREAM;
        SOCK_DGRAM :: DGRAM;
        SOCK_RAW :: RAW;
        SOCK_RDM :: RDM;
        SOCK_SEQPACKET :: SEQPACKET;
        SOCK_DCCP :: DCCP;
        SOCK_PACKET :: PACKET;
        SOCK_CLOEXEC :: CLOEXEC;
        SOCK_NONBLOCK :: NONBLOCK;
    }

}

#if OS == .ANDROID {
    SOCK :: enum u32 {
        STREAM :: 1;
        DGRAM :: 2;
        RAW :: 3;
        RDM :: 4;
        SEQPACKET :: 5;
        DCCP :: 6;
        PACKET :: 10;

        SOCK_STREAM :: STREAM;
        SOCK_DGRAM :: DGRAM;
        SOCK_RAW :: RAW;
        SOCK_RDM :: RDM;
        SOCK_SEQPACKET :: SEQPACKET;
        SOCK_DCCP :: DCCP;
        SOCK_PACKET :: PACKET;
    }

}

#if OS == .MACOS {
    SOCK :: enum u32 {
        STREAM :: 1;
        DGRAM :: 2;
        RAW :: 3;
        RDM :: 4;
        SEQPACKET :: 5;

        SOCK_STREAM :: STREAM;
        SOCK_DGRAM :: DGRAM;
        SOCK_RAW :: RAW;
        SOCK_RDM :: RDM;
        SOCK_SEQPACKET :: SEQPACKET;
    }

}

#if OS == .WINDOWS {
    SOCK_NOTIFY_REGISTRATION :: struct {
        socket: SOCKET;
        completionKey: *void;
        eventFilter: u16;
        operation: u8;
        triggerFlags: u8;
        registrationResult: u32;
    }

}

#if OS == .WINDOWS {
    sockaddr :: struct {
        sa_family: ADDRESS_FAMILY;

        sa_data: [14] u8;
    }

}

#if OS == .LINUX {
    sockaddr :: struct {
        sa_family: sa_family_t;
        sa_data: [14] u8;
    }

}

#if OS == .MACOS {
    sockaddr :: struct {
        sa_len: u8;
        sa_family: sa_family_t;
        sa_data: [14] u8;
    }

}

#if OS == .WINDOWS {
    sockaddr_dl :: struct {
        sdl_family: ADDRESS_FAMILY;
        sdl_data: [8] u8;
        sdl_zero: [4] u8;
    }

}

#if OS == .WINDOWS {
    sockaddr_gen :: struct {
        AddressIn6: sockaddr_in6_old;

        #overlay (AddressIn6) Address: sockaddr;
        #overlay (AddressIn6) AddressIn: sockaddr_in;
    }

}

#if OS == .WINDOWS {
    sockaddr_in :: struct {
        sin_family: ADDRESS_FAMILY;

        sin_port: u16;
        sin_addr: IN_ADDR;
        sin_zero: [8] u8;
    }

}

#if OS == .LINUX {
    sockaddr_in :: struct {
        sin_family: sa_family_t;
        sin_port: in_port_t;
        sin_addr: in_addr;

        sin_zero: [8] u8;
    }

}

#if OS == .ANDROID {
    sockaddr_in :: struct {
        sin_family: u16;
        sin_port: u16;
        sin_addr: in_addr;
        __pad: [8] u8;
    }

}

#if OS == .MACOS {
    sockaddr_in :: struct {
        sin_len: u8;
        sin_family: sa_family_t;
        sin_port: in_port_t;
        sin_addr: in_addr;
        sin_zero: [8] u8;
    }

}

#if OS == .WINDOWS {
    sockaddr_in6 :: struct {
        sin6_family: ADDRESS_FAMILY;
        sin6_port: u16;
        sin6_flowinfo: u32;
        sin6_addr: IN6_ADDR;
        struct {
            sin6_scope_id: u32;

            #overlay (sin6_scope_id) sin6_scope_struct: SCOPE_ID;
        }
    }

}

#if OS == .LINUX {
    sockaddr_in6 :: struct {
        sin6_family: sa_family_t;
        sin6_port: in_port_t;
        sin6_flowinfo: u32;
        sin6_addr: in6_addr;
        sin6_scope_id: u32;
    }

}

#if OS == .ANDROID {
    sockaddr_in6 :: struct {
        sin6_family: u16;
        sin6_port: u16;
        sin6_flowinfo: u32;
        sin6_addr: in6_addr;
        sin6_scope_id: u32;
    }

}

#if OS == .MACOS {
    sockaddr_in6 :: struct {
        sin6_len: u8;
        sin6_family: sa_family_t;
        sin6_port: in_port_t;
        sin6_flowinfo: u32;
        sin6_addr: in6_addr;
        sin6_scope_id: u32;
    }

}

#if OS == .WINDOWS {
    sockaddr_in6_old :: struct {
        sin6_family: s16;
        sin6_port: u16;
        sin6_flowinfo: u32;
        sin6_addr: IN6_ADDR;
    }

}

#if OS == .WINDOWS {
    sockaddr_in6_pair :: struct {
        SourceAddress: *SOCKADDR_IN6_LH;
        DestinationAddress: *SOCKADDR_IN6_LH;
    }

}

#if OS == .WINDOWS {
    sockaddr_in6_w2ksp1 :: struct {
        sin6_family: s16;
        sin6_port: u16;
        sin6_flowinfo: u32;
        sin6_addr: in6_addr;
        sin6_scope_id: u32;
    }

}

#if OS == .WINDOWS {
    SOCKADDR_INET :: struct {
        Ipv6: SOCKADDR_IN6;

        #overlay (Ipv6) Ipv4: SOCKADDR_IN;
        #overlay (Ipv6) si_family: ADDRESS_FAMILY;
    }

}

#if OS == .WINDOWS {
    sockaddr_storage :: struct {
        ss_family: ADDRESS_FAMILY;

        __ss_pad1: [6] u8;

        __ss_align: s64;
        __ss_pad2: [112] u8;
    }

}

#if OS == .LINUX {
    sockaddr_storage :: struct {
        ss_family: sa_family_t;
        __ss_padding: [118] u8;
        __ss_align: u64;
    }

}

#if OS == .ANDROID {
    sockaddr_storage :: struct {
        struct {
            using anon_struct: struct {
                ss_family: sa_family_t;
                __data: [126] u8;
            };

            #overlay (anon_struct) __align: *void;
        }
    }

}

#if OS == .MACOS {
    sockaddr_storage :: struct {
        ss_len: u8;
        ss_family: sa_family_t;
        __ss_pad1: [6] u8;
        __ss_align: s64;
        __ss_pad2: [112] u8;
    }

}

#if OS == .WINDOWS {
    sockaddr_storage_xp :: struct {
        ss_family: s16;

        __ss_pad1: [6] u8;

        __ss_align: s64;
        __ss_pad2: [112] u8;
    }

}

#if OS == .WINDOWS {
    SOCKET_ADDRESS :: struct {
        lpSockaddr: *sockaddr;

        iSockaddrLength: s32;
    }

}

#if OS == .WINDOWS {
    SOCKET_ADDRESS_LIST :: struct {
        iAddressCount: s32;
        Address: [1] SOCKET_ADDRESS;
    }

}

#if OS == .WINDOWS {
    SOCKET_PEER_TARGET_NAME :: struct {
        SecurityProtocol: SOCKET_SECURITY_PROTOCOL;
        PeerAddress: SOCKADDR_STORAGE;
        PeerTargetNameStringLen: u32;
        AllStrings: [0] s16;
    }

}

#if OS == .WINDOWS {
    SOCKET_PRIORITY_HINT :: enum s32 {
        PriorityHintVeryLow :: 0;
        PriorityHintLow :: 1;
        PriorityHintNormal :: 2;
        MaximumPriorityHintType :: 3;

        SocketPriorityHintVeryLow :: PriorityHintVeryLow;
        SocketPriorityHintLow :: PriorityHintLow;
        SocketPriorityHintNormal :: PriorityHintNormal;
        SocketMaximumPriorityHintType :: MaximumPriorityHintType;
    }

}

#if OS == .WINDOWS {
    SOCKET_SECURITY_PROTOCOL :: enum s32 {
        DEFAULT :: 0;
        IPSEC :: 1;

        IPSEC2 :: 2;

        INVALID :: 3;

        SOCKET_SECURITY_PROTOCOL_DEFAULT :: DEFAULT;
        SOCKET_SECURITY_PROTOCOL_IPSEC :: IPSEC;

        SOCKET_SECURITY_PROTOCOL_IPSEC2 :: IPSEC2;

        SOCKET_SECURITY_PROTOCOL_INVALID :: INVALID;
    }

}

#if OS == .WINDOWS {
    SOCKET_SECURITY_QUERY_INFO :: struct {
        SecurityProtocol: SOCKET_SECURITY_PROTOCOL;
        Flags: u32;
        PeerApplicationAccessTokenHandle: u64;
        PeerMachineAccessTokenHandle: u64;
    }

}

#if OS == .WINDOWS {
    SOCKET_SECURITY_QUERY_INFO_IPSEC2 :: struct {
        SecurityProtocol: SOCKET_SECURITY_PROTOCOL;
        Flags: u32;
        PeerApplicationAccessTokenHandle: u64;
        PeerMachineAccessTokenHandle: u64;
        MmSaId: u64;
        QmSaId: u64;
        NegotiationWinerr: u32;
        SaLookupContext: GUID;
    }

}

#if OS == .WINDOWS {
    SOCKET_SECURITY_QUERY_TEMPLATE :: struct {
        SecurityProtocol: SOCKET_SECURITY_PROTOCOL;
        PeerAddress: SOCKADDR_STORAGE;
        PeerTokenAccessMask: u32;
    }

}

#if OS == .WINDOWS {
    SOCKET_SECURITY_QUERY_TEMPLATE_IPSEC2 :: struct {
        SecurityProtocol: SOCKET_SECURITY_PROTOCOL;
        PeerAddress: SOCKADDR_STORAGE;
        PeerTokenAccessMask: u32;
        Flags: u32;
        FieldMask: u32;
    }

}

#if OS == .WINDOWS {
    SOCKET_SECURITY_SETTINGS :: struct {
        SecurityProtocol: SOCKET_SECURITY_PROTOCOL;
        SecurityFlags: u32;
    }

}

#if OS == .WINDOWS {
    SOCKET_SECURITY_SETTINGS_IPSEC :: struct {
        SecurityProtocol: SOCKET_SECURITY_PROTOCOL;
        SecurityFlags: u32;
        IpsecFlags: u32;
        AuthipMMPolicyKey: GUID;
        AuthipQMPolicyKey: GUID;
        Reserved: GUID;
        Reserved2: u64;
        UserNameStringLen: u32;
        DomainNameStringLen: u32;
        PasswordStringLen: u32;
        AllStrings: [0] s16;
    }

}

#if OS == .WINDOWS {
    SOCKET_USAGE_TYPE :: enum s32 {
        SYSTEM_CRITICAL_SOCKET :: 1;
    }

}

#if OS == .WINDOWS {
    sockproto :: struct {
        sp_family: u16;
        sp_protocol: u16;
    }

}

#if OS == .MACOS {
    sockproto :: struct {
        sp_family: u16;
        sp_protocol: u16;
    }

}

#if OS == .LINUX {
    TCP :: enum u32 {
        ESTABLISHED :: 1;
        SYN_SENT :: 2;
        SYN_RECV :: 3;
        FIN_WAIT1 :: 4;
        FIN_WAIT2 :: 5;
        TIME_WAIT :: 6;
        CLOSE :: 7;
        CLOSE_WAIT :: 8;
        LAST_ACK :: 9;
        LISTEN :: 10;
        CLOSING :: 11;

        TCP_ESTABLISHED :: ESTABLISHED;
        TCP_SYN_SENT :: SYN_SENT;
        TCP_SYN_RECV :: SYN_RECV;
        TCP_FIN_WAIT1 :: FIN_WAIT1;
        TCP_FIN_WAIT2 :: FIN_WAIT2;
        TCP_TIME_WAIT :: TIME_WAIT;
        TCP_CLOSE :: CLOSE;
        TCP_CLOSE_WAIT :: CLOSE_WAIT;
        TCP_LAST_ACK :: LAST_ACK;
        TCP_LISTEN :: LISTEN;
        TCP_CLOSING :: CLOSING;
    }

}

#if OS == .LINUX {
    TCP_1 :: enum u32 {
        NO_QUEUE :: 0;
        RECV_QUEUE :: 1;
        SEND_QUEUE :: 2;
        QUEUES_NR :: 3;

        TCP_NO_QUEUE :: NO_QUEUE;
        TCP_RECV_QUEUE :: RECV_QUEUE;
        TCP_SEND_QUEUE :: SEND_QUEUE;
        TCP_QUEUES_NR :: QUEUES_NR;
    }

}

#if OS == .ANDROID {
    TCP_2 :: enum u32 {
        ESTABLISHED :: 1;
        SYN_SENT :: 2;
        SYN_RECV :: 3;
        FIN_WAIT1 :: 4;
        FIN_WAIT2 :: 5;
        TIME_WAIT :: 6;
        CLOSE :: 7;
        CLOSE_WAIT :: 8;
        LAST_ACK :: 9;
        LISTEN :: 10;
        CLOSING :: 11;

        TCP_ESTABLISHED :: ESTABLISHED;
        TCP_SYN_SENT :: SYN_SENT;
        TCP_SYN_RECV :: SYN_RECV;
        TCP_FIN_WAIT1 :: FIN_WAIT1;
        TCP_FIN_WAIT2 :: FIN_WAIT2;
        TCP_TIME_WAIT :: TIME_WAIT;
        TCP_CLOSE :: CLOSE;
        TCP_CLOSE_WAIT :: CLOSE_WAIT;
        TCP_LAST_ACK :: LAST_ACK;
        TCP_LISTEN :: LISTEN;
        TCP_CLOSING :: CLOSING;
    }

}

#if OS == .WINDOWS {
    TCP_ACK_FREQUENCY_PARAMETERS :: struct {
        TcpDelayedAckFrequency: u8;
    }

}

#if OS == .LINUX {
    tcp_ca_state :: enum u32 {
        Open :: 0;
        Disorder :: 1;
        CWR :: 2;
        Recovery :: 3;
        Loss :: 4;

        TCP_CA_Open :: Open;
        TCP_CA_Disorder :: Disorder;
        TCP_CA_CWR :: CWR;
        TCP_CA_Recovery :: Recovery;
        TCP_CA_Loss :: Loss;
    }

}

#if OS == .MACOS {
    tcp_connection_info :: struct {
        tcpi_state: u8;
        tcpi_snd_wscale: u8;
        tcpi_rcv_wscale: u8;
        __pad1: u8;
        tcpi_options: u32;

        tcpi_flags: u32;

        tcpi_rto: u32;
        tcpi_maxseg: u32;
        tcpi_snd_ssthresh: u32;
        tcpi_snd_cwnd: u32;
        tcpi_snd_wnd: u32;
        tcpi_snd_sbbytes: u32;
        tcpi_rcv_wnd: u32;
        tcpi_rttcur: u32;
        tcpi_srtt: u32;
        tcpi_rttvar: u32;

        __bitfield: u32;

        tcpi_txpackets: u64;
        tcpi_txbytes: u64;
        tcpi_txretransmitbytes: u64;
        tcpi_rxpackets: u64;
        tcpi_rxbytes: u64;
        tcpi_rxoutoforderbytes: u64;
        tcpi_txretransmitpackets: u64;
    }

}

#if OS == .LINUX {
    tcp_cookie_transactions :: struct {
        tcpct_flags: u16;
        __tcpct_pad1: u8;
        tcpct_cookie_desired: u8;
        tcpct_s_data_desired: u16;
        tcpct_used: u16;
        tcpct_value: [536] u8;
    }

}

#if OS == .WINDOWS {
    TCP_ICW_LEVEL :: enum s32 {
        DEFAULT :: 0;
        HIGH :: 1;
        VERY_HIGH :: 2;
        AGGRESSIVE :: 3;
        EXPERIMENTAL :: 4;
        COMPAT :: 254;
        MAX :: 255;

        TCP_ICW_LEVEL_DEFAULT :: DEFAULT;
        TCP_ICW_LEVEL_HIGH :: HIGH;
        TCP_ICW_LEVEL_VERY_HIGH :: VERY_HIGH;
        TCP_ICW_LEVEL_AGGRESSIVE :: AGGRESSIVE;
        TCP_ICW_LEVEL_EXPERIMENTAL :: EXPERIMENTAL;
        TCP_ICW_LEVEL_COMPAT :: COMPAT;
        TCP_ICW_LEVEL_MAX :: MAX;
    }

}

#if OS == .WINDOWS {
    TCP_ICW_PARAMETERS :: struct {
        Level: TCP_ICW_LEVEL;
    }

}

#if OS == .LINUX {
    tcp_info :: struct {
        tcpi_state: u8;
        tcpi_ca_state: u8;
        tcpi_retransmits: u8;
        tcpi_probes: u8;
        tcpi_backoff: u8;
        tcpi_options: u8;
        __bitfield: u8;

        tcpi_rto: u32;
        tcpi_ato: u32;
        tcpi_snd_mss: u32;
        tcpi_rcv_mss: u32;

        tcpi_unacked: u32;
        tcpi_sacked: u32;
        tcpi_lost: u32;
        tcpi_retrans: u32;
        tcpi_fackets: u32;

        tcpi_last_data_sent: u32;
        tcpi_last_ack_sent: u32;
        tcpi_last_data_recv: u32;
        tcpi_last_ack_recv: u32;

        tcpi_pmtu: u32;
        tcpi_rcv_ssthresh: u32;
        tcpi_rtt: u32;
        tcpi_rttvar: u32;
        tcpi_snd_ssthresh: u32;
        tcpi_snd_cwnd: u32;
        tcpi_advmss: u32;
        tcpi_reordering: u32;

        tcpi_rcv_rtt: u32;
        tcpi_rcv_space: u32;

        tcpi_total_retrans: u32;
    }

}

#if OS == .WINDOWS {
    TCP_INFO_v0 :: struct {
        State: TCPSTATE;
        Mss: u32;
        ConnectionTimeMs: u64;
        TimestampsEnabled: BOOLEAN;
        RttUs: u32;
        MinRttUs: u32;
        BytesInFlight: u32;
        Cwnd: u32;
        SndWnd: u32;
        RcvWnd: u32;
        RcvBuf: u32;
        BytesOut: u64;
        BytesIn: u64;
        BytesReordered: u32;
        BytesRetrans: u32;
        FastRetrans: u32;
        DupAcksIn: u32;
        TimeoutEpisodes: u32;
        SynRetrans: u8;
    }

}

#if OS == .WINDOWS {
    TCP_INFO_v1 :: struct {
        State: TCPSTATE;
        Mss: u32;
        ConnectionTimeMs: u64;
        TimestampsEnabled: BOOLEAN;
        RttUs: u32;
        MinRttUs: u32;
        BytesInFlight: u32;
        Cwnd: u32;
        SndWnd: u32;
        RcvWnd: u32;
        RcvBuf: u32;
        BytesOut: u64;
        BytesIn: u64;
        BytesReordered: u32;
        BytesRetrans: u32;
        FastRetrans: u32;
        DupAcksIn: u32;
        TimeoutEpisodes: u32;
        SynRetrans: u8;

        SndLimTransRwin: u32;
        SndLimTimeRwin: u32;
        SndLimBytesRwin: u64;
        SndLimTransCwnd: u32;
        SndLimTimeCwnd: u32;
        SndLimBytesCwnd: u64;
        SndLimTransSnd: u32;
        SndLimTimeSnd: u32;
        SndLimBytesSnd: u64;
    }

}

#if OS == .WINDOWS {
    TCP_INFO_v2 :: struct {
        State: TCPSTATE;
        Mss: u32;
        ConnectionTimeMs: u64;
        TimestampsEnabled: BOOLEAN;
        RttUs: u32;
        MinRttUs: u32;
        BytesInFlight: u32;
        Cwnd: u32;
        SndWnd: u32;
        RcvWnd: u32;
        RcvBuf: u32;
        BytesOut: u64;
        BytesIn: u64;

        BytesReordered: u32;
        BytesRetrans: u32;
        FastRetrans: u32;
        DupAcksIn: u32;
        TimeoutEpisodes: u32;
        SynRetrans: u8;

        SndLimTransRwin: u32;
        SndLimTimeRwin: u32;
        SndLimBytesRwin: u64;
        SndLimTransCwnd: u32;
        SndLimTimeCwnd: u32;
        SndLimBytesCwnd: u64;
        SndLimTransSnd: u32;
        SndLimTimeSnd: u32;
        SndLimBytesSnd: u64;

        OutOfOrderPktsIn: u32;

        EcnNegotiated: BOOLEAN;
        EceAcksIn: u32;
        PtoEpisodes: u32;
    }

}

#if OS == .WINDOWS {
    TCP_INITIAL_RTO_PARAMETERS :: struct {
        Rtt: u16;

        MaxSynRetransmissions: u8;
    }

}

#if OS == .WINDOWS {
    tcp_keepalive :: struct {
        onoff: u32;
        keepalivetime: u32;
        keepaliveinterval: u32;
    }

}

#if OS == .LINUX {
    tcp_md5sig :: struct {
        tcpm_addr: sockaddr_storage;
        tcpm_flags: u8;
        tcpm_prefixlen: u8;
        tcpm_keylen: u16;
        __tcpm_pad: u32;
        tcpm_key: [80] u8;
    }

}

#if OS == .LINUX {
    tcp_repair_opt :: struct {
        opt_code: u32;
        opt_val: u32;
    }

}

#if OS == .LINUX {
    tcp_repair_window :: struct {
        snd_wl1: u32;
        snd_wnd: u32;
        max_window: u32;
        rcv_wnd: u32;
        rcv_wup: u32;
    }

}

#if OS == .LINUX {
    tcp_zerocopy_receive :: struct {
        address: u64;
        length: u32;
        recv_skip_hint: u32;
    }

}

#if OS == .LINUX {
    tcphdr :: struct {
        struct {
            using anon_struct: struct {
                th_sport: u16;
                th_dport: u16;
                th_seq: tcp_seq;
                th_ack: tcp_seq;

                __bitfield: u8;

                th_flags: u8;

                th_win: u16;
                th_sum: u16;
                th_urp: u16;
            };

            #overlay (anon_struct) using anon_struct_1: struct {
                source: u16;
                dest: u16;
                seq: u32;
                ack_seq: u32;

                __bitfield_1: u16;

                window: u16;
                check: u16;
                urg_ptr: u16;
            };
        }
    }

}

#if OS == .ANDROID {
    tcphdr :: struct {
        struct {
            using anon_struct: struct {
                th_sport: u16;
                th_dport: u16;
                th_seq: u32;
                th_ack: u32;
                __bitfield: u8;

                th_flags: u8;
                th_win: u16;
                th_sum: u16;
                th_urp: u16;
            };

            #overlay (anon_struct) using anon_struct_1: struct {
                source: u16;
                dest: u16;
                seq: u32;
                ack_seq: u32;
                __bitfield_1: u16;

                window: u16;
                check: u16;
                urg_ptr: u16;
            };
        }
    }

}

#if OS == .MACOS {
    tcphdr :: struct {
        th_sport: u16;
        th_dport: u16;
        th_seq: tcp_seq;
        th_ack: tcp_seq;

        __bitfield: u8;

        th_flags: u8;

        th_win: u16;
        th_sum: u16;
        th_urp: u16;
    }

}

#if OS == .WINDOWS {
    TCPSTATE :: enum s32 {
        CLOSED :: 0;
        LISTEN :: 1;
        SYN_SENT :: 2;
        SYN_RCVD :: 3;
        ESTABLISHED :: 4;
        FIN_WAIT_1 :: 5;
        FIN_WAIT_2 :: 6;
        CLOSE_WAIT :: 7;
        CLOSING :: 8;
        LAST_ACK :: 9;
        TIME_WAIT :: 10;
        MAX :: 11;

        TCPSTATE_CLOSED :: CLOSED;
        TCPSTATE_LISTEN :: LISTEN;
        TCPSTATE_SYN_SENT :: SYN_SENT;
        TCPSTATE_SYN_RCVD :: SYN_RCVD;
        TCPSTATE_ESTABLISHED :: ESTABLISHED;
        TCPSTATE_FIN_WAIT_1 :: FIN_WAIT_1;
        TCPSTATE_FIN_WAIT_2 :: FIN_WAIT_2;
        TCPSTATE_CLOSE_WAIT :: CLOSE_WAIT;
        TCPSTATE_CLOSING :: CLOSING;
        TCPSTATE_LAST_ACK :: LAST_ACK;
        TCPSTATE_TIME_WAIT :: TIME_WAIT;
        TCPSTATE_MAX :: MAX;
    }

}

#if OS == .WINDOWS {
    TIMESTAMPING_CONFIG :: struct {
        Flags: u32;
        TxTimestampsBuffered: u16;
    }

}

#if OS == .WINDOWS {
    timeval :: struct {
        tv_sec: s32;
        tv_usec: s32;
    }

}

#if OS == .MACOS {
    timeval32 :: struct {
        tv_sec: s32;
        tv_usec: s32;
    }

}

#if OS == .WINDOWS {
    TRANSPORT_SETTING_ID :: struct {
        Guid: GUID;
    }

}

#if OS == .LINUX {
    ucred :: struct {
        pid: pid_t;
        uid: uid_t;
        gid: gid_t;
    }

}

#if OS == .ANDROID {
    ucred :: struct {
        pid: pid_t;
        uid: uid_t;
        gid: gid_t;
    }

}

#if OS == .WINDOWS {
    WSABUF :: struct {
        len: u32;
        buf: *u8;
    }

}

#if OS == .WINDOWS {
    WSACOMPLETION :: struct {
        Type: WSACOMPLETIONTYPE;
        Parameters: struct {
            WindowMessage: struct {
                hWnd: HWND;
                uMsg: u32;
                _context: WPARAM;
            };

            #overlay (WindowMessage) Event: struct {
                lpOverlapped: *OVERLAPPED;
            };
            #overlay (WindowMessage) Apc: struct {
                lpOverlapped: *OVERLAPPED;
                lpfnCompletionProc: LPWSAOVERLAPPED_COMPLETION_ROUTINE;
            };
            #overlay (WindowMessage) Port: struct {
                lpOverlapped: *OVERLAPPED;
                hPort: HANDLE;
                Key: u64;
            };
        };
    }

}

#if OS == .WINDOWS {
    WSACOMPLETIONTYPE :: enum s32 {
        IMMEDIATELY :: 0;
        HWND :: 1;
        EVENT :: 2;
        PORT :: 3;
        APC :: 4;

        NSP_NOTIFY_IMMEDIATELY :: IMMEDIATELY;
        NSP_NOTIFY_HWND :: HWND;
        NSP_NOTIFY_EVENT :: EVENT;
        NSP_NOTIFY_PORT :: PORT;
        NSP_NOTIFY_APC :: APC;
    }

}

#if OS == .WINDOWS {
    WSAData :: struct {
        wVersion: u16;
        wHighVersion: u16;

        iMaxSockets: u16;
        iMaxUdpDg: u16;
        lpVendorInfo: *u8;
        szDescription: [257] u8;
        szSystemStatus: [129] u8;
    }

}

#if OS == .WINDOWS {
    WSAEcomparator :: enum s32 {
        EQUAL :: 0;
        NOTLESS :: 1;

        COMP_EQUAL :: EQUAL;
        COMP_NOTLESS :: NOTLESS;
    }

}

#if OS == .WINDOWS {
    WSAESETSERVICEOP :: enum s32 {
        REGISTER :: 0;
        DEREGISTER :: 1;
        DELETE :: 2;

        RNRSERVICE_REGISTER :: REGISTER;
        RNRSERVICE_DEREGISTER :: DEREGISTER;
        RNRSERVICE_DELETE :: DELETE;
    }

}

#if OS == .WINDOWS {
    WSAMSG :: struct {
        name: *sockaddr;
        namelen: s32;
        lpBuffers: *WSABUF;

        dwBufferCount: u32;

        Control: WSABUF;

        dwFlags: u32;
    }

}

#if OS == .WINDOWS {
    WSANETWORKEVENTS :: struct {
        lNetworkEvents: s32;
        iErrorCode: [10] s32;
    }

}

#if OS == .WINDOWS {
    WSANSClassInfoA :: struct {
        lpszName: *u8;
        dwNameSpace: u32;
        dwValueType: u32;
        dwValueSize: u32;
        lpValue: *void;
    }

}

#if OS == .WINDOWS {
    WSANSClassInfoW :: struct {
        lpszName: *s16;
        dwNameSpace: u32;
        dwValueType: u32;
        dwValueSize: u32;
        lpValue: *void;
    }

}

#if OS == .WINDOWS {
    WSAPROTOCOL_INFOA :: struct {
        dwServiceFlags1: u32;
        dwServiceFlags2: u32;
        dwServiceFlags3: u32;
        dwServiceFlags4: u32;
        dwProviderFlags: u32;
        ProviderId: GUID;
        dwCatalogEntryId: u32;
        ProtocolChain: WSAPROTOCOLCHAIN;
        iVersion: s32;
        iAddressFamily: s32;
        iMaxSockAddr: s32;
        iMinSockAddr: s32;
        iSocketType: s32;
        iProtocol: s32;
        iProtocolMaxOffset: s32;
        iNetworkByteOrder: s32;
        iSecurityScheme: s32;
        dwMessageSize: u32;
        dwProviderReserved: u32;
        szProtocol: [256] u8;
    }

}

#if OS == .WINDOWS {
    WSAPROTOCOL_INFOW :: struct {
        dwServiceFlags1: u32;
        dwServiceFlags2: u32;
        dwServiceFlags3: u32;
        dwServiceFlags4: u32;
        dwProviderFlags: u32;
        ProviderId: GUID;
        dwCatalogEntryId: u32;
        ProtocolChain: WSAPROTOCOLCHAIN;
        iVersion: s32;
        iAddressFamily: s32;
        iMaxSockAddr: s32;
        iMinSockAddr: s32;
        iSocketType: s32;
        iProtocol: s32;
        iProtocolMaxOffset: s32;
        iNetworkByteOrder: s32;
        iSecurityScheme: s32;
        dwMessageSize: u32;
        dwProviderReserved: u32;
        szProtocol: [256] s16;
    }

}

#if OS == .WINDOWS {
    WSAPROTOCOLCHAIN :: struct {
        ChainLen: s32;

        ChainEntries: [7] u32;
    }

}

#if OS == .WINDOWS {
    WSAServiceClassInfoA :: struct {
        lpServiceClassId: *GUID;
        lpszServiceClassName: *u8;
        dwCount: u32;
        lpClassInfos: *WSANSClassInfoA;
    }

}

#if OS == .WINDOWS {
    WSAServiceClassInfoW :: struct {
        lpServiceClassId: *GUID;
        lpszServiceClassName: *s16;
        dwCount: u32;
        lpClassInfos: *WSANSClassInfoW;
    }

}

#if OS == .WINDOWS {
    WSAVersion :: struct {
        dwVersion: u32;
        ecHow: WSAECOMPARATOR;
    }

}

#if OS == .LINUX {
    __cmsg_nxthdr :: (__mhdr: *msghdr, __cmsg: *cmsghdr) -> *cmsghdr #foreign libc;
}

#if OS == .ANDROID {
    __cmsg_nxthdr :: (__msg: *msghdr, __cmsg: *cmsghdr) -> *cmsghdr #foreign libc;
}

#if OS == .ANDROID {
    __FD_CLR_chk :: (unknown0: s32, unknown1: *fd_set, unknown2: u64) -> void #foreign libc;
}

#if OS == .ANDROID {
    __FD_ISSET_chk :: (unknown0: s32, unknown1: *fd_set, unknown2: u64) -> s32 #foreign libc;
}

#if OS == .ANDROID {
    __FD_SET_chk :: (unknown0: s32, unknown1: *fd_set, unknown2: u64) -> void #foreign libc;
}

#if OS == .ANDROID {
    __get_h_errno :: () -> *s32 #foreign libc;
}

#if OS == .LINUX {
    __h_errno_location :: () -> *s32 #foreign libc;
}

#if OS == .WINDOWS {
    __WSAFDIsSet :: (fd: SOCKET, unknown0: *fd_set) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    accept :: (s: SOCKET, addr: *sockaddr, addrlen: *s32) -> SOCKET #foreign ws2_32;
}

#if OS == .LINUX {
    accept :: (__fd: s32, __addr: *sockaddr, __addr_len: *socklen_t) -> s32 #foreign libc;
}

#if OS == .ANDROID {
    accept :: (__fd: s32, __addr: *sockaddr, __addr_length: *socklen_t) -> s32 #foreign libc;
}

#if OS == .MACOS {
    accept :: (unknown0: s32, unknown1: *sockaddr, unknown2: *socklen_t) -> s32 #foreign libc;
}

#if OS == .LINUX {
    accept4 :: (__fd: s32, __addr: *sockaddr, __addr_len: *socklen_t, __flags: SOCK) -> s32 #foreign libc;
}

#if OS == .ANDROID {
    accept4 :: (__fd: s32, __addr: *sockaddr, __addr_length: *socklen_t, __flags: SOCK) -> s32 #foreign libc;
}

#if OS == .MACOS {
    addr2ascii :: (unknown0: s32, unknown1: *void, unknown2: s32, unknown3: *u8) -> *u8 #foreign libc;
}

#if OS == .MACOS {
    ascii2addr :: (unknown0: s32, unknown1: *u8, unknown2: *void) -> s32 #foreign libc;
}

#if OS == .WINDOWS {
    bind :: (s: SOCKET, name: *sockaddr, namelen: s32) -> s32 #foreign ws2_32;
}

#if OS == .LINUX {
    bind :: (__fd: s32, __addr: *sockaddr, __len: socklen_t) -> s32 #foreign libc;
}

#if OS == .ANDROID {
    bind :: (__fd: s32, __addr: *sockaddr, __addr_length: socklen_t) -> s32 #foreign libc;
}

#if OS == .MACOS {
    bind :: (unknown0: s32, unknown1: *sockaddr, unknown2: socklen_t) -> s32 #foreign libc;
}

#if OS == .LINUX {
    bindresvport :: (__sockfd: s32, __sock_in: *sockaddr_in) -> s32 #foreign libc;
}

#if OS == .ANDROID {
    bindresvport :: (__fd: s32, __sin: *sockaddr_in) -> s32 #foreign libc;
}

#if OS == .MACOS {
    bindresvport :: (unknown0: s32, unknown1: *sockaddr_in) -> s32 #foreign libc;
}

#if OS == .MACOS {
    bindresvport_sa :: (unknown0: s32, unknown1: *sockaddr) -> s32 #foreign libc;
}

#if OS == .WINDOWS {
    closesocket :: (s: SOCKET) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    connect :: (s: SOCKET, name: *sockaddr, namelen: s32) -> s32 #foreign ws2_32;
}

#if OS == .LINUX {
    connect :: (__fd: s32, __addr: *sockaddr, __len: socklen_t) -> s32 #foreign libc;
}

#if OS == .ANDROID {
    connect :: (__fd: s32, __addr: *sockaddr, __addr_length: socklen_t) -> s32 #foreign libc;
}

#if OS == .MACOS {
    connect :: (unknown0: s32, unknown1: *sockaddr, unknown2: socklen_t) -> s32 #foreign libc;
}

#if OS == .MACOS {
    connectx :: (unknown0: s32, unknown1: *sa_endpoints_t, unknown2: sae_associd_t, unknown3: u32, unknown4: *iovec, unknown5: u32, unknown6: *u64, unknown7: *sae_connid_t) -> s32 #foreign libc;
}

#if OS == .MACOS {
    disconnectx :: (unknown0: s32, unknown1: sae_associd_t, unknown2: sae_connid_t) -> s32 #foreign libc;
}

#if OS == .LINUX {
    endhostent :: () -> void #foreign libc;
}

#if OS == .ANDROID {
    endhostent :: () -> void #foreign libc;
}

#if OS == .MACOS {
    endhostent :: () -> void #foreign libc;
}

#if OS == .LINUX {
    endnetent :: () -> void #foreign libc;
}

#if OS == .ANDROID {
    endnetent :: () -> void #foreign libc;
}

#if OS == .MACOS {
    endnetent :: () -> void #foreign libc;
}

#if OS == .LINUX {
    endnetgrent :: () -> void #foreign libc;
}

#if OS == .MACOS {
    endnetgrent :: () -> void #foreign libc;
}

#if OS == .LINUX {
    endprotoent :: () -> void #foreign libc;
}

#if OS == .ANDROID {
    endprotoent :: () -> void #foreign libc;
}

#if OS == .MACOS {
    endprotoent :: () -> void #foreign libc;
}

#if OS == .MACOS {
    endrpcent :: () -> void #foreign libc;
}

#if OS == .LINUX {
    endservent :: () -> void #foreign libc;
}

#if OS == .ANDROID {
    endservent :: () -> void #foreign libc;
}

#if OS == .MACOS {
    endservent :: () -> void #foreign libc;
}

#if OS == .WINDOWS {
    freeaddrinfo :: (pAddrInfo: *addrinfo) -> void #foreign ws2_32;
}

#if OS == .LINUX {
    freeaddrinfo :: (__ai: *addrinfo) -> void #foreign libc;
}

#if OS == .ANDROID {
    freeaddrinfo :: (__ptr: *addrinfo) -> void #foreign libc;
}

#if OS == .MACOS {
    freeaddrinfo :: (unknown0: *addrinfo) -> void #foreign libc;
}

#if OS == .WINDOWS {
    FreeAddrInfoEx :: (pAddrInfoEx: *addrinfoexA) -> void #foreign ws2_32;
}

#if OS == .WINDOWS {
    FreeAddrInfoExW :: (pAddrInfoEx: *ADDRINFOEXW) -> void #foreign ws2_32;
}

#if OS == .WINDOWS {
    FreeAddrInfoW :: (pAddrInfo: *ADDRINFOW) -> void #foreign ws2_32;
}

#if OS == .MACOS {
    freehostent :: (unknown0: *hostent) -> void #foreign libc;
}

#if OS == .LINUX {
    freeifaddrs :: (__ifa: *ifaddrs) -> void #foreign libc;
}

#if OS == .ANDROID {
    freeifaddrs :: (__ptr: *ifaddrs) -> void #foreign libc;
}

#if OS == .MACOS {
    freeifaddrs :: (unknown0: *ifaddrs) -> void #foreign libc;
}

#if OS == .MACOS {
    freeifmaddrs :: (unknown0: *ifmaddrs) -> void #foreign libc;
}

#if OS == .LINUX {
    gai_strerror :: (__ecode: s32) -> *u8 #foreign libc;
}

#if OS == .ANDROID {
    gai_strerror :: (__error: s32) -> *u8 #foreign libc;
}

#if OS == .MACOS {
    gai_strerror :: (unknown0: s32) -> *u8 #foreign libc;
}

#if OS == .WINDOWS {
    getaddrinfo :: (pNodeName: *u8, pServiceName: *u8, pHints: *ADDRINFOA, ppResult: **addrinfo) -> s32 #foreign ws2_32;
}

#if OS == .LINUX {
    getaddrinfo :: (__name: *u8, __service: *u8, __req: *addrinfo, __pai: **addrinfo) -> s32 #foreign libc;
}

#if OS == .ANDROID {
    getaddrinfo :: (__node: *u8, __service: *u8, __hints: *addrinfo, __result: **addrinfo) -> s32 #foreign libc;
}

#if OS == .MACOS {
    getaddrinfo :: (unknown0: *u8, unknown1: *u8, unknown2: *addrinfo, unknown3: **addrinfo) -> s32 #foreign libc;
}

#if OS == .WINDOWS {
    GetAddrInfoExA :: (pName: *u8, pServiceName: *u8, dwNameSpace: u32, lpNspId: *GUID, hints: *ADDRINFOEXA, ppResult: **addrinfoexA, timeout: *TIMEVAL, lpOverlapped: *OVERLAPPED, lpCompletionRoutine: LPLOOKUPSERVICE_COMPLETION_ROUTINE, lpNameHandle: *HANDLE) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    GetAddrInfoExCancel :: (lpHandle: *HANDLE) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    GetAddrInfoExOverlappedResult :: (lpOverlapped: *OVERLAPPED) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    GetAddrInfoExW :: (pName: *s16, pServiceName: *s16, dwNameSpace: u32, lpNspId: *GUID, hints: *ADDRINFOEXW, ppResult: **ADDRINFOEXW, timeout: *TIMEVAL, lpOverlapped: *OVERLAPPED, lpCompletionRoutine: LPLOOKUPSERVICE_COMPLETION_ROUTINE, lpHandle: *HANDLE) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    GetAddrInfoW :: (pNodeName: *s16, pServiceName: *s16, pHints: *ADDRINFOW, ppResult: **ADDRINFOW) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    gethostbyaddr :: (addr: *u8, len: s32, type: s32) -> *hostent #foreign ws2_32;
}

#if OS == .LINUX {
    gethostbyaddr :: (__addr: *void, __len: u32, __type: s32) -> *hostent #foreign libc;
}

#if OS == .ANDROID {
    gethostbyaddr :: (__addr: *void, __length: socklen_t, __type: s32) -> *hostent #foreign libc;
}

#if OS == .MACOS {
    gethostbyaddr :: (unknown0: *void, unknown1: socklen_t, unknown2: s32) -> *hostent #foreign libc;
}

#if OS == .LINUX {
    gethostbyaddr_r :: (__addr: *void, __len: u32, __type: s32, __result_buf: *hostent, __buf: *u8, __buflen: u64, __result: **hostent, __h_errnop: *s32) -> s32 #foreign libc;
}

#if OS == .ANDROID {
    gethostbyaddr_r :: (__addr: *void, __length: socklen_t, __type: s32, __ret: *hostent, __buf: *u8, __buf_size: u64, __result: **hostent, __h_errno_ptr: *s32) -> s32 #foreign libc;
}

#if OS == .WINDOWS {
    gethostbyname :: (name: *u8) -> *hostent #foreign ws2_32;
}

#if OS == .LINUX {
    gethostbyname :: (__name: *u8) -> *hostent #foreign libc;
}

#if OS == .ANDROID {
    gethostbyname :: (__name: *u8) -> *hostent #foreign libc;
}

#if OS == .MACOS {
    gethostbyname :: (unknown0: *u8) -> *hostent #foreign libc;
}

#if OS == .LINUX {
    gethostbyname2 :: (__name: *u8, __af: s32) -> *hostent #foreign libc;
}

#if OS == .ANDROID {
    gethostbyname2 :: (__name: *u8, __af: s32) -> *hostent #foreign libc;
}

#if OS == .MACOS {
    gethostbyname2 :: (unknown0: *u8, unknown1: s32) -> *hostent #foreign libc;
}

#if OS == .LINUX {
    gethostbyname2_r :: (__name: *u8, __af: s32, __result_buf: *hostent, __buf: *u8, __buflen: u64, __result: **hostent, __h_errnop: *s32) -> s32 #foreign libc;
}

#if OS == .ANDROID {
    gethostbyname2_r :: (__name: *u8, __af: s32, __ret: *hostent, __buf: *u8, __buf_size: u64, __result: **hostent, __h_errno_ptr: *s32) -> s32 #foreign libc;
}

#if OS == .LINUX {
    gethostbyname_r :: (__name: *u8, __result_buf: *hostent, __buf: *u8, __buflen: u64, __result: **hostent, __h_errnop: *s32) -> s32 #foreign libc;
}

#if OS == .ANDROID {
    gethostbyname_r :: (__name: *u8, __ret: *hostent, __buf: *u8, __buf_size: u64, __result: **hostent, __h_errno_ptr: *s32) -> s32 #foreign libc;
}

#if OS == .LINUX {
    gethostent :: () -> *hostent #foreign libc;
}

#if OS == .ANDROID {
    gethostent :: () -> *hostent #foreign libc;
}

#if OS == .MACOS {
    gethostent :: () -> *hostent #foreign libc;
}

#if OS == .LINUX {
    gethostent_r :: (__result_buf: *hostent, __buf: *u8, __buflen: u64, __result: **hostent, __h_errnop: *s32) -> s32 #foreign libc;
}

#if OS == .WINDOWS {
    gethostname :: (name: *u8, namelen: s32) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    GetHostNameW :: (name: *s16, namelen: s32) -> s32 #foreign ws2_32;
}

#if OS == .LINUX {
    getifaddrs :: (__ifap: **ifaddrs) -> s32 #foreign libc;
}

#if OS == .ANDROID {
    getifaddrs :: (__list_ptr: **ifaddrs) -> s32 #foreign libc;
}

#if OS == .MACOS {
    getifaddrs :: (unknown0: **ifaddrs) -> s32 #foreign libc;
}

#if OS == .MACOS {
    getifmaddrs :: (unknown0: **ifmaddrs) -> s32 #foreign libc;
}

#if OS == .MACOS {
    getipnodebyaddr :: (unknown0: *void, unknown1: u64, unknown2: s32, unknown3: *s32) -> *hostent #foreign libc;
}

#if OS == .MACOS {
    getipnodebyname :: (unknown0: *u8, unknown1: s32, unknown2: s32, unknown3: *s32) -> *hostent #foreign libc;
}

#if OS == .LINUX {
    getipv4sourcefilter :: (__s: s32, __interface_addr: in_addr, __group: in_addr, __fmode: *u32, __numsrc: *u32, __slist: *in_addr) -> s32 #foreign libc;
}

#if OS == .WINDOWS {
    getnameinfo :: (pSockaddr: *SOCKADDR, SockaddrLength: socklen_t, pNodeBuffer: *u8, NodeBufferSize: u32, pServiceBuffer: *u8, ServiceBufferSize: u32, Flags: s32) -> s32 #foreign ws2_32;
}

#if OS == .LINUX {
    getnameinfo :: (__sa: *sockaddr, __salen: socklen_t, __host: *u8, __hostlen: socklen_t, __serv: *u8, __servlen: socklen_t, __flags: s32) -> s32 #foreign libc;
}

#if OS == .ANDROID {
    getnameinfo :: (__sa: *sockaddr, __sa_length: socklen_t, __host: *u8, __host_length: u64, __service: *u8, __service_length: u64, __flags: s32) -> s32 #foreign libc;
}

#if OS == .MACOS {
    getnameinfo :: (unknown0: *sockaddr, unknown1: socklen_t, unknown2: *u8, unknown3: socklen_t, unknown4: *u8, unknown5: socklen_t, unknown6: s32) -> s32 #foreign libc;
}

#if OS == .WINDOWS {
    GetNameInfoW :: (pSockaddr: *SOCKADDR, SockaddrLength: socklen_t, pNodeBuffer: *s16, NodeBufferSize: u32, pServiceBuffer: *s16, ServiceBufferSize: u32, Flags: s32) -> s32 #foreign ws2_32;
}

#if OS == .LINUX {
    getnetbyaddr :: (__net: u32, __type: s32) -> *netent #foreign libc;
}

#if OS == .ANDROID {
    getnetbyaddr :: (__net: u32, __type: s32) -> *netent #foreign libc;
}

#if OS == .MACOS {
    getnetbyaddr :: (unknown0: u32, unknown1: s32) -> *netent #foreign libc;
}

#if OS == .LINUX {
    getnetbyaddr_r :: (__net: u32, __type: s32, __result_buf: *netent, __buf: *u8, __buflen: u64, __result: **netent, __h_errnop: *s32) -> s32 #foreign libc;
}

#if OS == .LINUX {
    getnetbyname :: (__name: *u8) -> *netent #foreign libc;
}

#if OS == .ANDROID {
    getnetbyname :: (__name: *u8) -> *netent #foreign libc;
}

#if OS == .MACOS {
    getnetbyname :: (unknown0: *u8) -> *netent #foreign libc;
}

#if OS == .LINUX {
    getnetbyname_r :: (__name: *u8, __result_buf: *netent, __buf: *u8, __buflen: u64, __result: **netent, __h_errnop: *s32) -> s32 #foreign libc;
}

#if OS == .LINUX {
    getnetent :: () -> *netent #foreign libc;
}

#if OS == .ANDROID {
    getnetent :: () -> *netent #foreign libc;
}

#if OS == .MACOS {
    getnetent :: () -> *netent #foreign libc;
}

#if OS == .LINUX {
    getnetent_r :: (__result_buf: *netent, __buf: *u8, __buflen: u64, __result: **netent, __h_errnop: *s32) -> s32 #foreign libc;
}

#if OS == .LINUX {
    getnetgrent :: (__hostp: **u8, __userp: **u8, __domainp: **u8) -> s32 #foreign libc;
}

#if OS == .MACOS {
    getnetgrent :: (unknown0: **u8, unknown1: **u8, unknown2: **u8) -> s32 #foreign libc;
}

#if OS == .LINUX {
    getnetgrent_r :: (__hostp: **u8, __userp: **u8, __domainp: **u8, __buffer: *u8, __buflen: u64) -> s32 #foreign libc;
}

#if OS == .WINDOWS {
    getpeername :: (s: SOCKET, name: *sockaddr, namelen: *s32) -> s32 #foreign ws2_32;
}

#if OS == .LINUX {
    getpeername :: (__fd: s32, __addr: *sockaddr, __len: *socklen_t) -> s32 #foreign libc;
}

#if OS == .ANDROID {
    getpeername :: (__fd: s32, __addr: *sockaddr, __addr_length: *socklen_t) -> s32 #foreign libc;
}

#if OS == .MACOS {
    getpeername :: (unknown0: s32, unknown1: *sockaddr, unknown2: *socklen_t) -> s32 #foreign libc;
}

#if OS == .WINDOWS {
    getprotobyname :: (name: *u8) -> *protoent #foreign ws2_32;
}

#if OS == .LINUX {
    getprotobyname :: (__name: *u8) -> *protoent #foreign libc;
}

#if OS == .ANDROID {
    getprotobyname :: (__name: *u8) -> *protoent #foreign libc;
}

#if OS == .MACOS {
    getprotobyname :: (unknown0: *u8) -> *protoent #foreign libc;
}

#if OS == .LINUX {
    getprotobyname_r :: (__name: *u8, __result_buf: *protoent, __buf: *u8, __buflen: u64, __result: **protoent) -> s32 #foreign libc;
}

#if OS == .WINDOWS {
    getprotobynumber :: (number: s32) -> *protoent #foreign ws2_32;
}

#if OS == .LINUX {
    getprotobynumber :: (__proto: s32) -> *protoent #foreign libc;
}

#if OS == .ANDROID {
    getprotobynumber :: (__proto: s32) -> *protoent #foreign libc;
}

#if OS == .MACOS {
    getprotobynumber :: (unknown0: s32) -> *protoent #foreign libc;
}

#if OS == .LINUX {
    getprotobynumber_r :: (__proto: s32, __result_buf: *protoent, __buf: *u8, __buflen: u64, __result: **protoent) -> s32 #foreign libc;
}

#if OS == .LINUX {
    getprotoent :: () -> *protoent #foreign libc;
}

#if OS == .ANDROID {
    getprotoent :: () -> *protoent #foreign libc;
}

#if OS == .MACOS {
    getprotoent :: () -> *protoent #foreign libc;
}

#if OS == .LINUX {
    getprotoent_r :: (__result_buf: *protoent, __buf: *u8, __buflen: u64, __result: **protoent) -> s32 #foreign libc;
}

#if OS == .MACOS {
    getrpcbyname :: (name: *u8) -> *rpcent #foreign libc;
}

#if OS == .MACOS {
    getrpcbynumber :: (number: s32) -> *rpcent #foreign libc;
}

#if OS == .MACOS {
    getrpcent :: () -> *rpcent #foreign libc;
}

#if OS == .WINDOWS {
    getservbyname :: (name: *u8, proto: *u8) -> *servent #foreign ws2_32;
}

#if OS == .LINUX {
    getservbyname :: (__name: *u8, __proto: *u8) -> *servent #foreign libc;
}

#if OS == .ANDROID {
    getservbyname :: (__name: *u8, __proto: *u8) -> *servent #foreign libc;
}

#if OS == .MACOS {
    getservbyname :: (unknown0: *u8, unknown1: *u8) -> *servent #foreign libc;
}

#if OS == .LINUX {
    getservbyname_r :: (__name: *u8, __proto: *u8, __result_buf: *servent, __buf: *u8, __buflen: u64, __result: **servent) -> s32 #foreign libc;
}

#if OS == .WINDOWS {
    getservbyport :: (port: s32, proto: *u8) -> *servent #foreign ws2_32;
}

#if OS == .LINUX {
    getservbyport :: (__port: s32, __proto: *u8) -> *servent #foreign libc;
}

#if OS == .ANDROID {
    getservbyport :: (__port_in_network_order: s32, __proto: *u8) -> *servent #foreign libc;
}

#if OS == .MACOS {
    getservbyport :: (unknown0: s32, unknown1: *u8) -> *servent #foreign libc;
}

#if OS == .LINUX {
    getservbyport_r :: (__port: s32, __proto: *u8, __result_buf: *servent, __buf: *u8, __buflen: u64, __result: **servent) -> s32 #foreign libc;
}

#if OS == .LINUX {
    getservent :: () -> *servent #foreign libc;
}

#if OS == .ANDROID {
    getservent :: () -> *servent #foreign libc;
}

#if OS == .MACOS {
    getservent :: () -> *servent #foreign libc;
}

#if OS == .LINUX {
    getservent_r :: (__result_buf: *servent, __buf: *u8, __buflen: u64, __result: **servent) -> s32 #foreign libc;
}

#if OS == .WINDOWS {
    getsockname :: (s: SOCKET, name: *sockaddr, namelen: *s32) -> s32 #foreign ws2_32;
}

#if OS == .LINUX {
    getsockname :: (__fd: s32, __addr: *sockaddr, __len: *socklen_t) -> s32 #foreign libc;
}

#if OS == .ANDROID {
    getsockname :: (__fd: s32, __addr: *sockaddr, __addr_length: *socklen_t) -> s32 #foreign libc;
}

#if OS == .MACOS {
    getsockname :: (unknown0: s32, unknown1: *sockaddr, unknown2: *socklen_t) -> s32 #foreign libc;
}

#if OS == .WINDOWS {
    getsockopt :: (s: SOCKET, level: s32, optname: s32, optval: *u8, optlen: *s32) -> s32 #foreign ws2_32;
}

#if OS == .LINUX {
    getsockopt :: (__fd: s32, __level: s32, __optname: s32, __optval: *void, __optlen: *socklen_t) -> s32 #foreign libc;
}

#if OS == .ANDROID {
    getsockopt :: (__fd: s32, __level: s32, __option: s32, __value: *void, __value_length: *socklen_t) -> s32 #foreign libc;
}

#if OS == .MACOS {
    getsockopt :: (unknown0: s32, unknown1: s32, unknown2: s32, unknown3: *void, unknown4: *socklen_t) -> s32 #foreign libc;
}

#if OS == .LINUX {
    getsourcefilter :: (__s: s32, __interface_addr: u32, __group: *sockaddr, __grouplen: socklen_t, __fmode: *u32, __numsrc: *u32, __slist: *sockaddr_storage) -> s32 #foreign libc;
}

#if OS == .MACOS {
    h_errno: s32 #elsewhere libc;
}

#if OS == .LINUX {
    herror :: (__str: *u8) -> void #foreign libc;
}

#if OS == .ANDROID {
    herror :: (__s: *u8) -> void #foreign libc;
}

#if OS == .MACOS {
    herror :: (unknown0: *u8) -> void #foreign libc;
}

#if OS == .LINUX {
    hstrerror :: (__err_num: s32) -> *u8 #foreign libc;
}

#if OS == .ANDROID {
    hstrerror :: (__error: s32) -> *u8 #foreign libc;
}

#if OS == .MACOS {
    hstrerror :: (unknown0: s32) -> *u8 #foreign libc;
}

#if OS == .ANDROID {
    htonl :: (hostlong: u32) -> u32 #foreign libc;
}

#if OS == .MACOS {
    htonl :: (hostlong: u32) -> u32 #foreign libc;
}

#if OS == .WINDOWS {
    htonl :: (hostlong: u32) -> u32 #foreign ws2_32;
}

#if OS == .LINUX {
    htonl :: (__hostlong: u32) -> u32 #foreign libc;
}

#if OS == .ANDROID {
    htons :: (hostshort: u16) -> u16 #foreign libc;
}

#if OS == .MACOS {
    htons :: (hostshort: u16) -> u16 #foreign libc;
}

#if OS == .WINDOWS {
    htons :: (hostshort: u16) -> u16 #foreign ws2_32;
}

#if OS == .LINUX {
    htons :: (__hostshort: u16) -> u16 #foreign libc;
}

#if OS == .LINUX {
    if_freenameindex :: (__ptr: *if_nameindex) -> void #foreign libc;
}

#if OS == .ANDROID {
    if_freenameindex :: (__ptr: *if_nameindex) -> void #foreign libc;
}

#if OS == .MACOS {
    if_freenameindex :: (unknown0: *if_nameindex) -> void #foreign libc;
}

#if OS == .LINUX {
    if_indextoname :: (__ifindex: u32, __ifname: *u8) -> *u8 #foreign libc;
}

#if OS == .ANDROID {
    if_indextoname :: (__index: u32, __buf: *u8) -> *u8 #foreign libc;
}

#if OS == .MACOS {
    if_indextoname :: (unknown0: u32, unknown1: *u8) -> *u8 #foreign libc;
}

#if OS == .LINUX {
    if_nameindex_ :: () -> *if_nameindex #foreign libc "if_nameindex";
}

#if OS == .ANDROID {
    if_nameindex_ :: () -> *if_nameindex #foreign libc "if_nameindex";
}

#if OS == .MACOS {
    if_nameindex_ :: () -> *if_nameindex #foreign libc "if_nameindex";
}

#if OS == .LINUX {
    if_nametoindex :: (__ifname: *u8) -> u32 #foreign libc;
}

#if OS == .ANDROID {
    if_nametoindex :: (__name: *u8) -> u32 #foreign libc;
}

#if OS == .MACOS {
    if_nametoindex :: (unknown0: *u8) -> u32 #foreign libc;
}

#if OS == .LINUX {
    in6addr_any: in6_addr #elsewhere libc;
}

#if OS == .ANDROID {
    in6addr_any: in6_addr #elsewhere libc;
}

#if OS == .MACOS {
    in6addr_any: in6_addr #elsewhere libc;
}

#if OS == .MACOS {
    in6addr_linklocal_allnodes: in6_addr #elsewhere libc;
}

#if OS == .MACOS {
    in6addr_linklocal_allrouters: in6_addr #elsewhere libc;
}

#if OS == .LINUX {
    in6addr_loopback: in6_addr #elsewhere libc;
}

#if OS == .ANDROID {
    in6addr_loopback: in6_addr #elsewhere libc;
}

#if OS == .MACOS {
    in6addr_loopback: in6_addr #elsewhere libc;
}

#if OS == .MACOS {
    in6addr_nodelocal_allnodes: in6_addr #elsewhere libc;
}

#if OS == .LINUX {
    inet6_opt_append :: (__extbuf: *void, __extlen: socklen_t, __offset: s32, __type: u8, __len: socklen_t, __align: u8, __databufp: **void) -> s32 #foreign libc;
}

#if OS == .MACOS {
    inet6_opt_append :: (unknown0: *void, unknown1: socklen_t, unknown2: s32, unknown3: u8, unknown4: socklen_t, unknown5: u8, unknown6: **void) -> s32 #foreign libc;
}

#if OS == .LINUX {
    inet6_opt_find :: (__extbuf: *void, __extlen: socklen_t, __offset: s32, __type: u8, __lenp: *socklen_t, __databufp: **void) -> s32 #foreign libc;
}

#if OS == .MACOS {
    inet6_opt_find :: (unknown0: *void, unknown1: socklen_t, unknown2: s32, unknown3: u8, unknown4: *socklen_t, unknown5: **void) -> s32 #foreign libc;
}

#if OS == .LINUX {
    inet6_opt_finish :: (__extbuf: *void, __extlen: socklen_t, __offset: s32) -> s32 #foreign libc;
}

#if OS == .MACOS {
    inet6_opt_finish :: (unknown0: *void, unknown1: socklen_t, unknown2: s32) -> s32 #foreign libc;
}

#if OS == .LINUX {
    inet6_opt_get_val :: (__databuf: *void, __offset: s32, __val: *void, __vallen: socklen_t) -> s32 #foreign libc;
}

#if OS == .MACOS {
    inet6_opt_get_val :: (unknown0: *void, unknown1: s32, unknown2: *void, unknown3: socklen_t) -> s32 #foreign libc;
}

#if OS == .LINUX {
    inet6_opt_init :: (__extbuf: *void, __extlen: socklen_t) -> s32 #foreign libc;
}

#if OS == .MACOS {
    inet6_opt_init :: (unknown0: *void, unknown1: socklen_t) -> s32 #foreign libc;
}

#if OS == .LINUX {
    inet6_opt_next :: (__extbuf: *void, __extlen: socklen_t, __offset: s32, __typep: *u8, __lenp: *socklen_t, __databufp: **void) -> s32 #foreign libc;
}

#if OS == .MACOS {
    inet6_opt_next :: (unknown0: *void, unknown1: socklen_t, unknown2: s32, unknown3: *u8, unknown4: *socklen_t, unknown5: **void) -> s32 #foreign libc;
}

#if OS == .LINUX {
    inet6_opt_set_val :: (__databuf: *void, __offset: s32, __val: *void, __vallen: socklen_t) -> s32 #foreign libc;
}

#if OS == .MACOS {
    inet6_opt_set_val :: (unknown0: *void, unknown1: s32, unknown2: *void, unknown3: socklen_t) -> s32 #foreign libc;
}

#if OS == .LINUX {
    inet6_option_alloc :: (__cmsg: *cmsghdr, __datalen: s32, __multx: s32, __plusy: s32) -> *u8 #foreign libc;
}

#if OS == .MACOS {
    inet6_option_alloc :: (unknown0: *cmsghdr, unknown1: s32, unknown2: s32, unknown3: s32) -> *u8 #foreign libc;
}

#if OS == .LINUX {
    inet6_option_append :: (__cmsg: *cmsghdr, __typep: *u8, __multx: s32, __plusy: s32) -> s32 #foreign libc;
}

#if OS == .MACOS {
    inet6_option_append :: (unknown0: *cmsghdr, unknown1: *u8, unknown2: s32, unknown3: s32) -> s32 #foreign libc;
}

#if OS == .LINUX {
    inet6_option_find :: (__cmsg: *cmsghdr, __tptrp: **u8, __type: s32) -> s32 #foreign libc;
}

#if OS == .MACOS {
    inet6_option_find :: (unknown0: *cmsghdr, unknown1: **u8, unknown2: s32) -> s32 #foreign libc;
}

#if OS == .LINUX {
    inet6_option_init :: (__bp: *void, __cmsgp: **cmsghdr, __type: s32) -> s32 #foreign libc;
}

#if OS == .MACOS {
    inet6_option_init :: (unknown0: *void, unknown1: **cmsghdr, unknown2: s32) -> s32 #foreign libc;
}

#if OS == .LINUX {
    inet6_option_next :: (__cmsg: *cmsghdr, __tptrp: **u8) -> s32 #foreign libc;
}

#if OS == .MACOS {
    inet6_option_next :: (unknown0: *cmsghdr, unknown1: **u8) -> s32 #foreign libc;
}

#if OS == .LINUX {
    inet6_option_space :: (__nbytes: s32) -> s32 #foreign libc;
}

#if OS == .MACOS {
    inet6_option_space :: (unknown0: s32) -> s32 #foreign libc;
}

#if OS == .LINUX {
    inet6_rth_add :: (__bp: *void, __addr: *in6_addr) -> s32 #foreign libc;
}

#if OS == .MACOS {
    inet6_rth_add :: (unknown0: *void, unknown1: *in6_addr) -> s32 #foreign libc;
}

#if OS == .LINUX {
    inet6_rth_getaddr :: (__bp: *void, __index: s32) -> *in6_addr #foreign libc;
}

#if OS == .MACOS {
    inet6_rth_getaddr :: (unknown0: *void, unknown1: s32) -> *in6_addr #foreign libc;
}

#if OS == .LINUX {
    inet6_rth_init :: (__bp: *void, __bp_len: socklen_t, __type: s32, __segments: s32) -> *void #foreign libc;
}

#if OS == .MACOS {
    inet6_rth_init :: (unknown0: *void, unknown1: socklen_t, unknown2: s32, unknown3: s32) -> *void #foreign libc;
}

#if OS == .LINUX {
    inet6_rth_reverse :: (__in: *void, __out: *void) -> s32 #foreign libc;
}

#if OS == .MACOS {
    inet6_rth_reverse :: (unknown0: *void, unknown1: *void) -> s32 #foreign libc;
}

#if OS == .LINUX {
    inet6_rth_segments :: (__bp: *void) -> s32 #foreign libc;
}

#if OS == .MACOS {
    inet6_rth_segments :: (unknown0: *void) -> s32 #foreign libc;
}

#if OS == .LINUX {
    inet6_rth_space :: (__type: s32, __segments: s32) -> socklen_t #foreign libc;
}

#if OS == .MACOS {
    inet6_rth_space :: (unknown0: s32, unknown1: s32) -> socklen_t #foreign libc;
}

#if OS == .MACOS {
    inet6_rthdr_add :: (unknown0: *cmsghdr, unknown1: *in6_addr, unknown2: u32) -> s32 #foreign libc;
}

#if OS == .MACOS {
    inet6_rthdr_getaddr :: (unknown0: *cmsghdr, unknown1: s32) -> *in6_addr #foreign libc;
}

#if OS == .MACOS {
    inet6_rthdr_getflags :: (unknown0: *cmsghdr, unknown1: s32) -> s32 #foreign libc;
}

#if OS == .MACOS {
    inet6_rthdr_init :: (unknown0: *void, unknown1: s32) -> *cmsghdr #foreign libc;
}

#if OS == .MACOS {
    inet6_rthdr_lasthop :: (unknown0: *cmsghdr, unknown1: u32) -> s32 #foreign libc;
}

#if OS == .MACOS {
    inet6_rthdr_segments :: (unknown0: *cmsghdr) -> s32 #foreign libc;
}

#if OS == .MACOS {
    inet6_rthdr_space :: (unknown0: s32, unknown1: s32) -> u64 #foreign libc;
}

#if OS == .WINDOWS {
    inet_addr :: (cp: *u8) -> u32 #foreign ws2_32;
}

#if OS == .LINUX {
    inet_addr :: (__cp: *u8) -> in_addr_t #foreign libc;
}

#if OS == .ANDROID {
    inet_addr :: (__s: *u8) -> in_addr_t #foreign libc;
}

#if OS == .MACOS {
    inet_addr :: (unknown0: *u8) -> in_addr_t #foreign libc;
}

#if OS == .LINUX {
    inet_aton :: (__cp: *u8, __inp: *in_addr) -> s32 #foreign libc;
}

#if OS == .ANDROID {
    inet_aton :: (__s: *u8, __addr: *in_addr) -> s32 #foreign libc;
}

#if OS == .MACOS {
    inet_aton :: (unknown0: *u8, unknown1: *in_addr) -> s32 #foreign libc;
}

#if OS == .LINUX {
    inet_lnaof :: (__in: in_addr) -> in_addr_t #foreign libc;
}

#if OS == .ANDROID {
    inet_lnaof :: (__addr: in_addr) -> in_addr_t #foreign libc;
}

#if OS == .MACOS {
    inet_lnaof :: (unknown0: in_addr) -> in_addr_t #foreign libc;
}

#if OS == .LINUX {
    inet_makeaddr :: (__net: in_addr_t, __host: in_addr_t) -> in_addr #foreign libc;
}

#if OS == .ANDROID {
    inet_makeaddr :: (__net: in_addr_t, __host: in_addr_t) -> in_addr #foreign libc;
}

#if OS == .MACOS {
    inet_makeaddr :: (unknown0: in_addr_t, unknown1: in_addr_t) -> in_addr #foreign libc;
}

#if OS == .MACOS {
    inet_net_ntop :: (unknown0: s32, unknown1: *void, unknown2: s32, unknown3: *u8, unknown4: u64) -> *u8 #foreign libc;
}

#if OS == .MACOS {
    inet_net_pton :: (unknown0: s32, unknown1: *u8, unknown2: *void, unknown3: u64) -> s32 #foreign libc;
}

#if OS == .MACOS {
    inet_neta :: (unknown0: in_addr_t, unknown1: *u8, unknown2: u64) -> *u8 #foreign libc;
}

#if OS == .LINUX {
    inet_netof :: (__in: in_addr) -> in_addr_t #foreign libc;
}

#if OS == .ANDROID {
    inet_netof :: (__addr: in_addr) -> in_addr_t #foreign libc;
}

#if OS == .MACOS {
    inet_netof :: (unknown0: in_addr) -> in_addr_t #foreign libc;
}

#if OS == .LINUX {
    inet_network :: (__cp: *u8) -> in_addr_t #foreign libc;
}

#if OS == .ANDROID {
    inet_network :: (__s: *u8) -> in_addr_t #foreign libc;
}

#if OS == .MACOS {
    inet_network :: (unknown0: *u8) -> in_addr_t #foreign libc;
}

#if OS == .LINUX {
    inet_nsap_addr :: (__cp: *u8, __buf: *u8, __len: s32) -> u32 #foreign libc;
}

#if OS == .ANDROID {
    inet_nsap_addr :: (__ascii: *u8, __binary: *u8, __n: s32) -> u32 #foreign libc;
}

#if OS == .MACOS {
    inet_nsap_addr :: (unknown0: *u8, unknown1: *u8, unknown2: s32) -> u32 #foreign libc;
}

#if OS == .LINUX {
    inet_nsap_ntoa :: (__len: s32, __cp: *u8, __buf: *u8) -> *u8 #foreign libc;
}

#if OS == .ANDROID {
    inet_nsap_ntoa :: (__binary_length: s32, __binary: *u8, __ascii: *u8) -> *u8 #foreign libc;
}

#if OS == .MACOS {
    inet_nsap_ntoa :: (unknown0: s32, unknown1: *u8, unknown2: *u8) -> *u8 #foreign libc;
}

#if OS == .WINDOWS {
    inet_ntoa :: (in: in_addr) -> *u8 #foreign ws2_32;
}

#if OS == .LINUX {
    inet_ntoa :: (__in: in_addr) -> *u8 #foreign libc;
}

#if OS == .ANDROID {
    inet_ntoa :: (__addr: in_addr) -> *u8 #foreign libc;
}

#if OS == .MACOS {
    inet_ntoa :: (unknown0: in_addr) -> *u8 #foreign libc;
}

#if OS == .WINDOWS {
    inet_ntop :: (Family: s32, pAddr: *void, pStringBuf: *u8, StringBufSize: u64) -> *u8 #foreign ws2_32;
}

#if OS == .LINUX {
    inet_ntop :: (__af: s32, __cp: *void, __buf: *u8, __len: socklen_t) -> *u8 #foreign libc;
}

#if OS == .ANDROID {
    inet_ntop :: (__af: s32, __src: *void, __dst: *u8, __size: socklen_t) -> *u8 #foreign libc;
}

#if OS == .MACOS {
    inet_ntop :: (unknown0: s32, unknown1: *void, unknown2: *u8, unknown3: socklen_t) -> *u8 #foreign libc;
}

#if OS == .WINDOWS {
    inet_pton :: (Family: s32, pszAddrString: *u8, pAddrBuf: *void) -> s32 #foreign ws2_32;
}

#if OS == .LINUX {
    inet_pton :: (__af: s32, __cp: *u8, __buf: *void) -> s32 #foreign libc;
}

#if OS == .ANDROID {
    inet_pton :: (__af: s32, __src: *u8, __dst: *void) -> s32 #foreign libc;
}

#if OS == .MACOS {
    inet_pton :: (unknown0: s32, unknown1: *u8, unknown2: *void) -> s32 #foreign libc;
}

#if OS == .WINDOWS {
    InetNtopW :: (Family: s32, pAddr: *void, pStringBuf: *s16, StringBufSize: u64) -> *s16 #foreign ws2_32;
}

#if OS == .WINDOWS {
    InetPtonW :: (Family: s32, pszAddrString: *s16, pAddrBuf: *void) -> s32 #foreign ws2_32;
}

#if OS == .LINUX {
    innetgr :: (__netgroup: *u8, __host: *u8, __user: *u8, __domain: *u8) -> s32 #foreign libc;
}

#if OS == .MACOS {
    innetgr :: (unknown0: *u8, unknown1: *u8, unknown2: *u8, unknown3: *u8) -> s32 #foreign libc;
}

#if OS == .WINDOWS {
    ioctlsocket :: (s: SOCKET, cmd: s32, argp: *u32) -> s32 #foreign ws2_32;
}

#if OS == .LINUX {
    iruserok :: (__raddr: u32, __suser: s32, __remuser: *u8, __locuser: *u8) -> s32 #foreign libc;
}

#if OS == .LINUX {
    iruserok_af :: (__raddr: *void, __suser: s32, __remuser: *u8, __locuser: *u8, __af: sa_family_t) -> s32 #foreign libc;
}

#if OS == .LINUX {
    isfdtype :: (__fd: s32, __fdtype: s32) -> s32 #foreign libc;
}

#if OS == .WINDOWS {
    listen :: (s: SOCKET, backlog: s32) -> s32 #foreign ws2_32;
}

#if OS == .LINUX {
    listen :: (__fd: s32, __n: s32) -> s32 #foreign libc;
}

#if OS == .ANDROID {
    listen :: (__fd: s32, __backlog: s32) -> s32 #foreign libc;
}

#if OS == .MACOS {
    listen :: (unknown0: s32, unknown1: s32) -> s32 #foreign libc;
}

#if OS == .ANDROID {
    ntohl :: (netlong: u32) -> u32 #foreign libc;
}

#if OS == .MACOS {
    ntohl :: (netlong: u32) -> u32 #foreign libc;
}

#if OS == .WINDOWS {
    ntohl :: (netlong: u32) -> u32 #foreign ws2_32;
}

#if OS == .LINUX {
    ntohl :: (__netlong: u32) -> u32 #foreign libc;
}

#if OS == .ANDROID {
    ntohs :: (netshort: u16) -> u16 #foreign libc;
}

#if OS == .MACOS {
    ntohs :: (netshort: u16) -> u16 #foreign libc;
}

#if OS == .WINDOWS {
    ntohs :: (netshort: u16) -> u16 #foreign ws2_32;
}

#if OS == .LINUX {
    ntohs :: (__netshort: u16) -> u16 #foreign libc;
}

#if OS == .LINUX {
    pselect :: (__nfds: s32, __readfds: *fd_set, __writefds: *fd_set, __exceptfds: *fd_set, __timeout: *timespec, __sigmask: *__sigset_t) -> s32 #foreign libc;
}

#if OS == .ANDROID {
    pselect :: (__max_fd_plus_one: s32, __read_fds: *fd_set, __write_fds: *fd_set, __exception_fds: *fd_set, __timeout: *timespec, __mask: *sigset_t) -> s32 #foreign libc;
}

#if OS == .ANDROID {
    pselect64 :: (__max_fd_plus_one: s32, __read_fds: *fd_set, __write_fds: *fd_set, __exception_fds: *fd_set, __timeout: *timespec, __mask: *sigset64_t) -> s32 #foreign libc;
}

#if OS == .LINUX {
    rcmd :: (__ahost: **u8, __rport: u16, __locuser: *u8, __remuser: *u8, __cmd: *u8, __fd2p: *s32) -> s32 #foreign libc;
}

#if OS == .LINUX {
    rcmd_af :: (__ahost: **u8, __rport: u16, __locuser: *u8, __remuser: *u8, __cmd: *u8, __fd2p: *s32, __af: sa_family_t) -> s32 #foreign libc;
}

#if OS == .WINDOWS {
    recv :: (s: SOCKET, buf: *u8, len: s32, flags: MSG) -> s32 #foreign ws2_32;
}

#if OS == .LINUX {
    recv :: (__fd: s32, __buf: *void, __n: u64, __flags: MSG) -> s64 #foreign libc;
}

#if OS == .ANDROID {
    recv :: (__fd: s32, __buf: *void, __n: u64, __flags: MSG) -> s64 #foreign libc;
}

#if OS == .MACOS {
    recv :: (unknown0: s32, unknown1: *void, unknown2: u64, unknown3: MSG) -> s64 #foreign libc;
}

#if OS == .WINDOWS {
    recvfrom :: (s: SOCKET, buf: *u8, len: s32, flags: MSG, from: *sockaddr, fromlen: *s32) -> s32 #foreign ws2_32;
}

#if OS == .LINUX {
    recvfrom :: (__fd: s32, __buf: *void, __n: u64, __flags: MSG, __addr: *sockaddr, __addr_len: *socklen_t) -> s64 #foreign libc;
}

#if OS == .ANDROID {
    recvfrom :: (__fd: s32, __buf: *void, __n: u64, __flags: MSG, __src_addr: *sockaddr, __src_addr_length: *socklen_t) -> s64 #foreign libc;
}

#if OS == .MACOS {
    recvfrom :: (unknown0: s32, unknown1: *void, unknown2: u64, unknown3: MSG, unknown4: *sockaddr, unknown5: *socklen_t) -> s64 #foreign libc;
}

#if OS == .LINUX {
    recvmmsg :: (__fd: s32, __vmessages: *mmsghdr, __vlen: u32, __flags: MSG, __tmo: *timespec) -> s32 #foreign libc;
}

#if OS == .ANDROID {
    recvmmsg :: (__fd: s32, __msgs: *mmsghdr, __msg_count: u32, __flags: MSG, __timeout: *timespec) -> s32 #foreign libc;
}

#if OS == .LINUX {
    recvmsg :: (__fd: s32, __message: *msghdr, __flags: MSG) -> s64 #foreign libc;
}

#if OS == .ANDROID {
    recvmsg :: (__fd: s32, __msg: *msghdr, __flags: MSG) -> s64 #foreign libc;
}

#if OS == .MACOS {
    recvmsg :: (unknown0: s32, unknown1: *msghdr, unknown2: MSG) -> s64 #foreign libc;
}

#if OS == .LINUX {
    rexec :: (__ahost: **u8, __rport: s32, __name: *u8, __pass: *u8, __cmd: *u8, __fd2p: *s32) -> s32 #foreign libc;
}

#if OS == .LINUX {
    rexec_af :: (__ahost: **u8, __rport: s32, __name: *u8, __pass: *u8, __cmd: *u8, __fd2p: *s32, __af: sa_family_t) -> s32 #foreign libc;
}

#if OS == .LINUX {
    rresvport :: (__alport: *s32) -> s32 #foreign libc;
}

#if OS == .LINUX {
    rresvport_af :: (__alport: *s32, __af: sa_family_t) -> s32 #foreign libc;
}

#if OS == .LINUX {
    ruserok :: (__rhost: *u8, __suser: s32, __remuser: *u8, __locuser: *u8) -> s32 #foreign libc;
}

#if OS == .LINUX {
    ruserok_af :: (__rhost: *u8, __suser: s32, __remuser: *u8, __locuser: *u8, __af: sa_family_t) -> s32 #foreign libc;
}

#if OS == .WINDOWS {
    select :: (nfds: s32, readfds: *fd_set, writefds: *fd_set, exceptfds: *fd_set, timeout: *TIMEVAL) -> s32 #foreign ws2_32;
}

#if OS == .LINUX {
    select :: (__nfds: s32, __readfds: *fd_set, __writefds: *fd_set, __exceptfds: *fd_set, __timeout: *TIMEVAL) -> s32 #foreign libc;
}

#if OS == .ANDROID {
    select :: (__max_fd_plus_one: s32, __read_fds: *fd_set, __write_fds: *fd_set, __exception_fds: *fd_set, __timeout: *TIMEVAL) -> s32 #foreign libc;
}

#if CPU == .X64 && OS == .MACOS {
    select :: (nfds: s32, readfds: *fd_set, writefds: *fd_set, exceptfds: *fd_set, timeout: *TIMEVAL) -> s32 #foreign libc "select$1050";
}

#if OS != .WINDOWS && OS == .MACOS {
    select :: (nfds: s32, readfds: *fd_set, writefds: *fd_set, exceptfds: *fd_set, timeout: *TIMEVAL) -> s32 #foreign libc;
}

#if OS == .WINDOWS {
    send :: (s: SOCKET, buf: *u8, len: s32, flags: MSG) -> s32 #foreign ws2_32;
}

#if OS == .LINUX {
    send :: (__fd: s32, __buf: *void, __n: u64, __flags: MSG) -> s64 #foreign libc;
}

#if OS == .ANDROID {
    send :: (__fd: s32, __buf: *void, __n: u64, __flags: MSG) -> s64 #foreign libc;
}

#if OS == .MACOS {
    send :: (unknown0: s32, unknown1: *void, unknown2: u64, unknown3: MSG) -> s64 #foreign libc;
}

#if OS == .MACOS {
    sendfile :: (unknown0: s32, unknown1: s32, unknown2: s64, unknown3: *s64, unknown4: *sf_hdtr, unknown5: s32) -> s32 #foreign libc;
}

#if OS == .LINUX {
    sendmmsg :: (__fd: s32, __vmessages: *mmsghdr, __vlen: u32, __flags: MSG) -> s32 #foreign libc;
}

#if OS == .ANDROID {
    sendmmsg :: (__fd: s32, __msgs: *mmsghdr, __msg_count: u32, __flags: MSG) -> s32 #foreign libc;
}

#if OS == .LINUX {
    sendmsg :: (__fd: s32, __message: *msghdr, __flags: MSG) -> s64 #foreign libc;
}

#if OS == .ANDROID {
    sendmsg :: (__fd: s32, __msg: *msghdr, __flags: MSG) -> s64 #foreign libc;
}

#if OS == .MACOS {
    sendmsg :: (unknown0: s32, unknown1: *msghdr, unknown2: MSG) -> s64 #foreign libc;
}

#if OS == .WINDOWS {
    sendto :: (s: SOCKET, buf: *u8, len: s32, flags: MSG, to: *sockaddr, tolen: s32) -> s32 #foreign ws2_32;
}

#if OS == .LINUX {
    sendto :: (__fd: s32, __buf: *void, __n: u64, __flags: MSG, __addr: *sockaddr, __addr_len: socklen_t) -> s64 #foreign libc;
}

#if OS == .ANDROID {
    sendto :: (__fd: s32, __buf: *void, __n: u64, __flags: MSG, __dst_addr: *sockaddr, __dst_addr_length: socklen_t) -> s64 #foreign libc;
}

#if OS == .MACOS {
    sendto :: (unknown0: s32, unknown1: *void, unknown2: u64, unknown3: MSG, unknown4: *sockaddr, unknown5: socklen_t) -> s64 #foreign libc;
}

#if OS == .WINDOWS {
    SetAddrInfoExA :: (pName: *u8, pServiceName: *u8, pAddresses: *SOCKET_ADDRESS, dwAddressCount: u32, lpBlob: *BLOB, dwFlags: u32, dwNameSpace: u32, lpNspId: *GUID, timeout: *TIMEVAL, lpOverlapped: *OVERLAPPED, lpCompletionRoutine: LPLOOKUPSERVICE_COMPLETION_ROUTINE, lpNameHandle: *HANDLE) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    SetAddrInfoExW :: (pName: *s16, pServiceName: *s16, pAddresses: *SOCKET_ADDRESS, dwAddressCount: u32, lpBlob: *BLOB, dwFlags: u32, dwNameSpace: u32, lpNspId: *GUID, timeout: *TIMEVAL, lpOverlapped: *OVERLAPPED, lpCompletionRoutine: LPLOOKUPSERVICE_COMPLETION_ROUTINE, lpNameHandle: *HANDLE) -> s32 #foreign ws2_32;
}

#if OS == .LINUX {
    sethostent :: (__stay_open: s32) -> void #foreign libc;
}

#if OS == .ANDROID {
    sethostent :: (__stay_open: s32) -> void #foreign libc;
}

#if OS == .MACOS {
    sethostent :: (unknown0: s32) -> void #foreign libc;
}

#if OS == .LINUX {
    setipv4sourcefilter :: (__s: s32, __interface_addr: in_addr, __group: in_addr, __fmode: u32, __numsrc: u32, __slist: *in_addr) -> s32 #foreign libc;
}

#if OS == .LINUX {
    setnetent :: (__stay_open: s32) -> void #foreign libc;
}

#if OS == .ANDROID {
    setnetent :: (__stay_open: s32) -> void #foreign libc;
}

#if OS == .MACOS {
    setnetent :: (unknown0: s32) -> void #foreign libc;
}

#if OS == .LINUX {
    setnetgrent :: (__netgroup: *u8) -> s32 #foreign libc;
}

#if OS == .MACOS {
    setnetgrent :: (unknown0: *u8) -> void #foreign libc;
}

#if OS == .LINUX {
    setprotoent :: (__stay_open: s32) -> void #foreign libc;
}

#if OS == .ANDROID {
    setprotoent :: (__stay_open: s32) -> void #foreign libc;
}

#if OS == .MACOS {
    setprotoent :: (unknown0: s32) -> void #foreign libc;
}

#if OS == .MACOS {
    setrpcent :: (stayopen: s32) -> void #foreign libc;
}

#if OS == .LINUX {
    setservent :: (__stay_open: s32) -> void #foreign libc;
}

#if OS == .ANDROID {
    setservent :: (__stay_open: s32) -> void #foreign libc;
}

#if OS == .MACOS {
    setservent :: (unknown0: s32) -> void #foreign libc;
}

#if OS == .WINDOWS {
    setsockopt :: (s: SOCKET, level: s32, optname: s32, optval: *u8, optlen: s32) -> s32 #foreign ws2_32;
}

#if OS == .LINUX {
    setsockopt :: (__fd: s32, __level: s32, __optname: s32, __optval: *void, __optlen: socklen_t) -> s32 #foreign libc;
}

#if OS == .ANDROID {
    setsockopt :: (__fd: s32, __level: s32, __option: s32, __value: *void, __value_length: socklen_t) -> s32 #foreign libc;
}

#if OS == .MACOS {
    setsockopt :: (unknown0: s32, unknown1: s32, unknown2: s32, unknown3: *void, unknown4: socklen_t) -> s32 #foreign libc;
}

#if OS == .LINUX {
    setsourcefilter :: (__s: s32, __interface_addr: u32, __group: *sockaddr, __grouplen: socklen_t, __fmode: u32, __numsrc: u32, __slist: *sockaddr_storage) -> s32 #foreign libc;
}

#if OS == .WINDOWS {
    shutdown :: (s: SOCKET, how: s32) -> s32 #foreign ws2_32;
}

#if OS == .LINUX {
    shutdown :: (__fd: s32, __how: SHUT) -> s32 #foreign libc;
}

#if OS == .ANDROID {
    shutdown :: (__fd: s32, __how: SHUT) -> s32 #foreign libc;
}

#if OS == .MACOS {
    shutdown :: (unknown0: s32, unknown1: SHUT) -> s32 #foreign libc;
}

#if OS == .LINUX {
    sockatmark :: (__fd: s32) -> s32 #foreign libc;
}

#if OS == .MACOS {
    sockatmark :: (unknown0: s32) -> s32 #foreign libc;
}

#if OS == .WINDOWS {
    socket :: (af: s32, type: SOCK, protocol: IPPROTO) -> SOCKET #foreign ws2_32;
}

#if OS == .LINUX {
    socket :: (__domain: s32, __type: SOCK, __protocol: IPPROTO) -> s32 #foreign libc;
}

#if OS == .ANDROID {
    socket :: (__af: s32, __type: SOCK, __protocol: IPPROTO) -> s32 #foreign libc;
}

#if OS == .MACOS {
    socket :: (unknown0: s32, unknown1: SOCK, unknown2: IPPROTO) -> s32 #foreign libc;
}

#if OS == .LINUX {
    socketpair :: (__domain: s32, __type: SOCK, __protocol: IPPROTO, __fds: *[2] s32) -> s32 #foreign libc;
}

#if OS == .ANDROID {
    socketpair :: (__af: s32, __type: SOCK, __protocol: IPPROTO, __fds: *[2] s32) -> s32 #foreign libc;
}

#if OS == .MACOS {
    socketpair :: (unknown0: s32, unknown1: SOCK, unknown2: IPPROTO, unknown3: *[2] s32) -> s32 #foreign libc;
}

#if OS == .WINDOWS {
    WSAAccept :: (s: SOCKET, addr: *sockaddr, addrlen: *s32, lpfnCondition: LPCONDITIONPROC, dwCallbackData: u64) -> SOCKET #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAAddressToStringA :: (lpsaAddress: *sockaddr, dwAddressLength: u32, lpProtocolInfo: *WSAPROTOCOL_INFO, lpszAddressString: *u8, lpdwAddressStringLength: *u32) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAAddressToStringW :: (lpsaAddress: *sockaddr, dwAddressLength: u32, lpProtocolInfo: *WSAPROTOCOL_INFOW, lpszAddressString: *s16, lpdwAddressStringLength: *u32) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAAsyncGetHostByAddr :: (hWnd: HWND, wMsg: u32, addr: *u8, len: s32, type: s32, buf: *u8, buflen: s32) -> HANDLE #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAAsyncGetHostByName :: (hWnd: HWND, wMsg: u32, name: *u8, buf: *u8, buflen: s32) -> HANDLE #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAAsyncGetProtoByName :: (hWnd: HWND, wMsg: u32, name: *u8, buf: *u8, buflen: s32) -> HANDLE #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAAsyncGetProtoByNumber :: (hWnd: HWND, wMsg: u32, number: s32, buf: *u8, buflen: s32) -> HANDLE #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAAsyncGetServByName :: (hWnd: HWND, wMsg: u32, name: *u8, proto: *u8, buf: *u8, buflen: s32) -> HANDLE #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAAsyncGetServByPort :: (hWnd: HWND, wMsg: u32, port: s32, proto: *u8, buf: *u8, buflen: s32) -> HANDLE #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAAsyncSelect :: (s: SOCKET, hWnd: HWND, wMsg: u32, lEvent: s32) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSACancelAsyncRequest :: (hAsyncTaskHandle: HANDLE) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSACancelBlockingCall :: () -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSACleanup :: () -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSACloseEvent :: (hEvent: HANDLE) -> BOOL #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAConnect :: (s: SOCKET, name: *sockaddr, namelen: s32, lpCallerData: *WSABUF, lpCalleeData: *WSABUF, lpSQOS: *QOS, lpGQOS: *QOS) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAConnectByList :: (s: SOCKET, SocketAddress: *SOCKET_ADDRESS_LIST, LocalAddressLength: *u32, LocalAddress: *sockaddr, RemoteAddressLength: *u32, RemoteAddress: *sockaddr, timeout: *TIMEVAL, Reserved: *OVERLAPPED) -> BOOL #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAConnectByNameA :: (s: SOCKET, nodename: *u8, servicename: *u8, LocalAddressLength: *u32, LocalAddress: *sockaddr, RemoteAddressLength: *u32, RemoteAddress: *sockaddr, timeout: *TIMEVAL, Reserved: *OVERLAPPED) -> BOOL #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAConnectByNameW :: (s: SOCKET, nodename: *s16, servicename: *s16, LocalAddressLength: *u32, LocalAddress: *sockaddr, RemoteAddressLength: *u32, RemoteAddress: *sockaddr, timeout: *TIMEVAL, Reserved: *OVERLAPPED) -> BOOL #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSACreateEvent :: () -> HANDLE #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSADuplicateSocketA :: (s: SOCKET, dwProcessId: u32, lpProtocolInfo: *WSAPROTOCOL_INFO) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSADuplicateSocketW :: (s: SOCKET, dwProcessId: u32, lpProtocolInfo: *WSAPROTOCOL_INFOW) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAEnumNetworkEvents :: (s: SOCKET, hEventObject: HANDLE, lpNetworkEvents: *WSANETWORKEVENTS) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAEnumProtocolsA :: (lpiProtocols: *s32, lpProtocolBuffer: *WSAPROTOCOL_INFO, lpdwBufferLength: *u32) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAEnumProtocolsW :: (lpiProtocols: *s32, lpProtocolBuffer: *WSAPROTOCOL_INFOW, lpdwBufferLength: *u32) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAEventSelect :: (s: SOCKET, hEventObject: HANDLE, lNetworkEvents: s32) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAGetLastError :: () -> OS_Error_Code #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAGetOverlappedResult :: (s: SOCKET, lpOverlapped: *OVERLAPPED, lpcbTransfer: *u32, fWait: BOOL, lpdwFlags: *u32) -> BOOL #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAGetQOSByName :: (s: SOCKET, lpQOSName: *WSABUF, lpQOS: *QOS) -> BOOL #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAGetServiceClassInfoA :: (lpProviderId: *GUID, lpServiceClassId: *GUID, lpdwBufSize: *u32, lpServiceClassInfo: *WSAServiceClassInfoA) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAGetServiceClassInfoW :: (lpProviderId: *GUID, lpServiceClassId: *GUID, lpdwBufSize: *u32, lpServiceClassInfo: *WSAServiceClassInfoW) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAGetServiceClassNameByClassIdA :: (lpServiceClassId: *GUID, lpszServiceClassName: *u8, lpdwBufferLength: *u32) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAGetServiceClassNameByClassIdW :: (lpServiceClassId: *GUID, lpszServiceClassName: *s16, lpdwBufferLength: *u32) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAHtonl :: (s: SOCKET, hostlong: u32, lpnetlong: *u32) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAHtons :: (s: SOCKET, hostshort: u16, lpnetshort: *u16) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAInstallServiceClassA :: (lpServiceClassInfo: *WSAServiceClassInfoA) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAInstallServiceClassW :: (lpServiceClassInfo: *WSAServiceClassInfoW) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAIoctl :: (s: SOCKET, dwIoControlCode: u32, lpvInBuffer: *void, cbInBuffer: u32, lpvOutBuffer: *void, cbOutBuffer: u32, lpcbBytesReturned: *u32, lpOverlapped: *OVERLAPPED, lpCompletionRoutine: LPWSAOVERLAPPED_COMPLETION_ROUTINE) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAIsBlocking :: () -> BOOL #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAJoinLeaf :: (s: SOCKET, name: *sockaddr, namelen: s32, lpCallerData: *WSABUF, lpCalleeData: *WSABUF, lpSQOS: *QOS, lpGQOS: *QOS, dwFlags: u32) -> SOCKET #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSANSPIoctl :: (hLookup: HANDLE, dwControlCode: u32, lpvInBuffer: *void, cbInBuffer: u32, lpvOutBuffer: *void, cbOutBuffer: u32, lpcbBytesReturned: *u32, lpCompletion: *WSACOMPLETION) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSANtohl :: (s: SOCKET, netlong: u32, lphostlong: *u32) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSANtohs :: (s: SOCKET, netshort: u16, lphostshort: *u16) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAPoll :: (fdArray: *WSAPOLLFD, fds: u32, timeout: s32) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAProviderConfigChange :: (lpNotificationHandle: *HANDLE, lpOverlapped: *OVERLAPPED, lpCompletionRoutine: LPWSAOVERLAPPED_COMPLETION_ROUTINE) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSARecv :: (s: SOCKET, lpBuffers: *WSABUF, dwBufferCount: u32, lpNumberOfBytesRecvd: *u32, lpFlags: *u32, lpOverlapped: *OVERLAPPED, lpCompletionRoutine: LPWSAOVERLAPPED_COMPLETION_ROUTINE) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSARecvDisconnect :: (s: SOCKET, lpInboundDisconnectData: *WSABUF) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSARecvFrom :: (s: SOCKET, lpBuffers: *WSABUF, dwBufferCount: u32, lpNumberOfBytesRecvd: *u32, lpFlags: *u32, lpFrom: *sockaddr, lpFromlen: *s32, lpOverlapped: *OVERLAPPED, lpCompletionRoutine: LPWSAOVERLAPPED_COMPLETION_ROUTINE) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSARemoveServiceClass :: (lpServiceClassId: *GUID) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAResetEvent :: (hEvent: HANDLE) -> BOOL #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSASend :: (s: SOCKET, lpBuffers: *WSABUF, dwBufferCount: u32, lpNumberOfBytesSent: *u32, dwFlags: u32, lpOverlapped: *OVERLAPPED, lpCompletionRoutine: LPWSAOVERLAPPED_COMPLETION_ROUTINE) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSASendDisconnect :: (s: SOCKET, lpOutboundDisconnectData: *WSABUF) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSASendMsg :: (Handle: SOCKET, lpMsg: *WSAMSG, dwFlags: u32, lpNumberOfBytesSent: *u32, lpOverlapped: *OVERLAPPED, lpCompletionRoutine: LPWSAOVERLAPPED_COMPLETION_ROUTINE) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSASendTo :: (s: SOCKET, lpBuffers: *WSABUF, dwBufferCount: u32, lpNumberOfBytesSent: *u32, dwFlags: u32, lpTo: *sockaddr, iTolen: s32, lpOverlapped: *OVERLAPPED, lpCompletionRoutine: LPWSAOVERLAPPED_COMPLETION_ROUTINE) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSASetBlockingHook :: (lpBlockFunc: #type () -> s64 #c_call) -> (() -> s64 #c_call) #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSASetEvent :: (hEvent: HANDLE) -> BOOL #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSASetLastError :: (iError: s32) -> void #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSASocketA :: (af: s32, type: SOCK, protocol: IPPROTO, lpProtocolInfo: *WSAPROTOCOL_INFO, g: GROUP, dwFlags: u32) -> SOCKET #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSASocketW :: (af: s32, type: SOCK, protocol: IPPROTO, lpProtocolInfo: *WSAPROTOCOL_INFOW, g: GROUP, dwFlags: u32) -> SOCKET #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAStartup :: (wVersionRequested: u16, lpWSAData: *WSADATA) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAStringToAddressA :: (AddressString: *u8, AddressFamily: s32, lpProtocolInfo: *WSAPROTOCOL_INFO, lpAddress: *sockaddr, lpAddressLength: *s32) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAStringToAddressW :: (AddressString: *s16, AddressFamily: s32, lpProtocolInfo: *WSAPROTOCOL_INFOW, lpAddress: *sockaddr, lpAddressLength: *s32) -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAUnhookBlockingHook :: () -> s32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WSAWaitForMultipleEvents :: (cEvents: u32, lphEvents: *HANDLE, fWaitAll: BOOL, dwTimeout: u32, fAlertable: BOOL) -> u32 #foreign ws2_32;
}

#if OS == .WINDOWS {
    WINSOCK_VERSION: u16 : 0x0202;
}

#if OS == .MACOS {
    __DARWIN_IPPORT_RESERVED :: 1024;
}

#if OS == .LINUX {
    __fd_mask :: s64;
}

#if OS == .MACOS {
    __IPV6_ADDR_SCOPE_GLOBAL :: 0x0e;
}

#if OS == .MACOS {
    __IPV6_ADDR_SCOPE_INTFACELOCAL :: 0x01;
}

#if OS == .MACOS {
    __IPV6_ADDR_SCOPE_LINKLOCAL :: 0x02;
}

#if OS == .MACOS {
    __IPV6_ADDR_SCOPE_NODELOCAL :: 0x01;
}

#if OS == .MACOS {
    __IPV6_ADDR_SCOPE_ORGLOCAL :: 0x08;
}

#if OS == .MACOS {
    __IPV6_ADDR_SCOPE_SITELOCAL :: 0x05;
}

#if OS == .MACOS {
    __KAME_VERSION :: "2009/apple-darwin";
}

#if OS == .ANDROID {
    __kernel_sockaddr_storage :: sockaddr_storage;
}

#if OS == .ANDROID {
    __SOCK_SIZE__ :: 16;
}

#if OS == .LINUX {
    __USE_KERNEL_IPV6_DEFS :: 0;
}

#if OS == .LINUX {
    _PATH_HEQUIV :: "/etc/hosts.equiv";
}

#if OS == .ANDROID {
    _PATH_HEQUIV :: "/system/etc/hosts.equiv";
}

#if OS == .MACOS {
    _PATH_HEQUIV :: "/etc/hosts.equiv";
}

#if OS == .LINUX {
    _PATH_HOSTS :: "/etc/hosts";
}

#if OS == .ANDROID {
    _PATH_HOSTS :: "/system/etc/hosts";
}

#if OS == .MACOS {
    _PATH_HOSTS :: "/etc/hosts";
}

#if OS == .LINUX {
    _PATH_NETWORKS :: "/etc/networks";
}

#if OS == .ANDROID {
    _PATH_NETWORKS :: "/system/etc/networks";
}

#if OS == .MACOS {
    _PATH_NETWORKS :: "/etc/networks";
}

#if OS == .LINUX {
    _PATH_NSSWITCH_CONF :: "/etc/nsswitch.conf";
}

#if OS == .LINUX {
    _PATH_PROTOCOLS :: "/etc/protocols";
}

#if OS == .ANDROID {
    _PATH_PROTOCOLS :: "/system/etc/protocols";
}

#if OS == .MACOS {
    _PATH_PROTOCOLS :: "/etc/protocols";
}

#if OS == .LINUX {
    _PATH_SERVICES :: "/etc/services";
}

#if OS == .ANDROID {
    _PATH_SERVICES :: "/system/etc/services";
}

#if OS == .MACOS {
    _PATH_SERVICES :: "/etc/services";
}

#if OS == .WINDOWS {
    _SS_MAXSIZE :: 128;
}

#if OS == .MACOS {
    _SS_MAXSIZE :: 128;
}

#if OS == .WINDOWS {
    _WSACMSGHDR :: cmsghdr;
}

#if OS == .WINDOWS {
    ADDR_ANY :: INADDR_ANY;
}

#if OS == .WINDOWS {
    ADDRESS_FAMILY :: u16;
}

#if OS == .WINDOWS {
    ADDRINFO :: ADDRINFOA;
}

#if OS == .WINDOWS {
    ADDRINFO_DNS_SERVER :: addrinfo_dns_server;
}

#if OS == .WINDOWS {
    ADDRINFOA :: addrinfo;
}

#if OS == .WINDOWS {
    ADDRINFOEX :: ADDRINFOEXA;
}

#if OS == .WINDOWS {
    ADDRINFOEX2A :: addrinfoex2A;
}

#if OS == .WINDOWS {
    ADDRINFOEX2W :: addrinfoex2W;
}

#if OS == .WINDOWS {
    ADDRINFOEX3 :: addrinfoex3;
}

#if OS == .WINDOWS {
    ADDRINFOEX4 :: addrinfoex4;
}

#if OS == .WINDOWS {
    ADDRINFOEX5 :: addrinfoex5;
}

#if OS == .WINDOWS {
    ADDRINFOEX6 :: addrinfoex6;
}

#if OS == .WINDOWS {
    ADDRINFOEX7 :: addrinfoex7;
}

#if OS == .WINDOWS {
    ADDRINFOEX_VERSION_2 :: 2;
}

#if OS == .WINDOWS {
    ADDRINFOEX_VERSION_3 :: 3;
}

#if OS == .WINDOWS {
    ADDRINFOEX_VERSION_4 :: 4;
}

#if OS == .WINDOWS {
    ADDRINFOEX_VERSION_5 :: 5;
}

#if OS == .WINDOWS {
    ADDRINFOEX_VERSION_6 :: 6;
}

#if OS == .WINDOWS {
    ADDRINFOEX_VERSION_7 :: 7;
}

#if OS == .WINDOWS {
    ADDRINFOEXA :: addrinfoexA;
}

#if OS == .WINDOWS {
    ADDRINFOEXW :: addrinfoexW;
}

#if OS == .WINDOWS {
    ADDRINFOT :: ADDRINFOA;
}

#if OS == .WINDOWS {
    ADDRINFOW :: addrinfoW;
}

#if OS == .WINDOWS {
    AF_12844 :: 25;
}

#if OS == .LINUX {
    AF_ALG :: PF_ALG;
}

#if OS == .ANDROID {
    AF_ALG :: 38;
}

#if OS == .WINDOWS {
    AF_APPLETALK :: 16;
}

#if OS == .LINUX {
    AF_APPLETALK :: PF_APPLETALK;
}

#if OS == .ANDROID {
    AF_APPLETALK :: 5;
}

#if OS == .MACOS {
    AF_APPLETALK :: 16;
}

#if OS == .LINUX {
    AF_ASH :: PF_ASH;
}

#if OS == .ANDROID {
    AF_ASH :: 18;
}

#if OS == .WINDOWS {
    AF_ATM :: 22;
}

#if OS == .LINUX {
    AF_ATMPVC :: PF_ATMPVC;
}

#if OS == .ANDROID {
    AF_ATMPVC :: 8;
}

#if OS == .LINUX {
    AF_ATMSVC :: PF_ATMSVC;
}

#if OS == .ANDROID {
    AF_ATMSVC :: 20;
}

#if OS == .LINUX {
    AF_AX25 :: PF_AX25;
}

#if OS == .ANDROID {
    AF_AX25 :: 3;
}

#if OS == .WINDOWS {
    AF_BAN :: 21;
}

#if OS == .LINUX {
    AF_BLUETOOTH :: PF_BLUETOOTH;
}

#if OS == .ANDROID {
    AF_BLUETOOTH :: 31;
}

#if OS == .LINUX {
    AF_BRIDGE :: PF_BRIDGE;
}

#if OS == .ANDROID {
    AF_BRIDGE :: 7;
}

#if OS == .WINDOWS {
    AF_BTH :: 32;
}

#if OS == .LINUX {
    AF_CAIF :: PF_CAIF;
}

#if OS == .ANDROID {
    AF_CAIF :: 37;
}

#if OS == .LINUX {
    AF_CAN :: PF_CAN;
}

#if OS == .ANDROID {
    AF_CAN :: 29;
}

#if OS == .WINDOWS {
    AF_CCITT :: 10;
}

#if OS == .MACOS {
    AF_CCITT :: 10;
}

#if OS == .WINDOWS {
    AF_CHAOS :: 5;
}

#if OS == .MACOS {
    AF_CHAOS :: 5;
}

#if OS == .WINDOWS {
    AF_CLUSTER :: 24;
}

#if OS == .MACOS {
    AF_CNT :: 21;
}

#if OS == .MACOS {
    AF_COIP :: 20;
}

#if OS == .WINDOWS {
    AF_DATAKIT :: 9;
}

#if OS == .MACOS {
    AF_DATAKIT :: 9;
}

#if OS == .WINDOWS {
    AF_DECnet :: 12;
}

#if OS == .LINUX {
    AF_DECnet :: PF_DECnet;
}

#if OS == .ANDROID {
    AF_DECnet :: 12;
}

#if OS == .MACOS {
    AF_DECnet :: 12;
}

#if OS == .WINDOWS {
    AF_DLI :: 13;
}

#if OS == .MACOS {
    AF_DLI :: 13;
}

#if OS == .MACOS {
    AF_E164 :: AF_ISDN;
}

#if OS == .WINDOWS {
    AF_ECMA :: 8;
}

#if OS == .MACOS {
    AF_ECMA :: 8;
}

#if OS == .LINUX {
    AF_ECONET :: PF_ECONET;
}

#if OS == .ANDROID {
    AF_ECONET :: 19;
}

#if OS == .LINUX {
    AF_FILE :: PF_FILE;
}

#if OS == .WINDOWS {
    AF_FIREFOX :: 19;
}

#if OS == .WINDOWS {
    AF_HYLINK :: 15;
}

#if OS == .MACOS {
    AF_HYLINK :: 15;
}

#if OS == .WINDOWS {
    AF_HYPERV :: 34;
}

#if OS == .LINUX {
    AF_IB :: PF_IB;
}

#if OS == .WINDOWS {
    AF_ICLFXBM :: 31;
}

#if OS == .MACOS {
    AF_IEEE80211 :: 37;
}

#if OS == .LINUX {
    AF_IEEE802154 :: PF_IEEE802154;
}

#if OS == .ANDROID {
    AF_IEEE802154 :: 36;
}

#if OS == .WINDOWS {
    AF_IMPLINK :: 3;
}

#if OS == .MACOS {
    AF_IMPLINK :: 3;
}

#if OS == .WINDOWS {
    AF_INET :: 2;
}

#if OS == .LINUX {
    AF_INET :: PF_INET;
}

#if OS == .ANDROID {
    AF_INET :: 2;
}

#if OS == .MACOS {
    AF_INET :: 2;
}

#if OS == .WINDOWS {
    AF_INET6 :: 23;
}

#if OS == .LINUX {
    AF_INET6 :: PF_INET6;
}

#if OS == .ANDROID {
    AF_INET6 :: 10;
}

#if OS == .MACOS {
    AF_INET6 :: 30;
}

#if OS == .WINDOWS {
    AF_IPX :: AF_NS;
}

#if OS == .LINUX {
    AF_IPX :: PF_IPX;
}

#if OS == .ANDROID {
    AF_IPX :: 4;
}

#if OS == .MACOS {
    AF_IPX :: 23;
}

#if OS == .WINDOWS {
    AF_IRDA :: 26;
}

#if OS == .LINUX {
    AF_IRDA :: PF_IRDA;
}

#if OS == .ANDROID {
    AF_IRDA :: 23;
}

#if OS == .LINUX {
    AF_ISDN :: PF_ISDN;
}

#if OS == .ANDROID {
    AF_ISDN :: 34;
}

#if OS == .MACOS {
    AF_ISDN :: 28;
}

#if OS == .WINDOWS {
    AF_ISO :: 7;
}

#if OS == .MACOS {
    AF_ISO :: 7;
}

#if OS == .LINUX {
    AF_IUCV :: PF_IUCV;
}

#if OS == .ANDROID {
    AF_IUCV :: 32;
}

#if OS == .LINUX {
    AF_KCM :: PF_KCM;
}

#if OS == .ANDROID {
    AF_KCM :: 41;
}

#if OS == .LINUX {
    AF_KEY :: PF_KEY;
}

#if OS == .ANDROID {
    AF_KEY :: 15;
}

#if OS == .WINDOWS {
    AF_LAT :: 14;
}

#if OS == .MACOS {
    AF_LAT :: 14;
}

#if OS == .WINDOWS {
    AF_LINK :: 33;
}

#if OS == .MACOS {
    AF_LINK :: 18;
}

#if OS == .LINUX {
    AF_LLC :: PF_LLC;
}

#if OS == .ANDROID {
    AF_LLC :: 26;
}

#if OS == .LINUX {
    AF_LOCAL :: PF_LOCAL;
}

#if OS == .ANDROID {
    AF_LOCAL :: 1;
}

#if OS == .MACOS {
    AF_LOCAL :: AF_UNIX;
}

#if OS == .WINDOWS {
    AF_MAX :: 35;
}

#if OS == .LINUX {
    AF_MAX :: PF_MAX;
}

#if OS == .ANDROID {
    AF_MAX :: 43;
}

#if OS == .MACOS {
    AF_MAX :: 40;
}

#if OS == .LINUX {
    AF_MPLS :: PF_MPLS;
}

#if OS == .MACOS {
    AF_NATM :: 31;
}

#if OS == .MACOS {
    AF_NDRV :: 27;
}

#if OS == .LINUX {
    AF_NETBEUI :: PF_NETBEUI;
}

#if OS == .ANDROID {
    AF_NETBEUI :: 13;
}

#if OS == .WINDOWS {
    AF_NETBIOS :: 17;
}

#if OS == .MACOS {
    AF_NETBIOS :: 33;
}

#if OS == .WINDOWS {
    AF_NETDES :: 28;
}

#if OS == .LINUX {
    AF_NETLINK :: PF_NETLINK;
}

#if OS == .ANDROID {
    AF_NETLINK :: 16;
}

#if OS == .LINUX {
    AF_NETROM :: PF_NETROM;
}

#if OS == .ANDROID {
    AF_NETROM :: 6;
}

#if OS == .LINUX {
    AF_NFC :: PF_NFC;
}

#if OS == .ANDROID {
    AF_NFC :: 39;
}

#if OS == .WINDOWS {
    AF_NS :: 6;
}

#if OS == .MACOS {
    AF_NS :: 6;
}

#if OS == .WINDOWS {
    AF_OSI :: AF_ISO;
}

#if OS == .MACOS {
    AF_OSI :: AF_ISO;
}

#if OS == .LINUX {
    AF_PACKET :: PF_PACKET;
}

#if OS == .ANDROID {
    AF_PACKET :: 17;
}

#if OS == .LINUX {
    AF_PHONET :: PF_PHONET;
}

#if OS == .ANDROID {
    AF_PHONET :: 35;
}

#if OS == .MACOS {
    AF_PPP :: 34;
}

#if OS == .LINUX {
    AF_PPPOX :: PF_PPPOX;
}

#if OS == .ANDROID {
    AF_PPPOX :: 24;
}

#if OS == .WINDOWS {
    AF_PUP :: 4;
}

#if OS == .MACOS {
    AF_PUP :: 4;
}

#if OS == .LINUX {
    AF_QIPCRTR :: PF_QIPCRTR;
}

#if OS == .ANDROID {
    AF_QIPCRTR :: 42;
}

#if OS == .LINUX {
    AF_RDS :: PF_RDS;
}

#if OS == .ANDROID {
    AF_RDS :: 21;
}

#if OS == .MACOS {
    AF_RESERVED_36 :: 36;
}

#if OS == .LINUX {
    AF_ROSE :: PF_ROSE;
}

#if OS == .ANDROID {
    AF_ROSE :: 11;
}

#if OS == .LINUX {
    AF_ROUTE :: PF_ROUTE;
}

#if OS == .ANDROID {
    AF_ROUTE :: AF_NETLINK;
}

#if OS == .MACOS {
    AF_ROUTE :: 17;
}

#if OS == .LINUX {
    AF_RXRPC :: PF_RXRPC;
}

#if OS == .ANDROID {
    AF_RXRPC :: 33;
}

#if OS == .LINUX {
    AF_SECURITY :: PF_SECURITY;
}

#if OS == .ANDROID {
    AF_SECURITY :: 14;
}

#if OS == .MACOS {
    AF_SIP :: 24;
}

#if OS == .LINUX {
    AF_SMC :: PF_SMC;
}

#if OS == .WINDOWS {
    AF_SNA :: 11;
}

#if OS == .LINUX {
    AF_SNA :: PF_SNA;
}

#if OS == .ANDROID {
    AF_SNA :: 22;
}

#if OS == .MACOS {
    AF_SNA :: 11;
}

#if OS == .MACOS {
    AF_SYSTEM :: 32;
}

#if OS == .WINDOWS {
    AF_TCNMESSAGE :: 30;
}

#if OS == .WINDOWS {
    AF_TCNPROCESS :: 29;
}

#if OS == .LINUX {
    AF_TIPC :: PF_TIPC;
}

#if OS == .ANDROID {
    AF_TIPC :: 30;
}

#if OS == .WINDOWS {
    AF_UNIX :: 1;
}

#if OS == .LINUX {
    AF_UNIX :: PF_UNIX;
}

#if OS == .ANDROID {
    AF_UNIX :: 1;
}

#if OS == .MACOS {
    AF_UNIX :: 1;
}

#if OS == .WINDOWS {
    AF_UNKNOWN1 :: 20;
}

#if OS == .WINDOWS {
    AF_UNSPEC :: 0;
}

#if OS == .LINUX {
    AF_UNSPEC :: PF_UNSPEC;
}

#if OS == .ANDROID {
    AF_UNSPEC :: 0;
}

#if OS == .MACOS {
    AF_UNSPEC :: 0;
}

#if OS == .MACOS {
    AF_UTUN :: 38;
}

#if OS == .WINDOWS {
    AF_VOICEVIEW :: 18;
}

#if OS == .LINUX {
    AF_VSOCK :: PF_VSOCK;
}

#if OS == .ANDROID {
    AF_VSOCK :: 40;
}

#if OS == .LINUX {
    AF_WANPIPE :: PF_WANPIPE;
}

#if OS == .ANDROID {
    AF_WANPIPE :: 25;
}

#if OS == .LINUX {
    AF_X25 :: PF_X25;
}

#if OS == .ANDROID {
    AF_X25 :: 9;
}

#if OS == .LINUX {
    AF_XDP :: PF_XDP;
}

#if OS == .MACOS {
    APPLE_IF_FAM_BOND :: 14;
}

#if OS == .MACOS {
    APPLE_IF_FAM_DISC :: 8;
}

#if OS == .MACOS {
    APPLE_IF_FAM_ETHERNET :: 2;
}

#if OS == .MACOS {
    APPLE_IF_FAM_FAITH :: 11;
}

#if OS == .MACOS {
    APPLE_IF_FAM_FIREWIRE :: 13;
}

#if OS == .MACOS {
    APPLE_IF_FAM_GIF :: 10;
}

#if OS == .MACOS {
    APPLE_IF_FAM_LOOPBACK :: 1;
}

#if OS == .MACOS {
    APPLE_IF_FAM_MDECAP :: 9;
}

#if OS == .MACOS {
    APPLE_IF_FAM_PPP :: 6;
}

#if OS == .MACOS {
    APPLE_IF_FAM_PVC :: 7;
}

#if OS == .MACOS {
    APPLE_IF_FAM_SLIP :: 3;
}

#if OS == .MACOS {
    APPLE_IF_FAM_STF :: 12;
}

#if OS == .MACOS {
    APPLE_IF_FAM_TUN :: 4;
}

#if OS == .MACOS {
    APPLE_IF_FAM_VLAN :: 5;
}

#if OS == .WINDOWS {
    BASE_PROTOCOL :: 1;
}

#if OS == .WINDOWS {
    BIGENDIAN :: 0x0000;
}

#if OS != .WINDOWS && OS_IS_UNIX {
    BITS_PER_FD_SET_MASK :: size_of(FD_BITS_ELEMENT_TYPE) * 8;
}

#if OS == .WINDOWS {
    CF_ACCEPT :: 0x0000;
}

#if OS == .WINDOWS {
    CF_DEFER :: 0x0002;
}

#if OS == .WINDOWS {
    CF_REJECT :: 0x0001;
}

#if OS == .WINDOWS {
    CMSGHDR :: WSACMSGHDR;
}

#if OS == .MACOS {
    CONNECT_DATA_AUTHENTICATED :: 0x4;
}

#if OS == .MACOS {
    CONNECT_DATA_IDEMPOTENT :: 0x2;
}

#if OS == .MACOS {
    CONNECT_RESUME_ON_READ_WRITE :: 0x1;
}

#if OS == .LINUX {
    EAI_ADDRFAMILY :: -9;
}

#if OS == .ANDROID {
    EAI_ADDRFAMILY :: 1;
}

#if OS == .MACOS {
    EAI_ADDRFAMILY :: 1;
}

#if OS == .LINUX {
    EAI_AGAIN :: -3;
}

#if OS == .ANDROID {
    EAI_AGAIN :: 2;
}

#if OS == .MACOS {
    EAI_AGAIN :: 2;
}

#if OS == .LINUX {
    EAI_ALLDONE :: -103;
}

#if OS == .LINUX {
    EAI_BADFLAGS :: -1;
}

#if OS == .ANDROID {
    EAI_BADFLAGS :: 3;
}

#if OS == .MACOS {
    EAI_BADFLAGS :: 3;
}

#if OS == .ANDROID {
    EAI_BADHINTS :: 12;
}

#if OS == .MACOS {
    EAI_BADHINTS :: 12;
}

#if OS == .LINUX {
    EAI_CANCELED :: -101;
}

#if OS == .LINUX {
    EAI_FAIL :: -4;
}

#if OS == .ANDROID {
    EAI_FAIL :: 4;
}

#if OS == .MACOS {
    EAI_FAIL :: 4;
}

#if OS == .LINUX {
    EAI_FAMILY :: -6;
}

#if OS == .ANDROID {
    EAI_FAMILY :: 5;
}

#if OS == .MACOS {
    EAI_FAMILY :: 5;
}

#if OS == .LINUX {
    EAI_IDN_ENCODE :: -105;
}

#if OS == .LINUX {
    EAI_INPROGRESS :: -100;
}

#if OS == .LINUX {
    EAI_INTR :: -104;
}

#if OS == .ANDROID {
    EAI_MAX :: 15;
}

#if OS == .MACOS {
    EAI_MAX :: 15;
}

#if OS == .LINUX {
    EAI_MEMORY :: -10;
}

#if OS == .ANDROID {
    EAI_MEMORY :: 6;
}

#if OS == .MACOS {
    EAI_MEMORY :: 6;
}

#if OS == .LINUX {
    EAI_NODATA :: -5;
}

#if OS == .ANDROID {
    EAI_NODATA :: 7;
}

#if OS == .MACOS {
    EAI_NODATA :: 7;
}

#if OS == .LINUX {
    EAI_NONAME :: -2;
}

#if OS == .ANDROID {
    EAI_NONAME :: 8;
}

#if OS == .MACOS {
    EAI_NONAME :: 8;
}

#if OS == .LINUX {
    EAI_NOTCANCELED :: -102;
}

#if OS == .LINUX {
    EAI_OVERFLOW :: -12;
}

#if OS == .ANDROID {
    EAI_OVERFLOW :: 14;
}

#if OS == .MACOS {
    EAI_OVERFLOW :: 14;
}

#if OS == .ANDROID {
    EAI_PROTOCOL :: 13;
}

#if OS == .MACOS {
    EAI_PROTOCOL :: 13;
}

#if OS == .LINUX {
    EAI_SERVICE :: -8;
}

#if OS == .ANDROID {
    EAI_SERVICE :: 9;
}

#if OS == .MACOS {
    EAI_SERVICE :: 9;
}

#if OS == .LINUX {
    EAI_SOCKTYPE :: -7;
}

#if OS == .ANDROID {
    EAI_SOCKTYPE :: 10;
}

#if OS == .MACOS {
    EAI_SOCKTYPE :: 10;
}

#if OS == .LINUX {
    EAI_SYSTEM :: -11;
}

#if OS == .ANDROID {
    EAI_SYSTEM :: 11;
}

#if OS == .MACOS {
    EAI_SYSTEM :: 11;
}

#if OS == .WINDOWS {
    FD_ACCEPT :: 1 << FD_ACCEPT_BIT;
}

#if OS == .WINDOWS {
    FD_ACCEPT_BIT :: 3;
}

#if OS == .WINDOWS {
    FD_ADDRESS_LIST_CHANGE :: 1 << FD_ADDRESS_LIST_CHANGE_BIT;
}

#if OS == .WINDOWS {
    FD_ADDRESS_LIST_CHANGE_BIT :: 9;
}

#if OS == .WINDOWS {
    FD_ALL_EVENTS :: (1 << FD_MAX_EVENTS) - 1;
}

#if OS != .WINDOWS && OS_IS_UNIX {
    FD_BITS_ELEMENT_TYPE :: type_of(fd_set.fds_bits[0]);
}

#if OS == .WINDOWS {
    FD_CLOSE :: 1 << FD_CLOSE_BIT;
}

#if OS == .WINDOWS {
    FD_CLOSE_BIT :: 5;
}

#if OS == .WINDOWS {
    FD_CONNECT :: 1 << FD_CONNECT_BIT;
}

#if OS == .WINDOWS {
    FD_CONNECT_BIT :: 4;
}

#if OS == .WINDOWS {
    FD_GROUP_QOS :: 1 << FD_GROUP_QOS_BIT;
}

#if OS == .WINDOWS {
    FD_GROUP_QOS_BIT :: 7;
}

#if OS == .WINDOWS {
    FD_ISSET :: __WSAFDIsSet;
}

#if OS == .LINUX {
    fd_mask :: __fd_mask;
}

#if OS == .ANDROID {
    fd_mask :: u64;
}

#if OS == .MACOS {
    fd_mask :: s32;
}

#if OS == .WINDOWS {
    FD_MAX_EVENTS :: 10;
}

#if OS == .WINDOWS {
    FD_OOB :: 1 << FD_OOB_BIT;
}

#if OS == .WINDOWS {
    FD_OOB_BIT :: 2;
}

#if OS == .WINDOWS {
    FD_QOS :: 1 << FD_QOS_BIT;
}

#if OS == .WINDOWS {
    FD_QOS_BIT :: 6;
}

#if OS == .WINDOWS {
    FD_READ :: 1 << FD_READ_BIT;
}

#if OS == .WINDOWS {
    FD_READ_BIT :: 0;
}

#if OS == .WINDOWS {
    FD_ROUTING_INTERFACE_CHANGE :: 1 << FD_ROUTING_INTERFACE_CHANGE_BIT;
}

#if OS == .WINDOWS {
    FD_ROUTING_INTERFACE_CHANGE_BIT :: 8;
}

#if OS == .WINDOWS {
    FD_SETSIZE :: 64;
}

#if OS == .ANDROID {
    FD_SETSIZE :: 1024;
}

#if OS == .WINDOWS {
    FD_WRITE :: 1 << FD_WRITE_BIT;
}

#if OS == .WINDOWS {
    FD_WRITE_BIT :: 1;
}

#if OS == .WINDOWS {
    FLOWSPEC :: flowspec;
}

#if OS == .WINDOWS {
    FreeAddrInfo :: FreeAddrInfoA;
}

#if OS == .WINDOWS {
    FreeAddrInfoA :: freeaddrinfo;
}

#if OS == .WINDOWS {
    FreeAddrInfoExA :: FreeAddrInfoEx;
}

#if OS == .WINDOWS {
    FROM_PROTOCOL_INFO :: -1;
}

#if OS == .LINUX {
    GAI_NOWAIT :: 1;
}

#if OS == .WINDOWS {
    GAI_STRERROR_BUFFER_SIZE :: 1024;
}

#if OS == .LINUX {
    GAI_WAIT :: 0;
}

#if OS == .WINDOWS {
    GetAddrInfo :: GetAddrInfoA;
}

#if OS == .WINDOWS {
    GetAddrInfoA :: getaddrinfo;
}

#if OS == .WINDOWS {
    GetAddrInfoEx :: GetAddrInfoExA;
}

#if OS == .WINDOWS {
    GetNameInfo :: GetNameInfoA;
}

#if OS == .WINDOWS {
    GetNameInfoA :: getnameinfo;
}

#if OS == .WINDOWS {
    GROUP :: u32;
}

#if OS == .WINDOWS {
    GROUP_FILTER :: group_filter;
}

#if OS == .WINDOWS {
    GROUP_REQ :: group_req;
}

#if OS == .WINDOWS {
    GROUP_SOURCE_REQ :: group_source_req;
}

#if OS == .LINUX {
    HOST_NOT_FOUND :: 1;
}

#if OS == .ANDROID {
    HOST_NOT_FOUND :: 1;
}

#if OS == .MACOS {
    HOST_NOT_FOUND :: 1;
}

#if OS == .WINDOWS {
    HOSTENT :: hostent;
}

#if OS == .MACOS {
    ICMP6_FILTER :: 18;
}

#if OS == .WINDOWS {
    ICMP_ERROR_INFO :: icmp_error_info;
}

#if OS == .MACOS {
    ICMPV6CTL_ND6_ONLINKNSRFC4861 :: 50;
}

#if OS == .MACOS {
    IF_DATA_TIMEVAL :: timeval32;
}

#if OS == .MACOS {
    IF_MAXMTU :: 65535;
}

#if OS == .MACOS {
    IF_MINMTU :: 72;
}

#if OS == .LINUX {
    IF_NAMESIZE :: 16;
}

#if OS == .MACOS {
    IF_NAMESIZE :: 16;
}

#if OS == .MACOS {
    IF_WAKE_ON_MAGIC_PACKET :: 0x01;
}

#if OS == .MACOS {
    IFCAP_AV :: 0x00100;
}

#if OS == .MACOS {
    IFCAP_CSUM_PARTIAL :: 0x02000;
}

#if OS == .MACOS {
    IFCAP_CSUM_ZERO_INVERT :: 0x04000;
}

#if OS == .MACOS {
    IFCAP_HW_TIMESTAMP :: 0x00800;
}

#if OS == .MACOS {
    IFCAP_HWCSUM :: IFCAP_RXCSUM | IFCAP_TXCSUM;
}

#if OS == .MACOS {
    IFCAP_JUMBO_MTU :: 0x00010;
}

#if OS == .MACOS {
    IFCAP_LRO :: 0x00080;
}

#if OS == .MACOS {
    IFCAP_RXCSUM :: 0x00001;
}

#if OS == .MACOS {
    IFCAP_SKYWALK :: 0x00400;
}

#if OS == .MACOS {
    IFCAP_SW_TIMESTAMP :: 0x01000;
}

#if OS == .MACOS {
    IFCAP_TSO :: IFCAP_TSO4 | IFCAP_TSO6;
}

#if OS == .MACOS {
    IFCAP_TSO4 :: 0x00020;
}

#if OS == .MACOS {
    IFCAP_TSO6 :: 0x00040;
}

#if OS == .MACOS {
    IFCAP_TXCSUM :: 0x00002;
}

#if OS == .MACOS {
    IFCAP_TXSTATUS :: 0x00200;
}

#if OS == .MACOS {
    IFCAP_VALID :: IFCAP_HWCSUM | IFCAP_TSO | IFCAP_LRO | IFCAP_VLAN_MTU | IFCAP_VLAN_HWTAGGING | IFCAP_JUMBO_MTU | IFCAP_AV | IFCAP_TXSTATUS | IFCAP_SKYWALK | IFCAP_SW_TIMESTAMP | IFCAP_HW_TIMESTAMP | IFCAP_CSUM_PARTIAL | IFCAP_CSUM_ZERO_INVERT;
}

#if OS == .MACOS {
    IFCAP_VLAN_HWTAGGING :: 0x00008;
}

#if OS == .MACOS {
    IFCAP_VLAN_MTU :: 0x00004;
}

#if OS == .WINDOWS {
    IFF_BROADCAST :: 0x00000002;
}

#if OS == .WINDOWS {
    IFF_LOOPBACK :: 0x00000004;
}

#if OS == .WINDOWS {
    IFF_MULTICAST :: 0x00000010;
}

#if OS == .WINDOWS {
    IFF_POINTTOPOINT :: 0x00000008;
}

#if OS == .WINDOWS {
    IFF_UP :: 0x00000001;
}

#if OS == .LINUX {
    IFHWADDRLEN :: 6;
}

#if OS == .LINUX {
    IFNAMSIZ :: IF_NAMESIZE;
}

#if OS == .MACOS {
    IFNAMSIZ :: 16;
}

#if OS == .MACOS {
    IFNET_SLOWHZ :: 1;
}

#if OS == .MACOS {
    IFQ_MAXLEN :: 128;
}

#if OS == .MACOS {
    IFQ_TARGET_DELAY :: 10 * 1000 * 1000;
}

#if OS == .MACOS {
    IFQ_UPDATE_INTERVAL :: 100 * 1000 * 1000;
}

#if OS == .MACOS {
    IFRTYPE_FUNCTIONAL_CELLULAR :: 5;
}

#if OS == .MACOS {
    IFRTYPE_FUNCTIONAL_INTCOPROC :: 6;
}

#if OS == .MACOS {
    IFRTYPE_FUNCTIONAL_LAST :: 6;
}

#if OS == .MACOS {
    IFRTYPE_FUNCTIONAL_LOOPBACK :: 1;
}

#if OS == .MACOS {
    IFRTYPE_FUNCTIONAL_UNKNOWN :: 0;
}

#if OS == .MACOS {
    IFRTYPE_FUNCTIONAL_WIFI_AWDL :: 4;
}

#if OS == .MACOS {
    IFRTYPE_FUNCTIONAL_WIFI_INFRA :: 3;
}

#if OS == .MACOS {
    IFRTYPE_FUNCTIONAL_WIRED :: 2;
}

#if OS == .MACOS {
    IFSTATMAX :: 800;
}

#if OS == .WINDOWS {
    IMPLINK_HIGHEXPER :: 158;
}

#if OS == .WINDOWS {
    IMPLINK_IP :: 155;
}

#if OS == .WINDOWS {
    IMPLINK_LOWEXPER :: 156;
}

#if OS == .WINDOWS {
    IN4ADDR_ANY :: INADDR_ANY;
}

#if OS == .WINDOWS {
    IN4ADDR_BROADCAST :: INADDR_BROADCAST;
}

#if OS == .WINDOWS {
    IN4ADDR_LINKLOCALPREFIX_LENGTH :: 16;
}

#if OS == .WINDOWS {
    IN4ADDR_LOOPBACK :: 0x0100007f;
}

#if OS == .WINDOWS {
    IN4ADDR_LOOPBACKPREFIX_LENGTH :: 8;
}

#if OS == .WINDOWS {
    IN4ADDR_MULTICASTPREFIX_LENGTH :: 4;
}

#if OS == .WINDOWS {
    IN4ADDR_NONE :: INADDR_NONE;
}

#if OS == .WINDOWS {
    IN6_ADDR :: in6_addr;
}

#if OS == .WINDOWS {
    IN6_PKTINFO :: in6_pktinfo;
}

#if OS == .WINDOWS {
    IN6_PKTINFO_EX :: in6_pktinfo_ex;
}

#if OS == .WINDOWS {
    IN6ADDR_6TO4PREFIX_LENGTH :: 16;
}

#if OS == .WINDOWS {
    IN6ADDR_LINKLOCALPREFIX_LENGTH :: 64;
}

#if OS == .WINDOWS {
    IN6ADDR_MULTICASTPREFIX_LENGTH :: 8;
}

#if OS == .WINDOWS {
    IN6ADDR_SOLICITEDNODEMULTICASTPREFIX_LENGTH :: 104;
}

#if OS == .WINDOWS {
    IN6ADDR_TEREDOPREFIX_LENGTH :: 32;
}

#if OS == .WINDOWS {
    IN6ADDR_V4MAPPEDPREFIX_LENGTH :: 96;
}

#if OS == .WINDOWS {
    IN_ADDR :: in_addr;
}

#if OS == .WINDOWS {
    in_addr6 :: in6_addr;
}

#if OS == .LINUX {
    in_addr_t :: u32;
}

#if OS == .ANDROID {
    in_addr_t :: u32;
}

#if OS == .MACOS {
    in_addr_t :: u32;
}

#if OS == .WINDOWS {
    IN_CLASSA_HOST :: 0x00ffffff;
}

#if OS == .LINUX {
    IN_CLASSA_HOST :: 0xffffffff & ~IN_CLASSA_NET;
}

#if OS == .ANDROID {
    IN_CLASSA_HOST :: 0xffffffff & ~IN_CLASSA_NET;
}

#if OS == .MACOS {
    IN_CLASSA_HOST :: 0x00ffffff;
}

#if OS == .WINDOWS {
    IN_CLASSA_MAX :: 128;
}

#if OS == .LINUX {
    IN_CLASSA_MAX :: 128;
}

#if OS == .ANDROID {
    IN_CLASSA_MAX :: 128;
}

#if OS == .MACOS {
    IN_CLASSA_MAX :: 128;
}

#if OS == .WINDOWS {
    IN_CLASSA_NET :: 0xff000000;
}

#if OS == .LINUX {
    IN_CLASSA_NET :: 0xff000000;
}

#if OS == .ANDROID {
    IN_CLASSA_NET :: 0xff000000;
}

#if OS == .MACOS {
    IN_CLASSA_NET :: 0xff000000;
}

#if OS == .WINDOWS {
    IN_CLASSA_NSHIFT :: 24;
}

#if OS == .LINUX {
    IN_CLASSA_NSHIFT :: 24;
}

#if OS == .ANDROID {
    IN_CLASSA_NSHIFT :: 24;
}

#if OS == .MACOS {
    IN_CLASSA_NSHIFT :: 24;
}

#if OS == .WINDOWS {
    IN_CLASSB_HOST :: 0x0000ffff;
}

#if OS == .LINUX {
    IN_CLASSB_HOST :: 0xffffffff & ~IN_CLASSB_NET;
}

#if OS == .ANDROID {
    IN_CLASSB_HOST :: 0xffffffff & ~IN_CLASSB_NET;
}

#if OS == .MACOS {
    IN_CLASSB_HOST :: 0x0000ffff;
}

#if OS == .WINDOWS {
    IN_CLASSB_MAX :: 65536;
}

#if OS == .LINUX {
    IN_CLASSB_MAX :: 65536;
}

#if OS == .ANDROID {
    IN_CLASSB_MAX :: 65536;
}

#if OS == .MACOS {
    IN_CLASSB_MAX :: 65536;
}

#if OS == .WINDOWS {
    IN_CLASSB_NET :: 0xffff0000;
}

#if OS == .LINUX {
    IN_CLASSB_NET :: 0xffff0000;
}

#if OS == .ANDROID {
    IN_CLASSB_NET :: 0xffff0000;
}

#if OS == .MACOS {
    IN_CLASSB_NET :: 0xffff0000;
}

#if OS == .WINDOWS {
    IN_CLASSB_NSHIFT :: 16;
}

#if OS == .LINUX {
    IN_CLASSB_NSHIFT :: 16;
}

#if OS == .ANDROID {
    IN_CLASSB_NSHIFT :: 16;
}

#if OS == .MACOS {
    IN_CLASSB_NSHIFT :: 16;
}

#if OS == .WINDOWS {
    IN_CLASSC_HOST :: 0x000000ff;
}

#if OS == .LINUX {
    IN_CLASSC_HOST :: 0xffffffff & ~IN_CLASSC_NET;
}

#if OS == .ANDROID {
    IN_CLASSC_HOST :: 0xffffffff & ~IN_CLASSC_NET;
}

#if OS == .MACOS {
    IN_CLASSC_HOST :: 0x000000ff;
}

#if OS == .WINDOWS {
    IN_CLASSC_NET :: 0xffffff00;
}

#if OS == .LINUX {
    IN_CLASSC_NET :: 0xffffff00;
}

#if OS == .ANDROID {
    IN_CLASSC_NET :: 0xffffff00;
}

#if OS == .MACOS {
    IN_CLASSC_NET :: 0xffffff00;
}

#if OS == .WINDOWS {
    IN_CLASSC_NSHIFT :: 8;
}

#if OS == .LINUX {
    IN_CLASSC_NSHIFT :: 8;
}

#if OS == .ANDROID {
    IN_CLASSC_NSHIFT :: 8;
}

#if OS == .MACOS {
    IN_CLASSC_NSHIFT :: 8;
}

#if OS == .WINDOWS {
    IN_CLASSD_HOST :: 0x0fffffff;
}

#if OS == .MACOS {
    IN_CLASSD_HOST :: 0x0fffffff;
}

#if OS == .WINDOWS {
    IN_CLASSD_NET :: 0xf0000000;
}

#if OS == .MACOS {
    IN_CLASSD_NET :: 0xf0000000;
}

#if OS == .WINDOWS {
    IN_CLASSD_NSHIFT :: 28;
}

#if OS == .MACOS {
    IN_CLASSD_NSHIFT :: 28;
}

#if OS == .ANDROID {
    IN_CLASSE_NET :: 0xffffffff;
}

#if OS == .ANDROID {
    IN_CLASSE_NSHIFT :: 0;
}

#if OS == .MACOS {
    IN_LINKLOCALNETNUM :: cast(u32) 0xA9FE0000;
}

#if OS == .LINUX {
    IN_LOOPBACKNET :: 127;
}

#if OS == .ANDROID {
    IN_LOOPBACKNET :: 127;
}

#if OS == .MACOS {
    IN_LOOPBACKNET :: 127;
}

#if OS == .ANDROID {
    IN_MULTICAST_NET :: 0xe0000000;
}

#if OS == .WINDOWS {
    IN_PKTINFO :: in_pktinfo;
}

#if OS == .WINDOWS {
    IN_PKTINFO_EX :: in_pktinfo_ex;
}

#if OS == .LINUX {
    in_port_t :: u16;
}

#if OS == .ANDROID {
    in_port_t :: u16;
}

#if OS == .MACOS {
    in_port_t :: u16;
}

#if OS == .WINDOWS {
    IN_RECVERR :: in_recverr;
}

#if OS == .LINUX {
    INADDR_ALLHOSTS_GROUP :: cast(in_addr_t) 0xe0000001;
}

#if OS == .ANDROID {
    INADDR_ALLHOSTS_GROUP :: 0xe0000001;
}

#if OS == .MACOS {
    INADDR_ALLHOSTS_GROUP :: cast(u32) 0xe0000001;
}

#if OS == .MACOS {
    INADDR_ALLMDNS_GROUP :: cast(u32) 0xe00000fb;
}

#if OS == .MACOS {
    INADDR_ALLRPTS_GROUP :: cast(u32) 0xe0000016;
}

#if OS == .LINUX {
    INADDR_ALLRTRS_GROUP :: cast(in_addr_t) 0xe0000002;
}

#if OS == .ANDROID {
    INADDR_ALLRTRS_GROUP :: 0xe0000002;
}

#if OS == .MACOS {
    INADDR_ALLRTRS_GROUP :: cast(u32) 0xe0000002;
}

#if OS == .LINUX {
    INADDR_ALLSNOOPERS_GROUP :: cast(in_addr_t) 0xe000006a;
}

#if OS == .ANDROID {
    INADDR_ALLSNOOPERS_GROUP :: 0xe000006a;
}

#if OS == .WINDOWS {
    INADDR_ANY :: cast(u32) 0x00000000;
}

#if OS == .LINUX {
    INADDR_ANY :: cast(in_addr_t) 0x00000000;
}

#if OS == .MACOS {
    INADDR_ANY :: cast(u32) 0x00000000;
}

#if OS == .WINDOWS {
    INADDR_BROADCAST :: cast(u32) 0xffffffff;
}

#if OS == .LINUX {
    INADDR_BROADCAST :: cast(in_addr_t) 0xffffffff;
}

#if OS == .MACOS {
    INADDR_BROADCAST :: cast(u32) 0xffffffff;
}

#if OS == .MACOS {
    INADDR_CARP_GROUP :: cast(u32) 0xe0000012;
}

#if OS == .WINDOWS {
    INADDR_LOOPBACK :: 0x7f000001;
}

#if OS == .LINUX {
    INADDR_LOOPBACK :: cast(in_addr_t) 0x7f000001;
}

#if OS == .ANDROID {
    INADDR_LOOPBACK :: 0x7f000001;
}

#if OS == .MACOS {
    INADDR_LOOPBACK :: cast(u32) 0x7f000001;
}

#if OS == .LINUX {
    INADDR_MAX_LOCAL_GROUP :: cast(in_addr_t) 0xe00000ff;
}

#if OS == .ANDROID {
    INADDR_MAX_LOCAL_GROUP :: 0xe00000ff;
}

#if OS == .MACOS {
    INADDR_MAX_LOCAL_GROUP :: cast(u32) 0xe00000ff;
}

#if OS == .WINDOWS {
    INADDR_NONE :: 0xffffffff;
}

#if OS == .LINUX {
    INADDR_NONE :: cast(in_addr_t) 0xffffffff;
}

#if OS == .MACOS {
    INADDR_NONE :: 0xffffffff;
}

#if OS == .MACOS {
    INADDR_PFSYNC_GROUP :: cast(u32) 0xe00000f0;
}

#if OS == .LINUX {
    INADDR_UNSPEC_GROUP :: cast(in_addr_t) 0xe0000000;
}

#if OS == .ANDROID {
    INADDR_UNSPEC_GROUP :: 0xe0000000;
}

#if OS == .MACOS {
    INADDR_UNSPEC_GROUP :: cast(u32) 0xe0000000;
}

#if OS == .WINDOWS {
    INCL_WINSOCK_API_PROTOTYPES :: 1;
}

#if OS == .WINDOWS {
    INCL_WINSOCK_API_TYPEDEFS :: 0;
}

#if OS == .WINDOWS {
    INET6_ADDRSTRLEN :: 65;
}

#if OS == .LINUX {
    INET6_ADDRSTRLEN :: 46;
}

#if OS == .ANDROID {
    INET6_ADDRSTRLEN :: 46;
}

#if OS == .MACOS {
    INET6_ADDRSTRLEN :: 46;
}

#if OS == .WINDOWS {
    INET_ADDRSTRLEN :: 22;
}

#if OS == .LINUX {
    INET_ADDRSTRLEN :: 16;
}

#if OS == .ANDROID {
    INET_ADDRSTRLEN :: 16;
}

#if OS == .MACOS {
    INET_ADDRSTRLEN :: 16;
}

#if OS == .WINDOWS {
    INET_PORT_RESERVATION :: INET_PORT_RANGE;
}

#if OS == .WINDOWS {
    InetNtop :: InetNtopA;
}

#if OS == .WINDOWS {
    InetNtopA :: inet_ntop;
}

#if OS == .WINDOWS {
    InetPton :: InetPtonA;
}

#if OS == .WINDOWS {
    InetPtonA :: inet_pton;
}

#if OS == .WINDOWS {
    INVALID_PORT_RESERVATION_TOKEN :: cast(u64) 0;
}

#if OS == .WINDOWS {
    IOC_IN :: 0x80000000;
}

#if OS == .WINDOWS {
    IOC_INOUT :: IOC_IN | IOC_OUT;
}

#if OS == .WINDOWS {
    IOC_OUT :: 0x40000000;
}

#if OS == .WINDOWS {
    IOC_PROTOCOL :: 0x10000000;
}

#if OS == .WINDOWS {
    IOC_UNIX :: 0x00000000;
}

#if OS == .WINDOWS {
    IOC_VENDOR :: 0x18000000;
}

#if OS == .WINDOWS {
    IOC_VOID :: 0x20000000;
}

#if OS == .WINDOWS {
    IOC_WS2 :: 0x08000000;
}

#if OS == .WINDOWS {
    IOC_WSK :: IOC_WS2 | 0x07000000;
}

#if OS == .WINDOWS {
    IOCPARM_MASK :: 0x7f;
}

#if OS == .WINDOWS {
    IP6T_SO_ORIGINAL_DST :: SO_ORIGINAL_DST;
}

#if OS == .WINDOWS {
    IP_ADD_IFLIST :: 29;
}

#if OS == .WINDOWS {
    IP_ADD_MEMBERSHIP :: 12;
}

#if OS == .LINUX {
    IP_ADD_MEMBERSHIP :: 35;
}

#if OS == .ANDROID {
    IP_ADD_MEMBERSHIP :: 35;
}

#if OS == .MACOS {
    IP_ADD_MEMBERSHIP :: 12;
}

#if OS == .WINDOWS {
    IP_ADD_SOURCE_MEMBERSHIP :: 15;
}

#if OS == .LINUX {
    IP_ADD_SOURCE_MEMBERSHIP :: 39;
}

#if OS == .ANDROID {
    IP_ADD_SOURCE_MEMBERSHIP :: 39;
}

#if OS == .MACOS {
    IP_ADD_SOURCE_MEMBERSHIP :: 70;
}

#if OS == .LINUX {
    IP_BIND_ADDRESS_NO_PORT :: 24;
}

#if OS == .ANDROID {
    IP_BIND_ADDRESS_NO_PORT :: 24;
}

#if OS == .WINDOWS {
    IP_BLOCK_SOURCE :: 17;
}

#if OS == .LINUX {
    IP_BLOCK_SOURCE :: 38;
}

#if OS == .ANDROID {
    IP_BLOCK_SOURCE :: 38;
}

#if OS == .MACOS {
    IP_BLOCK_SOURCE :: 72;
}

#if OS == .MACOS {
    IP_BOUND_IF :: 25;
}

#if OS == .LINUX {
    IP_CHECKSUM :: 23;
}

#if OS == .ANDROID {
    IP_CHECKSUM :: 23;
}

#if OS == .LINUX {
    IP_DEFAULT_MULTICAST_LOOP :: 1;
}

#if OS == .ANDROID {
    IP_DEFAULT_MULTICAST_LOOP :: 1;
}

#if OS == .MACOS {
    IP_DEFAULT_MULTICAST_LOOP :: 1;
}

#if OS == .LINUX {
    IP_DEFAULT_MULTICAST_TTL :: 1;
}

#if OS == .ANDROID {
    IP_DEFAULT_MULTICAST_TTL :: 1;
}

#if OS == .MACOS {
    IP_DEFAULT_MULTICAST_TTL :: 1;
}

#if OS == .WINDOWS {
    IP_DEL_IFLIST :: 30;
}

#if OS == .WINDOWS {
    IP_DONTFRAGMENT :: 14;
}

#if OS == .WINDOWS {
    IP_DROP_MEMBERSHIP :: 13;
}

#if OS == .LINUX {
    IP_DROP_MEMBERSHIP :: 36;
}

#if OS == .ANDROID {
    IP_DROP_MEMBERSHIP :: 36;
}

#if OS == .MACOS {
    IP_DROP_MEMBERSHIP :: 13;
}

#if OS == .WINDOWS {
    IP_DROP_SOURCE_MEMBERSHIP :: 16;
}

#if OS == .LINUX {
    IP_DROP_SOURCE_MEMBERSHIP :: 40;
}

#if OS == .ANDROID {
    IP_DROP_SOURCE_MEMBERSHIP :: 40;
}

#if OS == .MACOS {
    IP_DROP_SOURCE_MEMBERSHIP :: 71;
}

#if OS == .MACOS {
    IP_DUMMYNET_CONFIGURE :: 60;
}

#if OS == .MACOS {
    IP_DUMMYNET_DEL :: 61;
}

#if OS == .MACOS {
    IP_DUMMYNET_FLUSH :: 62;
}

#if OS == .MACOS {
    IP_DUMMYNET_GET :: 64;
}

#if OS == .WINDOWS {
    IP_ECN :: 50;
}

#if OS == .MACOS {
    IP_FAITH :: 22;
}

#if OS == .LINUX {
    IP_FREEBIND :: 15;
}

#if OS == .ANDROID {
    IP_FREEBIND :: 15;
}

#if OS == .MACOS {
    IP_FW_ADD :: 40;
}

#if OS == .MACOS {
    IP_FW_DEL :: 41;
}

#if OS == .MACOS {
    IP_FW_FLUSH :: 42;
}

#if OS == .MACOS {
    IP_FW_GET :: 44;
}

#if OS == .MACOS {
    IP_FW_RESETLOG :: 45;
}

#if OS == .MACOS {
    IP_FW_ZERO :: 43;
}

#if OS == .WINDOWS {
    IP_GET_IFLIST :: 33;
}

#if OS == .WINDOWS {
    IP_HDRINCL :: 2;
}

#if OS == .LINUX {
    IP_HDRINCL :: 3;
}

#if OS == .ANDROID {
    IP_HDRINCL :: 3;
}

#if OS == .MACOS {
    IP_HDRINCL :: 2;
}

#if OS == .WINDOWS {
    IP_HOPLIMIT :: 21;
}

#if OS == .WINDOWS {
    IP_IFLIST :: 28;
}

#if OS == .LINUX {
    IP_IPSEC_POLICY :: 16;
}

#if OS == .ANDROID {
    IP_IPSEC_POLICY :: 16;
}

#if OS == .MACOS {
    IP_IPSEC_POLICY :: 21;
}

#if OS == .ANDROID {
    IP_LOCAL_PORT_RANGE :: 51;
}

#if OS == .MACOS {
    IP_MAX_GROUP_SRC_FILTER :: 512;
}

#if OS == .LINUX {
    IP_MAX_MEMBERSHIPS :: 20;
}

#if OS == .MACOS {
    IP_MAX_MEMBERSHIPS :: 4095;
}

#if OS == .MACOS {
    IP_MAX_SOCK_MUTE_FILTER :: 128;
}

#if OS == .MACOS {
    IP_MAX_SOCK_SRC_FILTER :: 128;
}

#if OS == .MACOS {
    IP_MIN_MEMBERSHIPS :: 31;
}

#if OS == .LINUX {
    IP_MINTTL :: 21;
}

#if OS == .ANDROID {
    IP_MINTTL :: 21;
}

#if OS == .WINDOWS {
    IP_MREQ :: ip_mreq;
}

#if OS == .WINDOWS {
    IP_MREQ_SOURCE :: ip_mreq_source;
}

#if OS == .WINDOWS {
    IP_MSFILTER :: ip_msfilter;
}

#if OS == .LINUX {
    IP_MSFILTER :: 41;
}

#if OS == .ANDROID {
    IP_MSFILTER :: 41;
}

#if OS == .MACOS {
    IP_MSFILTER :: 74;
}

#if OS == .WINDOWS {
    IP_MTU :: 73;
}

#if OS == .LINUX {
    IP_MTU :: 14;
}

#if OS == .ANDROID {
    IP_MTU :: 14;
}

#if OS == .WINDOWS {
    IP_MTU_DISCOVER :: 71;
}

#if OS == .LINUX {
    IP_MTU_DISCOVER :: 10;
}

#if OS == .ANDROID {
    IP_MTU_DISCOVER :: 10;
}

#if OS == .LINUX {
    IP_MULTICAST_ALL :: 49;
}

#if OS == .ANDROID {
    IP_MULTICAST_ALL :: 49;
}

#if OS == .WINDOWS {
    IP_MULTICAST_IF :: 9;
}

#if OS == .LINUX {
    IP_MULTICAST_IF :: 32;
}

#if OS == .ANDROID {
    IP_MULTICAST_IF :: 32;
}

#if OS == .MACOS {
    IP_MULTICAST_IF :: 9;
}

#if OS == .MACOS {
    IP_MULTICAST_IFINDEX :: 66;
}

#if OS == .WINDOWS {
    IP_MULTICAST_LOOP :: 11;
}

#if OS == .LINUX {
    IP_MULTICAST_LOOP :: 34;
}

#if OS == .ANDROID {
    IP_MULTICAST_LOOP :: 34;
}

#if OS == .MACOS {
    IP_MULTICAST_LOOP :: 11;
}

#if OS == .WINDOWS {
    IP_MULTICAST_TTL :: 10;
}

#if OS == .LINUX {
    IP_MULTICAST_TTL :: 33;
}

#if OS == .ANDROID {
    IP_MULTICAST_TTL :: 33;
}

#if OS == .MACOS {
    IP_MULTICAST_TTL :: 10;
}

#if OS == .MACOS {
    IP_MULTICAST_VIF :: 14;
}

#if OS == .MACOS {
    IP_NAT__XXX :: 55;
}

#if OS == .LINUX {
    IP_NODEFRAG :: 22;
}

#if OS == .ANDROID {
    IP_NODEFRAG :: 22;
}

#if OS == .WINDOWS {
    IP_NRT_INTERFACE :: 74;
}

#if OS == .MACOS {
    IP_OLD_FW_ADD :: 50;
}

#if OS == .MACOS {
    IP_OLD_FW_DEL :: 51;
}

#if OS == .MACOS {
    IP_OLD_FW_FLUSH :: 52;
}

#if OS == .MACOS {
    IP_OLD_FW_GET :: 54;
}

#if OS == .MACOS {
    IP_OLD_FW_RESETLOG :: 56;
}

#if OS == .MACOS {
    IP_OLD_FW_ZERO :: 53;
}

#if OS == .WINDOWS {
    IP_OPTIONS :: 1;
}

#if OS == .LINUX {
    IP_OPTIONS :: 4;
}

#if OS == .ANDROID {
    IP_OPTIONS :: 4;
}

#if OS == .MACOS {
    IP_OPTIONS :: 1;
}

#if OS == .LINUX {
    IP_ORIGDSTADDR :: 20;
}

#if OS == .ANDROID {
    IP_ORIGDSTADDR :: 20;
}

#if OS == .WINDOWS {
    IP_ORIGINAL_ARRIVAL_IF :: 47;
}

#if OS == .LINUX {
    IP_PASSSEC :: 18;
}

#if OS == .ANDROID {
    IP_PASSSEC :: 18;
}

#if OS == .WINDOWS {
    IP_PKTINFO :: 19;
}

#if OS == .LINUX {
    IP_PKTINFO :: 8;
}

#if OS == .ANDROID {
    IP_PKTINFO :: 8;
}

#if OS == .MACOS {
    IP_PKTINFO :: 26;
}

#if OS == .WINDOWS {
    IP_PKTINFO_EX :: 51;
}

#if OS == .LINUX {
    IP_PKTOPTIONS :: 9;
}

#if OS == .ANDROID {
    IP_PKTOPTIONS :: 9;
}

#if OS == .LINUX {
    IP_PMTUDISC :: 10;
}

#if OS == .LINUX {
    IP_PMTUDISC_DO :: 2;
}

#if OS == .ANDROID {
    IP_PMTUDISC_DO :: 2;
}

#if OS == .LINUX {
    IP_PMTUDISC_DONT :: 0;
}

#if OS == .ANDROID {
    IP_PMTUDISC_DONT :: 0;
}

#if OS == .LINUX {
    IP_PMTUDISC_INTERFACE :: 4;
}

#if OS == .ANDROID {
    IP_PMTUDISC_INTERFACE :: 4;
}

#if OS == .LINUX {
    IP_PMTUDISC_OMIT :: 5;
}

#if OS == .ANDROID {
    IP_PMTUDISC_OMIT :: 5;
}

#if OS == .LINUX {
    IP_PMTUDISC_PROBE :: 3;
}

#if OS == .ANDROID {
    IP_PMTUDISC_PROBE :: 3;
}

#if OS == .LINUX {
    IP_PMTUDISC_WANT :: 1;
}

#if OS == .ANDROID {
    IP_PMTUDISC_WANT :: 1;
}

#if OS == .MACOS {
    IP_PORTRANGE :: 19;
}

#if OS == .MACOS {
    IP_PORTRANGE_DEFAULT :: 0;
}

#if OS == .MACOS {
    IP_PORTRANGE_HIGH :: 1;
}

#if OS == .MACOS {
    IP_PORTRANGE_LOW :: 2;
}

#if OS == .WINDOWS {
    IP_PROTECTION_LEVEL :: IPV6_PROTECTION_LEVEL;
}

#if OS == .WINDOWS {
    IP_RECEIVE_BROADCAST :: 22;
}

#if OS == .WINDOWS {
    IP_RECVDSTADDR :: 25;
}

#if OS == .MACOS {
    IP_RECVDSTADDR :: 7;
}

#if OS == .WINDOWS {
    IP_RECVECN :: 50;
}

#if OS == .WINDOWS {
    IP_RECVERR :: 75;
}

#if OS == .LINUX {
    IP_RECVERR :: 11;
}

#if OS == .ANDROID {
    IP_RECVERR :: 11;
}

#if OS == .ANDROID {
    IP_RECVERR_RFC4884 :: 26;
}

#if OS == .LINUX {
    IP_RECVFRAGSIZE :: 25;
}

#if OS == .ANDROID {
    IP_RECVFRAGSIZE :: 25;
}

#if OS == .WINDOWS {
    IP_RECVIF :: 24;
}

#if OS == .MACOS {
    IP_RECVIF :: 20;
}

#if OS == .LINUX {
    IP_RECVOPTS :: 6;
}

#if OS == .ANDROID {
    IP_RECVOPTS :: 6;
}

#if OS == .MACOS {
    IP_RECVOPTS :: 5;
}

#if OS == .LINUX {
    IP_RECVORIGDSTADDR :: IP_ORIGDSTADDR;
}

#if OS == .ANDROID {
    IP_RECVORIGDSTADDR :: IP_ORIGDSTADDR;
}

#if OS == .MACOS {
    IP_RECVPKTINFO :: IP_PKTINFO;
}

#if OS == .LINUX {
    IP_RECVRETOPTS :: IP_RETOPTS;
}

#if OS == .ANDROID {
    IP_RECVRETOPTS :: IP_RETOPTS;
}

#if OS == .MACOS {
    IP_RECVRETOPTS :: 6;
}

#if OS == .WINDOWS {
    IP_RECVRTHDR :: 38;
}

#if OS == .WINDOWS {
    IP_RECVTCLASS :: 40;
}

#if OS == .WINDOWS {
    IP_RECVTOS :: 40;
}

#if OS == .LINUX {
    IP_RECVTOS :: 13;
}

#if OS == .ANDROID {
    IP_RECVTOS :: 13;
}

#if OS == .MACOS {
    IP_RECVTOS :: 27;
}

#if OS == .WINDOWS {
    IP_RECVTTL :: 21;
}

#if OS == .LINUX {
    IP_RECVTTL :: 12;
}

#if OS == .ANDROID {
    IP_RECVTTL :: 12;
}

#if OS == .MACOS {
    IP_RECVTTL :: 24;
}

#if OS == .LINUX {
    IP_RETOPTS :: 7;
}

#if OS == .ANDROID {
    IP_RETOPTS :: 7;
}

#if OS == .MACOS {
    IP_RETOPTS :: 8;
}

#if OS == .LINUX {
    IP_ROUTER_ALERT :: 5;
}

#if OS == .ANDROID {
    IP_ROUTER_ALERT :: 5;
}

#if OS == .MACOS {
    IP_RSVP_OFF :: 16;
}

#if OS == .MACOS {
    IP_RSVP_ON :: 15;
}

#if OS == .MACOS {
    IP_RSVP_VIF_OFF :: 18;
}

#if OS == .MACOS {
    IP_RSVP_VIF_ON :: 17;
}

#if OS == .WINDOWS {
    IP_RTHDR :: 32;
}

#if OS == .MACOS {
    IP_STRIPHDR :: 23;
}

#if OS == .WINDOWS {
    IP_TCLASS :: 39;
}

#if OS == .WINDOWS {
    IP_TOS :: 3;
}

#if OS == .LINUX {
    IP_TOS :: 1;
}

#if OS == .ANDROID {
    IP_TOS :: 1;
}

#if OS == .MACOS {
    IP_TOS :: 3;
}

#if OS == .MACOS {
    IP_TRAFFIC_MGT_BACKGROUND :: 65;
}

#if OS == .LINUX {
    IP_TRANSPARENT :: 19;
}

#if OS == .ANDROID {
    IP_TRANSPARENT :: 19;
}

#if OS == .WINDOWS {
    IP_TTL :: 4;
}

#if OS == .LINUX {
    IP_TTL :: 2;
}

#if OS == .ANDROID {
    IP_TTL :: 2;
}

#if OS == .MACOS {
    IP_TTL :: 4;
}

#if OS == .WINDOWS {
    IP_UNBLOCK_SOURCE :: 18;
}

#if OS == .LINUX {
    IP_UNBLOCK_SOURCE :: 37;
}

#if OS == .ANDROID {
    IP_UNBLOCK_SOURCE :: 37;
}

#if OS == .MACOS {
    IP_UNBLOCK_SOURCE :: 73;
}

#if OS == .WINDOWS {
    IP_UNICAST_IF :: 31;
}

#if OS == .LINUX {
    IP_UNICAST_IF :: 50;
}

#if OS == .ANDROID {
    IP_UNICAST_IF :: 50;
}

#if OS == .WINDOWS {
    IP_UNSPECIFIED_HOP_LIMIT :: -1;
}

#if OS == .WINDOWS {
    IP_UNSPECIFIED_TYPE_OF_SERVICE :: -1;
}

#if OS == .WINDOWS {
    IP_USER_MTU :: 76;
}

#if OS == .WINDOWS {
    IP_WFP_REDIRECT_CONTEXT :: 70;
}

#if OS == .WINDOWS {
    IP_WFP_REDIRECT_RECORDS :: 60;
}

#if OS == .LINUX {
    IP_XFRM_POLICY :: 17;
}

#if OS == .ANDROID {
    IP_XFRM_POLICY :: 17;
}

#if OS == .MACOS {
    IPCTL_ACCEPTSOURCEROUTE :: 13;
}

#if OS == .MACOS {
    IPCTL_DEFTTL :: 3;
}

#if OS == .MACOS {
    IPCTL_DIRECTEDBROADCAST :: 9;
}

#if OS == .MACOS {
    IPCTL_FASTFORWARDING :: 14;
}

#if OS == .MACOS {
    IPCTL_FORWARDING :: 1;
}

#if OS == .MACOS {
    IPCTL_GIF_TTL :: 16;
}

#if OS == .MACOS {
    IPCTL_INTRQDROPS :: 11;
}

#if OS == .MACOS {
    IPCTL_INTRQMAXLEN :: 10;
}

#if OS == .MACOS {
    IPCTL_KEEPFAITH :: 15;
}

#if OS == .MACOS {
    IPCTL_MAXID :: 17;
}

#if OS == .MACOS {
    IPCTL_RTEXPIRE :: 5;
}

#if OS == .MACOS {
    IPCTL_RTMAXCACHE :: 7;
}

#if OS == .MACOS {
    IPCTL_RTMINEXPIRE :: 6;
}

#if OS == .MACOS {
    IPCTL_SENDREDIRECTS :: 2;
}

#if OS == .MACOS {
    IPCTL_SOURCEROUTE :: 8;
}

#if OS == .MACOS {
    IPCTL_STATS :: 12;
}

#if OS == .WINDOWS {
    IPPORT_BIFFUDP :: 512;
}

#if OS == .WINDOWS {
    IPPORT_CHARGEN :: 19;
}

#if OS == .WINDOWS {
    IPPORT_CMDSERVER :: 514;
}

#if OS == .WINDOWS {
    IPPORT_DAYTIME :: 13;
}

#if OS == .WINDOWS {
    IPPORT_DISCARD :: 9;
}

#if OS == .WINDOWS {
    IPPORT_DYNAMIC_MAX :: 0xffff;
}

#if OS == .WINDOWS {
    IPPORT_DYNAMIC_MIN :: 0xc000;
}

#if OS == .WINDOWS {
    IPPORT_ECHO :: 7;
}

#if OS == .WINDOWS {
    IPPORT_EFSSERVER :: 520;
}

#if OS == .WINDOWS {
    IPPORT_EPMAP :: 135;
}

#if OS == .WINDOWS {
    IPPORT_EXECSERVER :: 512;
}

#if OS == .WINDOWS {
    IPPORT_FINGER :: 79;
}

#if OS == .WINDOWS {
    IPPORT_FTP :: 21;
}

#if OS == .WINDOWS {
    IPPORT_FTP_DATA :: 20;
}

#if OS == .MACOS {
    IPPORT_HIFIRSTAUTO :: 49152;
}

#if OS == .MACOS {
    IPPORT_HILASTAUTO :: 65535;
}

#if OS == .WINDOWS {
    IPPORT_HTTPS :: 443;
}

#if OS == .WINDOWS {
    IPPORT_IMAP :: 143;
}

#if OS == .WINDOWS {
    IPPORT_IMAP3 :: 220;
}

#if OS == .WINDOWS {
    IPPORT_LDAP :: 389;
}

#if OS == .WINDOWS {
    IPPORT_LOGINSERVER :: 513;
}

#if OS == .WINDOWS {
    IPPORT_MICROSOFT_DS :: 445;
}

#if OS == .WINDOWS {
    IPPORT_MSP :: 18;
}

#if OS == .WINDOWS {
    IPPORT_MTP :: 57;
}

#if OS == .WINDOWS {
    IPPORT_NAMESERVER :: 42;
}

#if OS == .WINDOWS {
    IPPORT_NETBIOS_DGM :: 138;
}

#if OS == .WINDOWS {
    IPPORT_NETBIOS_NS :: 137;
}

#if OS == .WINDOWS {
    IPPORT_NETBIOS_SSN :: 139;
}

#if OS == .WINDOWS {
    IPPORT_NETSTAT :: 15;
}

#if OS == .WINDOWS {
    IPPORT_NTP :: 123;
}

#if OS == .WINDOWS {
    IPPORT_POP3 :: 110;
}

#if OS == .WINDOWS {
    IPPORT_QOTD :: 17;
}

#if OS == .WINDOWS {
    IPPORT_REGISTERED_MAX :: 0xbfff;
}

#if OS == .WINDOWS {
    IPPORT_REGISTERED_MIN :: IPPORT_RESERVED;
}

#if OS == .WINDOWS {
    IPPORT_RESERVED :: 1024;
}

#if OS == .LINUX {
    IPPORT_RESERVED :: 1024;
}

#if OS == .ANDROID {
    IPPORT_RESERVED :: 1024;
}

#if OS == .MACOS {
    IPPORT_RESERVED :: __DARWIN_IPPORT_RESERVED;
}

#if OS == .MACOS {
    IPPORT_RESERVEDSTART :: 600;
}

#if OS == .WINDOWS {
    IPPORT_RJE :: 77;
}

#if OS == .WINDOWS {
    IPPORT_ROUTESERVER :: 520;
}

#if OS == .WINDOWS {
    IPPORT_SMTP :: 25;
}

#if OS == .WINDOWS {
    IPPORT_SNMP :: 161;
}

#if OS == .WINDOWS {
    IPPORT_SNMP_TRAP :: 162;
}

#if OS == .WINDOWS {
    IPPORT_SUPDUP :: 95;
}

#if OS == .WINDOWS {
    IPPORT_SYSTAT :: 11;
}

#if OS == .WINDOWS {
    IPPORT_TCPMUX :: 1;
}

#if OS == .WINDOWS {
    IPPORT_TELNET :: 23;
}

#if OS == .WINDOWS {
    IPPORT_TFTP :: 69;
}

#if OS == .WINDOWS {
    IPPORT_TIMESERVER :: 37;
}

#if OS == .WINDOWS {
    IPPORT_TTYLINK :: 87;
}

#if OS == .MACOS {
    IPPORT_USERRESERVED :: 5000;
}

#if OS == .WINDOWS {
    IPPORT_WHOIS :: 43;
}

#if OS == .WINDOWS {
    IPPORT_WHOSERVER :: 513;
}

#if OS == .WINDOWS {
    IPPROTO_IP :: 0;
}

#if OS == .LINUX {
    IPV6_2292DSTOPTS :: 4;
}

#if OS == .ANDROID {
    IPV6_2292DSTOPTS :: 4;
}

#if OS == .MACOS {
    IPV6_2292DSTOPTS :: 23;
}

#if OS == .LINUX {
    IPV6_2292HOPLIMIT :: 8;
}

#if OS == .ANDROID {
    IPV6_2292HOPLIMIT :: 8;
}

#if OS == .MACOS {
    IPV6_2292HOPLIMIT :: 20;
}

#if OS == .LINUX {
    IPV6_2292HOPOPTS :: 3;
}

#if OS == .ANDROID {
    IPV6_2292HOPOPTS :: 3;
}

#if OS == .MACOS {
    IPV6_2292HOPOPTS :: 22;
}

#if OS == .MACOS {
    IPV6_2292NEXTHOP :: 21;
}

#if OS == .LINUX {
    IPV6_2292PKTINFO :: 2;
}

#if OS == .ANDROID {
    IPV6_2292PKTINFO :: 2;
}

#if OS == .MACOS {
    IPV6_2292PKTINFO :: 19;
}

#if OS == .LINUX {
    IPV6_2292PKTOPTIONS :: 6;
}

#if OS == .ANDROID {
    IPV6_2292PKTOPTIONS :: 6;
}

#if OS == .MACOS {
    IPV6_2292PKTOPTIONS :: 25;
}

#if OS == .LINUX {
    IPV6_2292RTHDR :: 5;
}

#if OS == .ANDROID {
    IPV6_2292RTHDR :: 5;
}

#if OS == .MACOS {
    IPV6_2292RTHDR :: 24;
}

#if OS == .WINDOWS {
    IPV6_ADD_IFLIST :: 29;
}

#if OS == .WINDOWS {
    IPV6_ADD_MEMBERSHIP :: 12;
}

#if OS == .LINUX {
    IPV6_ADD_MEMBERSHIP :: IPV6_JOIN_GROUP;
}

#if OS == .ANDROID {
    IPV6_ADD_MEMBERSHIP :: 20;
}

#if OS == .LINUX {
    IPV6_ADDR_PREFERENCES :: 72;
}

#if OS == .ANDROID {
    IPV6_ADDR_PREFERENCES :: 72;
}

#if OS == .ANDROID {
    IPV6_ADDR_SCOPE_GLOBAL :: 0x0e;
}

#if OS == .ANDROID {
    IPV6_ADDR_SCOPE_INTFACELOCAL :: 0x01;
}

#if OS == .ANDROID {
    IPV6_ADDR_SCOPE_LINKLOCAL :: 0x02;
}

#if OS == .ANDROID {
    IPV6_ADDR_SCOPE_NODELOCAL :: 0x01;
}

#if OS == .ANDROID {
    IPV6_ADDR_SCOPE_ORGLOCAL :: 0x08;
}

#if OS == .ANDROID {
    IPV6_ADDR_SCOPE_SITELOCAL :: 0x05;
}

#if OS == .LINUX {
    IPV6_ADDRFORM :: 1;
}

#if OS == .ANDROID {
    IPV6_ADDRFORM :: 1;
}

#if OS == .LINUX {
    IPV6_AUTHHDR :: 10;
}

#if OS == .ANDROID {
    IPV6_AUTHHDR :: 10;
}

#if OS == .LINUX {
    IPV6_AUTOFLOWLABEL :: 70;
}

#if OS == .ANDROID {
    IPV6_AUTOFLOWLABEL :: 70;
}

#if OS == .MACOS {
    IPV6_BINDV6ONLY :: IPV6_V6ONLY;
}

#if OS == .MACOS {
    IPV6_BOUND_IF :: 125;
}

#if OS == .WINDOWS {
    IPV6_CHECKSUM :: 26;
}

#if OS == .LINUX {
    IPV6_CHECKSUM :: 7;
}

#if OS == .ANDROID {
    IPV6_CHECKSUM :: 7;
}

#if OS == .MACOS {
    IPV6_CHECKSUM :: 26;
}

#if OS == .MACOS {
    IPV6_DEFAULT_MULTICAST_HOPS :: 1;
}

#if OS == .MACOS {
    IPV6_DEFAULT_MULTICAST_LOOP :: 1;
}

#if OS == .WINDOWS {
    IPV6_DEL_IFLIST :: 30;
}

#if OS == .WINDOWS {
    IPV6_DONTFRAG :: 14;
}

#if OS == .LINUX {
    IPV6_DONTFRAG :: 62;
}

#if OS == .ANDROID {
    IPV6_DONTFRAG :: 62;
}

#if OS == .WINDOWS {
    IPV6_DROP_MEMBERSHIP :: 13;
}

#if OS == .LINUX {
    IPV6_DROP_MEMBERSHIP :: IPV6_LEAVE_GROUP;
}

#if OS == .ANDROID {
    IPV6_DROP_MEMBERSHIP :: 21;
}

#if OS == .LINUX {
    IPV6_DSTOPTS :: 59;
}

#if OS == .ANDROID {
    IPV6_DSTOPTS :: 59;
}

#if OS == .WINDOWS {
    IPV6_ECN :: 50;
}

#if OS == .MACOS {
    IPV6_FAITH :: 29;
}

#if OS == .ANDROID {
    IPV6_FL_A_GET :: 0;
}

#if OS == .ANDROID {
    IPV6_FL_A_PUT :: 1;
}

#if OS == .ANDROID {
    IPV6_FL_A_RENEW :: 2;
}

#if OS == .ANDROID {
    IPV6_FL_F_CREATE :: 1;
}

#if OS == .ANDROID {
    IPV6_FL_F_EXCL :: 2;
}

#if OS == .ANDROID {
    IPV6_FL_F_REFLECT :: 4;
}

#if OS == .ANDROID {
    IPV6_FL_F_REMOTE :: 8;
}

#if OS == .ANDROID {
    IPV6_FL_S_ANY :: 255;
}

#if OS == .ANDROID {
    IPV6_FL_S_EXCL :: 1;
}

#if OS == .ANDROID {
    IPV6_FL_S_NONE :: 0;
}

#if OS == .ANDROID {
    IPV6_FL_S_PROCESS :: 2;
}

#if OS == .ANDROID {
    IPV6_FL_S_USER :: 3;
}

#if OS == .ANDROID {
    IPV6_FLOWINFO :: 11;
}

#if OS == .ANDROID {
    IPV6_FLOWINFO_FLOWLABEL :: 0x000fffff;
}

#if OS == .ANDROID {
    IPV6_FLOWINFO_PRIORITY :: 0x0ff00000;
}

#if OS == .ANDROID {
    IPV6_FLOWINFO_SEND :: 33;
}

#if OS == .ANDROID {
    IPV6_FLOWLABEL_MGR :: 32;
}

#if OS == .LINUX {
    IPV6_FREEBIND :: 78;
}

#if OS == .ANDROID {
    IPV6_FREEBIND :: 78;
}

#if OS == .MACOS {
    IPV6_FW_ADD :: 30;
}

#if OS == .MACOS {
    IPV6_FW_DEL :: 31;
}

#if OS == .MACOS {
    IPV6_FW_FLUSH :: 32;
}

#if OS == .MACOS {
    IPV6_FW_GET :: 34;
}

#if OS == .MACOS {
    IPV6_FW_ZERO :: 33;
}

#if OS == .WINDOWS {
    IPV6_GET_IFLIST :: 33;
}

#if OS == .WINDOWS {
    IPV6_HDRINCL :: 2;
}

#if OS == .LINUX {
    IPV6_HDRINCL :: 36;
}

#if OS == .ANDROID {
    IPV6_HDRINCL :: 36;
}

#if OS == .WINDOWS {
    IPV6_HOPLIMIT :: 21;
}

#if OS == .LINUX {
    IPV6_HOPLIMIT :: 52;
}

#if OS == .ANDROID {
    IPV6_HOPLIMIT :: 52;
}

#if OS == .WINDOWS {
    IPV6_HOPOPTS :: 1;
}

#if OS == .LINUX {
    IPV6_HOPOPTS :: 54;
}

#if OS == .ANDROID {
    IPV6_HOPOPTS :: 54;
}

#if OS == .WINDOWS {
    IPV6_IFLIST :: 28;
}

#if OS == .LINUX {
    IPV6_IPSEC_POLICY :: 34;
}

#if OS == .ANDROID {
    IPV6_IPSEC_POLICY :: 34;
}

#if OS == .MACOS {
    IPV6_IPSEC_POLICY :: 28;
}

#if OS == .LINUX {
    IPV6_JOIN_ANYCAST :: 27;
}

#if OS == .ANDROID {
    IPV6_JOIN_ANYCAST :: 27;
}

#if OS == .WINDOWS {
    IPV6_JOIN_GROUP :: IPV6_ADD_MEMBERSHIP;
}

#if OS == .LINUX {
    IPV6_JOIN_GROUP :: 20;
}

#if OS == .ANDROID {
    IPV6_JOIN_GROUP :: IPV6_ADD_MEMBERSHIP;
}

#if OS == .MACOS {
    IPV6_JOIN_GROUP :: 12;
}

#if OS == .LINUX {
    IPV6_LEAVE_ANYCAST :: 28;
}

#if OS == .ANDROID {
    IPV6_LEAVE_ANYCAST :: 28;
}

#if OS == .WINDOWS {
    IPV6_LEAVE_GROUP :: IPV6_DROP_MEMBERSHIP;
}

#if OS == .LINUX {
    IPV6_LEAVE_GROUP :: 21;
}

#if OS == .ANDROID {
    IPV6_LEAVE_GROUP :: IPV6_DROP_MEMBERSHIP;
}

#if OS == .MACOS {
    IPV6_LEAVE_GROUP :: 13;
}

#if OS == .MACOS {
    IPV6_MAX_GROUP_SRC_FILTER :: 512;
}

#if OS == .MACOS {
    IPV6_MAX_MEMBERSHIPS :: 4095;
}

#if OS == .MACOS {
    IPV6_MAX_SOCK_SRC_FILTER :: 128;
}

#if OS == .MACOS {
    IPV6_MIN_MEMBERSHIPS :: 31;
}

#if OS == .LINUX {
    IPV6_MINHOPCOUNT :: 73;
}

#if OS == .ANDROID {
    IPV6_MINHOPCOUNT :: 73;
}

#if OS == .WINDOWS {
    IPV6_MREQ :: ipv6_mreq;
}

#if OS == .WINDOWS {
    IPV6_MTU :: 72;
}

#if OS == .LINUX {
    IPV6_MTU :: 24;
}

#if OS == .ANDROID {
    IPV6_MTU :: 24;
}

#if OS == .WINDOWS {
    IPV6_MTU_DISCOVER :: 71;
}

#if OS == .LINUX {
    IPV6_MTU_DISCOVER :: 23;
}

#if OS == .ANDROID {
    IPV6_MTU_DISCOVER :: 23;
}

#if OS == .LINUX {
    IPV6_MULTICAST_ALL :: 29;
}

#if OS == .ANDROID {
    IPV6_MULTICAST_ALL :: 29;
}

#if OS == .WINDOWS {
    IPV6_MULTICAST_HOPS :: 10;
}

#if OS == .LINUX {
    IPV6_MULTICAST_HOPS :: 18;
}

#if OS == .ANDROID {
    IPV6_MULTICAST_HOPS :: 18;
}

#if OS == .MACOS {
    IPV6_MULTICAST_HOPS :: 10;
}

#if OS == .WINDOWS {
    IPV6_MULTICAST_IF :: 9;
}

#if OS == .LINUX {
    IPV6_MULTICAST_IF :: 17;
}

#if OS == .ANDROID {
    IPV6_MULTICAST_IF :: 17;
}

#if OS == .MACOS {
    IPV6_MULTICAST_IF :: 9;
}

#if OS == .WINDOWS {
    IPV6_MULTICAST_LOOP :: 11;
}

#if OS == .LINUX {
    IPV6_MULTICAST_LOOP :: 19;
}

#if OS == .ANDROID {
    IPV6_MULTICAST_LOOP :: 19;
}

#if OS == .MACOS {
    IPV6_MULTICAST_LOOP :: 11;
}

#if OS == .LINUX {
    IPV6_NEXTHOP :: 9;
}

#if OS == .ANDROID {
    IPV6_NEXTHOP :: 9;
}

#if OS == .WINDOWS {
    IPV6_NRT_INTERFACE :: 74;
}

#if OS == .LINUX {
    IPV6_ORIGDSTADDR :: 74;
}

#if OS == .ANDROID {
    IPV6_ORIGDSTADDR :: 74;
}

#if OS == .LINUX {
    IPV6_PATHMTU :: 61;
}

#if OS == .ANDROID {
    IPV6_PATHMTU :: 61;
}

#if OS == .WINDOWS {
    IPV6_PKTINFO :: 19;
}

#if OS == .LINUX {
    IPV6_PKTINFO :: 50;
}

#if OS == .ANDROID {
    IPV6_PKTINFO :: 50;
}

#if OS == .WINDOWS {
    IPV6_PKTINFO_EX :: 51;
}

#if OS == .LINUX {
    IPV6_PMTUDISC_DO :: 2;
}

#if OS == .ANDROID {
    IPV6_PMTUDISC_DO :: 2;
}

#if OS == .LINUX {
    IPV6_PMTUDISC_DONT :: 0;
}

#if OS == .ANDROID {
    IPV6_PMTUDISC_DONT :: 0;
}

#if OS == .LINUX {
    IPV6_PMTUDISC_INTERFACE :: 4;
}

#if OS == .ANDROID {
    IPV6_PMTUDISC_INTERFACE :: 4;
}

#if OS == .LINUX {
    IPV6_PMTUDISC_OMIT :: 5;
}

#if OS == .ANDROID {
    IPV6_PMTUDISC_OMIT :: 5;
}

#if OS == .LINUX {
    IPV6_PMTUDISC_PROBE :: 3;
}

#if OS == .ANDROID {
    IPV6_PMTUDISC_PROBE :: 3;
}

#if OS == .LINUX {
    IPV6_PMTUDISC_WANT :: 1;
}

#if OS == .ANDROID {
    IPV6_PMTUDISC_WANT :: 1;
}

#if OS == .MACOS {
    IPV6_PORTRANGE :: 14;
}

#if OS == .MACOS {
    IPV6_PORTRANGE_DEFAULT :: 0;
}

#if OS == .MACOS {
    IPV6_PORTRANGE_HIGH :: 1;
}

#if OS == .MACOS {
    IPV6_PORTRANGE_LOW :: 2;
}

#if OS == .ANDROID {
    IPV6_PREFER_SRC_CGA :: 0x0008;
}

#if OS == .ANDROID {
    IPV6_PREFER_SRC_COA :: 0x0004;
}

#if OS == .ANDROID {
    IPV6_PREFER_SRC_HOME :: 0x0400;
}

#if OS == .ANDROID {
    IPV6_PREFER_SRC_NONCGA :: 0x0800;
}

#if OS == .ANDROID {
    IPV6_PREFER_SRC_PUBLIC :: 0x0002;
}

#if OS == .ANDROID {
    IPV6_PREFER_SRC_PUBTMP_DEFAULT :: 0x0100;
}

#if OS == .ANDROID {
    IPV6_PREFER_SRC_TMP :: 0x0001;
}

#if OS == .ANDROID {
    IPV6_PRIORITY_10 :: 0x0a00;
}

#if OS == .ANDROID {
    IPV6_PRIORITY_11 :: 0x0b00;
}

#if OS == .ANDROID {
    IPV6_PRIORITY_12 :: 0x0c00;
}

#if OS == .ANDROID {
    IPV6_PRIORITY_13 :: 0x0d00;
}

#if OS == .ANDROID {
    IPV6_PRIORITY_14 :: 0x0e00;
}

#if OS == .ANDROID {
    IPV6_PRIORITY_15 :: 0x0f00;
}

#if OS == .ANDROID {
    IPV6_PRIORITY_8 :: 0x0800;
}

#if OS == .ANDROID {
    IPV6_PRIORITY_9 :: 0x0900;
}

#if OS == .ANDROID {
    IPV6_PRIORITY_BULK :: 0x0400;
}

#if OS == .ANDROID {
    IPV6_PRIORITY_CONTROL :: 0x0700;
}

#if OS == .ANDROID {
    IPV6_PRIORITY_FILLER :: 0x0100;
}

#if OS == .ANDROID {
    IPV6_PRIORITY_INTERACTIVE :: 0x0600;
}

#if OS == .ANDROID {
    IPV6_PRIORITY_RESERVED1 :: 0x0300;
}

#if OS == .ANDROID {
    IPV6_PRIORITY_RESERVED2 :: 0x0500;
}

#if OS == .ANDROID {
    IPV6_PRIORITY_UNATTENDED :: 0x0200;
}

#if OS == .ANDROID {
    IPV6_PRIORITY_UNCHARACTERIZED :: 0x0000;
}

#if OS == .WINDOWS {
    IPV6_PROTECTION_LEVEL :: 23;
}

#if OS == .WINDOWS {
    IPV6_RECVDSTADDR :: 25;
}

#if OS == .LINUX {
    IPV6_RECVDSTOPTS :: 58;
}

#if OS == .ANDROID {
    IPV6_RECVDSTOPTS :: 58;
}

#if OS == .WINDOWS {
    IPV6_RECVECN :: 50;
}

#if OS == .WINDOWS {
    IPV6_RECVERR :: 75;
}

#if OS == .LINUX {
    IPV6_RECVERR :: 25;
}

#if OS == .ANDROID {
    IPV6_RECVERR :: 25;
}

#if OS == .ANDROID {
    IPV6_RECVERR_RFC4884 :: 31;
}

#if OS == .LINUX {
    IPV6_RECVFRAGSIZE :: 77;
}

#if OS == .ANDROID {
    IPV6_RECVFRAGSIZE :: 77;
}

#if OS == .LINUX {
    IPV6_RECVHOPLIMIT :: 51;
}

#if OS == .ANDROID {
    IPV6_RECVHOPLIMIT :: 51;
}

#if OS == .LINUX {
    IPV6_RECVHOPOPTS :: 53;
}

#if OS == .ANDROID {
    IPV6_RECVHOPOPTS :: 53;
}

#if OS == .WINDOWS {
    IPV6_RECVIF :: 24;
}

#if OS == .LINUX {
    IPV6_RECVORIGDSTADDR :: IPV6_ORIGDSTADDR;
}

#if OS == .ANDROID {
    IPV6_RECVORIGDSTADDR :: IPV6_ORIGDSTADDR;
}

#if OS == .LINUX {
    IPV6_RECVPATHMTU :: 60;
}

#if OS == .ANDROID {
    IPV6_RECVPATHMTU :: 60;
}

#if OS == .LINUX {
    IPV6_RECVPKTINFO :: 49;
}

#if OS == .ANDROID {
    IPV6_RECVPKTINFO :: 49;
}

#if OS == .WINDOWS {
    IPV6_RECVRTHDR :: 38;
}

#if OS == .LINUX {
    IPV6_RECVRTHDR :: 56;
}

#if OS == .ANDROID {
    IPV6_RECVRTHDR :: 56;
}

#if OS == .WINDOWS {
    IPV6_RECVTCLASS :: 40;
}

#if OS == .LINUX {
    IPV6_RECVTCLASS :: 66;
}

#if OS == .ANDROID {
    IPV6_RECVTCLASS :: 66;
}

#if OS == .MACOS {
    IPV6_RECVTCLASS :: 35;
}

#if OS == .LINUX {
    IPV6_ROUTER_ALERT :: 22;
}

#if OS == .ANDROID {
    IPV6_ROUTER_ALERT :: 22;
}

#if OS == .LINUX {
    IPV6_ROUTER_ALERT_ISOLATE :: 30;
}

#if OS == .ANDROID {
    IPV6_ROUTER_ALERT_ISOLATE :: 30;
}

#if OS == .WINDOWS {
    IPV6_RTHDR :: 32;
}

#if OS == .LINUX {
    IPV6_RTHDR :: 57;
}

#if OS == .ANDROID {
    IPV6_RTHDR :: 57;
}

#if OS == .LINUX {
    IPV6_RTHDR_LOOSE :: 0;
}

#if OS == .MACOS {
    IPV6_RTHDR_LOOSE :: 0;
}

#if OS == .LINUX {
    IPV6_RTHDR_STRICT :: 1;
}

#if OS == .MACOS {
    IPV6_RTHDR_STRICT :: 1;
}

#if OS == .LINUX {
    IPV6_RTHDR_TYPE_0 :: 0;
}

#if OS == .MACOS {
    IPV6_RTHDR_TYPE_0 :: 0;
}

#if OS == .LINUX {
    IPV6_RTHDRDSTOPTS :: 55;
}

#if OS == .ANDROID {
    IPV6_RTHDRDSTOPTS :: 55;
}

#if OS == .LINUX {
    IPV6_RXDSTOPTS :: IPV6_DSTOPTS;
}

#if OS == .LINUX {
    IPV6_RXHOPOPTS :: IPV6_HOPOPTS;
}

#if OS == .MACOS {
    IPV6_SOCKOPT_RESERVED1 :: 3;
}

#if OS == .WINDOWS {
    IPV6_TCLASS :: 39;
}

#if OS == .LINUX {
    IPV6_TCLASS :: 67;
}

#if OS == .ANDROID {
    IPV6_TCLASS :: 67;
}

#if OS == .MACOS {
    IPV6_TCLASS :: 36;
}

#if OS == .ANDROID {
    IPV6_TLV_CALIPSO :: 7;
}

#if OS == .ANDROID {
    IPV6_TLV_HAO :: 201;
}

#if OS == .ANDROID {
    IPV6_TLV_IOAM :: 49;
}

#if OS == .ANDROID {
    IPV6_TLV_JUMBO :: 194;
}

#if OS == .ANDROID {
    IPV6_TLV_PAD1 :: 0;
}

#if OS == .ANDROID {
    IPV6_TLV_PADN :: 1;
}

#if OS == .ANDROID {
    IPV6_TLV_ROUTERALERT :: 5;
}

#if OS == .LINUX {
    IPV6_TRANSPARENT :: 75;
}

#if OS == .ANDROID {
    IPV6_TRANSPARENT :: 75;
}

#if OS == .WINDOWS {
    IPV6_UNICAST_HOPS :: 4;
}

#if OS == .LINUX {
    IPV6_UNICAST_HOPS :: 16;
}

#if OS == .ANDROID {
    IPV6_UNICAST_HOPS :: 16;
}

#if OS == .MACOS {
    IPV6_UNICAST_HOPS :: 4;
}

#if OS == .WINDOWS {
    IPV6_UNICAST_IF :: 31;
}

#if OS == .LINUX {
    IPV6_UNICAST_IF :: 76;
}

#if OS == .ANDROID {
    IPV6_UNICAST_IF :: 76;
}

#if OS == .WINDOWS {
    IPV6_USER_MTU :: 76;
}

#if OS == .WINDOWS {
    IPV6_V6ONLY :: 27;
}

#if OS == .LINUX {
    IPV6_V6ONLY :: 26;
}

#if OS == .ANDROID {
    IPV6_V6ONLY :: 26;
}

#if OS == .MACOS {
    IPV6_V6ONLY :: 27;
}

#if OS == .WINDOWS {
    IPV6_WFP_REDIRECT_CONTEXT :: 70;
}

#if OS == .WINDOWS {
    IPV6_WFP_REDIRECT_RECORDS :: 60;
}

#if OS == .LINUX {
    IPV6_XFRM_POLICY :: 35;
}

#if OS == .ANDROID {
    IPV6_XFRM_POLICY :: 35;
}

#if OS == .MACOS {
    IPV6CTL_ACCEPT_RTADV :: 12;
}

#if OS == .MACOS {
    IPV6CTL_ADDRCTLPOLICY :: 38;
}

#if OS == .MACOS {
    IPV6CTL_AUTO_FLOWLABEL :: 17;
}

#if OS == .MACOS {
    IPV6CTL_AUTO_LINKLOCAL :: 35;
}

#if OS == .MACOS {
    IPV6CTL_DAD_COUNT :: 16;
}

#if OS == .MACOS {
    IPV6CTL_DEFHLIM :: 3;
}

#if OS == .MACOS {
    IPV6CTL_DEFMCASTHLIM :: 18;
}

#if OS == .MACOS {
    IPV6CTL_FORWARDING :: 1;
}

#if OS == .MACOS {
    IPV6CTL_FORWSRCRT :: 5;
}

#if OS == .MACOS {
    IPV6CTL_GIF_HLIM :: 19;
}

#if OS == .MACOS {
    IPV6CTL_HDRNESTLIMIT :: 15;
}

#if OS == .MACOS {
    IPV6CTL_KAME_VERSION :: 20;
}

#if OS == .MACOS {
    IPV6CTL_KEEPFAITH :: 13;
}

#if OS == .MACOS {
    IPV6CTL_LOG_INTERVAL :: 14;
}

#if OS == .MACOS {
    IPV6CTL_MAXDYNROUTES :: 49;
}

#if OS == .MACOS {
    IPV6CTL_MAXFRAGPACKETS :: 9;
}

#if OS == .MACOS {
    IPV6CTL_MAXFRAGS :: 41;
}

#if OS == .MACOS {
    IPV6CTL_MAXID :: 51;
}

#if OS == .MACOS {
    IPV6CTL_MAXIFDEFROUTERS :: 48;
}

#if OS == .MACOS {
    IPV6CTL_MAXIFPREFIXES :: 47;
}

#if OS == .MACOS {
    IPV6CTL_MCAST_PMTU :: 44;
}

#if OS == .MACOS {
    IPV6CTL_MRTPROTO :: 8;
}

#if OS == .MACOS {
    IPV6CTL_MRTSTATS :: 7;
}

#if OS == .MACOS {
    IPV6CTL_NEIGHBORGCTHRESH :: 46;
}

#if OS == .MACOS {
    IPV6CTL_PREFER_TEMPADDR :: 37;
}

#if OS == .MACOS {
    IPV6CTL_RIP6STATS :: 36;
}

#if OS == .MACOS {
    IPV6CTL_RR_PRUNE :: 22;
}

#if OS == .MACOS {
    IPV6CTL_RTEXPIRE :: 25;
}

#if OS == .MACOS {
    IPV6CTL_RTMAXCACHE :: 27;
}

#if OS == .MACOS {
    IPV6CTL_RTMINEXPIRE :: 26;
}

#if OS == .MACOS {
    IPV6CTL_SENDREDIRECTS :: 2;
}

#if OS == .MACOS {
    IPV6CTL_SOURCECHECK :: 10;
}

#if OS == .MACOS {
    IPV6CTL_SOURCECHECK_LOGINT :: 11;
}

#if OS == .MACOS {
    IPV6CTL_STATS :: 6;
}

#if OS == .MACOS {
    IPV6CTL_TEMPPLTIME :: 33;
}

#if OS == .MACOS {
    IPV6CTL_TEMPVLTIME :: 34;
}

#if OS == .MACOS {
    IPV6CTL_USE_DEFAULTZONE :: 39;
}

#if OS == .MACOS {
    IPV6CTL_USE_DEPRECATED :: 21;
}

#if OS == .MACOS {
    IPV6CTL_USETEMPADDR :: 32;
}

#if OS == .MACOS {
    IPV6CTL_V6ONLY :: 24;
}

#if OS == .MACOS {
    IPV6PORT_ANONMAX :: 65535;
}

#if OS == .MACOS {
    IPV6PORT_ANONMIN :: 49152;
}

#if OS == .MACOS {
    IPV6PORT_RESERVED :: 1024;
}

#if OS == .MACOS {
    IPV6PORT_RESERVEDMAX :: IPV6PORT_RESERVED - 1;
}

#if OS == .MACOS {
    IPV6PORT_RESERVEDMIN :: 600;
}

#if OS == .MACOS {
    IPV6PROTO_MAXID :: xx IPPROTO.PIM + 1;
}

#if OS == .ANDROID {
    IPX_TYPE :: 1;
}

#if OS == .WINDOWS {
    JL_BOTH :: 0x04;
}

#if OS == .WINDOWS {
    JL_RECEIVER_ONLY :: 0x02;
}

#if OS == .WINDOWS {
    JL_SENDER_ONLY :: 0x01;
}

#if OS == .WINDOWS {
    LAYERED_PROTOCOL :: 0;
}

#if OS == .WINDOWS {
    LINGER :: linger;
}

#if OS == .WINDOWS {
    LITTLEENDIAN :: 0x0001;
}

#if OS == .WINDOWS {
    LUP_ADDRCONFIG :: 0x00100000;
}

#if OS == .WINDOWS {
    LUP_API_ANSI :: 0x01000000;
}

#if OS == .WINDOWS {
    LUP_CONTAINERS :: 0x00000002;
}

#if OS == .WINDOWS {
    LUP_DEEP :: 0x00000001;
}

#if OS == .WINDOWS {
    LUP_DISABLE_IDN_ENCODING :: 0x00800000;
}

#if OS == .WINDOWS {
    LUP_DNS_ONLY :: 0x00020000;
}

#if OS == .WINDOWS {
    LUP_DUAL_ADDR :: 0x00200000;
}

#if OS == .WINDOWS {
    LUP_EXCLUSIVE_CUSTOM_SERVERS :: 0x08000000;
}

#if OS == .WINDOWS {
    LUP_EXTENDED_QUERYSET :: 0x02000000;
}

#if OS == .WINDOWS {
    LUP_FILESERVER :: 0x00400000;
}

#if OS == .WINDOWS {
    LUP_FLUSHCACHE :: 0x00001000;
}

#if OS == .WINDOWS {
    LUP_FLUSHPREVIOUS :: 0x00002000;
}

#if OS == .WINDOWS {
    LUP_FORCE_CLEAR_TEXT :: 0x40000000;
}

#if OS == .WINDOWS {
    LUP_NEAREST :: 0x00000008;
}

#if OS == .WINDOWS {
    LUP_NOCONTAINERS :: 0x00000004;
}

#if OS == .WINDOWS {
    LUP_NON_AUTHORITATIVE :: 0x00004000;
}

#if OS == .WINDOWS {
    LUP_REQUIRE_SECURE :: 0x10000000;
}

#if OS == .WINDOWS {
    LUP_RES_SERVICE :: 0x00008000;
}

#if OS == .WINDOWS {
    LUP_RESERVED_UNUSED :: 0x00080000;
}

#if OS == .WINDOWS {
    LUP_RESOLUTION_HANDLE :: 0x80000000;
}

#if OS == .WINDOWS {
    LUP_RETURN_ADDR :: 0x00000100;
}

#if OS == .WINDOWS {
    LUP_RETURN_ALIASES :: 0x00000400;
}

#if OS == .WINDOWS {
    LUP_RETURN_ALL :: 0x00000FF0;
}

#if OS == .WINDOWS {
    LUP_RETURN_BLOB :: 0x00000200;
}

#if OS == .WINDOWS {
    LUP_RETURN_COMMENT :: 0x00000080;
}

#if OS == .WINDOWS {
    LUP_RETURN_NAME :: 0x00000010;
}

#if OS == .WINDOWS {
    LUP_RETURN_PREFERRED_NAMES :: 0x00010000;
}

#if OS == .WINDOWS {
    LUP_RETURN_QUERY_STRING :: 0x00000800;
}

#if OS == .WINDOWS {
    LUP_RETURN_RESPONSE_FLAGS :: 0x00040000;
}

#if OS == .WINDOWS {
    LUP_RETURN_TTL :: 0x20000000;
}

#if OS == .WINDOWS {
    LUP_RETURN_TYPE :: 0x00000020;
}

#if OS == .WINDOWS {
    LUP_RETURN_VERSION :: 0x00000040;
}

#if OS == .WINDOWS {
    LUP_SECURE :: 0x00008000;
}

#if OS == .WINDOWS {
    LUP_SECURE_WITH_FALLBACK :: 0x04000000;
}

#if OS == .WINDOWS {
    MAX_PROTOCOL_CHAIN :: 7;
}

#if OS == .MACOS {
    MAX_SACK_BLKS :: 6;
}

#if OS == .MACOS {
    MAX_TCPOPTLEN :: 40;
}

#if OS == .WINDOWS {
    MAXGETHOSTSTRUCT :: 1024;
}

#if OS == .WINDOWS {
    MCAST_BLOCK_SOURCE :: 43;
}

#if OS == .LINUX {
    MCAST_BLOCK_SOURCE :: 43;
}

#if OS == .ANDROID {
    MCAST_BLOCK_SOURCE :: 43;
}

#if OS == .MACOS {
    MCAST_BLOCK_SOURCE :: 84;
}

#if OS == .LINUX {
    MCAST_EXCLUDE :: 0;
}

#if OS == .ANDROID {
    MCAST_EXCLUDE :: 0;
}

#if OS == .MACOS {
    MCAST_EXCLUDE :: 2;
}

#if OS == .LINUX {
    MCAST_INCLUDE :: 1;
}

#if OS == .ANDROID {
    MCAST_INCLUDE :: 1;
}

#if OS == .MACOS {
    MCAST_INCLUDE :: 1;
}

#if OS == .WINDOWS {
    MCAST_JOIN_GROUP :: 41;
}

#if OS == .LINUX {
    MCAST_JOIN_GROUP :: 42;
}

#if OS == .ANDROID {
    MCAST_JOIN_GROUP :: 42;
}

#if OS == .MACOS {
    MCAST_JOIN_GROUP :: 80;
}

#if OS == .WINDOWS {
    MCAST_JOIN_SOURCE_GROUP :: 45;
}

#if OS == .LINUX {
    MCAST_JOIN_SOURCE_GROUP :: 46;
}

#if OS == .ANDROID {
    MCAST_JOIN_SOURCE_GROUP :: 46;
}

#if OS == .MACOS {
    MCAST_JOIN_SOURCE_GROUP :: 82;
}

#if OS == .WINDOWS {
    MCAST_LEAVE_GROUP :: 42;
}

#if OS == .LINUX {
    MCAST_LEAVE_GROUP :: 45;
}

#if OS == .ANDROID {
    MCAST_LEAVE_GROUP :: 45;
}

#if OS == .MACOS {
    MCAST_LEAVE_GROUP :: 81;
}

#if OS == .WINDOWS {
    MCAST_LEAVE_SOURCE_GROUP :: 46;
}

#if OS == .LINUX {
    MCAST_LEAVE_SOURCE_GROUP :: 47;
}

#if OS == .ANDROID {
    MCAST_LEAVE_SOURCE_GROUP :: 47;
}

#if OS == .MACOS {
    MCAST_LEAVE_SOURCE_GROUP :: 83;
}

#if OS == .LINUX {
    MCAST_MSFILTER :: 48;
}

#if OS == .ANDROID {
    MCAST_MSFILTER :: 48;
}

#if OS == .WINDOWS {
    MCAST_UNBLOCK_SOURCE :: 44;
}

#if OS == .LINUX {
    MCAST_UNBLOCK_SOURCE :: 44;
}

#if OS == .ANDROID {
    MCAST_UNBLOCK_SOURCE :: 44;
}

#if OS == .MACOS {
    MCAST_UNBLOCK_SOURCE :: 85;
}

#if OS == .MACOS {
    MCAST_UNDEFINED :: 0;
}

#if OS == .MACOS {
    NET_MAXID :: AF_MAX;
}

#if OS == .MACOS {
    NET_RT_DUMP :: 1;
}

#if OS == .MACOS {
    NET_RT_DUMP2 :: 7;
}

#if OS == .MACOS {
    NET_RT_FLAGS :: 2;
}

#if OS == .MACOS {
    NET_RT_IFLIST :: 3;
}

#if OS == .MACOS {
    NET_RT_IFLIST2 :: 6;
}

#if OS == .MACOS {
    NET_RT_MAXID :: 10;
}

#if OS == .MACOS {
    NET_RT_STAT :: 4;
}

#if OS == .MACOS {
    NET_RT_TRASH :: 5;
}

#if OS == .MACOS {
    NET_SERVICE_TYPE_AV :: 6;
}

#if OS == .MACOS {
    NET_SERVICE_TYPE_BE :: 0;
}

#if OS == .MACOS {
    NET_SERVICE_TYPE_BK :: 1;
}

#if OS == .MACOS {
    NET_SERVICE_TYPE_OAM :: 7;
}

#if OS == .MACOS {
    NET_SERVICE_TYPE_RD :: 8;
}

#if OS == .MACOS {
    NET_SERVICE_TYPE_RV :: 5;
}

#if OS == .MACOS {
    NET_SERVICE_TYPE_SIG :: 2;
}

#if OS == .MACOS {
    NET_SERVICE_TYPE_VI :: 3;
}

#if OS == .MACOS {
    NET_SERVICE_TYPE_VO :: 4;
}

#if OS == .LINUX {
    NETDB_INTERNAL :: -1;
}

#if OS == .ANDROID {
    NETDB_INTERNAL :: -1;
}

#if OS == .MACOS {
    NETDB_INTERNAL :: -1;
}

#if OS == .LINUX {
    NETDB_SUCCESS :: 0;
}

#if OS == .ANDROID {
    NETDB_SUCCESS :: 0;
}

#if OS == .MACOS {
    NETDB_SUCCESS :: 0;
}

#if OS == .MACOS {
    NETSVC_MRKNG_LVL_L2 :: 1;
}

#if OS == .MACOS {
    NETSVC_MRKNG_LVL_L3L2_ALL :: 2;
}

#if OS == .MACOS {
    NETSVC_MRKNG_LVL_L3L2_BK :: 3;
}

#if OS == .MACOS {
    NETSVC_MRKNG_UNKNOWN :: 0;
}

#if OS == .WINDOWS {
    NI_DGRAM :: 0x10;
}

#if OS == .LINUX {
    NI_DGRAM :: 16;
}

#if OS == .ANDROID {
    NI_DGRAM :: 0x00000010;
}

#if OS == .MACOS {
    NI_DGRAM :: 0x00000010;
}

#if OS == .LINUX {
    NI_IDN :: 32;
}

#if OS == .WINDOWS {
    NI_MAXHOST :: 1025;
}

#if OS == .LINUX {
    NI_MAXHOST :: 1025;
}

#if OS == .ANDROID {
    NI_MAXHOST :: 1025;
}

#if OS == .MACOS {
    NI_MAXHOST :: 1025;
}

#if OS == .WINDOWS {
    NI_MAXSERV :: 32;
}

#if OS == .LINUX {
    NI_MAXSERV :: 32;
}

#if OS == .ANDROID {
    NI_MAXSERV :: 32;
}

#if OS == .MACOS {
    NI_MAXSERV :: 32;
}

#if OS == .WINDOWS {
    NI_NAMEREQD :: 0x04;
}

#if OS == .LINUX {
    NI_NAMEREQD :: 8;
}

#if OS == .ANDROID {
    NI_NAMEREQD :: 0x00000004;
}

#if OS == .MACOS {
    NI_NAMEREQD :: 0x00000004;
}

#if OS == .WINDOWS {
    NI_NOFQDN :: 0x01;
}

#if OS == .LINUX {
    NI_NOFQDN :: 4;
}

#if OS == .ANDROID {
    NI_NOFQDN :: 0x00000001;
}

#if OS == .MACOS {
    NI_NOFQDN :: 0x00000001;
}

#if OS == .WINDOWS {
    NI_NUMERICHOST :: 0x02;
}

#if OS == .LINUX {
    NI_NUMERICHOST :: 1;
}

#if OS == .ANDROID {
    NI_NUMERICHOST :: 0x00000002;
}

#if OS == .MACOS {
    NI_NUMERICHOST :: 0x00000002;
}

#if OS == .MACOS {
    NI_NUMERICSCOPE :: 0x00000100;
}

#if OS == .WINDOWS {
    NI_NUMERICSERV :: 0x08;
}

#if OS == .LINUX {
    NI_NUMERICSERV :: 2;
}

#if OS == .ANDROID {
    NI_NUMERICSERV :: 0x00000008;
}

#if OS == .MACOS {
    NI_NUMERICSERV :: 0x00000008;
}

#if OS == .MACOS {
    NI_WITHSCOPEID :: 0x00000020;
}

#if OS == .LINUX {
    NO_ADDRESS :: NO_DATA;
}

#if OS == .ANDROID {
    NO_ADDRESS :: NO_DATA;
}

#if OS == .MACOS {
    NO_ADDRESS :: NO_DATA;
}

#if OS == .LINUX {
    NO_DATA :: 4;
}

#if OS == .ANDROID {
    NO_DATA :: 4;
}

#if OS == .MACOS {
    NO_DATA :: 4;
}

#if OS == .LINUX {
    NO_RECOVERY :: 3;
}

#if OS == .ANDROID {
    NO_RECOVERY :: 3;
}

#if OS == .MACOS {
    NO_RECOVERY :: 3;
}

#if OS == .WINDOWS {
    NS_ALL :: 0;
}

#if OS == .WINDOWS {
    NS_BTH :: 16;
}

#if OS == .WINDOWS {
    NS_DHCP :: 6;
}

#if OS == .WINDOWS {
    NS_DNS :: 12;
}

#if OS == .WINDOWS {
    NS_EMAIL :: 37;
}

#if OS == .WINDOWS {
    NS_LOCALNAME :: 19;
}

#if OS == .WINDOWS {
    NS_MS :: 30;
}

#if OS == .WINDOWS {
    NS_NBP :: 20;
}

#if OS == .WINDOWS {
    NS_NDS :: 2;
}

#if OS == .WINDOWS {
    NS_NETBT :: 13;
}

#if OS == .WINDOWS {
    NS_NETDES :: 60;
}

#if OS == .WINDOWS {
    NS_NIS :: 41;
}

#if OS == .WINDOWS {
    NS_NISPLUS :: 42;
}

#if OS == .WINDOWS {
    NS_NLA :: 15;
}

#if OS == .WINDOWS {
    NS_NTDS :: 32;
}

#if OS == .WINDOWS {
    NS_PEER_BROWSE :: 3;
}

#if OS == .WINDOWS {
    NS_PNRPCLOUD :: 39;
}

#if OS == .WINDOWS {
    NS_PNRPNAME :: 38;
}

#if OS == .WINDOWS {
    NS_SAP :: 1;
}

#if OS == .WINDOWS {
    NS_SLP :: 5;
}

#if OS == .WINDOWS {
    NS_STDA :: 31;
}

#if OS == .WINDOWS {
    NS_TCPIP_HOSTS :: 11;
}

#if OS == .WINDOWS {
    NS_TCPIP_LOCAL :: 10;
}

#if OS == .WINDOWS {
    NS_WINS :: 14;
}

#if OS == .WINDOWS {
    NS_WRQ :: 50;
}

#if OS == .WINDOWS {
    NS_X500 :: 40;
}

#if OS == .MACOS {
    PERSIST_TIMEOUT :: 0x40;
}

#if OS == .LINUX {
    PF_ALG :: 38;
}

#if OS == .ANDROID {
    PF_ALG :: AF_ALG;
}

#if OS == .WINDOWS {
    PF_APPLETALK :: AF_APPLETALK;
}

#if OS == .LINUX {
    PF_APPLETALK :: 5;
}

#if OS == .ANDROID {
    PF_APPLETALK :: AF_APPLETALK;
}

#if OS == .MACOS {
    PF_APPLETALK :: AF_APPLETALK;
}

#if OS == .LINUX {
    PF_ASH :: 18;
}

#if OS == .ANDROID {
    PF_ASH :: AF_ASH;
}

#if OS == .WINDOWS {
    PF_ATM :: AF_ATM;
}

#if OS == .LINUX {
    PF_ATMPVC :: 8;
}

#if OS == .ANDROID {
    PF_ATMPVC :: AF_ATMPVC;
}

#if OS == .LINUX {
    PF_ATMSVC :: 20;
}

#if OS == .ANDROID {
    PF_ATMSVC :: AF_ATMSVC;
}

#if OS == .LINUX {
    PF_AX25 :: 3;
}

#if OS == .ANDROID {
    PF_AX25 :: AF_AX25;
}

#if OS == .WINDOWS {
    PF_BAN :: AF_BAN;
}

#if OS == .LINUX {
    PF_BLUETOOTH :: 31;
}

#if OS == .ANDROID {
    PF_BLUETOOTH :: AF_BLUETOOTH;
}

#if OS == .MACOS {
    PF_BOND :: cast(u32) 0x626f6e64;
}

#if OS == .LINUX {
    PF_BRIDGE :: 7;
}

#if OS == .ANDROID {
    PF_BRIDGE :: AF_BRIDGE;
}

#if OS == .WINDOWS {
    PF_BTH :: AF_BTH;
}

#if OS == .LINUX {
    PF_CAIF :: 37;
}

#if OS == .ANDROID {
    PF_CAIF :: AF_CAIF;
}

#if OS == .LINUX {
    PF_CAN :: 29;
}

#if OS == .ANDROID {
    PF_CAN :: AF_CAN;
}

#if OS == .WINDOWS {
    PF_CCITT :: AF_CCITT;
}

#if OS == .MACOS {
    PF_CCITT :: AF_CCITT;
}

#if OS == .WINDOWS {
    PF_CHAOS :: AF_CHAOS;
}

#if OS == .MACOS {
    PF_CHAOS :: AF_CHAOS;
}

#if OS == .MACOS {
    PF_CNT :: AF_CNT;
}

#if OS == .MACOS {
    PF_COIP :: AF_COIP;
}

#if OS == .WINDOWS {
    PF_DATAKIT :: AF_DATAKIT;
}

#if OS == .MACOS {
    PF_DATAKIT :: AF_DATAKIT;
}

#if OS == .WINDOWS {
    PF_DECnet :: AF_DECnet;
}

#if OS == .LINUX {
    PF_DECnet :: 12;
}

#if OS == .ANDROID {
    PF_DECnet :: AF_DECnet;
}

#if OS == .MACOS {
    PF_DECnet :: AF_DECnet;
}

#if OS == .WINDOWS {
    PF_DLI :: AF_DLI;
}

#if OS == .MACOS {
    PF_DLI :: AF_DLI;
}

#if OS == .WINDOWS {
    PF_ECMA :: AF_ECMA;
}

#if OS == .MACOS {
    PF_ECMA :: AF_ECMA;
}

#if OS == .LINUX {
    PF_ECONET :: 19;
}

#if OS == .ANDROID {
    PF_ECONET :: AF_ECONET;
}

#if OS == .LINUX {
    PF_FILE :: PF_LOCAL;
}

#if OS == .WINDOWS {
    PF_FIREFOX :: AF_FIREFOX;
}

#if OS == .WINDOWS {
    PF_HYLINK :: AF_HYLINK;
}

#if OS == .MACOS {
    PF_HYLINK :: AF_HYLINK;
}

#if OS == .LINUX {
    PF_IB :: 27;
}

#if OS == .LINUX {
    PF_IEEE802154 :: 36;
}

#if OS == .ANDROID {
    PF_IEEE802154 :: AF_IEEE802154;
}

#if OS == .WINDOWS {
    PF_IMPLINK :: AF_IMPLINK;
}

#if OS == .MACOS {
    PF_IMPLINK :: AF_IMPLINK;
}

#if OS == .WINDOWS {
    PF_INET :: AF_INET;
}

#if OS == .LINUX {
    PF_INET :: 2;
}

#if OS == .ANDROID {
    PF_INET :: AF_INET;
}

#if OS == .MACOS {
    PF_INET :: AF_INET;
}

#if OS == .WINDOWS {
    PF_INET6 :: AF_INET6;
}

#if OS == .LINUX {
    PF_INET6 :: 10;
}

#if OS == .ANDROID {
    PF_INET6 :: AF_INET6;
}

#if OS == .MACOS {
    PF_INET6 :: AF_INET6;
}

#if OS == .WINDOWS {
    PF_IPX :: AF_IPX;
}

#if OS == .LINUX {
    PF_IPX :: 4;
}

#if OS == .ANDROID {
    PF_IPX :: AF_IPX;
}

#if OS == .MACOS {
    PF_IPX :: AF_IPX;
}

#if OS == .LINUX {
    PF_IRDA :: 23;
}

#if OS == .ANDROID {
    PF_IRDA :: AF_IRDA;
}

#if OS == .LINUX {
    PF_ISDN :: 34;
}

#if OS == .ANDROID {
    PF_ISDN :: AF_ISDN;
}

#if OS == .MACOS {
    PF_ISDN :: AF_ISDN;
}

#if OS == .WINDOWS {
    PF_ISO :: AF_ISO;
}

#if OS == .MACOS {
    PF_ISO :: AF_ISO;
}

#if OS == .LINUX {
    PF_IUCV :: 32;
}

#if OS == .ANDROID {
    PF_IUCV :: AF_IUCV;
}

#if OS == .LINUX {
    PF_KCM :: 41;
}

#if OS == .ANDROID {
    PF_KCM :: AF_KCM;
}

#if OS == .LINUX {
    PF_KEY :: 15;
}

#if OS == .ANDROID {
    PF_KEY :: AF_KEY;
}

#if OS == .MACOS {
    PF_KEY :: pseudo_AF_KEY;
}

#if OS == .WINDOWS {
    PF_LAT :: AF_LAT;
}

#if OS == .MACOS {
    PF_LAT :: AF_LAT;
}

#if OS == .MACOS {
    PF_LINK :: AF_LINK;
}

#if OS == .LINUX {
    PF_LLC :: 26;
}

#if OS == .ANDROID {
    PF_LLC :: AF_LLC;
}

#if OS == .LINUX {
    PF_LOCAL :: 1;
}

#if OS == .ANDROID {
    PF_LOCAL :: AF_LOCAL;
}

#if OS == .MACOS {
    PF_LOCAL :: AF_LOCAL;
}

#if OS == .WINDOWS {
    PF_MAX :: AF_MAX;
}

#if OS == .LINUX {
    PF_MAX :: 45;
}

#if OS == .ANDROID {
    PF_MAX :: AF_MAX;
}

#if OS == .MACOS {
    PF_MAX :: AF_MAX;
}

#if OS == .LINUX {
    PF_MPLS :: 28;
}

#if OS == .MACOS {
    PF_NATM :: AF_NATM;
}

#if OS == .MACOS {
    PF_NDRV :: AF_NDRV;
}

#if OS == .LINUX {
    PF_NETBEUI :: 13;
}

#if OS == .ANDROID {
    PF_NETBEUI :: AF_NETBEUI;
}

#if OS == .MACOS {
    PF_NETBIOS :: AF_NETBIOS;
}

#if OS == .LINUX {
    PF_NETLINK :: 16;
}

#if OS == .ANDROID {
    PF_NETLINK :: AF_NETLINK;
}

#if OS == .LINUX {
    PF_NETROM :: 6;
}

#if OS == .ANDROID {
    PF_NETROM :: AF_NETROM;
}

#if OS == .LINUX {
    PF_NFC :: 39;
}

#if OS == .ANDROID {
    PF_NFC :: AF_NFC;
}

#if OS == .WINDOWS {
    PF_NS :: AF_NS;
}

#if OS == .MACOS {
    PF_NS :: AF_NS;
}

#if OS == .WINDOWS {
    PF_OSI :: AF_OSI;
}

#if OS == .MACOS {
    PF_OSI :: AF_ISO;
}

#if OS == .LINUX {
    PF_PACKET :: 17;
}

#if OS == .ANDROID {
    PF_PACKET :: AF_PACKET;
}

#if OS == .LINUX {
    PF_PHONET :: 35;
}

#if OS == .ANDROID {
    PF_PHONET :: AF_PHONET;
}

#if OS == .MACOS {
    PF_PIP :: pseudo_AF_PIP;
}

#if OS == .MACOS {
    PF_PPP :: AF_PPP;
}

#if OS == .LINUX {
    PF_PPPOX :: 24;
}

#if OS == .ANDROID {
    PF_PPPOX :: AF_PPPOX;
}

#if OS == .WINDOWS {
    PF_PUP :: AF_PUP;
}

#if OS == .MACOS {
    PF_PUP :: AF_PUP;
}

#if OS == .LINUX {
    PF_QIPCRTR :: 42;
}

#if OS == .ANDROID {
    PF_QIPCRTR :: AF_QIPCRTR;
}

#if OS == .LINUX {
    PF_RDS :: 21;
}

#if OS == .ANDROID {
    PF_RDS :: AF_RDS;
}

#if OS == .MACOS {
    PF_RESERVED_36 :: AF_RESERVED_36;
}

#if OS == .LINUX {
    PF_ROSE :: 11;
}

#if OS == .ANDROID {
    PF_ROSE :: AF_ROSE;
}

#if OS == .LINUX {
    PF_ROUTE :: PF_NETLINK;
}

#if OS == .ANDROID {
    PF_ROUTE :: AF_ROUTE;
}

#if OS == .MACOS {
    PF_ROUTE :: AF_ROUTE;
}

#if OS == .MACOS {
    PF_RTIP :: pseudo_AF_RTIP;
}

#if OS == .LINUX {
    PF_RXRPC :: 33;
}

#if OS == .ANDROID {
    PF_RXRPC :: AF_RXRPC;
}

#if OS == .LINUX {
    PF_SECURITY :: 14;
}

#if OS == .ANDROID {
    PF_SECURITY :: AF_SECURITY;
}

#if OS == .MACOS {
    PF_SIP :: AF_SIP;
}

#if OS == .LINUX {
    PF_SMC :: 43;
}

#if OS == .WINDOWS {
    PF_SNA :: AF_SNA;
}

#if OS == .LINUX {
    PF_SNA :: 22;
}

#if OS == .ANDROID {
    PF_SNA :: AF_SNA;
}

#if OS == .MACOS {
    PF_SNA :: AF_SNA;
}

#if OS == .MACOS {
    PF_SYSTEM :: AF_SYSTEM;
}

#if OS == .LINUX {
    PF_TIPC :: 30;
}

#if OS == .ANDROID {
    PF_TIPC :: AF_TIPC;
}

#if OS == .WINDOWS {
    PF_UNIX :: AF_UNIX;
}

#if OS == .LINUX {
    PF_UNIX :: PF_LOCAL;
}

#if OS == .ANDROID {
    PF_UNIX :: AF_UNIX;
}

#if OS == .MACOS {
    PF_UNIX :: PF_LOCAL;
}

#if OS == .WINDOWS {
    PF_UNKNOWN1 :: AF_UNKNOWN1;
}

#if OS == .WINDOWS {
    PF_UNSPEC :: AF_UNSPEC;
}

#if OS == .LINUX {
    PF_UNSPEC :: 0;
}

#if OS == .ANDROID {
    PF_UNSPEC :: AF_UNSPEC;
}

#if OS == .MACOS {
    PF_UNSPEC :: AF_UNSPEC;
}

#if OS == .MACOS {
    PF_UTUN :: AF_UTUN;
}

#if OS == .MACOS {
    PF_VLAN :: cast(u32) 0x766c616e;
}

#if OS == .WINDOWS {
    PF_VOICEVIEW :: AF_VOICEVIEW;
}

#if OS == .LINUX {
    PF_VSOCK :: 40;
}

#if OS == .ANDROID {
    PF_VSOCK :: AF_VSOCK;
}

#if OS == .LINUX {
    PF_WANPIPE :: 25;
}

#if OS == .ANDROID {
    PF_WANPIPE :: AF_WANPIPE;
}

#if OS == .LINUX {
    PF_X25 :: 9;
}

#if OS == .ANDROID {
    PF_X25 :: AF_X25;
}

#if OS == .LINUX {
    PF_XDP :: 44;
}

#if OS == .MACOS {
    PF_XTP :: pseudo_AF_XTP;
}

#if OS == .WINDOWS {
    PFL_HIDDEN :: 0x00000004;
}

#if OS == .WINDOWS {
    PFL_MATCHES_PROTOCOL_ZERO :: 0x00000008;
}

#if OS == .WINDOWS {
    PFL_MULTIPLE_PROTO_ENTRIES :: 0x00000001;
}

#if OS == .WINDOWS {
    PFL_NETWORKDIRECT_PROVIDER :: 0x00000010;
}

#if OS == .WINDOWS {
    PFL_RECOMMENDED_PROTO_ENTRY :: 0x00000002;
}

#if OS == .WINDOWS {
    POLLERR :: 0x0001;
}

#if OS == .WINDOWS {
    POLLHUP :: 0x0002;
}

#if OS == .WINDOWS {
    POLLIN :: POLLRDNORM | POLLRDBAND;
}

#if OS == .WINDOWS {
    POLLNVAL :: 0x0004;
}

#if OS == .WINDOWS {
    POLLOUT :: POLLWRNORM;
}

#if OS == .WINDOWS {
    POLLPRI :: 0x0400;
}

#if OS == .WINDOWS {
    POLLRDBAND :: 0x0200;
}

#if OS == .WINDOWS {
    POLLRDNORM :: 0x0100;
}

#if OS == .WINDOWS {
    POLLWRBAND :: 0x0020;
}

#if OS == .WINDOWS {
    POLLWRNORM :: 0x0010;
}

#if OS == .WINDOWS {
    POSITIVE_INFINITY_RATE :: 0xFFFFFFFE;
}

#if OS == .WINDOWS {
    PROTECTION_LEVEL_DEFAULT :: cast,trunc(u32) -1;
}

#if OS == .WINDOWS {
    PROTECTION_LEVEL_EDGERESTRICTED :: 20;
}

#if OS == .WINDOWS {
    PROTECTION_LEVEL_RESTRICTED :: 30;
}

#if OS == .WINDOWS {
    PROTECTION_LEVEL_UNRESTRICTED :: 10;
}

#if OS == .WINDOWS {
    PROTOENT :: protoent;
}

#if OS == .MACOS {
    pseudo_AF_HDRCMPLT :: 35;
}

#if OS == .MACOS {
    pseudo_AF_KEY :: 29;
}

#if OS == .MACOS {
    pseudo_AF_PIP :: 25;
}

#if OS == .MACOS {
    pseudo_AF_RTIP :: 22;
}

#if OS == .MACOS {
    pseudo_AF_XTP :: 19;
}

#if OS == .WINDOWS {
    PVD_CONFIG :: 0x3001;
}

#if OS == .WINDOWS {
    QOS :: QualityOfService;
}

#if OS == .WINDOWS {
    QOS_GENERAL_ID_BASE :: 2000;
}

#if OS == .WINDOWS {
    QOS_NOT_SPECIFIED :: 0xFFFFFFFF;
}

#if OS == .WINDOWS {
    QOS_OBJECT_DESTADDR :: 0x00000004 + QOS_GENERAL_ID_BASE;
}

#if OS == .WINDOWS {
    QOS_OBJECT_END_OF_LIST :: 0x00000001 + QOS_GENERAL_ID_BASE;
}

#if OS == .WINDOWS {
    QOS_OBJECT_SD_MODE :: 0x00000002 + QOS_GENERAL_ID_BASE;
}

#if OS == .WINDOWS {
    QOS_OBJECT_SHAPING_RATE :: 0x00000003 + QOS_GENERAL_ID_BASE;
}

#if OS == .WINDOWS {
    RES_FLUSH_CACHE :: 0x00000002;
}

#if OS == .WINDOWS {
    RES_SERVICE :: 0x00000004;
}

#if OS == .WINDOWS {
    RES_UNUSED_1 :: 0x00000001;
}

#if OS == .WINDOWS {
    RESULT_IS_ADDED :: 0x0010;
}

#if OS == .WINDOWS {
    RESULT_IS_ALIAS :: 0x0001;
}

#if OS == .WINDOWS {
    RESULT_IS_CHANGED :: 0x0020;
}

#if OS == .WINDOWS {
    RESULT_IS_DELETED :: 0x0040;
}

#if OS == .MACOS {
    sa_endpoints_t :: sa_endpoints;
}

#if OS == .LINUX {
    sa_family_t :: u16;
}

#if OS == .ANDROID {
    sa_family_t :: u16;
}

#if OS == .MACOS {
    sa_family_t :: u8;
}

#if OS == .MACOS {
    SAE_ASSOCID_ALL :: cast,trunc(sae_associd_t) (-1);
}

#if OS == .MACOS {
    SAE_ASSOCID_ANY :: 0;
}

#if OS == .MACOS {
    sae_associd_t :: u32;
}

#if OS == .MACOS {
    SAE_CONNID_ALL :: cast,trunc(sae_connid_t) (-1);
}

#if OS == .MACOS {
    SAE_CONNID_ANY :: 0;
}

#if OS == .MACOS {
    sae_connid_t :: u32;
}

#if OS == .WINDOWS {
    SD_BOTH :: 0x02;
}

#if OS == .WINDOWS {
    SD_RECEIVE :: 0x00;
}

#if OS == .WINDOWS {
    SD_SEND :: 0x01;
}

#if OS == .WINDOWS {
    SECURITY_PROTOCOL_NONE :: 0x0000;
}

#if OS == .WINDOWS {
    SERVENT :: servent;
}

#if OS == .WINDOWS {
    SERVICE_BESTEFFORT :: 0x80010000;
}

#if OS == .WINDOWS {
    SERVICE_CONTROLLEDLOAD :: 0x80020000;
}

#if OS == .WINDOWS {
    SERVICE_GUARANTEED :: 0x80040000;
}

#if OS == .WINDOWS {
    SERVICE_MULTIPLE :: 0x00000001;
}

#if OS == .WINDOWS {
    SERVICE_NO_QOS_SIGNALING :: 0x40000000;
}

#if OS == .WINDOWS {
    SERVICE_NO_TRAFFIC_CONTROL :: 0x81000000;
}

#if OS == .WINDOWS {
    SERVICE_QUALITATIVE :: 0x80200000;
}

#if OS == .WINDOWS {
    SERVICE_TYPE_VALUE_IPXPORTA :: "IpxSocket";
}

#if OS == .WINDOWS {
    SERVICE_TYPE_VALUE_OBJECTID :: SERVICE_TYPE_VALUE_OBJECTIDA;
}

#if OS == .WINDOWS {
    SERVICE_TYPE_VALUE_OBJECTIDA :: "ObjectId";
}

#if OS == .WINDOWS {
    SERVICE_TYPE_VALUE_SAPID :: SERVICE_TYPE_VALUE_SAPIDA;
}

#if OS == .WINDOWS {
    SERVICE_TYPE_VALUE_SAPIDA :: "SapId";
}

#if OS == .WINDOWS {
    SERVICE_TYPE_VALUE_TCPPORT :: SERVICE_TYPE_VALUE_TCPPORTA;
}

#if OS == .WINDOWS {
    SERVICE_TYPE_VALUE_TCPPORTA :: "TcpPort";
}

#if OS == .WINDOWS {
    SERVICE_TYPE_VALUE_UDPPORT :: SERVICE_TYPE_VALUE_UDPPORTA;
}

#if OS == .WINDOWS {
    SERVICE_TYPE_VALUE_UDPPORTA :: "UdpPort";
}

#if OS == .WINDOWS {
    SERVICETYPE :: u32;
}

#if OS == .WINDOWS {
    SERVICETYPE_BESTEFFORT :: 0x00000001;
}

#if OS == .WINDOWS {
    SERVICETYPE_CONTROLLEDLOAD :: 0x00000002;
}

#if OS == .WINDOWS {
    SERVICETYPE_GENERAL_INFORMATION :: 0x00000005;
}

#if OS == .WINDOWS {
    SERVICETYPE_GUARANTEED :: 0x00000003;
}

#if OS == .WINDOWS {
    SERVICETYPE_NETWORK_CONTROL :: 0x0000000A;
}

#if OS == .WINDOWS {
    SERVICETYPE_NETWORK_UNAVAILABLE :: 0x00000004;
}

#if OS == .WINDOWS {
    SERVICETYPE_NOCHANGE :: 0x00000006;
}

#if OS == .WINDOWS {
    SERVICETYPE_NONCONFORMING :: 0x00000009;
}

#if OS == .WINDOWS {
    SERVICETYPE_NOTRAFFIC :: 0x00000000;
}

#if OS == .WINDOWS {
    SERVICETYPE_QUALITATIVE :: 0x0000000D;
}

#if OS == .WINDOWS {
    SetAddrInfoEx :: SetAddrInfoExA;
}

#if OS == .WINDOWS {
    SG_CONSTRAINED_GROUP :: 0x02;
}

#if OS == .WINDOWS {
    SG_UNCONSTRAINED_GROUP :: 0x01;
}

#if OS == .WINDOWS {
    SIO_KEEPALIVE_VALS :: 0x98000004;
}

#if OS == .WINDOWS {
    SO_ACCEPTCONN :: 0x0002;
}

#if OS == .LINUX {
    SO_ACCEPTCONN :: 30;
}

#if OS == .ANDROID {
    SO_ACCEPTCONN :: 30;
}

#if OS == .MACOS {
    SO_ACCEPTCONN :: 0x0002;
}

#if OS == .LINUX {
    SO_ATTACH_BPF :: 50;
}

#if OS == .ANDROID {
    SO_ATTACH_BPF :: 50;
}

#if OS == .LINUX {
    SO_ATTACH_FILTER :: 26;
}

#if OS == .ANDROID {
    SO_ATTACH_FILTER :: 26;
}

#if OS == .LINUX {
    SO_ATTACH_REUSEPORT_CBPF :: 51;
}

#if OS == .ANDROID {
    SO_ATTACH_REUSEPORT_CBPF :: 51;
}

#if OS == .LINUX {
    SO_ATTACH_REUSEPORT_EBPF :: 52;
}

#if OS == .ANDROID {
    SO_ATTACH_REUSEPORT_EBPF :: 52;
}

#if OS == .LINUX {
    SO_BINDTODEVICE :: 25;
}

#if OS == .ANDROID {
    SO_BINDTODEVICE :: 25;
}

#if OS == .LINUX {
    SO_BINDTOIFINDEX :: 62;
}

#if OS == .ANDROID {
    SO_BINDTOIFINDEX :: 62;
}

#if OS == .LINUX {
    SO_BPF_EXTENSIONS :: 48;
}

#if OS == .ANDROID {
    SO_BPF_EXTENSIONS :: 48;
}

#if OS == .WINDOWS {
    SO_BROADCAST :: 0x0020;
}

#if OS == .LINUX {
    SO_BROADCAST :: 6;
}

#if OS == .ANDROID {
    SO_BROADCAST :: 6;
}

#if OS == .MACOS {
    SO_BROADCAST :: 0x0020;
}

#if OS == .LINUX {
    SO_BSDCOMPAT :: 14;
}

#if OS == .ANDROID {
    SO_BSDCOMPAT :: 14;
}

#if OS == .WINDOWS {
    SO_BSP_STATE :: 0x1009;
}

#if OS == .ANDROID {
    SO_BUF_LOCK :: 72;
}

#if OS == .LINUX {
    SO_BUSY_POLL :: 46;
}

#if OS == .ANDROID {
    SO_BUSY_POLL :: 46;
}

#if OS == .ANDROID {
    SO_BUSY_POLL_BUDGET :: 70;
}

#if OS == .LINUX {
    SO_CNX_ADVICE :: 53;
}

#if OS == .ANDROID {
    SO_CNX_ADVICE :: 53;
}

#if OS == .WINDOWS {
    SO_COMPARTMENT_ID :: 0x3004;
}

#if OS == .WINDOWS {
    SO_CONDITIONAL_ACCEPT :: 0x3002;
}

#if OS == .LINUX {
    SO_COOKIE :: 57;
}

#if OS == .ANDROID {
    SO_COOKIE :: 57;
}

#if OS == .WINDOWS {
    SO_DEBUG :: 0x0001;
}

#if OS == .LINUX {
    SO_DEBUG :: 1;
}

#if OS == .ANDROID {
    SO_DEBUG :: 1;
}

#if OS == .MACOS {
    SO_DEBUG :: 0x0001;
}

#if OS == .LINUX {
    SO_DETACH_BPF :: SO_DETACH_FILTER;
}

#if OS == .ANDROID {
    SO_DETACH_BPF :: SO_DETACH_FILTER;
}

#if OS == .LINUX {
    SO_DETACH_FILTER :: 27;
}

#if OS == .ANDROID {
    SO_DETACH_FILTER :: 27;
}

#if OS == .LINUX {
    SO_DETACH_REUSEPORT_BPF :: 68;
}

#if OS == .ANDROID {
    SO_DETACH_REUSEPORT_BPF :: 68;
}

#if OS == .LINUX {
    SO_DOMAIN :: 39;
}

#if OS == .ANDROID {
    SO_DOMAIN :: 39;
}

#if OS == .WINDOWS {
    SO_DONTLINGER :: cast,trunc(s32) (~SO_LINGER);
}

#if OS == .WINDOWS {
    SO_DONTROUTE :: 0x0010;
}

#if OS == .LINUX {
    SO_DONTROUTE :: 5;
}

#if OS == .ANDROID {
    SO_DONTROUTE :: 5;
}

#if OS == .MACOS {
    SO_DONTROUTE :: 0x0010;
}

#if OS == .MACOS {
    SO_DONTTRUNC :: 0x2000;
}

#if OS == .WINDOWS {
    SO_ERROR :: 0x1007;
}

#if OS == .LINUX {
    SO_ERROR :: 4;
}

#if OS == .ANDROID {
    SO_ERROR :: 4;
}

#if OS == .MACOS {
    SO_ERROR :: 0x1007;
}

#if OS == .WINDOWS {
    SO_EXCLUSIVEADDRUSE :: cast,trunc(s32) (~SO_REUSEADDR);
}

#if OS == .LINUX {
    SO_GET_FILTER :: SO_ATTACH_FILTER;
}

#if OS == .ANDROID {
    SO_GET_FILTER :: SO_ATTACH_FILTER;
}

#if OS == .WINDOWS {
    SO_GROUP_ID :: 0x2001;
}

#if OS == .WINDOWS {
    SO_GROUP_PRIORITY :: 0x2002;
}

#if OS == .LINUX {
    SO_INCOMING_CPU :: 49;
}

#if OS == .ANDROID {
    SO_INCOMING_CPU :: 49;
}

#if OS == .LINUX {
    SO_INCOMING_NAPI_ID :: 56;
}

#if OS == .ANDROID {
    SO_INCOMING_NAPI_ID :: 56;
}

#if OS == .WINDOWS {
    SO_KEEPALIVE :: 0x0008;
}

#if OS == .LINUX {
    SO_KEEPALIVE :: 9;
}

#if OS == .ANDROID {
    SO_KEEPALIVE :: 9;
}

#if OS == .MACOS {
    SO_KEEPALIVE :: 0x0008;
}

#if OS == .MACOS {
    SO_LABEL :: 0x1010;
}

#if OS == .WINDOWS {
    SO_LINGER :: 0x0080;
}

#if OS == .LINUX {
    SO_LINGER :: 13;
}

#if OS == .ANDROID {
    SO_LINGER :: 13;
}

#if OS == .MACOS {
    SO_LINGER :: 0x0080;
}

#if OS == .MACOS {
    SO_LINGER_SEC :: 0x1080;
}

#if OS == .LINUX {
    SO_LOCK_FILTER :: 44;
}

#if OS == .ANDROID {
    SO_LOCK_FILTER :: 44;
}

#if OS == .LINUX {
    SO_MARK :: 36;
}

#if OS == .ANDROID {
    SO_MARK :: 36;
}

#if OS == .WINDOWS {
    SO_MAX_MSG_SIZE :: 0x2003;
}

#if OS == .LINUX {
    SO_MAX_PACING_RATE :: 47;
}

#if OS == .ANDROID {
    SO_MAX_PACING_RATE :: 47;
}

#if OS == .LINUX {
    SO_MEMINFO :: 55;
}

#if OS == .ANDROID {
    SO_MEMINFO :: 55;
}

#if OS == .MACOS {
    SO_NET_SERVICE_TYPE :: 0x1116;
}

#if OS == .ANDROID {
    SO_NETNS_COOKIE :: 71;
}

#if OS == .MACOS {
    SO_NETSVC_MARKING_LEVEL :: 0x1119;
}

#if OS == .MACOS {
    SO_NKE :: 0x1021;
}

#if OS == .LINUX {
    SO_NO_CHECK :: 11;
}

#if OS == .ANDROID {
    SO_NO_CHECK :: 11;
}

#if OS == .MACOS {
    SO_NOADDRERR :: 0x1023;
}

#if OS == .LINUX {
    SO_NOFCS :: 43;
}

#if OS == .ANDROID {
    SO_NOFCS :: 43;
}

#if OS == .MACOS {
    SO_NOSIGPIPE :: 0x1022;
}

#if OS == .MACOS {
    SO_NOTIFYCONFLICT :: 0x1026;
}

#if OS == .MACOS {
    SO_NP_EXTENSIONS :: 0x1083;
}

#if OS == .MACOS {
    SO_NREAD :: 0x1020;
}

#if OS == .MACOS {
    SO_NUMRCVPKT :: 0x1112;
}

#if OS == .MACOS {
    SO_NWRITE :: 0x1024;
}

#if OS == .WINDOWS {
    SO_OOBINLINE :: 0x0100;
}

#if OS == .LINUX {
    SO_OOBINLINE :: 10;
}

#if OS == .ANDROID {
    SO_OOBINLINE :: 10;
}

#if OS == .MACOS {
    SO_OOBINLINE :: 0x0100;
}

#if OS == .WINDOWS {
    SO_ORIGINAL_DST :: 0x300F;
}

#if OS == .LINUX {
    SO_PASSCRED :: 16;
}

#if OS == .ANDROID {
    SO_PASSCRED :: 16;
}

#if OS == .LINUX {
    SO_PASSSEC :: 34;
}

#if OS == .ANDROID {
    SO_PASSSEC :: 34;
}

#if OS == .WINDOWS {
    SO_PAUSE_ACCEPT :: 0x3003;
}

#if OS == .LINUX {
    SO_PEEK_OFF :: 42;
}

#if OS == .ANDROID {
    SO_PEEK_OFF :: 42;
}

#if OS == .LINUX {
    SO_PEERCRED :: 17;
}

#if OS == .ANDROID {
    SO_PEERCRED :: 17;
}

#if OS == .LINUX {
    SO_PEERGROUPS :: 59;
}

#if OS == .ANDROID {
    SO_PEERGROUPS :: 59;
}

#if OS == .MACOS {
    SO_PEERLABEL :: 0x1011;
}

#if OS == .LINUX {
    SO_PEERNAME :: 28;
}

#if OS == .ANDROID {
    SO_PEERNAME :: 28;
}

#if OS == .LINUX {
    SO_PEERSEC :: 31;
}

#if OS == .ANDROID {
    SO_PEERSEC :: 31;
}

#if OS == .WINDOWS {
    SO_PORT_SCALABILITY :: 0x3006;
}

#if OS == .ANDROID {
    SO_PREFER_BUSY_POLL :: 69;
}

#if OS == .LINUX {
    SO_PRIORITY :: 12;
}

#if OS == .ANDROID {
    SO_PRIORITY :: 12;
}

#if OS == .LINUX {
    SO_PROTOCOL :: 38;
}

#if OS == .ANDROID {
    SO_PROTOCOL :: 38;
}

#if OS == .WINDOWS {
    SO_PROTOCOL_INFO :: SO_PROTOCOL_INFOA;
}

#if OS == .WINDOWS {
    SO_PROTOCOL_INFOA :: 0x2004;
}

#if OS == .WINDOWS {
    SO_PROTOCOL_INFOW :: 0x2005;
}

#if OS == .WINDOWS {
    SO_RANDOMIZE_PORT :: 0x3005;
}

#if OS == .MACOS {
    SO_RANDOMPORT :: 0x1082;
}

#if OS == .WINDOWS {
    SO_RCVBUF :: 0x1002;
}

#if OS == .LINUX {
    SO_RCVBUF :: 8;
}

#if OS == .ANDROID {
    SO_RCVBUF :: 8;
}

#if OS == .MACOS {
    SO_RCVBUF :: 0x1002;
}

#if OS == .LINUX {
    SO_RCVBUFFORCE :: 33;
}

#if OS == .ANDROID {
    SO_RCVBUFFORCE :: 33;
}

#if OS == .WINDOWS {
    SO_RCVLOWAT :: 0x1004;
}

#if OS == .LINUX {
    SO_RCVLOWAT :: 18;
}

#if OS == .ANDROID {
    SO_RCVLOWAT :: 18;
}

#if OS == .MACOS {
    SO_RCVLOWAT :: 0x1004;
}

#if OS == .ANDROID {
    SO_RCVMARK :: 75;
}

#if OS == .WINDOWS {
    SO_RCVTIMEO :: 0x1006;
}

#if OS == .LINUX {
    SO_RCVTIMEO :: SO_RCVTIMEO_OLD;
}

#if OS == .ANDROID {
    SO_RCVTIMEO :: SO_RCVTIMEO_OLD;
}

#if OS == .MACOS {
    SO_RCVTIMEO :: 0x1006;
}

#if OS == .LINUX {
    SO_RCVTIMEO_NEW :: 66;
}

#if OS == .ANDROID {
    SO_RCVTIMEO_NEW :: 66;
}

#if OS == .LINUX {
    SO_RCVTIMEO_OLD :: 20;
}

#if OS == .ANDROID {
    SO_RCVTIMEO_OLD :: 20;
}

#if OS == .WINDOWS {
    SO_RECEIVED_HOPLIMIT :: 0x3010;
}

#if OS == .WINDOWS {
    SO_RECEIVED_PROCESSOR :: 0x3011;
}

#if OS == .ANDROID {
    SO_RESERVE_MEM :: 73;
}

#if OS == .WINDOWS {
    SO_REUSE_MULTICASTPORT :: 0x3008;
}

#if OS == .WINDOWS {
    SO_REUSE_UNICASTPORT :: 0x3007;
}

#if OS == .WINDOWS {
    SO_REUSEADDR :: 0x0004;
}

#if OS == .LINUX {
    SO_REUSEADDR :: 2;
}

#if OS == .ANDROID {
    SO_REUSEADDR :: 2;
}

#if OS == .MACOS {
    SO_REUSEADDR :: 0x0004;
}

#if OS == .LINUX {
    SO_REUSEPORT :: 15;
}

#if OS == .ANDROID {
    SO_REUSEPORT :: 15;
}

#if OS == .MACOS {
    SO_REUSEPORT :: 0x0200;
}

#if OS == .MACOS {
    SO_REUSESHAREUID :: 0x1025;
}

#if OS == .LINUX {
    SO_RXQ_OVFL :: 40;
}

#if OS == .ANDROID {
    SO_RXQ_OVFL :: 40;
}

#if OS == .LINUX {
    SO_SECURITY_AUTHENTICATION :: 22;
}

#if OS == .ANDROID {
    SO_SECURITY_AUTHENTICATION :: 22;
}

#if OS == .LINUX {
    SO_SECURITY_ENCRYPTION_NETWORK :: 24;
}

#if OS == .ANDROID {
    SO_SECURITY_ENCRYPTION_NETWORK :: 24;
}

#if OS == .LINUX {
    SO_SECURITY_ENCRYPTION_TRANSPORT :: 23;
}

#if OS == .ANDROID {
    SO_SECURITY_ENCRYPTION_TRANSPORT :: 23;
}

#if OS == .LINUX {
    SO_SELECT_ERR_QUEUE :: 45;
}

#if OS == .ANDROID {
    SO_SELECT_ERR_QUEUE :: 45;
}

#if OS == .WINDOWS {
    SO_SNDBUF :: 0x1001;
}

#if OS == .LINUX {
    SO_SNDBUF :: 7;
}

#if OS == .ANDROID {
    SO_SNDBUF :: 7;
}

#if OS == .MACOS {
    SO_SNDBUF :: 0x1001;
}

#if OS == .LINUX {
    SO_SNDBUFFORCE :: 32;
}

#if OS == .ANDROID {
    SO_SNDBUFFORCE :: 32;
}

#if OS == .WINDOWS {
    SO_SNDLOWAT :: 0x1003;
}

#if OS == .LINUX {
    SO_SNDLOWAT :: 19;
}

#if OS == .ANDROID {
    SO_SNDLOWAT :: 19;
}

#if OS == .MACOS {
    SO_SNDLOWAT :: 0x1003;
}

#if OS == .WINDOWS {
    SO_SNDTIMEO :: 0x1005;
}

#if OS == .LINUX {
    SO_SNDTIMEO :: SO_SNDTIMEO_OLD;
}

#if OS == .ANDROID {
    SO_SNDTIMEO :: SO_SNDTIMEO_OLD;
}

#if OS == .MACOS {
    SO_SNDTIMEO :: 0x1005;
}

#if OS == .LINUX {
    SO_SNDTIMEO_NEW :: 67;
}

#if OS == .ANDROID {
    SO_SNDTIMEO_NEW :: 67;
}

#if OS == .LINUX {
    SO_SNDTIMEO_OLD :: 21;
}

#if OS == .ANDROID {
    SO_SNDTIMEO_OLD :: 21;
}

#if OS == .WINDOWS {
    SO_TIMESTAMP :: 0x300A;
}

#if OS == .LINUX {
    SO_TIMESTAMP :: SO_TIMESTAMP_OLD;
}

#if OS == .ANDROID {
    SO_TIMESTAMP :: SO_TIMESTAMP_OLD;
}

#if OS == .MACOS {
    SO_TIMESTAMP :: 0x0400;
}

#if OS == .WINDOWS {
    SO_TIMESTAMP_ID :: 0x300B;
}

#if OS == .MACOS {
    SO_TIMESTAMP_MONOTONIC :: 0x0800;
}

#if OS == .LINUX {
    SO_TIMESTAMP_NEW :: 63;
}

#if OS == .ANDROID {
    SO_TIMESTAMP_NEW :: 63;
}

#if OS == .LINUX {
    SO_TIMESTAMP_OLD :: 29;
}

#if OS == .ANDROID {
    SO_TIMESTAMP_OLD :: 29;
}

#if OS == .LINUX {
    SO_TIMESTAMPING :: SO_TIMESTAMPING_OLD;
}

#if OS == .ANDROID {
    SO_TIMESTAMPING :: SO_TIMESTAMPING_OLD;
}

#if OS == .LINUX {
    SO_TIMESTAMPING_NEW :: 65;
}

#if OS == .ANDROID {
    SO_TIMESTAMPING_NEW :: 65;
}

#if OS == .LINUX {
    SO_TIMESTAMPING_OLD :: 37;
}

#if OS == .ANDROID {
    SO_TIMESTAMPING_OLD :: 37;
}

#if OS == .LINUX {
    SO_TIMESTAMPNS :: SO_TIMESTAMPNS_OLD;
}

#if OS == .ANDROID {
    SO_TIMESTAMPNS :: SO_TIMESTAMPNS_OLD;
}

#if OS == .LINUX {
    SO_TIMESTAMPNS_NEW :: 64;
}

#if OS == .ANDROID {
    SO_TIMESTAMPNS_NEW :: 64;
}

#if OS == .LINUX {
    SO_TIMESTAMPNS_OLD :: 35;
}

#if OS == .ANDROID {
    SO_TIMESTAMPNS_OLD :: 35;
}

#if OS == .ANDROID {
    SO_TXREHASH :: 74;
}

#if OS == .LINUX {
    SO_TXTIME :: 61;
}

#if OS == .ANDROID {
    SO_TXTIME :: 61;
}

#if OS == .WINDOWS {
    SO_TYPE :: 0x1008;
}

#if OS == .LINUX {
    SO_TYPE :: 3;
}

#if OS == .ANDROID {
    SO_TYPE :: 3;
}

#if OS == .MACOS {
    SO_TYPE :: 0x1008;
}

#if OS == .MACOS {
    SO_UPCALLCLOSEWAIT :: 0x1027;
}

#if OS == .WINDOWS {
    SO_USELOOPBACK :: 0x0040;
}

#if OS == .MACOS {
    SO_USELOOPBACK :: 0x0040;
}

#if OS == .MACOS {
    SO_WANTMORE :: 0x4000;
}

#if OS == .MACOS {
    SO_WANTOOBFLAG :: 0x8000;
}

#if OS == .LINUX {
    SO_WIFI_STATUS :: 41;
}

#if OS == .ANDROID {
    SO_WIFI_STATUS :: 41;
}

#if OS == .LINUX {
    SO_ZEROCOPY :: 60;
}

#if OS == .ANDROID {
    SO_ZEROCOPY :: 60;
}

#if OS == .MACOS {
    SOCK_MAXADDRLEN :: 255;
}

#if OS == .WINDOWS {
    SOCK_NOTIFY_EVENT_ERR :: 0x40;
}

#if OS == .WINDOWS {
    SOCK_NOTIFY_EVENT_HANGUP :: SOCK_NOTIFY_REGISTER_EVENT_HANGUP;
}

#if OS == .WINDOWS {
    SOCK_NOTIFY_EVENT_IN :: SOCK_NOTIFY_REGISTER_EVENT_IN;
}

#if OS == .WINDOWS {
    SOCK_NOTIFY_EVENT_OUT :: SOCK_NOTIFY_REGISTER_EVENT_OUT;
}

#if OS == .WINDOWS {
    SOCK_NOTIFY_EVENT_REMOVE :: 0x80;
}

#if OS == .WINDOWS {
    SOCK_NOTIFY_EVENTS_ALL :: SOCK_NOTIFY_REGISTER_EVENTS_ALL | SOCK_NOTIFY_EVENT_ERR | SOCK_NOTIFY_EVENT_REMOVE;
}

#if OS == .WINDOWS {
    SOCK_NOTIFY_OP_DISABLE :: 0x02;
}

#if OS == .WINDOWS {
    SOCK_NOTIFY_OP_ENABLE :: 0x01;
}

#if OS == .WINDOWS {
    SOCK_NOTIFY_OP_NONE :: 0x00;
}

#if OS == .WINDOWS {
    SOCK_NOTIFY_OP_REMOVE :: 0x04;
}

#if OS == .WINDOWS {
    SOCK_NOTIFY_REGISTER_EVENT_HANGUP :: 0x04;
}

#if OS == .WINDOWS {
    SOCK_NOTIFY_REGISTER_EVENT_IN :: 0x01;
}

#if OS == .WINDOWS {
    SOCK_NOTIFY_REGISTER_EVENT_NONE :: 0x00;
}

#if OS == .WINDOWS {
    SOCK_NOTIFY_REGISTER_EVENT_OUT :: 0x02;
}

#if OS == .WINDOWS {
    SOCK_NOTIFY_REGISTER_EVENTS_ALL :: SOCK_NOTIFY_REGISTER_EVENT_IN | SOCK_NOTIFY_REGISTER_EVENT_OUT | SOCK_NOTIFY_REGISTER_EVENT_HANGUP;
}

#if OS == .WINDOWS {
    SOCK_NOTIFY_TRIGGER_ALL :: SOCK_NOTIFY_TRIGGER_ONESHOT | SOCK_NOTIFY_TRIGGER_PERSISTENT | SOCK_NOTIFY_TRIGGER_LEVEL | SOCK_NOTIFY_TRIGGER_EDGE;
}

#if OS == .WINDOWS {
    SOCK_NOTIFY_TRIGGER_EDGE :: 0x08;
}

#if OS == .WINDOWS {
    SOCK_NOTIFY_TRIGGER_LEVEL :: 0x04;
}

#if OS == .WINDOWS {
    SOCK_NOTIFY_TRIGGER_ONESHOT :: 0x01;
}

#if OS == .WINDOWS {
    SOCK_NOTIFY_TRIGGER_PERSISTENT :: 0x02;
}

#if OS == .WINDOWS {
    SOCKADDR :: sockaddr;
}

#if OS == .WINDOWS {
    SOCKADDR_DL :: sockaddr_dl;
}

#if OS == .WINDOWS {
    SOCKADDR_IN :: sockaddr_in;
}

#if OS == .WINDOWS {
    SOCKADDR_IN6 :: SOCKADDR_IN6_LH;
}

#if OS == .WINDOWS {
    SOCKADDR_IN6_LH :: sockaddr_in6;
}

#if OS == .WINDOWS {
    SOCKADDR_IN6_PAIR :: sockaddr_in6_pair;
}

#if OS == .WINDOWS {
    SOCKADDR_IN6_W2KSP1 :: sockaddr_in6_w2ksp1;
}

#if OS == .WINDOWS {
    SOCKADDR_STORAGE :: SOCKADDR_STORAGE_LH;
}

#if OS == .WINDOWS {
    SOCKADDR_STORAGE_LH :: sockaddr_storage;
}

#if OS == .WINDOWS {
    SOCKADDR_STORAGE_XP :: sockaddr_storage_xp;
}

#if OS == .WINDOWS {
    SOCKET :: u64;
}

#if OS == .WINDOWS {
    SOCKET_INFO_CONNECTION_ENCRYPTED :: 0x2;
}

#if OS == .WINDOWS {
    SOCKET_INFO_CONNECTION_IMPERSONATED :: 0x4;
}

#if OS == .WINDOWS {
    SOCKET_INFO_CONNECTION_SECURED :: 0x1;
}

#if OS == .WINDOWS {
    SOCKET_QUERY_IPSEC2_ABORT_CONNECTION_ON_FIELD_CHANGE :: 0x1;
}

#if OS == .WINDOWS {
    SOCKET_QUERY_IPSEC2_FIELD_MASK_MM_SA_ID :: 0x1;
}

#if OS == .WINDOWS {
    SOCKET_QUERY_IPSEC2_FIELD_MASK_QM_SA_ID :: 0x2;
}

#if OS == .WINDOWS {
    SOCKET_SETTINGS_ALLOW_INSECURE :: 0x2;
}

#if OS == .WINDOWS {
    SOCKET_SETTINGS_GUARANTEE_ENCRYPTION :: 0x1;
}

#if OS == .WINDOWS {
    SOCKET_SETTINGS_IPSEC_ALLOW_FIRST_INBOUND_PKT_UNENCRYPTED :: 0x4;
}

#if OS == .WINDOWS {
    SOCKET_SETTINGS_IPSEC_OPTIONAL_PEER_NAME_VERIFICATION :: 0x2;
}

#if OS == .WINDOWS {
    SOCKET_SETTINGS_IPSEC_PEER_NAME_IS_RAW_FORMAT :: 0x8;
}

#if OS == .WINDOWS {
    SOCKET_SETTINGS_IPSEC_SKIP_FILTER_INSTANTIATION :: 0x1;
}

#if OS == .WINDOWS {
    socklen_t :: s32;
}

#if OS == .LINUX {
    socklen_t :: u32;
}

#if OS == .ANDROID {
    socklen_t :: u32;
}

#if OS == .MACOS {
    socklen_t :: u32;
}

#if OS == .LINUX {
    SOL_AAL :: 265;
}

#if OS == .ANDROID {
    SOL_AAL :: 265;
}

#if OS == .LINUX {
    SOL_ALG :: 279;
}

#if OS == .ANDROID {
    SOL_ALG :: 279;
}

#if OS == .ANDROID {
    SOL_ATALK :: 258;
}

#if OS == .LINUX {
    SOL_ATM :: 264;
}

#if OS == .ANDROID {
    SOL_ATM :: 264;
}

#if OS == .ANDROID {
    SOL_AX25 :: 257;
}

#if OS == .LINUX {
    SOL_BLUETOOTH :: 274;
}

#if OS == .ANDROID {
    SOL_BLUETOOTH :: 274;
}

#if OS == .LINUX {
    SOL_CAIF :: 278;
}

#if OS == .ANDROID {
    SOL_CAIF :: 278;
}

#if OS == .LINUX {
    SOL_DCCP :: 269;
}

#if OS == .ANDROID {
    SOL_DCCP :: 269;
}

#if OS == .LINUX {
    SOL_DECNET :: 261;
}

#if OS == .ANDROID {
    SOL_DECNET :: 261;
}

#if OS == .LINUX {
    SOL_ICMPV6 :: 58;
}

#if OS == .ANDROID {
    SOL_ICMPV6 :: 58;
}

#if OS == .WINDOWS {
    SOL_IP :: SOL_SOCKET - 4;
}

#if OS == .LINUX {
    SOL_IP :: 0;
}

#if OS == .ANDROID {
    SOL_IP :: 0;
}

#if OS == .WINDOWS {
    SOL_IPV6 :: SOL_SOCKET - 5;
}

#if OS == .LINUX {
    SOL_IPV6 :: 41;
}

#if OS == .ANDROID {
    SOL_IPV6 :: 41;
}

#if OS == .ANDROID {
    SOL_IPX :: 256;
}

#if OS == .LINUX {
    SOL_IRDA :: 266;
}

#if OS == .ANDROID {
    SOL_IRDA :: 266;
}

#if OS == .LINUX {
    SOL_IUCV :: 277;
}

#if OS == .ANDROID {
    SOL_IUCV :: 277;
}

#if OS == .LINUX {
    SOL_KCM :: 281;
}

#if OS == .ANDROID {
    SOL_KCM :: 281;
}

#if OS == .LINUX {
    SOL_LLC :: 268;
}

#if OS == .ANDROID {
    SOL_LLC :: 268;
}

#if OS == .LINUX {
    SOL_NETBEUI :: 267;
}

#if OS == .ANDROID {
    SOL_NETBEUI :: 267;
}

#if OS == .LINUX {
    SOL_NETLINK :: 270;
}

#if OS == .ANDROID {
    SOL_NETLINK :: 270;
}

#if OS == .ANDROID {
    SOL_NETROM :: 259;
}

#if OS == .LINUX {
    SOL_NFC :: 280;
}

#if OS == .ANDROID {
    SOL_NFC :: 280;
}

#if OS == .LINUX {
    SOL_PACKET :: 263;
}

#if OS == .ANDROID {
    SOL_PACKET :: 263;
}

#if OS == .LINUX {
    SOL_PNPIPE :: 275;
}

#if OS == .ANDROID {
    SOL_PNPIPE :: 275;
}

#if OS == .LINUX {
    SOL_PPPOL2TP :: 273;
}

#if OS == .ANDROID {
    SOL_PPPOL2TP :: 273;
}

#if OS == .LINUX {
    SOL_RAW :: 255;
}

#if OS == .ANDROID {
    SOL_RAW :: 255;
}

#if OS == .LINUX {
    SOL_RDS :: 276;
}

#if OS == .ANDROID {
    SOL_RDS :: 276;
}

#if OS == .ANDROID {
    SOL_ROSE :: 260;
}

#if OS == .LINUX {
    SOL_RXRPC :: 272;
}

#if OS == .ANDROID {
    SOL_RXRPC :: 272;
}

#if OS == .ANDROID {
    SOL_SCTP :: 132;
}

#if OS == .WINDOWS {
    SOL_SOCKET :: 0xffff;
}

#if OS == .LINUX {
    SOL_SOCKET :: 1;
}

#if OS == .ANDROID {
    SOL_SOCKET :: 1;
}

#if OS == .MACOS {
    SOL_SOCKET :: 0xffff;
}

#if OS == .LINUX {
    SOL_TCP :: 6;
}

#if OS == .ANDROID {
    SOL_TCP :: 6;
}

#if OS == .LINUX {
    SOL_TIPC :: 271;
}

#if OS == .ANDROID {
    SOL_TIPC :: 271;
}

#if OS == .LINUX {
    SOL_TLS :: 282;
}

#if OS == .ANDROID {
    SOL_TLS :: 282;
}

#if OS == .ANDROID {
    SOL_UDP :: 17;
}

#if OS == .LINUX {
    SOL_X25 :: 262;
}

#if OS == .ANDROID {
    SOL_X25 :: 262;
}

#if OS == .LINUX {
    SOL_XDP :: 283;
}

#if OS == .WINDOWS {
    SOMAXCONN :: 0x7fffffff;
}

#if OS == .LINUX {
    SOMAXCONN :: 4096;
}

#if OS == .ANDROID {
    SOMAXCONN :: 128;
}

#if OS == .MACOS {
    SOMAXCONN :: 128;
}

#if OS == .MACOS {
    SONPX_SETOPTSHUT :: 0x000000001;
}

#if OS == .WINDOWS {
    TC_NONCONF_BORROW :: 0;
}

#if OS == .WINDOWS {
    TC_NONCONF_BORROW_PLUS :: 3;
}

#if OS == .WINDOWS {
    TC_NONCONF_DISCARD :: 2;
}

#if OS == .WINDOWS {
    TC_NONCONF_SHAPE :: 1;
}

#if OS == .MACOS {
    TCP6_MSS :: 1024;
}

#if OS == .MACOS {
    tcp6_seq :: tcp_seq;
}

#if OS == .MACOS {
    tcp6hdr :: tcphdr;
}

#if OS == .WINDOWS {
    TCP_ATMARK :: 8;
}

#if OS == .MACOS {
    tcp_cc :: u32;
}

#if OS == .LINUX {
    TCP_CC_INFO :: 26;
}

#if OS == .LINUX {
    TCP_CM_INQ :: TCP_INQ;
}

#if OS == .LINUX {
    TCP_CONGESTION :: 13;
}

#if OS == .WINDOWS {
    TCP_CONGESTION_ALGORITHM :: 12;
}

#if OS == .MACOS {
    TCP_CONNECTION_INFO :: 0x106;
}

#if OS == .MACOS {
    TCP_CONNECTIONTIMEOUT :: 0x20;
}

#if OS == .LINUX {
    TCP_COOKIE_IN_ALWAYS :: 1 << 0;
}

#if OS == .LINUX {
    TCP_COOKIE_MAX :: 16;
}

#if OS == .LINUX {
    TCP_COOKIE_MIN :: 8;
}

#if OS == .LINUX {
    TCP_COOKIE_OUT_NEVER :: 1 << 1;
}

#if OS == .LINUX {
    TCP_COOKIE_PAIR_SIZE :: 2 * TCP_COOKIE_MAX;
}

#if OS == .LINUX {
    TCP_COOKIE_TRANSACTIONS :: 15;
}

#if OS == .LINUX {
    TCP_CORK :: 3;
}

#if OS == .LINUX {
    TCP_DEFER_ACCEPT :: 9;
}

#if OS == .WINDOWS {
    TCP_DELAY_FIN_ACK :: 13;
}

#if OS == .MACOS {
    TCP_ENABLE_ECN :: 0x104;
}

#if OS == .WINDOWS {
    TCP_EXPEDITED_1122 :: 0x0002;
}

#if OS == .WINDOWS {
    TCP_FAIL_CONNECT_ON_ICMP_ERROR :: 18;
}

#if OS == .WINDOWS {
    TCP_FASTOPEN :: 15;
}

#if OS == .LINUX {
    TCP_FASTOPEN :: 23;
}

#if OS == .MACOS {
    TCP_FASTOPEN :: 0x105;
}

#if OS == .LINUX {
    TCP_FASTOPEN_CONNECT :: 30;
}

#if OS == .LINUX {
    TCP_FASTOPEN_KEY :: 33;
}

#if OS == .LINUX {
    TCP_FASTOPEN_NO_COOKIE :: 34;
}

#if OS == .WINDOWS {
    TCP_ICMP_ERROR_INFO :: 19;
}

#if OS == .LINUX {
    TCP_INFO :: 11;
}

#if OS == .WINDOWS {
    TCP_INITIAL_RTO_DEFAULT_MAX_SYN_RETRANSMISSIONS :: 0;
}

#if OS == .WINDOWS {
    TCP_INITIAL_RTO_DEFAULT_RTT :: 0;
}

#if OS == .WINDOWS {
    TCP_INITIAL_RTO_NO_SYN_RETRANSMISSIONS :: cast,trunc(u8) -2;
}

#if OS == .WINDOWS {
    TCP_INITIAL_RTO_UNSPECIFIED_MAX_SYN_RETRANSMISSIONS :: cast,trunc(u8) -1;
}

#if OS == .WINDOWS {
    TCP_INITIAL_RTO_UNSPECIFIED_RTT :: cast,trunc(u16) -1;
}

#if OS == .LINUX {
    TCP_INQ :: 36;
}

#if OS == .WINDOWS {
    TCP_KEEPALIVE :: 3;
}

#if OS == .MACOS {
    TCP_KEEPALIVE :: 0x10;
}

#if OS == .WINDOWS {
    TCP_KEEPCNT :: 16;
}

#if OS == .LINUX {
    TCP_KEEPCNT :: 6;
}

#if OS == .MACOS {
    TCP_KEEPCNT :: 0x102;
}

#if OS == .WINDOWS {
    TCP_KEEPIDLE :: TCP_KEEPALIVE;
}

#if OS == .LINUX {
    TCP_KEEPIDLE :: 4;
}

#if OS == .WINDOWS {
    TCP_KEEPINTVL :: 17;
}

#if OS == .LINUX {
    TCP_KEEPINTVL :: 5;
}

#if OS == .MACOS {
    TCP_KEEPINTVL :: 0x101;
}

#if OS == .LINUX {
    TCP_LINGER2 :: 8;
}

#if OS == .MACOS {
    TCP_MAX_SACK :: 4;
}

#if OS == .LINUX {
    TCP_MAX_WINSHIFT :: 14;
}

#if OS == .MACOS {
    TCP_MAX_WINSHIFT :: 14;
}

#if OS == .MACOS {
    TCP_MAXHLEN :: 0xf << 2;
}

#if OS == .WINDOWS {
    TCP_MAXRT :: 5;
}

#if OS == .WINDOWS {
    TCP_MAXRTMS :: 14;
}

#if OS == .WINDOWS {
    TCP_MAXSEG :: 4;
}

#if OS == .LINUX {
    TCP_MAXSEG :: 2;
}

#if OS == .MACOS {
    TCP_MAXSEG :: 0x02;
}

#if OS == .LINUX {
    TCP_MAXWIN :: 65535;
}

#if OS == .MACOS {
    TCP_MAXWIN :: 65535;
}

#if OS == .LINUX {
    TCP_MD5SIG :: 14;
}

#if OS == .LINUX {
    TCP_MD5SIG_EXT :: 32;
}

#if OS == .LINUX {
    TCP_MD5SIG_FLAG_PREFIX :: 1;
}

#if OS == .LINUX {
    TCP_MD5SIG_MAXKEYLEN :: 80;
}

#if OS == .MACOS {
    TCP_MINMSS :: 216;
}

#if OS == .LINUX {
    TCP_MSS :: 512;
}

#if OS == .MACOS {
    TCP_MSS :: 512;
}

#if OS == .LINUX {
    TCP_MSS_DEFAULT :: 536;
}

#if OS == .LINUX {
    TCP_MSS_DESIRED :: 1220;
}

#if OS == .WINDOWS {
    TCP_NODELAY :: 0x0001;
}

#if OS == .LINUX {
    TCP_NODELAY :: 1;
}

#if OS == .MACOS {
    TCP_NODELAY :: 0x01;
}

#if OS == .MACOS {
    TCP_NOOPT :: 0x08;
}

#if OS == .MACOS {
    TCP_NOPUSH :: 0x04;
}

#if OS == .WINDOWS {
    TCP_NOSYNRETRIES :: 9;
}

#if OS == .LINUX {
    TCP_NOTSENT_LOWAT :: 25;
}

#if OS == .MACOS {
    TCP_NOTSENT_LOWAT :: 0x201;
}

#if OS == .WINDOWS {
    TCP_NOURG :: 7;
}

#if OS == .WINDOWS {
    TCP_OFFLOAD_NO_PREFERENCE :: 0;
}

#if OS == .WINDOWS {
    TCP_OFFLOAD_NOT_PREFERRED :: 1;
}

#if OS == .WINDOWS {
    TCP_OFFLOAD_PREFERENCE :: 11;
}

#if OS == .WINDOWS {
    TCP_OFFLOAD_PREFERRED :: 2;
}

#if OS == .LINUX {
    TCP_QUEUE_SEQ :: 21;
}

#if OS == .LINUX {
    TCP_QUICKACK :: 12;
}

#if OS == .LINUX {
    TCP_REPAIR :: 19;
}

#if OS == .LINUX {
    TCP_REPAIR_OFF :: 0;
}

#if OS == .LINUX {
    TCP_REPAIR_OFF_NO_WP :: -1;
}

#if OS == .LINUX {
    TCP_REPAIR_ON :: 1;
}

#if OS == .LINUX {
    TCP_REPAIR_OPTIONS :: 22;
}

#if OS == .LINUX {
    TCP_REPAIR_QUEUE :: 20;
}

#if OS == .LINUX {
    TCP_REPAIR_WINDOW :: 29;
}

#if OS == .MACOS {
    TCP_RXT_CONNDROPTIME :: 0x80;
}

#if OS == .MACOS {
    TCP_RXT_FINDROP :: 0x100;
}

#if OS == .LINUX {
    TCP_S_DATA_IN :: 1 << 2;
}

#if OS == .LINUX {
    TCP_S_DATA_OUT :: 1 << 3;
}

#if OS == .LINUX {
    TCP_SAVE_SYN :: 27;
}

#if OS == .LINUX {
    TCP_SAVED_SYN :: 28;
}

#if OS == .MACOS {
    TCP_SENDMOREACKS :: 0x103;
}

#if OS == .LINUX {
    tcp_seq :: u32;
}

#if OS == .MACOS {
    tcp_seq :: u32;
}

#if OS == .WINDOWS {
    TCP_STDURG :: 6;
}

#if OS == .LINUX {
    TCP_SYNCNT :: 7;
}

#if OS == .LINUX {
    TCP_THIN_DUPACK :: 17;
}

#if OS == .LINUX {
    TCP_THIN_LINEAR_TIMEOUTS :: 16;
}

#if OS == .LINUX {
    TCP_TIMESTAMP :: 24;
}

#if OS == .WINDOWS {
    TCP_TIMESTAMPS :: 10;
}

#if OS == .LINUX {
    TCP_TX_DELAY :: 37;
}

#if OS == .LINUX {
    TCP_ULP :: 31;
}

#if OS == .LINUX {
    TCP_USER_TIMEOUT :: 18;
}

#if OS == .LINUX {
    TCP_WINDOW_CLAMP :: 10;
}

#if OS == .LINUX {
    TCP_ZEROCOPY_RECEIVE :: 35;
}

#if OS == .MACOS {
    TCPCI_FLAG_LOSSRECOVERY :: 0x00000001;
}

#if OS == .MACOS {
    TCPCI_FLAG_REORDERING_DETECTED :: 0x00000002;
}

#if OS == .MACOS {
    TCPCI_OPT_ECN :: 0x00000008;
}

#if OS == .MACOS {
    TCPCI_OPT_SACK :: 0x00000002;
}

#if OS == .MACOS {
    TCPCI_OPT_TIMESTAMPS :: 0x00000001;
}

#if OS == .MACOS {
    TCPCI_OPT_WSCALE :: 0x00000004;
}

#if OS == .LINUX {
    TCPI_OPT_ECN :: 8;
}

#if OS == .LINUX {
    TCPI_OPT_ECN_SEEN :: 16;
}

#if OS == .LINUX {
    TCPI_OPT_SACK :: 2;
}

#if OS == .LINUX {
    TCPI_OPT_SYN_DATA :: 32;
}

#if OS == .LINUX {
    TCPI_OPT_TIMESTAMPS :: 1;
}

#if OS == .LINUX {
    TCPI_OPT_WSCALE :: 4;
}

#if OS == .MACOS {
    TCPOLEN_CC :: 6;
}

#if OS == .MACOS {
    TCPOLEN_CC_APPA :: TCPOLEN_CC + 2;
}

#if OS == .MACOS {
    TCPOLEN_FASTOPEN_REQ :: 2;
}

#if OS == .LINUX {
    TCPOLEN_MAXSEG :: 4;
}

#if OS == .ANDROID {
    TCPOLEN_MAXSEG :: 4;
}

#if OS == .MACOS {
    TCPOLEN_MAXSEG :: 4;
}

#if OS == .MACOS {
    TCPOLEN_SACK :: 8;
}

#if OS == .LINUX {
    TCPOLEN_SACK_PERMITTED :: 2;
}

#if OS == .ANDROID {
    TCPOLEN_SACK_PERMITTED :: 2;
}

#if OS == .MACOS {
    TCPOLEN_SACK_PERMITTED :: 2;
}

#if OS == .MACOS {
    TCPOLEN_SIGNATURE :: 18;
}

#if OS == .LINUX {
    TCPOLEN_TIMESTAMP :: 10;
}

#if OS == .ANDROID {
    TCPOLEN_TIMESTAMP :: 10;
}

#if OS == .MACOS {
    TCPOLEN_TIMESTAMP :: 10;
}

#if OS == .LINUX {
    TCPOLEN_TSTAMP_APPA :: TCPOLEN_TIMESTAMP + 2;
}

#if OS == .ANDROID {
    TCPOLEN_TSTAMP_APPA :: TCPOLEN_TIMESTAMP + 2;
}

#if OS == .MACOS {
    TCPOLEN_TSTAMP_APPA :: TCPOLEN_TIMESTAMP + 2;
}

#if OS == .LINUX {
    TCPOLEN_WINDOW :: 3;
}

#if OS == .ANDROID {
    TCPOLEN_WINDOW :: 3;
}

#if OS == .MACOS {
    TCPOLEN_WINDOW :: 3;
}

#if OS == .MACOS {
    TCPOPT_CC :: 11;
}

#if OS == .MACOS {
    TCPOPT_CCECHO :: 13;
}

#if OS == .MACOS {
    TCPOPT_CCNEW :: 12;
}

#if OS == .LINUX {
    TCPOPT_EOL :: 0;
}

#if OS == .ANDROID {
    TCPOPT_EOL :: 0;
}

#if OS == .MACOS {
    TCPOPT_EOL :: 0;
}

#if OS == .MACOS {
    TCPOPT_FASTOPEN :: 34;
}

#if OS == .LINUX {
    TCPOPT_MAXSEG :: 2;
}

#if OS == .ANDROID {
    TCPOPT_MAXSEG :: 2;
}

#if OS == .MACOS {
    TCPOPT_MAXSEG :: 2;
}

#if OS == .LINUX {
    TCPOPT_NOP :: 1;
}

#if OS == .ANDROID {
    TCPOPT_NOP :: 1;
}

#if OS == .MACOS {
    TCPOPT_NOP :: 1;
}

#if OS == .LINUX {
    TCPOPT_SACK :: 5;
}

#if OS == .ANDROID {
    TCPOPT_SACK :: 5;
}

#if OS == .MACOS {
    TCPOPT_SACK :: 5;
}

#if OS == .MACOS {
    TCPOPT_SACK_HDR :: TCPOPT_NOP << 24 | TCPOPT_NOP << 16 | TCPOPT_SACK << 8;
}

#if OS == .LINUX {
    TCPOPT_SACK_PERMITTED :: 4;
}

#if OS == .ANDROID {
    TCPOPT_SACK_PERMITTED :: 4;
}

#if OS == .MACOS {
    TCPOPT_SACK_PERMITTED :: 4;
}

#if OS == .MACOS {
    TCPOPT_SIGNATURE :: 19;
}

#if OS == .LINUX {
    TCPOPT_TIMESTAMP :: 8;
}

#if OS == .ANDROID {
    TCPOPT_TIMESTAMP :: 8;
}

#if OS == .MACOS {
    TCPOPT_TIMESTAMP :: 8;
}

#if OS == .LINUX {
    TCPOPT_TSTAMP_HDR :: TCPOPT_NOP << 24 | TCPOPT_NOP << 16 | TCPOPT_TIMESTAMP << 8 | TCPOLEN_TIMESTAMP;
}

#if OS == .ANDROID {
    TCPOPT_TSTAMP_HDR :: TCPOPT_NOP << 24 | TCPOPT_NOP << 16 | TCPOPT_TIMESTAMP << 8 | TCPOLEN_TIMESTAMP;
}

#if OS == .MACOS {
    TCPOPT_TSTAMP_HDR :: TCPOPT_NOP << 24 | TCPOPT_NOP << 16 | TCPOPT_TIMESTAMP << 8 | TCPOLEN_TIMESTAMP;
}

#if OS == .LINUX {
    TCPOPT_WINDOW :: 3;
}

#if OS == .ANDROID {
    TCPOPT_WINDOW :: 3;
}

#if OS == .MACOS {
    TCPOPT_WINDOW :: 3;
}

#if OS == .LINUX {
    TH_ACK :: 0x10;
}

#if OS == .ANDROID {
    TH_ACK :: 0x10;
}

#if OS == .MACOS {
    TH_ACK :: 0x10;
}

#if OS == .MACOS {
    TH_CWR :: 0x80;
}

#if OS == .MACOS {
    TH_ECE :: 0x40;
}

#if OS == .LINUX {
    TH_FIN :: 0x01;
}

#if OS == .ANDROID {
    TH_FIN :: 0x01;
}

#if OS == .MACOS {
    TH_FIN :: 0x01;
}

#if OS == .MACOS {
    TH_FLAGS :: TH_FIN | TH_SYN | TH_RST | TH_ACK | TH_URG | TH_ECE | TH_CWR;
}

#if OS == .WINDOWS {
    TH_NETDEV :: 0x00000001;
}

#if OS == .LINUX {
    TH_PUSH :: 0x08;
}

#if OS == .ANDROID {
    TH_PUSH :: 0x08;
}

#if OS == .MACOS {
    TH_PUSH :: 0x08;
}

#if OS == .LINUX {
    TH_RST :: 0x04;
}

#if OS == .ANDROID {
    TH_RST :: 0x04;
}

#if OS == .MACOS {
    TH_RST :: 0x04;
}

#if OS == .LINUX {
    TH_SYN :: 0x02;
}

#if OS == .ANDROID {
    TH_SYN :: 0x02;
}

#if OS == .MACOS {
    TH_SYN :: 0x02;
}

#if OS == .WINDOWS {
    TH_TAPI :: 0x00000002;
}

#if OS == .LINUX {
    TH_URG :: 0x20;
}

#if OS == .ANDROID {
    TH_URG :: 0x20;
}

#if OS == .MACOS {
    TH_URG :: 0x20;
}

#if OS == .WINDOWS {
    TIMESTAMPING_FLAG_RX :: 0x1;
}

#if OS == .WINDOWS {
    TIMESTAMPING_FLAG_TX :: 0x2;
}

#if OS == .WINDOWS {
    TIMEVAL :: timeval;
}

#if OS == .LINUX {
    TRY_AGAIN :: 2;
}

#if OS == .ANDROID {
    TRY_AGAIN :: 2;
}

#if OS == .MACOS {
    TRY_AGAIN :: 2;
}

#if OS == .MACOS {
    TTCP_CLIENT_SND_WND :: 4096;
}

#if OS == .WINDOWS {
    u_int64 :: u64;
}

#if OS == .WINDOWS {
    UDP_CHECKSUM_COVERAGE :: 20;
}

#if OS == .WINDOWS {
    UDP_COALESCED_INFO :: 3;
}

#if OS == .WINDOWS {
    UDP_NOCHECKSUM :: 1;
}

#if OS == .WINDOWS {
    UDP_RECV_MAX_COALESCED_SIZE :: 3;
}

#if OS == .WINDOWS {
    UDP_SEND_MSG_SIZE :: 2;
}

#if OS == .WINDOWS {
    WSA_E_CANCELLED :: (WSABASEERR + 111);
}

#if OS == .WINDOWS {
    WSA_E_NO_MORE :: (WSABASEERR + 110);
}

#if OS == .WINDOWS {
    WSA_FLAG_ACCESS_SYSTEM_SECURITY :: 0x40;
}

#if OS == .WINDOWS {
    WSA_FLAG_MULTIPOINT_C_LEAF :: 0x04;
}

#if OS == .WINDOWS {
    WSA_FLAG_MULTIPOINT_C_ROOT :: 0x02;
}

#if OS == .WINDOWS {
    WSA_FLAG_MULTIPOINT_D_LEAF :: 0x10;
}

#if OS == .WINDOWS {
    WSA_FLAG_MULTIPOINT_D_ROOT :: 0x08;
}

#if OS == .WINDOWS {
    WSA_FLAG_NO_HANDLE_INHERIT :: 0x80;
}

#if OS == .WINDOWS {
    WSA_FLAG_OVERLAPPED :: 0x01;
}

#if OS == .WINDOWS {
    WSA_FLAG_REGISTERED_IO :: 0x100;
}

#if OS == .WINDOWS {
    WSA_QOS_ADMISSION_FAILURE :: (WSABASEERR + 1010);
}

#if OS == .WINDOWS {
    WSA_QOS_BAD_OBJECT :: (WSABASEERR + 1013);
}

#if OS == .WINDOWS {
    WSA_QOS_BAD_STYLE :: (WSABASEERR + 1012);
}

#if OS == .WINDOWS {
    WSA_QOS_EFILTERCOUNT :: (WSABASEERR + 1021);
}

#if OS == .WINDOWS {
    WSA_QOS_EFILTERSTYLE :: (WSABASEERR + 1019);
}

#if OS == .WINDOWS {
    WSA_QOS_EFILTERTYPE :: (WSABASEERR + 1020);
}

#if OS == .WINDOWS {
    WSA_QOS_EFLOWCOUNT :: (WSABASEERR + 1023);
}

#if OS == .WINDOWS {
    WSA_QOS_EFLOWDESC :: (WSABASEERR + 1026);
}

#if OS == .WINDOWS {
    WSA_QOS_EFLOWSPEC :: (WSABASEERR + 1017);
}

#if OS == .WINDOWS {
    WSA_QOS_EOBJLENGTH :: (WSABASEERR + 1022);
}

#if OS == .WINDOWS {
    WSA_QOS_EPOLICYOBJ :: (WSABASEERR + 1025);
}

#if OS == .WINDOWS {
    WSA_QOS_EPROVSPECBUF :: (WSABASEERR + 1018);
}

#if OS == .WINDOWS {
    WSA_QOS_EPSFILTERSPEC :: (WSABASEERR + 1028);
}

#if OS == .WINDOWS {
    WSA_QOS_EPSFLOWSPEC :: (WSABASEERR + 1027);
}

#if OS == .WINDOWS {
    WSA_QOS_ESDMODEOBJ :: (WSABASEERR + 1029);
}

#if OS == .WINDOWS {
    WSA_QOS_ESERVICETYPE :: (WSABASEERR + 1016);
}

#if OS == .WINDOWS {
    WSA_QOS_ESHAPERATEOBJ :: (WSABASEERR + 1030);
}

#if OS == .WINDOWS {
    WSA_QOS_EUNKOWNPSOBJ :: (WSABASEERR + 1024);
}

#if OS == .WINDOWS {
    WSA_QOS_GENERIC_ERROR :: (WSABASEERR + 1015);
}

#if OS == .WINDOWS {
    WSA_QOS_NO_RECEIVERS :: (WSABASEERR + 1008);
}

#if OS == .WINDOWS {
    WSA_QOS_NO_SENDERS :: (WSABASEERR + 1007);
}

#if OS == .WINDOWS {
    WSA_QOS_POLICY_FAILURE :: (WSABASEERR + 1011);
}

#if OS == .WINDOWS {
    WSA_QOS_RECEIVERS :: (WSABASEERR + 1005);
}

#if OS == .WINDOWS {
    WSA_QOS_REQUEST_CONFIRMED :: (WSABASEERR + 1009);
}

#if OS == .WINDOWS {
    WSA_QOS_RESERVED_PETYPE :: (WSABASEERR + 1031);
}

#if OS == .WINDOWS {
    WSA_QOS_SENDERS :: (WSABASEERR + 1006);
}

#if OS == .WINDOWS {
    WSA_QOS_TRAFFIC_CTRL_ERROR :: (WSABASEERR + 1014);
}

#if OS == .WINDOWS {
    WSAAddressToString :: WSAAddressToStringA;
}

#if OS == .WINDOWS {
    WSABASEERR :: 10000;
}

#if OS == .WINDOWS {
    WSACMSGHDR :: cmsghdr;
}

#if OS == .WINDOWS {
    WSAConnectByName :: WSAConnectByNameA;
}

#if OS == .WINDOWS {
    WSADATA :: WSAData;
}

#if OS == .WINDOWS {
    WSADESCRIPTION_LEN :: 256;
}

#if OS == .WINDOWS {
    WSADuplicateSocket :: WSADuplicateSocketA;
}

#if OS == .WINDOWS {
    WSAEACCES :: (WSABASEERR + 13);
}

#if OS == .WINDOWS {
    WSAEADDRINUSE :: (WSABASEERR + 48);
}

#if OS == .WINDOWS {
    WSAEADDRNOTAVAIL :: (WSABASEERR + 49);
}

#if OS == .WINDOWS {
    WSAEAFNOSUPPORT :: (WSABASEERR + 47);
}

#if OS == .WINDOWS {
    WSAEALREADY :: (WSABASEERR + 37);
}

#if OS == .WINDOWS {
    WSAEBADF :: (WSABASEERR + 9);
}

#if OS == .WINDOWS {
    WSAECANCELLED :: (WSABASEERR + 103);
}

#if OS == .WINDOWS {
    WSAECOMPARATOR :: WSAEcomparator;
}

#if OS == .WINDOWS {
    WSAECONNABORTED :: (WSABASEERR + 53);
}

#if OS == .WINDOWS {
    WSAECONNREFUSED :: (WSABASEERR + 61);
}

#if OS == .WINDOWS {
    WSAECONNRESET :: (WSABASEERR + 54);
}

#if OS == .WINDOWS {
    WSAEDESTADDRREQ :: (WSABASEERR + 39);
}

#if OS == .WINDOWS {
    WSAEDISCON :: (WSABASEERR + 101);
}

#if OS == .WINDOWS {
    WSAEDQUOT :: (WSABASEERR + 69);
}

#if OS == .WINDOWS {
    WSAEFAULT :: (WSABASEERR + 14);
}

#if OS == .WINDOWS {
    WSAEHOSTDOWN :: (WSABASEERR + 64);
}

#if OS == .WINDOWS {
    WSAEHOSTUNREACH :: (WSABASEERR + 65);
}

#if OS == .WINDOWS {
    WSAEINPROGRESS :: (WSABASEERR + 36);
}

#if OS == .WINDOWS {
    WSAEINTR :: (WSABASEERR + 4);
}

#if OS == .WINDOWS {
    WSAEINVAL :: (WSABASEERR + 22);
}

#if OS == .WINDOWS {
    WSAEINVALIDPROCTABLE :: (WSABASEERR + 104);
}

#if OS == .WINDOWS {
    WSAEINVALIDPROVIDER :: (WSABASEERR + 105);
}

#if OS == .WINDOWS {
    WSAEISCONN :: (WSABASEERR + 56);
}

#if OS == .WINDOWS {
    WSAELOOP :: (WSABASEERR + 62);
}

#if OS == .WINDOWS {
    WSAEMFILE :: (WSABASEERR + 24);
}

#if OS == .WINDOWS {
    WSAEMSGSIZE :: (WSABASEERR + 40);
}

#if OS == .WINDOWS {
    WSAENAMETOOLONG :: (WSABASEERR + 63);
}

#if OS == .WINDOWS {
    WSAENETDOWN :: (WSABASEERR + 50);
}

#if OS == .WINDOWS {
    WSAENETRESET :: (WSABASEERR + 52);
}

#if OS == .WINDOWS {
    WSAENETUNREACH :: (WSABASEERR + 51);
}

#if OS == .WINDOWS {
    WSAENOBUFS :: (WSABASEERR + 55);
}

#if OS == .WINDOWS {
    WSAENOMORE :: (WSABASEERR + 102);
}

#if OS == .WINDOWS {
    WSAENOPROTOOPT :: (WSABASEERR + 42);
}

#if OS == .WINDOWS {
    WSAENOTCONN :: (WSABASEERR + 57);
}

#if OS == .WINDOWS {
    WSAENOTEMPTY :: (WSABASEERR + 66);
}

#if OS == .WINDOWS {
    WSAENOTSOCK :: (WSABASEERR + 38);
}

#if OS == .WINDOWS {
    WSAEnumProtocols :: WSAEnumProtocolsA;
}

#if OS == .WINDOWS {
    WSAEOPNOTSUPP :: (WSABASEERR + 45);
}

#if OS == .WINDOWS {
    WSAEPFNOSUPPORT :: (WSABASEERR + 46);
}

#if OS == .WINDOWS {
    WSAEPROCLIM :: (WSABASEERR + 67);
}

#if OS == .WINDOWS {
    WSAEPROTONOSUPPORT :: (WSABASEERR + 43);
}

#if OS == .WINDOWS {
    WSAEPROTOTYPE :: (WSABASEERR + 41);
}

#if OS == .WINDOWS {
    WSAEPROVIDERFAILEDINIT :: (WSABASEERR + 106);
}

#if OS == .WINDOWS {
    WSAEREFUSED :: (WSABASEERR + 112);
}

#if OS == .WINDOWS {
    WSAEREMOTE :: (WSABASEERR + 71);
}

#if OS == .WINDOWS {
    WSAESHUTDOWN :: (WSABASEERR + 58);
}

#if OS == .WINDOWS {
    WSAESOCKTNOSUPPORT :: (WSABASEERR + 44);
}

#if OS == .WINDOWS {
    WSAESTALE :: (WSABASEERR + 70);
}

#if OS == .WINDOWS {
    WSAETIMEDOUT :: (WSABASEERR + 60);
}

#if OS == .WINDOWS {
    WSAETOOMANYREFS :: (WSABASEERR + 59);
}

#if OS == .WINDOWS {
    WSAEUSERS :: (WSABASEERR + 68);
}

#if OS == .WINDOWS {
    WSAEWOULDBLOCK :: (WSABASEERR + 35);
}

#if OS == .WINDOWS {
    WSAGetServiceClassInfo :: WSAGetServiceClassInfoA;
}

#if OS == .WINDOWS {
    WSAGetServiceClassNameByClassId :: WSAGetServiceClassNameByClassIdA;
}

#if OS == .WINDOWS {
    WSAHOST_NOT_FOUND :: (WSABASEERR + 1001);
}

#if OS == .WINDOWS {
    WSAInstallServiceClass :: WSAInstallServiceClassA;
}

#if OS == .WINDOWS {
    WSANO_DATA :: (WSABASEERR + 1004);
}

#if OS == .WINDOWS {
    WSANO_RECOVERY :: (WSABASEERR + 1003);
}

#if OS == .WINDOWS {
    WSANOTINITIALISED :: (WSABASEERR + 93);
}

#if OS == .WINDOWS {
    WSANSCLASSINFO :: WSANSCLASSINFOA;
}

#if OS == .WINDOWS {
    WSANSCLASSINFOA :: WSANSClassInfoA;
}

#if OS == .WINDOWS {
    WSANSCLASSINFOW :: WSANSClassInfoW;
}

#if OS == .WINDOWS {
    WSAPOLLFD :: pollfd;
}

#if OS == .WINDOWS {
    WSAPROTOCOL_INFO :: WSAPROTOCOL_INFOA;
}

#if OS == .WINDOWS {
    WSAPROTOCOL_LEN :: 255;
}

#if OS == .WINDOWS {
    WSASERVICE_NOT_FOUND :: (WSABASEERR + 108);
}

#if OS == .WINDOWS {
    WSASocket :: WSASocketA;
}

#if OS == .WINDOWS {
    WSAStringToAddress :: WSAStringToAddressA;
}

#if OS == .WINDOWS {
    WSASYS_STATUS_LEN :: 128;
}

#if OS == .WINDOWS {
    WSASYSCALLFAILURE :: (WSABASEERR + 107);
}

#if OS == .WINDOWS {
    WSASYSNOTREADY :: (WSABASEERR + 91);
}

#if OS == .WINDOWS {
    WSATRY_AGAIN :: (WSABASEERR + 1002);
}

#if OS == .WINDOWS {
    WSATYPE_NOT_FOUND :: (WSABASEERR + 109);
}

#if OS == .WINDOWS {
    WSAVERNOTSUPPORTED :: (WSABASEERR + 92);
}

#if OS == .WINDOWS {
    WSAVERSION :: WSAVersion;
}

#if OS == .WINDOWS {
    WSK_SO_BASE :: 0x4000;
}

#if OS == .WINDOWS {
    XP1_CONNECT_DATA :: 0x00000080;
}

#if OS == .WINDOWS {
    XP1_CONNECTIONLESS :: 0x00000001;
}

#if OS == .WINDOWS {
    XP1_DISCONNECT_DATA :: 0x00000100;
}

#if OS == .WINDOWS {
    XP1_EXPEDITED_DATA :: 0x00000040;
}

#if OS == .WINDOWS {
    XP1_GRACEFUL_CLOSE :: 0x00000020;
}

#if OS == .WINDOWS {
    XP1_GUARANTEED_DELIVERY :: 0x00000002;
}

#if OS == .WINDOWS {
    XP1_GUARANTEED_ORDER :: 0x00000004;
}

#if OS == .WINDOWS {
    XP1_IFS_HANDLES :: 0x00020000;
}

#if OS == .WINDOWS {
    XP1_INTERRUPT :: 0x00004000;
}

#if OS == .WINDOWS {
    XP1_MESSAGE_ORIENTED :: 0x00000008;
}

#if OS == .WINDOWS {
    XP1_MULTIPOINT_CONTROL_PLANE :: 0x00000800;
}

#if OS == .WINDOWS {
    XP1_MULTIPOINT_DATA_PLANE :: 0x00001000;
}

#if OS == .WINDOWS {
    XP1_PARTIAL_MESSAGE :: 0x00040000;
}

#if OS == .WINDOWS {
    XP1_PSEUDO_STREAM :: 0x00000010;
}

#if OS == .WINDOWS {
    XP1_QOS_SUPPORTED :: 0x00002000;
}

#if OS == .WINDOWS {
    XP1_SAN_SUPPORT_SDP :: 0x00080000;
}

#if OS == .WINDOWS {
    XP1_SUPPORT_BROADCAST :: 0x00000200;
}

#if OS == .WINDOWS {
    XP1_SUPPORT_MULTIPOINT :: 0x00000400;
}

#if OS == .WINDOWS {
    XP1_UNI_RECV :: 0x00010000;
}

#if OS == .WINDOWS {
    XP1_UNI_SEND :: 0x00008000;
}
