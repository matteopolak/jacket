// Generated from docs/specs/stdlib/modules/curl.md.
// Regenerate with tools/native_bindings/generate.py; do not hand-edit.

#scope_file
FILE :: struct {};
time_t :: s64;
socklen_t :: u32;
SOCKET :: u64;

fd_set :: struct {
    #if OS == .WINDOWS {
        fd_count: u32;
        fd_array: [64] u64;
    } else #if OS == .MACOS || OS == .IOS {
        fds_bits: [32] s32;
    } else {
        fds_bits: [16] s64;
    }
}

sockaddr :: struct {
    #if OS == .MACOS || OS == .IOS {
        sa_len: u8;
        sa_family: u8;
    } else {
        sa_family: u16;
    }
    sa_data: [14] u8;
}

Native :: #import "internal/bindings/native";
Varargs :: #import "internal/bindings/varargs";

libcurl :: #library,system "curl";
#scope_export

// Types and layouts

#if OS != .WINDOWS {
    curl_blob :: struct {
        data: *void;
        len: u64;
        flags: u32;
    }

}

#if OS == .WINDOWS {
    curl_blob :: struct {
        data: *void;
        len: u64;
        flags: u32;
    }

}

#if OS != .WINDOWS {
    curl_certinfo :: struct {
        num_of_certs: s32;
        certinfo: **curl_slist;
    }

}

#if OS == .WINDOWS {
    curl_certinfo :: struct {
        num_of_certs: s32;
        certinfo: **curl_slist;
    }

}

#if OS != .WINDOWS {
    curl_closepolicy :: enum u32 {
        NONE :: 0;
        OLDEST :: 1;
        LEAST_RECENTLY_USED :: 2;
        LEAST_TRAFFIC :: 3;
        SLOWEST :: 4;
        CALLBACK :: 5;
        LAST :: 6;
    }

}

#if OS == .WINDOWS {
    curl_closepolicy :: enum s32 {
        NONE :: 0;
        OLDEST :: 1;
        LEAST_RECENTLY_USED :: 2;
        LEAST_TRAFFIC :: 3;
        SLOWEST :: 4;
        CALLBACK :: 5;
        LAST :: 6;
    }

}

#if OS != .WINDOWS {
    curl_easyoption :: struct {
        name: *u8;
        id: CURLoption;
        type: curl_easytype;
        flags: u32;
    }

}

#if OS == .WINDOWS {
    curl_easyoption :: struct {
        name: *u8;
        id: CURLoption;
        type: curl_easytype;
        flags: u32;
    }

}

#if OS != .WINDOWS {
    curl_easytype :: enum u32 {
        LONG :: 0;
        VALUES :: 1;
        OFF_T :: 2;
        OBJECT :: 3;
        STRING :: 4;
        SLIST :: 5;
        CBPTR :: 6;
        BLOB :: 7;
        FUNCTION :: 8;
    }

}

#if OS == .WINDOWS {
    curl_easytype :: enum s32 {
        LONG :: 0;
        VALUES :: 1;
        OFF_T :: 2;
        OBJECT :: 3;
        STRING :: 4;
        SLIST :: 5;
        CBPTR :: 6;
        BLOB :: 7;
        FUNCTION :: 8;
    }

}

#if OS != .WINDOWS {
    curl_fileinfo :: struct {
        filename: *u8;
        filetype: curlfiletype;
        time: time_t;
        perm: u32;
        uid: s32;
        gid: s32;
        size: curl_off_t;
        hardlinks: s64;
        strings: struct {
            time: *u8;
            perm: *u8;
            user: *u8;
            group: *u8;
            target: *u8;
        };
        flags: u32;
        b_data: *u8;
        b_size: u64;
        b_used: u64;
    }

}

#if OS == .WINDOWS {
    curl_fileinfo :: struct {
        filename: *u8;
        filetype: curlfiletype;
        time: time_t;
        perm: u32;
        uid: s32;
        gid: s32;
        size: curl_off_t;
        hardlinks: s32;
        strings: struct {
            time: *u8;
            perm: *u8;
            user: *u8;
            group: *u8;
            target: *u8;
        };
        flags: u32;
        b_data: *u8;
        b_size: u64;
        b_used: u64;
    }

}

#if OS != .WINDOWS {
    curl_forms :: struct {
        option: CURLformoption;
        value: *u8;
    }

}

#if OS == .WINDOWS {
    curl_forms :: struct {
        option: CURLformoption;
        value: *u8;
    }

}

#if OS != .WINDOWS {
    curl_ftpauth :: enum u32 {
        DEFAULT :: 0;
        SSL :: 1;
        TLS :: 2;
        LAST :: 3;
    }

}

#if OS == .WINDOWS {
    curl_ftpauth :: enum s32 {
        DEFAULT :: 0;
        SSL :: 1;
        TLS :: 2;
        LAST :: 3;
    }

}

#if OS != .WINDOWS {
    curl_ftpccc :: enum u32 {
        NONE :: 0;
        PASSIVE :: 1;
        ACTIVE :: 2;
        LAST :: 3;
    }

}

#if OS == .WINDOWS {
    curl_ftpccc :: enum s32 {
        NONE :: 0;
        PASSIVE :: 1;
        ACTIVE :: 2;
        LAST :: 3;
    }

}

#if OS != .WINDOWS {
    curl_ftpcreatedir :: enum u32 {
        DIR_NONE :: 0;
        DIR :: 1;
        DIR_RETRY :: 2;
        DIR_LAST :: 3;
    }

}

#if OS == .WINDOWS {
    curl_ftpcreatedir :: enum s32 {
        DIR_NONE :: 0;
        DIR :: 1;
        DIR_RETRY :: 2;
        DIR_LAST :: 3;
    }

}

#if OS != .WINDOWS {
    curl_ftpmethod :: enum u32 {
        DEFAULT :: 0;
        MULTICWD :: 1;
        NOCWD :: 2;
        SINGLECWD :: 3;
        LAST :: 4;
    }

}

#if OS == .WINDOWS {
    curl_ftpmethod :: enum s32 {
        DEFAULT :: 0;
        MULTICWD :: 1;
        NOCWD :: 2;
        SINGLECWD :: 3;
        LAST :: 4;
    }

}

#if OS != .WINDOWS {
    curl_header :: struct {
        name: *u8;
        value: *u8;
        amount: u64;
        index: u64;
        origin: u32;
        anchor: *void;
    }

}

#if OS == .WINDOWS {
    curl_header :: struct {
        name: *u8;
        value: *u8;
        amount: u64;
        index: u64;
        origin: u32;
        anchor: *void;
    }

}

#if OS != .WINDOWS {
    curl_hstsentry :: struct {
        name: *u8;
        namelen: u64;
        includeSubDomains: u8;
        expire: [18] u8;
    }

}

#if OS == .WINDOWS {
    curl_hstsentry :: struct {
        name: *u8;
        namelen: u64;
        includeSubDomains: u32;
        expire: [18] u8;
    }

}

#if OS != .WINDOWS {
    CURL_HTTP_VERSION :: enum u32 {
        NONE :: 0;
        _1_0 :: 1;
        _1_1 :: 2;
        _2_0 :: 3;
        _2TLS :: 4;
        _2_PRIOR_KNOWLEDGE :: 5;
        _3 :: 30;
        _3ONLY :: 31;
        LAST :: 32;
    }

}

#if OS == .WINDOWS {
    CURL_HTTP_VERSION :: enum s32 {
        NONE :: 0;
        _1_0 :: 1;
        _1_1 :: 2;
        _2_0 :: 3;
        _2TLS :: 4;
        _2_PRIOR_KNOWLEDGE :: 5;
        _3 :: 30;
        _3ONLY :: 31;
        LAST :: 32;
    }

}

#if OS != .WINDOWS {
    curl_httppost :: struct {
        next: *curl_httppost;
        name: *u8;
        namelength: s64;
        contents: *u8;
        contentslength: s64;
        buffer: *u8;
        bufferlength: s64;
        contenttype: *u8;
        contentheader: *curl_slist;
        more: *curl_httppost;
        flags: s64;
        showfilename: *u8;
        userp: *void;
        contentlen: curl_off_t;
    }

}

#if OS == .WINDOWS {
    curl_httppost :: struct {
        next: *curl_httppost;
        name: *u8;
        namelength: s32;
        contents: *u8;
        contentslength: s32;
        buffer: *u8;
        bufferlength: s32;
        contenttype: *u8;
        contentheader: *curl_slist;
        more: *curl_httppost;
        flags: s32;
        showfilename: *u8;
        userp: *void;
        contentlen: curl_off_t;
    }

}

#if OS != .WINDOWS {
    curl_index :: struct {
        index: u64;
        total: u64;
    }

}

#if OS == .WINDOWS {
    curl_index :: struct {
        index: u64;
        total: u64;
    }

}

#if OS != .WINDOWS {
    curl_infotype :: enum u32 {
        TEXT :: 0;
        HEADER_IN :: 1;
        HEADER_OUT :: 2;
        DATA_IN :: 3;
        DATA_OUT :: 4;
        SSL_DATA_IN :: 5;
        SSL_DATA_OUT :: 6;
        END :: 7;
    }

}

#if OS == .WINDOWS {
    curl_infotype :: enum s32 {
        TEXT :: 0;
        HEADER_IN :: 1;
        HEADER_OUT :: 2;
        DATA_IN :: 3;
        DATA_OUT :: 4;
        SSL_DATA_IN :: 5;
        SSL_DATA_OUT :: 6;
        END :: 7;
    }

}

#if OS != .WINDOWS {
    curl_khkey :: struct {
        key: *u8;
        len: u64;
        keytype: curl_khtype;
    }

}

#if OS == .WINDOWS {
    curl_khkey :: struct {
        key: *u8;
        len: u64;
        keytype: curl_khtype;
    }

}

#if OS != .WINDOWS {
    curl_khmatch :: enum u32 {
        OK :: 0;
        MISMATCH :: 1;
        MISSING :: 2;
        LAST :: 3;
    }

}

#if OS == .WINDOWS {
    curl_khmatch :: enum s32 {
        OK :: 0;
        MISMATCH :: 1;
        MISSING :: 2;
        LAST :: 3;
    }

}

#if OS != .WINDOWS {
    curl_khstat :: enum u32 {
        FINE_ADD_TO_FILE :: 0;
        FINE :: 1;
        REJECT :: 2;
        DEFER :: 3;
        FINE_REPLACE :: 4;
        LAST :: 5;
    }

}

#if OS == .WINDOWS {
    curl_khstat :: enum s32 {
        FINE_ADD_TO_FILE :: 0;
        FINE :: 1;
        REJECT :: 2;
        DEFER :: 3;
        FINE_REPLACE :: 4;
        LAST :: 5;
    }

}

#if OS != .WINDOWS {
    curl_khtype :: enum u32 {
        UNKNOWN :: 0;
        RSA1 :: 1;
        RSA :: 2;
        DSS :: 3;
        ECDSA :: 4;
        ED25519 :: 5;
    }

}

#if OS == .WINDOWS {
    curl_khtype :: enum s32 {
        UNKNOWN :: 0;
        RSA1 :: 1;
        RSA :: 2;
        DSS :: 3;
        ECDSA :: 4;
        ED25519 :: 5;
    }

}

#if OS != .WINDOWS {
    curl_lock_access :: enum u32 {
        NONE :: 0;
        SHARED :: 1;
        SINGLE :: 2;
        LAST :: 3;
    }

}

#if OS == .WINDOWS {
    curl_lock_access :: enum s32 {
        NONE :: 0;
        SHARED :: 1;
        SINGLE :: 2;
        LAST :: 3;
    }

}

#if OS != .WINDOWS {
    curl_lock_data :: enum u32 {
        NONE :: 0;
        SHARE :: 1;
        COOKIE :: 2;
        DNS :: 3;
        SSL_SESSION :: 4;
        CONNECT :: 5;
        PSL :: 6;
        HSTS :: 7;
        LAST :: 8;
    }

}

#if OS == .WINDOWS {
    curl_lock_data :: enum s32 {
        NONE :: 0;
        SHARE :: 1;
        COOKIE :: 2;
        DNS :: 3;
        SSL_SESSION :: 4;
        CONNECT :: 5;
        PSL :: 6;
        HSTS :: 7;
        LAST :: 8;
    }

}

#if OS != .WINDOWS {
    curl_mime :: struct {}

}

#if OS == .WINDOWS {
    curl_mime :: struct {}

}

#if OS != .WINDOWS {
    curl_mimepart :: struct {}

}

#if OS == .WINDOWS {
    curl_mimepart :: struct {}

}

#if OS != .WINDOWS {
    CURL_NETRC_OPTION :: enum u32 {
        CURL_NETRC_IGNORED :: 0;
        AL :: 1;
        CURL_NETRC_REQUIRED :: 2;
        CURL_NETRC_LAST :: 3;
    }

}

#if OS == .WINDOWS {
    CURL_NETRC_OPTION :: enum s32 {
        CURL_NETRC_IGNORED :: 0;
        AL :: 1;
        CURL_NETRC_REQUIRED :: 2;
        CURL_NETRC_LAST :: 3;
    }

}

#if OS != .WINDOWS {
    curl_proxytype :: enum u32 {
        HTTP :: 0;
        HTTP_1_0 :: 1;
        HTTPS :: 2;
        HTTPS2 :: 3;
        SOCKS4 :: 4;
        SOCKS5 :: 5;
        SOCKS4A :: 6;
        SOCKS5_HOSTNAME :: 7;
    }

}

#if OS == .WINDOWS {
    curl_proxytype :: enum s32 {
        HTTP :: 0;
        HTTP_1_0 :: 1;
        HTTPS :: 2;
        HTTPS2 :: 3;
        SOCKS4 :: 4;
        SOCKS5 :: 5;
        SOCKS4A :: 6;
        SOCKS5_HOSTNAME :: 7;
    }

}

#if OS != .WINDOWS {
    curl_pushheaders :: struct {}

}

#if OS == .WINDOWS {
    curl_pushheaders :: struct {}

}

#if OS != .WINDOWS {
    CURL_RTSPREQ :: enum u32 {
        NONE :: 0;
        OPTIONS :: 1;
        DESCRIBE :: 2;
        ANNOUNCE :: 3;
        SETUP :: 4;
        PLAY :: 5;
        PAUSE :: 6;
        TEARDOWN :: 7;
        GET_PARAMETER :: 8;
        SET_PARAMETER :: 9;
        RECORD :: 10;
        RECEIVE :: 11;
        LAST :: 12;
    }

}

#if OS == .WINDOWS {
    CURL_RTSPREQ :: enum s32 {
        NONE :: 0;
        OPTIONS :: 1;
        DESCRIBE :: 2;
        ANNOUNCE :: 3;
        SETUP :: 4;
        PLAY :: 5;
        PAUSE :: 6;
        TEARDOWN :: 7;
        GET_PARAMETER :: 8;
        SET_PARAMETER :: 9;
        RECORD :: 10;
        RECEIVE :: 11;
        LAST :: 12;
    }

}

#if OS != .WINDOWS {
    curl_slist :: struct {
        data: *u8;
        next: *curl_slist;
    }

}

#if OS == .WINDOWS {
    curl_slist :: struct {
        data: *u8;
        next: *curl_slist;
    }

}

#if OS != .WINDOWS {
    curl_sockaddr :: struct {
        family: s32;
        socktype: s32;
        protocol: s32;
        addrlen: u32;
        addr: sockaddr;
    }

}

#if OS == .WINDOWS {
    curl_sockaddr :: struct {
        family: s32;
        socktype: s32;
        protocol: s32;
        addrlen: u32;
        addr: sockaddr;
    }

}

#if OS != .WINDOWS {
    curl_ssl_backend :: struct {
        id: curl_sslbackend;
        name: *u8;
    }

}

#if OS == .WINDOWS {
    curl_ssl_backend :: struct {
        id: curl_sslbackend;
        name: *u8;
    }

}

#if OS != .WINDOWS {
    curl_sslbackend :: enum u32 {
        NONE :: 0;
        OPENSSL :: 1;
        GNUTLS :: 2;
        NSS :: 3;
        OBSOLETE4 :: 4;
        GSKIT :: 5;
        POLARSSL :: 6;
        WOLFSSL :: 7;
        SCHANNEL :: 8;
        SECURETRANSPORT :: 9;
        AXTLS :: 10;
        MBEDTLS :: 11;
        MESALINK :: 12;
        BEARSSL :: 13;
        RUSTLS :: 14;
    }

}

#if OS == .WINDOWS {
    curl_sslbackend :: enum s32 {
        NONE :: 0;
        OPENSSL :: 1;
        GNUTLS :: 2;
        NSS :: 3;
        OBSOLETE4 :: 4;
        GSKIT :: 5;
        POLARSSL :: 6;
        WOLFSSL :: 7;
        SCHANNEL :: 8;
        SECURETRANSPORT :: 9;
        AXTLS :: 10;
        MBEDTLS :: 11;
        MESALINK :: 12;
        BEARSSL :: 13;
        RUSTLS :: 14;
    }

}

#if OS != .WINDOWS {
    CURL_SSLVERSION :: enum u32 {
        DEFAULT :: 0;
        TLSv1 :: 1;
        SSLv2 :: 2;
        SSLv3 :: 3;
        TLSv1_0 :: 4;
        TLSv1_1 :: 5;
        TLSv1_2 :: 6;
        TLSv1_3 :: 7;
        LAST :: 8;
    }

}

#if OS == .WINDOWS {
    CURL_SSLVERSION :: enum s32 {
        DEFAULT :: 0;
        TLSv1 :: 1;
        SSLv2 :: 2;
        SSLv3 :: 3;
        TLSv1_0 :: 4;
        TLSv1_1 :: 5;
        TLSv1_2 :: 6;
        TLSv1_3 :: 7;
        LAST :: 8;
    }

}

#if OS != .WINDOWS {
    CURL_SSLVERSION_MAX :: enum u32 {
        NONE :: 0;
        DEFAULT :: 65536;
        TLSv1_0 :: 262144;
        TLSv1_1 :: 327680;
        TLSv1_2 :: 393216;
        TLSv1_3 :: 458752;
        LAST :: 524288;
    }

}

#if OS == .WINDOWS {
    CURL_SSLVERSION_MAX :: enum s32 {
        NONE :: 0;
        DEFAULT :: 65536;
        TLSv1_0 :: 262144;
        TLSv1_1 :: 327680;
        TLSv1_2 :: 393216;
        TLSv1_3 :: 458752;
        LAST :: 524288;
    }

}

#if OS != .WINDOWS {
    curl_TimeCond :: enum u32 {
        NONE :: 0;
        IFMODSINCE :: 1;
        IFUNMODSINCE :: 2;
        LASTMOD :: 3;
        LAST :: 4;
    }

}

#if OS == .WINDOWS {
    curl_TimeCond :: enum s32 {
        NONE :: 0;
        IFMODSINCE :: 1;
        IFUNMODSINCE :: 2;
        LASTMOD :: 3;
        LAST :: 4;
    }

}

#if OS != .WINDOWS {
    CURL_TLSAUTH :: enum u32 {
        NONE :: 0;
        SRP :: 1;
        LAST :: 2;
    }

}

#if OS == .WINDOWS {
    CURL_TLSAUTH :: enum s32 {
        NONE :: 0;
        SRP :: 1;
        LAST :: 2;
    }

}

#if OS != .WINDOWS {
    curl_tlssessioninfo :: struct {
        backend: curl_sslbackend;
        internals: *void;
    }

}

#if OS == .WINDOWS {
    curl_tlssessioninfo :: struct {
        backend: curl_sslbackend;
        internals: *void;
    }

}

#if OS != .WINDOWS {
    Curl_URL :: struct {}

}

#if OS == .WINDOWS {
    Curl_URL :: struct {}

}

#if OS != .WINDOWS {
    curl_usessl :: enum u32 {
        NONE :: 0;
        TRY :: 1;
        CONTROL :: 2;
        ALL :: 3;
        LAST :: 4;
    }

}

#if OS == .WINDOWS {
    curl_usessl :: enum s32 {
        NONE :: 0;
        TRY :: 1;
        CONTROL :: 2;
        ALL :: 3;
        LAST :: 4;
    }

}

#if OS != .WINDOWS {
    curl_version_info_data :: struct {
        age: CURLversion;
        version: *u8;
        version_num: u32;
        host: *u8;
        features: s32;
        ssl_version: *u8;
        ssl_version_num: s64;
        libz_version: *u8;
        protocols: **u8;
        ares: *u8;
        ares_num: s32;
        libidn: *u8;
        iconv_ver_num: s32;
        libssh_version: *u8;
        brotli_ver_num: u32;
        brotli_version: *u8;
        nghttp2_ver_num: u32;
        nghttp2_version: *u8;
        quic_version: *u8;
        cainfo: *u8;
        capath: *u8;
        zstd_ver_num: u32;
        zstd_version: *u8;
        hyper_version: *u8;
        gsasl_version: *u8;
        feature_names: **u8;
    }

}

#if OS == .WINDOWS {
    curl_version_info_data :: struct {
        age: CURLversion;
        version: *u8;
        version_num: u32;
        host: *u8;
        features: s32;
        ssl_version: *u8;
        ssl_version_num: s32;
        libz_version: *u8;
        protocols: **u8;
        ares: *u8;
        ares_num: s32;
        libidn: *u8;
        iconv_ver_num: s32;
        libssh_version: *u8;
        brotli_ver_num: u32;
        brotli_version: *u8;
        nghttp2_ver_num: u32;
        nghttp2_version: *u8;
        quic_version: *u8;
        cainfo: *u8;
        capath: *u8;
        zstd_ver_num: u32;
        zstd_version: *u8;
        hyper_version: *u8;
        gsasl_version: *u8;
        feature_names: **u8;
    }

}

#if OS != .WINDOWS {
    curl_waitfd :: struct {
        fd: curl_socket_t;
        events: s16;
        revents: s16;
    }

}

#if OS == .WINDOWS {
    curl_waitfd :: struct {
        fd: curl_socket_t;
        events: s16;
        revents: s16;
    }

}

#if OS != .WINDOWS {
    curl_ws_frame :: struct {
        age: s32;
        flags: s32;
        offset: curl_off_t;
        bytesleft: curl_off_t;
        len: u64;
    }

}

#if OS == .WINDOWS {
    curl_ws_frame :: struct {
        age: s32;
        flags: s32;
        offset: curl_off_t;
        bytesleft: curl_off_t;
        len: u64;
    }

}

#if OS != .WINDOWS {
    CURLcode :: enum u32 {
        OK :: 0;
        UNSUPPORTED_PROTOCOL :: 1;
        FAILED_INIT :: 2;
        URL_MALFORMAT :: 3;
        NOT_BUILT_IN :: 4;
        COULDNT_RESOLVE_PROXY :: 5;
        COULDNT_RESOLVE_HOST :: 6;
        COULDNT_CONNECT :: 7;
        WEIRD_SERVER_REPLY :: 8;
        REMOTE_ACCESS_DENIED :: 9;
        FTP_ACCEPT_FAILED :: 10;
        FTP_WEIRD_PASS_REPLY :: 11;
        FTP_ACCEPT_TIMEOUT :: 12;
        FTP_WEIRD_PASV_REPLY :: 13;
        FTP_WEIRD_227_FORMAT :: 14;
        FTP_CANT_GET_HOST :: 15;
        HTTP2 :: 16;
        FTP_COULDNT_SET_TYPE :: 17;
        PARTIAL_FILE :: 18;
        FTP_COULDNT_RETR_FILE :: 19;
        OBSOLETE20 :: 20;
        QUOTE_ERROR :: 21;
        HTTP_RETURNED_ERROR :: 22;
        WRITE_ERROR :: 23;
        OBSOLETE24 :: 24;
        UPLOAD_FAILED :: 25;
        READ_ERROR :: 26;
        OUT_OF_MEMORY :: 27;
        OPERATION_TIMEDOUT :: 28;
        OBSOLETE29 :: 29;
        FTP_PORT_FAILED :: 30;
        FTP_COULDNT_USE_REST :: 31;
        OBSOLETE32 :: 32;
        RANGE_ERROR :: 33;
        HTTP_POST_ERROR :: 34;
        SSL_CONNECT_ERROR :: 35;
        BAD_DOWNLOAD_RESUME :: 36;
        FILE_COULDNT_READ_FILE :: 37;
        LDAP_CANNOT_BIND :: 38;
        LDAP_SEARCH_FAILED :: 39;
        OBSOLETE40 :: 40;
        FUNCTION_NOT_FOUND :: 41;
        ABORTED_BY_CALLBACK :: 42;
        BAD_FUNCTION_ARGUMENT :: 43;
        OBSOLETE44 :: 44;
        INTERFACE_FAILED :: 45;
        OBSOLETE46 :: 46;
        TOO_MANY_REDIRECTS :: 47;
        UNKNOWN_OPTION :: 48;
        SETOPT_OPTION_SYNTAX :: 49;
        OBSOLETE50 :: 50;
        OBSOLETE51 :: 51;
        GOT_NOTHING :: 52;
        SSL_ENGINE_NOTFOUND :: 53;
        SSL_ENGINE_SETFAILED :: 54;
        SEND_ERROR :: 55;
        RECV_ERROR :: 56;
        OBSOLETE57 :: 57;
        SSL_CERTPROBLEM :: 58;
        SSL_CIPHER :: 59;
        PEER_FAILED_VERIFICATION :: 60;
        BAD_CONTENT_ENCODING :: 61;
        OBSOLETE62 :: 62;
        FILESIZE_EXCEEDED :: 63;
        USE_SSL_FAILED :: 64;
        SEND_FAIL_REWIND :: 65;
        SSL_ENGINE_INITFAILED :: 66;
        LOGIN_DENIED :: 67;
        TFTP_NOTFOUND :: 68;
        TFTP_PERM :: 69;
        REMOTE_DISK_FULL :: 70;
        TFTP_ILLEGAL :: 71;
        TFTP_UNKNOWNID :: 72;
        REMOTE_FILE_EXISTS :: 73;
        TFTP_NOSUCHUSER :: 74;
        OBSOLETE75 :: 75;
        OBSOLETE76 :: 76;
        SSL_CACERT_BADFILE :: 77;
        REMOTE_FILE_NOT_FOUND :: 78;
        SSH :: 79;
        SSL_SHUTDOWN_FAILED :: 80;
        AGAIN :: 81;
        SSL_CRL_BADFILE :: 82;
        SSL_ISSUER_ERROR :: 83;
        FTP_PRET_FAILED :: 84;
        RTSP_CSEQ_ERROR :: 85;
        RTSP_SESSION_ERROR :: 86;
        FTP_BAD_FILE_LIST :: 87;
        CHUNK_FAILED :: 88;
        NO_CONNECTION_AVAILABLE :: 89;
        SSL_PINNEDPUBKEYNOTMATCH :: 90;
        SSL_INVALIDCERTSTATUS :: 91;
        HTTP2_STREAM :: 92;
        RECURSIVE_API_CALL :: 93;
        AUTH_ERROR :: 94;
        HTTP3 :: 95;
        QUIC_CONNECT_ERROR :: 96;
        PROXY :: 97;
        SSL_CLIENTCERT :: 98;
        UNRECOVERABLE_POLL :: 99;
        CURL_LAST :: 100;
    }

}

#if OS == .WINDOWS {
    CURLcode :: enum s32 {
        OK :: 0;
        UNSUPPORTED_PROTOCOL :: 1;
        FAILED_INIT :: 2;
        URL_MALFORMAT :: 3;
        NOT_BUILT_IN :: 4;
        COULDNT_RESOLVE_PROXY :: 5;
        COULDNT_RESOLVE_HOST :: 6;
        COULDNT_CONNECT :: 7;
        WEIRD_SERVER_REPLY :: 8;
        REMOTE_ACCESS_DENIED :: 9;
        FTP_ACCEPT_FAILED :: 10;
        FTP_WEIRD_PASS_REPLY :: 11;
        FTP_ACCEPT_TIMEOUT :: 12;
        FTP_WEIRD_PASV_REPLY :: 13;
        FTP_WEIRD_227_FORMAT :: 14;
        FTP_CANT_GET_HOST :: 15;
        HTTP2 :: 16;
        FTP_COULDNT_SET_TYPE :: 17;
        PARTIAL_FILE :: 18;
        FTP_COULDNT_RETR_FILE :: 19;
        OBSOLETE20 :: 20;
        QUOTE_ERROR :: 21;
        HTTP_RETURNED_ERROR :: 22;
        WRITE_ERROR :: 23;
        OBSOLETE24 :: 24;
        UPLOAD_FAILED :: 25;
        READ_ERROR :: 26;
        OUT_OF_MEMORY :: 27;
        OPERATION_TIMEDOUT :: 28;
        OBSOLETE29 :: 29;
        FTP_PORT_FAILED :: 30;
        FTP_COULDNT_USE_REST :: 31;
        OBSOLETE32 :: 32;
        RANGE_ERROR :: 33;
        HTTP_POST_ERROR :: 34;
        SSL_CONNECT_ERROR :: 35;
        BAD_DOWNLOAD_RESUME :: 36;
        FILE_COULDNT_READ_FILE :: 37;
        LDAP_CANNOT_BIND :: 38;
        LDAP_SEARCH_FAILED :: 39;
        OBSOLETE40 :: 40;
        FUNCTION_NOT_FOUND :: 41;
        ABORTED_BY_CALLBACK :: 42;
        BAD_FUNCTION_ARGUMENT :: 43;
        OBSOLETE44 :: 44;
        INTERFACE_FAILED :: 45;
        OBSOLETE46 :: 46;
        TOO_MANY_REDIRECTS :: 47;
        UNKNOWN_OPTION :: 48;
        SETOPT_OPTION_SYNTAX :: 49;
        OBSOLETE50 :: 50;
        OBSOLETE51 :: 51;
        GOT_NOTHING :: 52;
        SSL_ENGINE_NOTFOUND :: 53;
        SSL_ENGINE_SETFAILED :: 54;
        SEND_ERROR :: 55;
        RECV_ERROR :: 56;
        OBSOLETE57 :: 57;
        SSL_CERTPROBLEM :: 58;
        SSL_CIPHER :: 59;
        PEER_FAILED_VERIFICATION :: 60;
        BAD_CONTENT_ENCODING :: 61;
        OBSOLETE62 :: 62;
        FILESIZE_EXCEEDED :: 63;
        USE_SSL_FAILED :: 64;
        SEND_FAIL_REWIND :: 65;
        SSL_ENGINE_INITFAILED :: 66;
        LOGIN_DENIED :: 67;
        TFTP_NOTFOUND :: 68;
        TFTP_PERM :: 69;
        REMOTE_DISK_FULL :: 70;
        TFTP_ILLEGAL :: 71;
        TFTP_UNKNOWNID :: 72;
        REMOTE_FILE_EXISTS :: 73;
        TFTP_NOSUCHUSER :: 74;
        OBSOLETE75 :: 75;
        OBSOLETE76 :: 76;
        SSL_CACERT_BADFILE :: 77;
        REMOTE_FILE_NOT_FOUND :: 78;
        SSH :: 79;
        SSL_SHUTDOWN_FAILED :: 80;
        AGAIN :: 81;
        SSL_CRL_BADFILE :: 82;
        SSL_ISSUER_ERROR :: 83;
        FTP_PRET_FAILED :: 84;
        RTSP_CSEQ_ERROR :: 85;
        RTSP_SESSION_ERROR :: 86;
        FTP_BAD_FILE_LIST :: 87;
        CHUNK_FAILED :: 88;
        NO_CONNECTION_AVAILABLE :: 89;
        SSL_PINNEDPUBKEYNOTMATCH :: 90;
        SSL_INVALIDCERTSTATUS :: 91;
        HTTP2_STREAM :: 92;
        RECURSIVE_API_CALL :: 93;
        AUTH_ERROR :: 94;
        HTTP3 :: 95;
        QUIC_CONNECT_ERROR :: 96;
        PROXY :: 97;
        SSL_CLIENTCERT :: 98;
        UNRECOVERABLE_POLL :: 99;
        CURL_LAST :: 100;
    }

}

#if OS != .WINDOWS {
    curlfiletype :: enum u32 {
        FILE :: 0;
        DIRECTORY :: 1;
        SYMLINK :: 2;
        DEVICE_BLOCK :: 3;
        DEVICE_CHAR :: 4;
        NAMEDPIPE :: 5;
        SOCKET :: 6;
        DOOR :: 7;
        UNKNOWN :: 8;
    }

}

#if OS == .WINDOWS {
    curlfiletype :: enum s32 {
        FILE :: 0;
        DIRECTORY :: 1;
        SYMLINK :: 2;
        DEVICE_BLOCK :: 3;
        DEVICE_CHAR :: 4;
        NAMEDPIPE :: 5;
        SOCKET :: 6;
        DOOR :: 7;
        UNKNOWN :: 8;
    }

}

#if OS != .WINDOWS {
    CURLFORMcode :: enum u32 {
        OK :: 0;
        MEMORY :: 1;
        OPTION_TWICE :: 2;
        NULL :: 3;
        UNKNOWN_OPTION :: 4;
        INCOMPLETE :: 5;
        ILLEGAL_ARRAY :: 6;
        DISABLED :: 7;
        LAST :: 8;
    }

}

#if OS == .WINDOWS {
    CURLFORMcode :: enum s32 {
        OK :: 0;
        MEMORY :: 1;
        OPTION_TWICE :: 2;
        NULL :: 3;
        UNKNOWN_OPTION :: 4;
        INCOMPLETE :: 5;
        ILLEGAL_ARRAY :: 6;
        DISABLED :: 7;
        LAST :: 8;
    }

}

#if OS != .WINDOWS {
    CURLformoption :: enum u32 {
        NOTHING :: 0;
        COPYNAME :: 1;
        PTRNAME :: 2;
        NAMELENGTH :: 3;
        COPYCONTENTS :: 4;
        PTRCONTENTS :: 5;
        CONTENTSLENGTH :: 6;
        FILECONTENT :: 7;
        ARRAY :: 8;
        OBSOLETE :: 9;
        FILE :: 10;
        BUFFER :: 11;
        BUFFERPTR :: 12;
        BUFFERLENGTH :: 13;
        CONTENTTYPE :: 14;
        CONTENTHEADER :: 15;
        FILENAME :: 16;
        END :: 17;
        OBSOLETE2 :: 18;
        STREAM :: 19;
        CONTENTLEN :: 20;
        LASTENTRY :: 21;
    }

}

#if OS == .WINDOWS {
    CURLformoption :: enum s32 {
        NOTHING :: 0;
        COPYNAME :: 1;
        PTRNAME :: 2;
        NAMELENGTH :: 3;
        COPYCONTENTS :: 4;
        PTRCONTENTS :: 5;
        CONTENTSLENGTH :: 6;
        FILECONTENT :: 7;
        ARRAY :: 8;
        OBSOLETE :: 9;
        FILE :: 10;
        BUFFER :: 11;
        BUFFERPTR :: 12;
        BUFFERLENGTH :: 13;
        CONTENTTYPE :: 14;
        CONTENTHEADER :: 15;
        FILENAME :: 16;
        END :: 17;
        OBSOLETE2 :: 18;
        STREAM :: 19;
        CONTENTLEN :: 20;
        LASTENTRY :: 21;
    }

}

#if OS != .WINDOWS {
    CURLHcode :: enum u32 {
        OK :: 0;
        BADINDEX :: 1;
        MISSING :: 2;
        NOHEADERS :: 3;
        NOREQUEST :: 4;
        OUT_OF_MEMORY :: 5;
        BAD_ARGUMENT :: 6;
        NOT_BUILT_IN :: 7;
    }

}

#if OS == .WINDOWS {
    CURLHcode :: enum s32 {
        OK :: 0;
        BADINDEX :: 1;
        MISSING :: 2;
        NOHEADERS :: 3;
        NOREQUEST :: 4;
        OUT_OF_MEMORY :: 5;
        BAD_ARGUMENT :: 6;
        NOT_BUILT_IN :: 7;
    }

}

#if OS != .WINDOWS {
    CURLINFO :: enum u32 {
        NONE :: 0;
        EFFECTIVE_URL :: 1048577;
        RESPONSE_CODE :: 2097154;
        TOTAL_TIME :: 3145731;
        NAMELOOKUP_TIME :: 3145732;
        CONNECT_TIME :: 3145733;
        PRETRANSFER_TIME :: 3145734;
        SIZE_UPLOAD :: 3145735;
        SIZE_UPLOAD_T :: 6291463;
        SIZE_DOWNLOAD :: 3145736;
        SIZE_DOWNLOAD_T :: 6291464;
        SPEED_DOWNLOAD :: 3145737;
        SPEED_DOWNLOAD_T :: 6291465;
        SPEED_UPLOAD :: 3145738;
        SPEED_UPLOAD_T :: 6291466;
        HEADER_SIZE :: 2097163;
        REQUEST_SIZE :: 2097164;
        SSL_VERIFYRESULT :: 2097165;
        FILETIME :: 2097166;
        FILETIME_T :: 6291470;
        CONTENT_LENGTH_DOWNLOAD :: 3145743;
        CONTENT_LENGTH_DOWNLOAD_T :: 6291471;
        CONTENT_LENGTH_UPLOAD :: 3145744;
        CONTENT_LENGTH_UPLOAD_T :: 6291472;
        STARTTRANSFER_TIME :: 3145745;
        CONTENT_TYPE :: 1048594;
        REDIRECT_TIME :: 3145747;
        REDIRECT_COUNT :: 2097172;
        PRIVATE :: 1048597;
        HTTP_CONNECTCODE :: 2097174;
        HTTPAUTH_AVAIL :: 2097175;
        PROXYAUTH_AVAIL :: 2097176;
        OS_ERRNO :: 2097177;
        NUM_CONNECTS :: 2097178;
        SSL_ENGINES :: 4194331;
        COOKIELIST :: 4194332;
        LASTSOCKET :: 2097181;
        FTP_ENTRY_PATH :: 1048606;
        REDIRECT_URL :: 1048607;
        PRIMARY_IP :: 1048608;
        APPCONNECT_TIME :: 3145761;
        CERTINFO :: 4194338;
        CONDITION_UNMET :: 2097187;
        RTSP_SESSION_ID :: 1048612;
        RTSP_CLIENT_CSEQ :: 2097189;
        RTSP_SERVER_CSEQ :: 2097190;
        RTSP_CSEQ_RECV :: 2097191;
        PRIMARY_PORT :: 2097192;
        LOCAL_IP :: 1048617;
        LOCAL_PORT :: 2097194;
        TLS_SESSION :: 4194347;
        ACTIVESOCKET :: 5242924;
        TLS_SSL_PTR :: 4194349;
        HTTP_VERSION :: 2097198;
        PROXY_SSL_VERIFYRESULT :: 2097199;
        PROTOCOL :: 2097200;
        SCHEME :: 1048625;
        TOTAL_TIME_T :: 6291506;
        NAMELOOKUP_TIME_T :: 6291507;
        CONNECT_TIME_T :: 6291508;
        PRETRANSFER_TIME_T :: 6291509;
        STARTTRANSFER_TIME_T :: 6291510;
        REDIRECT_TIME_T :: 6291511;
        APPCONNECT_TIME_T :: 6291512;
        RETRY_AFTER :: 6291513;
        EFFECTIVE_METHOD :: 1048634;
        PROXY_ERROR :: 2097211;
        REFERER :: 1048636;
        CAINFO :: 1048637;
        CAPATH :: 1048638;
        XFER_ID :: 6291519;
        CONN_ID :: 6291520;
        LASTONE :: 64;
    }

}

#if OS == .WINDOWS {
    CURLINFO :: enum s32 {
        NONE :: 0;
        EFFECTIVE_URL :: 1048577;
        RESPONSE_CODE :: 2097154;
        TOTAL_TIME :: 3145731;
        NAMELOOKUP_TIME :: 3145732;
        CONNECT_TIME :: 3145733;
        PRETRANSFER_TIME :: 3145734;
        SIZE_UPLOAD :: 3145735;
        SIZE_UPLOAD_T :: 6291463;
        SIZE_DOWNLOAD :: 3145736;
        SIZE_DOWNLOAD_T :: 6291464;
        SPEED_DOWNLOAD :: 3145737;
        SPEED_DOWNLOAD_T :: 6291465;
        SPEED_UPLOAD :: 3145738;
        SPEED_UPLOAD_T :: 6291466;
        HEADER_SIZE :: 2097163;
        REQUEST_SIZE :: 2097164;
        SSL_VERIFYRESULT :: 2097165;
        FILETIME :: 2097166;
        FILETIME_T :: 6291470;
        CONTENT_LENGTH_DOWNLOAD :: 3145743;
        CONTENT_LENGTH_DOWNLOAD_T :: 6291471;
        CONTENT_LENGTH_UPLOAD :: 3145744;
        CONTENT_LENGTH_UPLOAD_T :: 6291472;
        STARTTRANSFER_TIME :: 3145745;
        CONTENT_TYPE :: 1048594;
        REDIRECT_TIME :: 3145747;
        REDIRECT_COUNT :: 2097172;
        PRIVATE :: 1048597;
        HTTP_CONNECTCODE :: 2097174;
        HTTPAUTH_AVAIL :: 2097175;
        PROXYAUTH_AVAIL :: 2097176;
        OS_ERRNO :: 2097177;
        NUM_CONNECTS :: 2097178;
        SSL_ENGINES :: 4194331;
        COOKIELIST :: 4194332;
        LASTSOCKET :: 2097181;
        FTP_ENTRY_PATH :: 1048606;
        REDIRECT_URL :: 1048607;
        PRIMARY_IP :: 1048608;
        APPCONNECT_TIME :: 3145761;
        CERTINFO :: 4194338;
        CONDITION_UNMET :: 2097187;
        RTSP_SESSION_ID :: 1048612;
        RTSP_CLIENT_CSEQ :: 2097189;
        RTSP_SERVER_CSEQ :: 2097190;
        RTSP_CSEQ_RECV :: 2097191;
        PRIMARY_PORT :: 2097192;
        LOCAL_IP :: 1048617;
        LOCAL_PORT :: 2097194;
        TLS_SESSION :: 4194347;
        ACTIVESOCKET :: 5242924;
        TLS_SSL_PTR :: 4194349;
        HTTP_VERSION :: 2097198;
        PROXY_SSL_VERIFYRESULT :: 2097199;
        PROTOCOL :: 2097200;
        SCHEME :: 1048625;
        TOTAL_TIME_T :: 6291506;
        NAMELOOKUP_TIME_T :: 6291507;
        CONNECT_TIME_T :: 6291508;
        PRETRANSFER_TIME_T :: 6291509;
        STARTTRANSFER_TIME_T :: 6291510;
        REDIRECT_TIME_T :: 6291511;
        APPCONNECT_TIME_T :: 6291512;
        RETRY_AFTER :: 6291513;
        EFFECTIVE_METHOD :: 1048634;
        PROXY_ERROR :: 2097211;
        REFERER :: 1048636;
        CAINFO :: 1048637;
        CAPATH :: 1048638;
        XFER_ID :: 6291519;
        CONN_ID :: 6291520;
        LASTONE :: 64;
    }

}

#if OS != .WINDOWS {
    curliocmd :: enum u32 {
        NOP :: 0;
        RESTARTREAD :: 1;
        LAST :: 2;
    }

}

#if OS == .WINDOWS {
    curliocmd :: enum s32 {
        NOP :: 0;
        RESTARTREAD :: 1;
        LAST :: 2;
    }

}

#if OS != .WINDOWS {
    curlioerr :: enum u32 {
        OK :: 0;
        UNKNOWNCMD :: 1;
        FAILRESTART :: 2;
        LAST :: 3;
    }

}

#if OS == .WINDOWS {
    curlioerr :: enum s32 {
        OK :: 0;
        UNKNOWNCMD :: 1;
        FAILRESTART :: 2;
        LAST :: 3;
    }

}

#if OS != .WINDOWS {
    CURLMcode :: enum s32 {
        CALL_MULTI_PERFORM :: -1;
        OK :: 0;
        BAD_HANDLE :: 1;
        BAD_EASY_HANDLE :: 2;
        OUT_OF_MEMORY :: 3;
        INTERNAL_ERROR :: 4;
        BAD_SOCKET :: 5;
        UNKNOWN_OPTION :: 6;
        ADDED_ALREADY :: 7;
        RECURSIVE_API_CALL :: 8;
        WAKEUP_FAILURE :: 9;
        BAD_FUNCTION_ARGUMENT :: 10;
        ABORTED_BY_CALLBACK :: 11;
        UNRECOVERABLE_POLL :: 12;
        LAST :: 13;
    }

}

#if OS == .WINDOWS {
    CURLMcode :: enum s32 {
        CALL_MULTI_PERFORM :: -1;
        OK :: 0;
        BAD_HANDLE :: 1;
        BAD_EASY_HANDLE :: 2;
        OUT_OF_MEMORY :: 3;
        INTERNAL_ERROR :: 4;
        BAD_SOCKET :: 5;
        UNKNOWN_OPTION :: 6;
        ADDED_ALREADY :: 7;
        RECURSIVE_API_CALL :: 8;
        WAKEUP_FAILURE :: 9;
        BAD_FUNCTION_ARGUMENT :: 10;
        ABORTED_BY_CALLBACK :: 11;
        UNRECOVERABLE_POLL :: 12;
        LAST :: 13;
    }

}

#if OS != .WINDOWS {
    CURLMoption :: enum u32 {
        SOCKETFUNCTION :: 20001;
        SOCKETDATA :: 10002;
        PIPELINING :: 3;
        TIMERFUNCTION :: 20004;
        TIMERDATA :: 10005;
        MAXCONNECTS :: 6;
        MAX_HOST_CONNECTIONS :: 7;
        MAX_PIPELINE_LENGTH :: 8;
        CONTENT_LENGTH_PENALTY_SIZE :: 30009;
        CHUNK_LENGTH_PENALTY_SIZE :: 30010;
        PIPELINING_SITE_BL :: 10011;
        PIPELINING_SERVER_BL :: 10012;
        MAX_TOTAL_CONNECTIONS :: 13;
        PUSHFUNCTION :: 20014;
        PUSHDATA :: 10015;
        MAX_CONCURRENT_STREAMS :: 16;
        LASTENTRY :: 17;
    }

}

#if OS == .WINDOWS {
    CURLMoption :: enum s32 {
        SOCKETFUNCTION :: 20001;
        SOCKETDATA :: 10002;
        PIPELINING :: 3;
        TIMERFUNCTION :: 20004;
        TIMERDATA :: 10005;
        MAXCONNECTS :: 6;
        MAX_HOST_CONNECTIONS :: 7;
        MAX_PIPELINE_LENGTH :: 8;
        CONTENT_LENGTH_PENALTY_SIZE :: 30009;
        CHUNK_LENGTH_PENALTY_SIZE :: 30010;
        PIPELINING_SITE_BL :: 10011;
        PIPELINING_SERVER_BL :: 10012;
        MAX_TOTAL_CONNECTIONS :: 13;
        PUSHFUNCTION :: 20014;
        PUSHDATA :: 10015;
        MAX_CONCURRENT_STREAMS :: 16;
        LASTENTRY :: 17;
    }

}

#if OS != .WINDOWS {
    CURLMSG :: enum u32 {
        NONE :: 0;
        DONE :: 1;
        LAST :: 2;
    }

}

#if OS != .WINDOWS {
    CURLMsg :: struct {
        msg: CURLMSG;
        easy_handle: *CURL;
        data: struct {
            whatever: *void;
            #overlay (whatever) result: CURLcode;
        };
    }

}

#if OS == .WINDOWS {
    CURLMSG :: enum s32 {
        NONE :: 0;
        DONE :: 1;
        LAST :: 2;
    }

}

#if OS == .WINDOWS {
    CURLMsg :: struct {
        msg: CURLMSG;
        easy_handle: *CURL;
        data: struct {
            whatever: *void;
            #overlay (whatever) result: CURLcode;
        };
    }

}

#if OS != .WINDOWS {
    CURLoption :: enum u32 {
        WRITEDATA :: 10001;
        URL :: 10002;
        PORT :: 3;
        PROXY :: 10004;
        USERPWD :: 10005;
        PROXYUSERPWD :: 10006;
        RANGE :: 10007;
        READDATA :: 10009;
        ERRORBUFFER :: 10010;
        WRITEFUNCTION :: 20011;
        READFUNCTION :: 20012;
        TIMEOUT :: 13;
        INFILESIZE :: 14;
        POSTFIELDS :: 10015;
        REFERER :: 10016;
        FTPPORT :: 10017;
        USERAGENT :: 10018;
        LOW_SPEED_LIMIT :: 19;
        LOW_SPEED_TIME :: 20;
        RESUME_FROM :: 21;
        COOKIE :: 10022;
        HTTPHEADER :: 10023;
        HTTPPOST :: 10024;
        SSLCERT :: 10025;
        KEYPASSWD :: 10026;
        CRLF :: 27;
        QUOTE :: 10028;
        HEADERDATA :: 10029;
        COOKIEFILE :: 10031;
        SSLVERSION :: 32;
        TIMECONDITION :: 33;
        TIMEVALUE :: 34;
        CUSTOMREQUEST :: 10036;
        STDERR :: 10037;
        POSTQUOTE :: 10039;
        OBSOLETE40 :: 10040;
        VERBOSE :: 41;
        HEADER :: 42;
        NOPROGRESS :: 43;
        NOBODY :: 44;
        FAILONERROR :: 45;
        UPLOAD :: 46;
        POST :: 47;
        DIRLISTONLY :: 48;
        APPEND :: 50;
        NETRC :: 51;
        FOLLOWLOCATION :: 52;
        TRANSFERTEXT :: 53;
        PUT :: 54;
        PROGRESSFUNCTION :: 20056;
        XFERINFODATA :: 10057;
        AUTOREFERER :: 58;
        PROXYPORT :: 59;
        POSTFIELDSIZE :: 60;
        HTTPPROXYTUNNEL :: 61;
        INTERFACE :: 10062;
        KRBLEVEL :: 10063;
        SSL_VERIFYPEER :: 64;
        CAINFO :: 10065;
        MAXREDIRS :: 68;
        FILETIME :: 69;
        TELNETOPTIONS :: 10070;
        MAXCONNECTS :: 71;
        OBSOLETE72 :: 72;
        FRESH_CONNECT :: 74;
        FORBID_REUSE :: 75;
        RANDOM_FILE :: 10076;
        EGDSOCKET :: 10077;
        CONNECTTIMEOUT :: 78;
        HEADERFUNCTION :: 20079;
        HTTPGET :: 80;
        SSL_VERIFYHOST :: 81;
        COOKIEJAR :: 10082;
        SSL_CIPHER_LIST :: 10083;
        HTTP_VERSION :: 84;
        FTP_USE_EPSV :: 85;
        SSLCERTTYPE :: 10086;
        SSLKEY :: 10087;
        SSLKEYTYPE :: 10088;
        SSLENGINE :: 10089;
        SSLENGINE_DEFAULT :: 90;
        DNS_USE_GLOBAL_CACHE :: 91;
        DNS_CACHE_TIMEOUT :: 92;
        PREQUOTE :: 10093;
        DEBUGFUNCTION :: 20094;
        DEBUGDATA :: 10095;
        COOKIESESSION :: 96;
        CAPATH :: 10097;
        BUFFERSIZE :: 98;
        NOSIGNAL :: 99;
        SHARE :: 10100;
        PROXYTYPE :: 101;
        ACCEPT_ENCODING :: 10102;
        PRIVATE :: 10103;
        HTTP200ALIASES :: 10104;
        UNRESTRICTED_AUTH :: 105;
        FTP_USE_EPRT :: 106;
        HTTPAUTH :: 107;
        SSL_CTX_FUNCTION :: 20108;
        SSL_CTX_DATA :: 10109;
        FTP_CREATE_MISSING_DIRS :: 110;
        PROXYAUTH :: 111;
        SERVER_RESPONSE_TIMEOUT :: 112;
        IPRESOLVE :: 113;
        MAXFILESIZE :: 114;
        INFILESIZE_LARGE :: 30115;
        RESUME_FROM_LARGE :: 30116;
        MAXFILESIZE_LARGE :: 30117;
        NETRC_FILE :: 10118;
        USE_SSL :: 119;
        POSTFIELDSIZE_LARGE :: 30120;
        TCP_NODELAY :: 121;
        FTPSSLAUTH :: 129;
        IOCTLFUNCTION :: 20130;
        IOCTLDATA :: 10131;
        FTP_ACCOUNT :: 10134;
        COOKIELIST :: 10135;
        IGNORE_CONTENT_LENGTH :: 136;
        FTP_SKIP_PASV_IP :: 137;
        FTP_FILEMETHOD :: 138;
        LOCALPORT :: 139;
        LOCALPORTRANGE :: 140;
        CONNECT_ONLY :: 141;
        CONV_FROM_NETWORK_FUNCTION :: 20142;
        CONV_TO_NETWORK_FUNCTION :: 20143;
        CONV_FROM_UTF8_FUNCTION :: 20144;
        MAX_SEND_SPEED_LARGE :: 30145;
        MAX_RECV_SPEED_LARGE :: 30146;
        FTP_ALTERNATIVE_TO_USER :: 10147;
        SOCKOPTFUNCTION :: 20148;
        SOCKOPTDATA :: 10149;
        SSL_SESSIONID_CACHE :: 150;
        SSH_AUTH_TYPES :: 151;
        SSH_PUBLIC_KEYFILE :: 10152;
        SSH_PRIVATE_KEYFILE :: 10153;
        FTP_SSL_CCC :: 154;
        TIMEOUT_MS :: 155;
        CONNECTTIMEOUT_MS :: 156;
        HTTP_TRANSFER_DECODING :: 157;
        HTTP_CONTENT_DECODING :: 158;
        NEW_FILE_PERMS :: 159;
        NEW_DIRECTORY_PERMS :: 160;
        POSTREDIR :: 161;
        SSH_HOST_PUBLIC_KEY_MD5 :: 10162;
        OPENSOCKETFUNCTION :: 20163;
        OPENSOCKETDATA :: 10164;
        COPYPOSTFIELDS :: 10165;
        PROXY_TRANSFER_MODE :: 166;
        SEEKFUNCTION :: 20167;
        SEEKDATA :: 10168;
        CRLFILE :: 10169;
        ISSUERCERT :: 10170;
        ADDRESS_SCOPE :: 171;
        CERTINFO :: 172;
        USERNAME :: 10173;
        PASSWORD :: 10174;
        PROXYUSERNAME :: 10175;
        PROXYPASSWORD :: 10176;
        NOPROXY :: 10177;
        TFTP_BLKSIZE :: 178;
        SOCKS5_GSSAPI_SERVICE :: 10179;
        SOCKS5_GSSAPI_NEC :: 180;
        PROTOCOLS :: 181;
        REDIR_PROTOCOLS :: 182;
        SSH_KNOWNHOSTS :: 10183;
        SSH_KEYFUNCTION :: 20184;
        SSH_KEYDATA :: 10185;
        MAIL_FROM :: 10186;
        MAIL_RCPT :: 10187;
        FTP_USE_PRET :: 188;
        RTSP_REQUEST :: 189;
        RTSP_SESSION_ID :: 10190;
        RTSP_STREAM_URI :: 10191;
        RTSP_TRANSPORT :: 10192;
        RTSP_CLIENT_CSEQ :: 193;
        RTSP_SERVER_CSEQ :: 194;
        INTERLEAVEDATA :: 10195;
        INTERLEAVEFUNCTION :: 20196;
        WILDCARDMATCH :: 197;
        CHUNK_BGN_FUNCTION :: 20198;
        CHUNK_END_FUNCTION :: 20199;
        FNMATCH_FUNCTION :: 20200;
        CHUNK_DATA :: 10201;
        FNMATCH_DATA :: 10202;
        RESOLVE :: 10203;
        TLSAUTH_USERNAME :: 10204;
        TLSAUTH_PASSWORD :: 10205;
        TLSAUTH_TYPE :: 10206;
        TRANSFER_ENCODING :: 207;
        CLOSESOCKETFUNCTION :: 20208;
        CLOSESOCKETDATA :: 10209;
        GSSAPI_DELEGATION :: 210;
        DNS_SERVERS :: 10211;
        ACCEPTTIMEOUT_MS :: 212;
        TCP_KEEPALIVE :: 213;
        TCP_KEEPIDLE :: 214;
        TCP_KEEPINTVL :: 215;
        SSL_OPTIONS :: 216;
        MAIL_AUTH :: 10217;
        SASL_IR :: 218;
        XFERINFOFUNCTION :: 20219;
        XOAUTH2_BEARER :: 10220;
        DNS_INTERFACE :: 10221;
        DNS_LOCAL_IP4 :: 10222;
        DNS_LOCAL_IP6 :: 10223;
        LOGIN_OPTIONS :: 10224;
        SSL_ENABLE_NPN :: 225;
        SSL_ENABLE_ALPN :: 226;
        EXPECT_100_TIMEOUT_MS :: 227;
        PROXYHEADER :: 10228;
        HEADEROPT :: 229;
        PINNEDPUBLICKEY :: 10230;
        UNIX_SOCKET_PATH :: 10231;
        SSL_VERIFYSTATUS :: 232;
        SSL_FALSESTART :: 233;
        PATH_AS_IS :: 234;
        PROXY_SERVICE_NAME :: 10235;
        SERVICE_NAME :: 10236;
        PIPEWAIT :: 237;
        DEFAULT_PROTOCOL :: 10238;
        STREAM_WEIGHT :: 239;
        STREAM_DEPENDS :: 10240;
        STREAM_DEPENDS_E :: 10241;
        TFTP_NO_OPTIONS :: 242;
        CONNECT_TO :: 10243;
        TCP_FASTOPEN :: 244;
        KEEP_SENDING_ON_ERROR :: 245;
        PROXY_CAINFO :: 10246;
        PROXY_CAPATH :: 10247;
        PROXY_SSL_VERIFYPEER :: 248;
        PROXY_SSL_VERIFYHOST :: 249;
        PROXY_SSLVERSION :: 250;
        PROXY_TLSAUTH_USERNAME :: 10251;
        PROXY_TLSAUTH_PASSWORD :: 10252;
        PROXY_TLSAUTH_TYPE :: 10253;
        PROXY_SSLCERT :: 10254;
        PROXY_SSLCERTTYPE :: 10255;
        PROXY_SSLKEY :: 10256;
        PROXY_SSLKEYTYPE :: 10257;
        PROXY_KEYPASSWD :: 10258;
        PROXY_SSL_CIPHER_LIST :: 10259;
        PROXY_CRLFILE :: 10260;
        PROXY_SSL_OPTIONS :: 261;
        PRE_PROXY :: 10262;
        PROXY_PINNEDPUBLICKEY :: 10263;
        ABSTRACT_UNIX_SOCKET :: 10264;
        SUPPRESS_CONNECT_HEADERS :: 265;
        REQUEST_TARGET :: 10266;
        SOCKS5_AUTH :: 267;
        SSH_COMPRESSION :: 268;
        MIMEPOST :: 10269;
        TIMEVALUE_LARGE :: 30270;
        HAPPY_EYEBALLS_TIMEOUT_MS :: 271;
        RESOLVER_START_FUNCTION :: 20272;
        RESOLVER_START_DATA :: 10273;
        HAPROXYPROTOCOL :: 274;
        DNS_SHUFFLE_ADDRESSES :: 275;
        TLS13_CIPHERS :: 10276;
        PROXY_TLS13_CIPHERS :: 10277;
        DISALLOW_USERNAME_IN_URL :: 278;
        DOH_URL :: 10279;
        UPLOAD_BUFFERSIZE :: 280;
        UPKEEP_INTERVAL_MS :: 281;
        CURLU :: 10282;
        TRAILERFUNCTION :: 20283;
        TRAILERDATA :: 10284;
        HTTP09_ALLOWED :: 285;
        ALTSVC_CTRL :: 286;
        ALTSVC :: 10287;
        MAXAGE_CONN :: 288;
        SASL_AUTHZID :: 10289;
        MAIL_RCPT_ALLOWFAILS :: 290;
        SSLCERT_BLOB :: 40291;
        SSLKEY_BLOB :: 40292;
        PROXY_SSLCERT_BLOB :: 40293;
        PROXY_SSLKEY_BLOB :: 40294;
        ISSUERCERT_BLOB :: 40295;
        PROXY_ISSUERCERT :: 10296;
        PROXY_ISSUERCERT_BLOB :: 40297;
        SSL_EC_CURVES :: 10298;
        HSTS_CTRL :: 299;
        HSTS :: 10300;
        HSTSREADFUNCTION :: 20301;
        HSTSREADDATA :: 10302;
        HSTSWRITEFUNCTION :: 20303;
        HSTSWRITEDATA :: 10304;
        AWS_SIGV4 :: 10305;
        DOH_SSL_VERIFYPEER :: 306;
        DOH_SSL_VERIFYHOST :: 307;
        DOH_SSL_VERIFYSTATUS :: 308;
        CAINFO_BLOB :: 40309;
        PROXY_CAINFO_BLOB :: 40310;
        SSH_HOST_PUBLIC_KEY_SHA256 :: 10311;
        PREREQFUNCTION :: 20312;
        PREREQDATA :: 10313;
        MAXLIFETIME_CONN :: 314;
        MIME_OPTIONS :: 315;
        SSH_HOSTKEYFUNCTION :: 20316;
        SSH_HOSTKEYDATA :: 10317;
        PROTOCOLS_STR :: 10318;
        REDIR_PROTOCOLS_STR :: 10319;
        WS_OPTIONS :: 320;
        CA_CACHE_TIMEOUT :: 321;
        QUICK_EXIT :: 322;
        HAPROXY_CLIENT_IP :: 10323;
        LASTENTRY :: 10324;
    }

}

#if OS == .WINDOWS {
    CURLoption :: enum s32 {
        WRITEDATA :: 10001;
        URL :: 10002;
        PORT :: 3;
        PROXY :: 10004;
        USERPWD :: 10005;
        PROXYUSERPWD :: 10006;
        RANGE :: 10007;
        READDATA :: 10009;
        ERRORBUFFER :: 10010;
        WRITEFUNCTION :: 20011;
        READFUNCTION :: 20012;
        TIMEOUT :: 13;
        INFILESIZE :: 14;
        POSTFIELDS :: 10015;
        REFERER :: 10016;
        FTPPORT :: 10017;
        USERAGENT :: 10018;
        LOW_SPEED_LIMIT :: 19;
        LOW_SPEED_TIME :: 20;
        RESUME_FROM :: 21;
        COOKIE :: 10022;
        HTTPHEADER :: 10023;
        HTTPPOST :: 10024;
        SSLCERT :: 10025;
        KEYPASSWD :: 10026;
        CRLF :: 27;
        QUOTE :: 10028;
        HEADERDATA :: 10029;
        COOKIEFILE :: 10031;
        SSLVERSION :: 32;
        TIMECONDITION :: 33;
        TIMEVALUE :: 34;
        CUSTOMREQUEST :: 10036;
        STDERR :: 10037;
        POSTQUOTE :: 10039;
        OBSOLETE40 :: 10040;
        VERBOSE :: 41;
        HEADER :: 42;
        NOPROGRESS :: 43;
        NOBODY :: 44;
        FAILONERROR :: 45;
        UPLOAD :: 46;
        POST :: 47;
        DIRLISTONLY :: 48;
        APPEND :: 50;
        NETRC :: 51;
        FOLLOWLOCATION :: 52;
        TRANSFERTEXT :: 53;
        PUT :: 54;
        PROGRESSFUNCTION :: 20056;
        XFERINFODATA :: 10057;
        AUTOREFERER :: 58;
        PROXYPORT :: 59;
        POSTFIELDSIZE :: 60;
        HTTPPROXYTUNNEL :: 61;
        INTERFACE :: 10062;
        KRBLEVEL :: 10063;
        SSL_VERIFYPEER :: 64;
        CAINFO :: 10065;
        MAXREDIRS :: 68;
        FILETIME :: 69;
        TELNETOPTIONS :: 10070;
        MAXCONNECTS :: 71;
        OBSOLETE72 :: 72;
        FRESH_CONNECT :: 74;
        FORBID_REUSE :: 75;
        RANDOM_FILE :: 10076;
        EGDSOCKET :: 10077;
        CONNECTTIMEOUT :: 78;
        HEADERFUNCTION :: 20079;
        HTTPGET :: 80;
        SSL_VERIFYHOST :: 81;
        COOKIEJAR :: 10082;
        SSL_CIPHER_LIST :: 10083;
        HTTP_VERSION :: 84;
        FTP_USE_EPSV :: 85;
        SSLCERTTYPE :: 10086;
        SSLKEY :: 10087;
        SSLKEYTYPE :: 10088;
        SSLENGINE :: 10089;
        SSLENGINE_DEFAULT :: 90;
        DNS_USE_GLOBAL_CACHE :: 91;
        DNS_CACHE_TIMEOUT :: 92;
        PREQUOTE :: 10093;
        DEBUGFUNCTION :: 20094;
        DEBUGDATA :: 10095;
        COOKIESESSION :: 96;
        CAPATH :: 10097;
        BUFFERSIZE :: 98;
        NOSIGNAL :: 99;
        SHARE :: 10100;
        PROXYTYPE :: 101;
        ACCEPT_ENCODING :: 10102;
        PRIVATE :: 10103;
        HTTP200ALIASES :: 10104;
        UNRESTRICTED_AUTH :: 105;
        FTP_USE_EPRT :: 106;
        HTTPAUTH :: 107;
        SSL_CTX_FUNCTION :: 20108;
        SSL_CTX_DATA :: 10109;
        FTP_CREATE_MISSING_DIRS :: 110;
        PROXYAUTH :: 111;
        SERVER_RESPONSE_TIMEOUT :: 112;
        IPRESOLVE :: 113;
        MAXFILESIZE :: 114;
        INFILESIZE_LARGE :: 30115;
        RESUME_FROM_LARGE :: 30116;
        MAXFILESIZE_LARGE :: 30117;
        NETRC_FILE :: 10118;
        USE_SSL :: 119;
        POSTFIELDSIZE_LARGE :: 30120;
        TCP_NODELAY :: 121;
        FTPSSLAUTH :: 129;
        IOCTLFUNCTION :: 20130;
        IOCTLDATA :: 10131;
        FTP_ACCOUNT :: 10134;
        COOKIELIST :: 10135;
        IGNORE_CONTENT_LENGTH :: 136;
        FTP_SKIP_PASV_IP :: 137;
        FTP_FILEMETHOD :: 138;
        LOCALPORT :: 139;
        LOCALPORTRANGE :: 140;
        CONNECT_ONLY :: 141;
        CONV_FROM_NETWORK_FUNCTION :: 20142;
        CONV_TO_NETWORK_FUNCTION :: 20143;
        CONV_FROM_UTF8_FUNCTION :: 20144;
        MAX_SEND_SPEED_LARGE :: 30145;
        MAX_RECV_SPEED_LARGE :: 30146;
        FTP_ALTERNATIVE_TO_USER :: 10147;
        SOCKOPTFUNCTION :: 20148;
        SOCKOPTDATA :: 10149;
        SSL_SESSIONID_CACHE :: 150;
        SSH_AUTH_TYPES :: 151;
        SSH_PUBLIC_KEYFILE :: 10152;
        SSH_PRIVATE_KEYFILE :: 10153;
        FTP_SSL_CCC :: 154;
        TIMEOUT_MS :: 155;
        CONNECTTIMEOUT_MS :: 156;
        HTTP_TRANSFER_DECODING :: 157;
        HTTP_CONTENT_DECODING :: 158;
        NEW_FILE_PERMS :: 159;
        NEW_DIRECTORY_PERMS :: 160;
        POSTREDIR :: 161;
        SSH_HOST_PUBLIC_KEY_MD5 :: 10162;
        OPENSOCKETFUNCTION :: 20163;
        OPENSOCKETDATA :: 10164;
        COPYPOSTFIELDS :: 10165;
        PROXY_TRANSFER_MODE :: 166;
        SEEKFUNCTION :: 20167;
        SEEKDATA :: 10168;
        CRLFILE :: 10169;
        ISSUERCERT :: 10170;
        ADDRESS_SCOPE :: 171;
        CERTINFO :: 172;
        USERNAME :: 10173;
        PASSWORD :: 10174;
        PROXYUSERNAME :: 10175;
        PROXYPASSWORD :: 10176;
        NOPROXY :: 10177;
        TFTP_BLKSIZE :: 178;
        SOCKS5_GSSAPI_SERVICE :: 10179;
        SOCKS5_GSSAPI_NEC :: 180;
        PROTOCOLS :: 181;
        REDIR_PROTOCOLS :: 182;
        SSH_KNOWNHOSTS :: 10183;
        SSH_KEYFUNCTION :: 20184;
        SSH_KEYDATA :: 10185;
        MAIL_FROM :: 10186;
        MAIL_RCPT :: 10187;
        FTP_USE_PRET :: 188;
        RTSP_REQUEST :: 189;
        RTSP_SESSION_ID :: 10190;
        RTSP_STREAM_URI :: 10191;
        RTSP_TRANSPORT :: 10192;
        RTSP_CLIENT_CSEQ :: 193;
        RTSP_SERVER_CSEQ :: 194;
        INTERLEAVEDATA :: 10195;
        INTERLEAVEFUNCTION :: 20196;
        WILDCARDMATCH :: 197;
        CHUNK_BGN_FUNCTION :: 20198;
        CHUNK_END_FUNCTION :: 20199;
        FNMATCH_FUNCTION :: 20200;
        CHUNK_DATA :: 10201;
        FNMATCH_DATA :: 10202;
        RESOLVE :: 10203;
        TLSAUTH_USERNAME :: 10204;
        TLSAUTH_PASSWORD :: 10205;
        TLSAUTH_TYPE :: 10206;
        TRANSFER_ENCODING :: 207;
        CLOSESOCKETFUNCTION :: 20208;
        CLOSESOCKETDATA :: 10209;
        GSSAPI_DELEGATION :: 210;
        DNS_SERVERS :: 10211;
        ACCEPTTIMEOUT_MS :: 212;
        TCP_KEEPALIVE :: 213;
        TCP_KEEPIDLE :: 214;
        TCP_KEEPINTVL :: 215;
        SSL_OPTIONS :: 216;
        MAIL_AUTH :: 10217;
        SASL_IR :: 218;
        XFERINFOFUNCTION :: 20219;
        XOAUTH2_BEARER :: 10220;
        DNS_INTERFACE :: 10221;
        DNS_LOCAL_IP4 :: 10222;
        DNS_LOCAL_IP6 :: 10223;
        LOGIN_OPTIONS :: 10224;
        SSL_ENABLE_NPN :: 225;
        SSL_ENABLE_ALPN :: 226;
        EXPECT_100_TIMEOUT_MS :: 227;
        PROXYHEADER :: 10228;
        HEADEROPT :: 229;
        PINNEDPUBLICKEY :: 10230;
        UNIX_SOCKET_PATH :: 10231;
        SSL_VERIFYSTATUS :: 232;
        SSL_FALSESTART :: 233;
        PATH_AS_IS :: 234;
        PROXY_SERVICE_NAME :: 10235;
        SERVICE_NAME :: 10236;
        PIPEWAIT :: 237;
        DEFAULT_PROTOCOL :: 10238;
        STREAM_WEIGHT :: 239;
        STREAM_DEPENDS :: 10240;
        STREAM_DEPENDS_E :: 10241;
        TFTP_NO_OPTIONS :: 242;
        CONNECT_TO :: 10243;
        TCP_FASTOPEN :: 244;
        KEEP_SENDING_ON_ERROR :: 245;
        PROXY_CAINFO :: 10246;
        PROXY_CAPATH :: 10247;
        PROXY_SSL_VERIFYPEER :: 248;
        PROXY_SSL_VERIFYHOST :: 249;
        PROXY_SSLVERSION :: 250;
        PROXY_TLSAUTH_USERNAME :: 10251;
        PROXY_TLSAUTH_PASSWORD :: 10252;
        PROXY_TLSAUTH_TYPE :: 10253;
        PROXY_SSLCERT :: 10254;
        PROXY_SSLCERTTYPE :: 10255;
        PROXY_SSLKEY :: 10256;
        PROXY_SSLKEYTYPE :: 10257;
        PROXY_KEYPASSWD :: 10258;
        PROXY_SSL_CIPHER_LIST :: 10259;
        PROXY_CRLFILE :: 10260;
        PROXY_SSL_OPTIONS :: 261;
        PRE_PROXY :: 10262;
        PROXY_PINNEDPUBLICKEY :: 10263;
        ABSTRACT_UNIX_SOCKET :: 10264;
        SUPPRESS_CONNECT_HEADERS :: 265;
        REQUEST_TARGET :: 10266;
        SOCKS5_AUTH :: 267;
        SSH_COMPRESSION :: 268;
        MIMEPOST :: 10269;
        TIMEVALUE_LARGE :: 30270;
        HAPPY_EYEBALLS_TIMEOUT_MS :: 271;
        RESOLVER_START_FUNCTION :: 20272;
        RESOLVER_START_DATA :: 10273;
        HAPROXYPROTOCOL :: 274;
        DNS_SHUFFLE_ADDRESSES :: 275;
        TLS13_CIPHERS :: 10276;
        PROXY_TLS13_CIPHERS :: 10277;
        DISALLOW_USERNAME_IN_URL :: 278;
        DOH_URL :: 10279;
        UPLOAD_BUFFERSIZE :: 280;
        UPKEEP_INTERVAL_MS :: 281;
        CURLU :: 10282;
        TRAILERFUNCTION :: 20283;
        TRAILERDATA :: 10284;
        HTTP09_ALLOWED :: 285;
        ALTSVC_CTRL :: 286;
        ALTSVC :: 10287;
        MAXAGE_CONN :: 288;
        SASL_AUTHZID :: 10289;
        MAIL_RCPT_ALLOWFAILS :: 290;
        SSLCERT_BLOB :: 40291;
        SSLKEY_BLOB :: 40292;
        PROXY_SSLCERT_BLOB :: 40293;
        PROXY_SSLKEY_BLOB :: 40294;
        ISSUERCERT_BLOB :: 40295;
        PROXY_ISSUERCERT :: 10296;
        PROXY_ISSUERCERT_BLOB :: 40297;
        SSL_EC_CURVES :: 10298;
        HSTS_CTRL :: 299;
        HSTS :: 10300;
        HSTSREADFUNCTION :: 20301;
        HSTSREADDATA :: 10302;
        HSTSWRITEFUNCTION :: 20303;
        HSTSWRITEDATA :: 10304;
        AWS_SIGV4 :: 10305;
        DOH_SSL_VERIFYPEER :: 306;
        DOH_SSL_VERIFYHOST :: 307;
        DOH_SSL_VERIFYSTATUS :: 308;
        CAINFO_BLOB :: 40309;
        PROXY_CAINFO_BLOB :: 40310;
        SSH_HOST_PUBLIC_KEY_SHA256 :: 10311;
        PREREQFUNCTION :: 20312;
        PREREQDATA :: 10313;
        MAXLIFETIME_CONN :: 314;
        MIME_OPTIONS :: 315;
        SSH_HOSTKEYFUNCTION :: 20316;
        SSH_HOSTKEYDATA :: 10317;
        PROTOCOLS_STR :: 10318;
        REDIR_PROTOCOLS_STR :: 10319;
        WS_OPTIONS :: 320;
        CA_CACHE_TIMEOUT :: 321;
        QUICK_EXIT :: 322;
        HAPROXY_CLIENT_IP :: 10323;
        LASTENTRY :: 10324;
    }

}

#if OS != .WINDOWS {
    CURLproxycode :: enum u32 {
        OK :: 0;
        BAD_ADDRESS_TYPE :: 1;
        BAD_VERSION :: 2;
        CLOSED :: 3;
        GSSAPI :: 4;
        GSSAPI_PERMSG :: 5;
        GSSAPI_PROTECTION :: 6;
        IDENTD :: 7;
        IDENTD_DIFFER :: 8;
        LONG_HOSTNAME :: 9;
        LONG_PASSWD :: 10;
        LONG_USER :: 11;
        NO_AUTH :: 12;
        RECV_ADDRESS :: 13;
        RECV_AUTH :: 14;
        RECV_CONNECT :: 15;
        RECV_REQACK :: 16;
        REPLY_ADDRESS_TYPE_NOT_SUPPORTED :: 17;
        REPLY_COMMAND_NOT_SUPPORTED :: 18;
        REPLY_CONNECTION_REFUSED :: 19;
        REPLY_GENERAL_SERVER_FAILURE :: 20;
        REPLY_HOST_UNREACHABLE :: 21;
        REPLY_NETWORK_UNREACHABLE :: 22;
        REPLY_NOT_ALLOWED :: 23;
        REPLY_TTL_EXPIRED :: 24;
        REPLY_UNASSIGNED :: 25;
        REQUEST_FAILED :: 26;
        RESOLVE_HOST :: 27;
        SEND_AUTH :: 28;
        SEND_CONNECT :: 29;
        SEND_REQUEST :: 30;
        UNKNOWN_FAIL :: 31;
        UNKNOWN_MODE :: 32;
        USER_REJECTED :: 33;
        LAST :: 34;
    }

}

#if OS == .WINDOWS {
    CURLproxycode :: enum s32 {
        OK :: 0;
        BAD_ADDRESS_TYPE :: 1;
        BAD_VERSION :: 2;
        CLOSED :: 3;
        GSSAPI :: 4;
        GSSAPI_PERMSG :: 5;
        GSSAPI_PROTECTION :: 6;
        IDENTD :: 7;
        IDENTD_DIFFER :: 8;
        LONG_HOSTNAME :: 9;
        LONG_PASSWD :: 10;
        LONG_USER :: 11;
        NO_AUTH :: 12;
        RECV_ADDRESS :: 13;
        RECV_AUTH :: 14;
        RECV_CONNECT :: 15;
        RECV_REQACK :: 16;
        REPLY_ADDRESS_TYPE_NOT_SUPPORTED :: 17;
        REPLY_COMMAND_NOT_SUPPORTED :: 18;
        REPLY_CONNECTION_REFUSED :: 19;
        REPLY_GENERAL_SERVER_FAILURE :: 20;
        REPLY_HOST_UNREACHABLE :: 21;
        REPLY_NETWORK_UNREACHABLE :: 22;
        REPLY_NOT_ALLOWED :: 23;
        REPLY_TTL_EXPIRED :: 24;
        REPLY_UNASSIGNED :: 25;
        REQUEST_FAILED :: 26;
        RESOLVE_HOST :: 27;
        SEND_AUTH :: 28;
        SEND_CONNECT :: 29;
        SEND_REQUEST :: 30;
        UNKNOWN_FAIL :: 31;
        UNKNOWN_MODE :: 32;
        USER_REJECTED :: 33;
        LAST :: 34;
    }

}

#if OS != .WINDOWS {
    CURLSHcode :: enum u32 {
        OK :: 0;
        BAD_OPTION :: 1;
        IN_USE :: 2;
        INVALID :: 3;
        NOMEM :: 4;
        NOT_BUILT_IN :: 5;
        LAST :: 6;
    }

}

#if OS == .WINDOWS {
    CURLSHcode :: enum s32 {
        OK :: 0;
        BAD_OPTION :: 1;
        IN_USE :: 2;
        INVALID :: 3;
        NOMEM :: 4;
        NOT_BUILT_IN :: 5;
        LAST :: 6;
    }

}

#if OS != .WINDOWS {
    CURLSHoption :: enum u32 {
        NONE :: 0;
        SHARE :: 1;
        UNSHARE :: 2;
        LOCKFUNC :: 3;
        UNLOCKFUNC :: 4;
        USERDATA :: 5;
        LAST :: 6;
    }

}

#if OS == .WINDOWS {
    CURLSHoption :: enum s32 {
        NONE :: 0;
        SHARE :: 1;
        UNSHARE :: 2;
        LOCKFUNC :: 3;
        UNLOCKFUNC :: 4;
        USERDATA :: 5;
        LAST :: 6;
    }

}

#if OS != .WINDOWS {
    curlsocktype :: enum u32 {
        IPCXN :: 0;
        ACCEPT :: 1;
        LAST :: 2;
    }

}

#if OS == .WINDOWS {
    curlsocktype :: enum s32 {
        IPCXN :: 0;
        ACCEPT :: 1;
        LAST :: 2;
    }

}

#if OS != .WINDOWS {
    CURLsslset :: enum u32 {
        OK :: 0;
        UNKNOWN_BACKEND :: 1;
        TOO_LATE :: 2;
        NO_BACKENDS :: 3;
    }

}

#if OS == .WINDOWS {
    CURLsslset :: enum s32 {
        OK :: 0;
        UNKNOWN_BACKEND :: 1;
        TOO_LATE :: 2;
        NO_BACKENDS :: 3;
    }

}

#if OS != .WINDOWS {
    CURLSTScode :: enum u32 {
        OK :: 0;
        DONE :: 1;
        FAIL :: 2;
    }

}

#if OS == .WINDOWS {
    CURLSTScode :: enum s32 {
        OK :: 0;
        DONE :: 1;
        FAIL :: 2;
    }

}

#if OS != .WINDOWS {
    CURLUcode :: enum u32 {
        OK :: 0;
        BAD_HANDLE :: 1;
        BAD_PARTPOINTER :: 2;
        MALFORMED_INPUT :: 3;
        BAD_PORT_NUMBER :: 4;
        UNSUPPORTED_SCHEME :: 5;
        URLDECODE :: 6;
        OUT_OF_MEMORY :: 7;
        USER_NOT_ALLOWED :: 8;
        UNKNOWN_PART :: 9;
        NO_SCHEME :: 10;
        NO_USER :: 11;
        NO_PASSWORD :: 12;
        NO_OPTIONS :: 13;
        NO_HOST :: 14;
        NO_PORT :: 15;
        NO_QUERY :: 16;
        NO_FRAGMENT :: 17;
        NO_ZONEID :: 18;
        BAD_FILE_URL :: 19;
        BAD_FRAGMENT :: 20;
        BAD_HOSTNAME :: 21;
        BAD_IPV6 :: 22;
        BAD_LOGIN :: 23;
        BAD_PASSWORD :: 24;
        BAD_PATH :: 25;
        BAD_QUERY :: 26;
        BAD_SCHEME :: 27;
        BAD_SLASHES :: 28;
        BAD_USER :: 29;
        LACKS_IDN :: 30;
        LAST :: 31;
    }

}

#if OS == .WINDOWS {
    CURLUcode :: enum s32 {
        OK :: 0;
        BAD_HANDLE :: 1;
        BAD_PARTPOINTER :: 2;
        MALFORMED_INPUT :: 3;
        BAD_PORT_NUMBER :: 4;
        UNSUPPORTED_SCHEME :: 5;
        URLDECODE :: 6;
        OUT_OF_MEMORY :: 7;
        USER_NOT_ALLOWED :: 8;
        UNKNOWN_PART :: 9;
        NO_SCHEME :: 10;
        NO_USER :: 11;
        NO_PASSWORD :: 12;
        NO_OPTIONS :: 13;
        NO_HOST :: 14;
        NO_PORT :: 15;
        NO_QUERY :: 16;
        NO_FRAGMENT :: 17;
        NO_ZONEID :: 18;
        BAD_FILE_URL :: 19;
        BAD_FRAGMENT :: 20;
        BAD_HOSTNAME :: 21;
        BAD_IPV6 :: 22;
        BAD_LOGIN :: 23;
        BAD_PASSWORD :: 24;
        BAD_PATH :: 25;
        BAD_QUERY :: 26;
        BAD_SCHEME :: 27;
        BAD_SLASHES :: 28;
        BAD_USER :: 29;
        LACKS_IDN :: 30;
        LAST :: 31;
    }

}

#if OS != .WINDOWS {
    CURLUPart :: enum u32 {
        URL :: 0;
        SCHEME :: 1;
        USER :: 2;
        PASSWORD :: 3;
        OPTIONS :: 4;
        HOST :: 5;
        PORT :: 6;
        PATH :: 7;
        QUERY :: 8;
        FRAGMENT :: 9;
        ZONEID :: 10;
    }

}

#if OS == .WINDOWS {
    CURLUPart :: enum s32 {
        URL :: 0;
        SCHEME :: 1;
        USER :: 2;
        PASSWORD :: 3;
        OPTIONS :: 4;
        HOST :: 5;
        PORT :: 6;
        PATH :: 7;
        QUERY :: 8;
        FRAGMENT :: 9;
        ZONEID :: 10;
    }

}

#if OS != .WINDOWS {
    CURLversion :: enum u32 {
        FIRST :: 0;
        SECOND :: 1;
        THIRD :: 2;
        FOURTH :: 3;
        FIFTH :: 4;
        SIXTH :: 5;
        SEVENTH :: 6;
        EIGHTH :: 7;
        NINTH :: 8;
        TENTH :: 9;
        ELEVENTH :: 10;
        LAST :: 11;
    }

}

#if OS == .WINDOWS {
    CURLversion :: enum s32 {
        FIRST :: 0;
        SECOND :: 1;
        THIRD :: 2;
        FOURTH :: 3;
        FIFTH :: 4;
        SIXTH :: 5;
        SEVENTH :: 6;
        EIGHTH :: 7;
        NINTH :: 8;
        TENTH :: 9;
        ELEVENTH :: 10;
        LAST :: 11;
    }

}

// Callback and opaque types

#if OS != .WINDOWS {
    curl_calloc_callback :: #type (nmemb: u64, size: u64) -> *void #c_call;
}

#if OS == .WINDOWS {
    curl_calloc_callback :: #type (nmemb: u64, size: u64) -> *void #c_call;
}

#if OS != .WINDOWS {
    curl_chunk_bgn_callback :: #type (transfer_info: *void, ptr: *void, remains: s32) -> s64 #c_call;
}

#if OS == .WINDOWS {
    curl_chunk_bgn_callback :: #type (transfer_info: *void, ptr: *void, remains: s32) -> s32 #c_call;
}

#if OS != .WINDOWS {
    curl_chunk_end_callback :: #type (ptr: *void) -> s64 #c_call;
}

#if OS == .WINDOWS {
    curl_chunk_end_callback :: #type (ptr: *void) -> s32 #c_call;
}

#if OS != .WINDOWS {
    curl_closesocket_callback :: #type (clientp: *void, item: curl_socket_t) -> s32 #c_call;
}

#if OS == .WINDOWS {
    curl_closesocket_callback :: #type (clientp: *void, item: curl_socket_t) -> s32 #c_call;
}

#if OS != .WINDOWS {
    curl_conv_callback :: #type (buffer: *u8, length: u64) -> CURLcode #c_call;
}

#if OS == .WINDOWS {
    curl_conv_callback :: #type (buffer: *u8, length: u64) -> CURLcode #c_call;
}

#if OS != .WINDOWS {
    curl_debug_callback :: #type (handle: *CURL, type: curl_infotype, data: *u8, size: u64, userptr: *void) -> s32 #c_call;
}

#if OS == .WINDOWS {
    curl_debug_callback :: #type (handle: *CURL, type: curl_infotype, data: *u8, size: u64, userptr: *void) -> s32 #c_call;
}

#if OS != .WINDOWS {
    curl_fnmatch_callback :: #type (ptr: *void, pattern: *u8, _string: *u8) -> s32 #c_call;
}

#if OS == .WINDOWS {
    curl_fnmatch_callback :: #type (ptr: *void, pattern: *u8, _string: *u8) -> s32 #c_call;
}

#if OS != .WINDOWS {
    curl_formget_callback :: #type (arg: *void, buf: *u8, len: u64) -> u64 #c_call;
}

#if OS == .WINDOWS {
    curl_formget_callback :: #type (arg: *void, buf: *u8, len: u64) -> u64 #c_call;
}

#if OS != .WINDOWS {
    curl_free_callback :: #type (ptr: *void) -> void #c_call;
}

#if OS == .WINDOWS {
    curl_free_callback :: #type (ptr: *void) -> void #c_call;
}

#if OS != .WINDOWS {
    curl_hstsread_callback :: #type (easy: *CURL, e: *curl_hstsentry, userp: *void) -> CURLSTScode #c_call;
}

#if OS == .WINDOWS {
    curl_hstsread_callback :: #type (easy: *CURL, e: *curl_hstsentry, userp: *void) -> CURLSTScode #c_call;
}

#if OS != .WINDOWS {
    curl_hstswrite_callback :: #type (easy: *CURL, e: *curl_hstsentry, i: *curl_index, userp: *void) -> CURLSTScode #c_call;
}

#if OS == .WINDOWS {
    curl_hstswrite_callback :: #type (easy: *CURL, e: *curl_hstsentry, i: *curl_index, userp: *void) -> CURLSTScode #c_call;
}

#if OS != .WINDOWS {
    curl_ioctl_callback :: #type (handle: *CURL, cmd: s32, clientp: *void) -> curlioerr #c_call;
}

#if OS == .WINDOWS {
    curl_ioctl_callback :: #type (handle: *CURL, cmd: s32, clientp: *void) -> curlioerr #c_call;
}

#if OS != .WINDOWS {
    curl_lock_function :: #type (handle: *CURL, data: curl_lock_data, locktype: curl_lock_access, userptr: *void) -> void #c_call;
}

#if OS == .WINDOWS {
    curl_lock_function :: #type (handle: *CURL, data: curl_lock_data, locktype: curl_lock_access, userptr: *void) -> void #c_call;
}

#if OS != .WINDOWS {
    curl_malloc_callback :: #type (size: u64) -> *void #c_call;
}

#if OS == .WINDOWS {
    curl_malloc_callback :: #type (size: u64) -> *void #c_call;
}

#if OS != .WINDOWS {
    curl_multi_timer_callback :: #type (multi: *CURLM, timeout_ms: s64, userp: *void) -> s32 #c_call;
}

#if OS == .WINDOWS {
    curl_multi_timer_callback :: #type (multi: *CURLM, timeout_ms: s32, userp: *void) -> s32 #c_call;
}

#if OS != .WINDOWS {
    curl_opensocket_callback :: #type (clientp: *void, purpose: curlsocktype, address: *curl_sockaddr) -> curl_socket_t #c_call;
}

#if OS == .WINDOWS {
    curl_opensocket_callback :: #type (clientp: *void, purpose: curlsocktype, address: *curl_sockaddr) -> curl_socket_t #c_call;
}

#if OS != .WINDOWS {
    curl_prereq_callback :: #type (clientp: *void, conn_primary_ip: *u8, conn_local_ip: *u8, conn_primary_port: s32, conn_local_port: s32) -> s32 #c_call;
}

#if OS == .WINDOWS {
    curl_prereq_callback :: #type (clientp: *void, conn_primary_ip: *u8, conn_local_ip: *u8, conn_primary_port: s32, conn_local_port: s32) -> s32 #c_call;
}

#if OS != .WINDOWS {
    curl_progress_callback :: #type (clientp: *void, dltotal: float64, dlnow: float64, ultotal: float64, ulnow: float64) -> s32 #c_call;
}

#if OS == .WINDOWS {
    curl_progress_callback :: #type (clientp: *void, dltotal: float64, dlnow: float64, ultotal: float64, ulnow: float64) -> s32 #c_call;
}

#if OS != .WINDOWS {
    curl_push_callback :: #type (parent: *CURL, easy: *CURL, num_headers: u64, headers: *curl_pushheaders, userp: *void) -> s32 #c_call;
}

#if OS == .WINDOWS {
    curl_push_callback :: #type (parent: *CURL, easy: *CURL, num_headers: u64, headers: *curl_pushheaders, userp: *void) -> s32 #c_call;
}

#if OS != .WINDOWS {
    curl_read_callback :: #type (buffer: *u8, size: u64, nitems: u64, instream: *void) -> u64 #c_call;
}

#if OS == .WINDOWS {
    curl_read_callback :: #type (buffer: *u8, size: u64, nitems: u64, instream: *void) -> u64 #c_call;
}

#if OS != .WINDOWS {
    curl_realloc_callback :: #type (ptr: *void, size: u64) -> *void #c_call;
}

#if OS == .WINDOWS {
    curl_realloc_callback :: #type (ptr: *void, size: u64) -> *void #c_call;
}

#if OS != .WINDOWS {
    curl_resolver_start_callback :: #type (resolver_state: *void, reserved: *void, userdata: *void) -> s32 #c_call;
}

#if OS == .WINDOWS {
    curl_resolver_start_callback :: #type (resolver_state: *void, reserved: *void, userdata: *void) -> s32 #c_call;
}

#if OS != .WINDOWS {
    curl_seek_callback :: #type (instream: *void, offset: curl_off_t, origin: s32) -> s32 #c_call;
}

#if OS == .WINDOWS {
    curl_seek_callback :: #type (instream: *void, offset: curl_off_t, origin: s32) -> s32 #c_call;
}

#if OS != .WINDOWS {
    curl_socket_callback :: #type (easy: *CURL, s: curl_socket_t, what: s32, userp: *void, socketp: *void) -> s32 #c_call;
}

#if OS == .WINDOWS {
    curl_socket_callback :: #type (easy: *CURL, s: curl_socket_t, what: s32, userp: *void, socketp: *void) -> s32 #c_call;
}

#if OS != .WINDOWS {
    curl_sockopt_callback :: #type (clientp: *void, curlfd: curl_socket_t, purpose: curlsocktype) -> s32 #c_call;
}

#if OS == .WINDOWS {
    curl_sockopt_callback :: #type (clientp: *void, curlfd: curl_socket_t, purpose: curlsocktype) -> s32 #c_call;
}

#if OS != .WINDOWS {
    curl_sshhostkeycallback :: #type (clientp: *void, keytype: s32, key: *u8, keylen: u64) -> s32 #c_call;
}

#if OS == .WINDOWS {
    curl_sshhostkeycallback :: #type (clientp: *void, keytype: s32, key: *u8, keylen: u64) -> s32 #c_call;
}

#if OS != .WINDOWS {
    curl_sshkeycallback :: #type (easy: *CURL, knownkey: *curl_khkey, foundkey: *curl_khkey, unknown0: curl_khmatch, clientp: *void) -> s32 #c_call;
}

#if OS == .WINDOWS {
    curl_sshkeycallback :: #type (easy: *CURL, knownkey: *curl_khkey, foundkey: *curl_khkey, unknown0: curl_khmatch, clientp: *void) -> s32 #c_call;
}

#if OS != .WINDOWS {
    curl_ssl_ctx_callback :: #type (curl: *CURL, ssl_ctx: *void, userptr: *void) -> CURLcode #c_call;
}

#if OS == .WINDOWS {
    curl_ssl_ctx_callback :: #type (curl: *CURL, ssl_ctx: *void, userptr: *void) -> CURLcode #c_call;
}

#if OS != .WINDOWS {
    curl_strdup_callback :: #type (str: *u8) -> *u8 #c_call;
}

#if OS == .WINDOWS {
    curl_strdup_callback :: #type (str: *u8) -> *u8 #c_call;
}

#if OS != .WINDOWS {
    curl_trailer_callback :: #type (list: **curl_slist, userdata: *void) -> s32 #c_call;
}

#if OS == .WINDOWS {
    curl_trailer_callback :: #type (list: **curl_slist, userdata: *void) -> s32 #c_call;
}

#if OS != .WINDOWS {
    curl_unlock_function :: #type (handle: *CURL, data: curl_lock_data, userptr: *void) -> void #c_call;
}

#if OS == .WINDOWS {
    curl_unlock_function :: #type (handle: *CURL, data: curl_lock_data, userptr: *void) -> void #c_call;
}

#if OS != .WINDOWS {
    curl_write_callback :: #type (buffer: *u8, size: u64, nitems: u64, outstream: *void) -> u64 #c_call;
}

#if OS == .WINDOWS {
    curl_write_callback :: #type (buffer: *u8, size: u64, nitems: u64, outstream: *void) -> u64 #c_call;
}

#if OS != .WINDOWS {
    curl_xferinfo_callback :: #type (clientp: *void, dltotal: curl_off_t, dlnow: curl_off_t, ultotal: curl_off_t, ulnow: curl_off_t) -> s32 #c_call;
}

#if OS == .WINDOWS {
    curl_xferinfo_callback :: #type (clientp: *void, dltotal: curl_off_t, dlnow: curl_off_t, ultotal: curl_off_t, ulnow: curl_off_t) -> s32 #c_call;
}

// Aliases and constants

#if OS != .WINDOWS {
    CURL :: void;
}

#if OS == .WINDOWS {
    CURL :: void;
}

#if OS != .WINDOWS {
    CURL_BLOB_COPY :: 1;
}

#if OS == .WINDOWS {
    CURL_BLOB_COPY :: 1;
}

#if OS != .WINDOWS {
    CURL_BLOB_NOCOPY :: 0;
}

#if OS == .WINDOWS {
    CURL_BLOB_NOCOPY :: 0;
}

#if OS != .WINDOWS {
    CURL_CHUNK_BGN_FUNC_FAIL :: 1;
}

#if OS == .WINDOWS {
    CURL_CHUNK_BGN_FUNC_FAIL :: 1;
}

#if OS != .WINDOWS {
    CURL_CHUNK_BGN_FUNC_OK :: 0;
}

#if OS == .WINDOWS {
    CURL_CHUNK_BGN_FUNC_OK :: 0;
}

#if OS != .WINDOWS {
    CURL_CHUNK_BGN_FUNC_SKIP :: 2;
}

#if OS == .WINDOWS {
    CURL_CHUNK_BGN_FUNC_SKIP :: 2;
}

#if OS != .WINDOWS {
    CURL_CHUNK_END_FUNC_FAIL :: 1;
}

#if OS == .WINDOWS {
    CURL_CHUNK_END_FUNC_FAIL :: 1;
}

#if OS != .WINDOWS {
    CURL_CHUNK_END_FUNC_OK :: 0;
}

#if OS == .WINDOWS {
    CURL_CHUNK_END_FUNC_OK :: 0;
}

#if OS != .WINDOWS {
    CURL_CSELECT_ERR :: 0x04;
}

#if OS == .WINDOWS {
    CURL_CSELECT_ERR :: 0x04;
}

#if OS != .WINDOWS {
    CURL_CSELECT_IN :: 0x01;
}

#if OS == .WINDOWS {
    CURL_CSELECT_IN :: 0x01;
}

#if OS != .WINDOWS {
    CURL_CSELECT_OUT :: 0x02;
}

#if OS == .WINDOWS {
    CURL_CSELECT_OUT :: 0x02;
}

#if OS != .WINDOWS {
    CURL_ERROR_SIZE :: 256;
}

#if OS == .WINDOWS {
    CURL_ERROR_SIZE :: 256;
}

#if OS != .WINDOWS {
    CURL_FNMATCHFUNC_FAIL :: 2;
}

#if OS == .WINDOWS {
    CURL_FNMATCHFUNC_FAIL :: 2;
}

#if OS != .WINDOWS {
    CURL_FNMATCHFUNC_MATCH :: 0;
}

#if OS == .WINDOWS {
    CURL_FNMATCHFUNC_MATCH :: 0;
}

#if OS != .WINDOWS {
    CURL_FNMATCHFUNC_NOMATCH :: 1;
}

#if OS == .WINDOWS {
    CURL_FNMATCHFUNC_NOMATCH :: 1;
}

#if OS != .WINDOWS {
    CURL_FORMAT_CURL_OFF_T :: "ld";
}

#if OS != .WINDOWS {
    CURL_FORMAT_CURL_OFF_TU :: "lu";
}

#if OS != .WINDOWS {
    curl_ftpssl :: curl_usessl;
}

#if OS == .WINDOWS {
    curl_ftpssl :: curl_usessl;
}

#if OS != .WINDOWS {
    CURL_GLOBAL_ACK_EINTR :: 1 << 2;
}

#if OS == .WINDOWS {
    CURL_GLOBAL_ACK_EINTR :: 1 << 2;
}

#if OS != .WINDOWS {
    CURL_GLOBAL_ALL :: CURL_GLOBAL_SSL | CURL_GLOBAL_WIN32;
}

#if OS == .WINDOWS {
    CURL_GLOBAL_ALL :: CURL_GLOBAL_SSL | CURL_GLOBAL_WIN32;
}

#if OS != .WINDOWS {
    CURL_GLOBAL_DEFAULT :: CURL_GLOBAL_ALL;
}

#if OS == .WINDOWS {
    CURL_GLOBAL_DEFAULT :: CURL_GLOBAL_ALL;
}

#if OS != .WINDOWS {
    CURL_GLOBAL_NOTHING :: 0;
}

#if OS == .WINDOWS {
    CURL_GLOBAL_NOTHING :: 0;
}

#if OS != .WINDOWS {
    CURL_GLOBAL_SSL :: 1 << 0;
}

#if OS == .WINDOWS {
    CURL_GLOBAL_SSL :: 1 << 0;
}

#if OS != .WINDOWS {
    CURL_GLOBAL_WIN32 :: 1 << 1;
}

#if OS == .WINDOWS {
    CURL_GLOBAL_WIN32 :: 1 << 1;
}

#if OS != .WINDOWS {
    CURL_HET_DEFAULT :: 200;
}

#if OS == .WINDOWS {
    CURL_HET_DEFAULT :: 200;
}

#if OS != .WINDOWS {
    CURL_HTTPPOST_BUFFER :: 1 << 4;
}

#if OS == .WINDOWS {
    CURL_HTTPPOST_BUFFER :: 1 << 4;
}

#if OS != .WINDOWS {
    CURL_HTTPPOST_CALLBACK :: 1 << 6;
}

#if OS == .WINDOWS {
    CURL_HTTPPOST_CALLBACK :: 1 << 6;
}

#if OS != .WINDOWS {
    CURL_HTTPPOST_FILENAME :: 1 << 0;
}

#if OS == .WINDOWS {
    CURL_HTTPPOST_FILENAME :: 1 << 0;
}

#if OS != .WINDOWS {
    CURL_HTTPPOST_LARGE :: 1 << 7;
}

#if OS == .WINDOWS {
    CURL_HTTPPOST_LARGE :: 1 << 7;
}

#if OS != .WINDOWS {
    CURL_HTTPPOST_PTRBUFFER :: 1 << 5;
}

#if OS == .WINDOWS {
    CURL_HTTPPOST_PTRBUFFER :: 1 << 5;
}

#if OS != .WINDOWS {
    CURL_HTTPPOST_PTRCONTENTS :: 1 << 3;
}

#if OS == .WINDOWS {
    CURL_HTTPPOST_PTRCONTENTS :: 1 << 3;
}

#if OS != .WINDOWS {
    CURL_HTTPPOST_PTRNAME :: 1 << 2;
}

#if OS == .WINDOWS {
    CURL_HTTPPOST_PTRNAME :: 1 << 2;
}

#if OS != .WINDOWS {
    CURL_HTTPPOST_READFILE :: 1 << 1;
}

#if OS == .WINDOWS {
    CURL_HTTPPOST_READFILE :: 1 << 1;
}

#if OS != .WINDOWS {
    CURL_IPRESOLVE_V4 :: 1;
}

#if OS == .WINDOWS {
    CURL_IPRESOLVE_V4 :: 1;
}

#if OS != .WINDOWS {
    CURL_IPRESOLVE_V6 :: 2;
}

#if OS == .WINDOWS {
    CURL_IPRESOLVE_V6 :: 2;
}

#if OS != .WINDOWS {
    CURL_IPRESOLVE_WHATEVER :: 0;
}

#if OS == .WINDOWS {
    CURL_IPRESOLVE_WHATEVER :: 0;
}

#if OS != .WINDOWS {
    CURL_MAX_HTTP_HEADER :: 100 * 1024;
}

#if OS == .WINDOWS {
    CURL_MAX_HTTP_HEADER :: 100 * 1024;
}

#if OS != .WINDOWS {
    CURL_MAX_READ_SIZE :: 10 * 1024 * 1024;
}

#if OS == .WINDOWS {
    CURL_MAX_READ_SIZE :: 10 * 1024 * 1024;
}

#if OS != .WINDOWS {
    CURL_MAX_WRITE_SIZE :: 16384;
}

#if OS == .WINDOWS {
    CURL_MAX_WRITE_SIZE :: 16384;
}

#if OS != .WINDOWS {
    curl_off_t :: s64;
}

#if OS == .WINDOWS {
    curl_off_t :: s64;
}

#if OS != .WINDOWS {
    CURL_POLL_IN :: 1;
}

#if OS == .WINDOWS {
    CURL_POLL_IN :: 1;
}

#if OS != .WINDOWS {
    CURL_POLL_INOUT :: 3;
}

#if OS == .WINDOWS {
    CURL_POLL_INOUT :: 3;
}

#if OS != .WINDOWS {
    CURL_POLL_NONE :: 0;
}

#if OS == .WINDOWS {
    CURL_POLL_NONE :: 0;
}

#if OS != .WINDOWS {
    CURL_POLL_OUT :: 2;
}

#if OS == .WINDOWS {
    CURL_POLL_OUT :: 2;
}

#if OS != .WINDOWS {
    CURL_POLL_REMOVE :: 4;
}

#if OS == .WINDOWS {
    CURL_POLL_REMOVE :: 4;
}

#if OS != .WINDOWS {
    CURL_PREREQFUNC_ABORT :: 1;
}

#if OS == .WINDOWS {
    CURL_PREREQFUNC_ABORT :: 1;
}

#if OS != .WINDOWS {
    CURL_PREREQFUNC_OK :: 0;
}

#if OS == .WINDOWS {
    CURL_PREREQFUNC_OK :: 0;
}

#if OS != .WINDOWS {
    CURL_PROGRESSFUNC_CONTINUE :: 0x10000001;
}

#if OS == .WINDOWS {
    CURL_PROGRESSFUNC_CONTINUE :: 0x10000001;
}

#if OS != .WINDOWS {
    CURL_PULL_SYS_SOCKET_H :: 1;
}

#if OS != .WINDOWS {
    CURL_PULL_SYS_TYPES_H :: 1;
}

#if OS != .WINDOWS {
    CURL_PUSH_DENY :: 1;
}

#if OS == .WINDOWS {
    CURL_PUSH_DENY :: 1;
}

#if OS != .WINDOWS {
    CURL_PUSH_ERROROUT :: 2;
}

#if OS == .WINDOWS {
    CURL_PUSH_ERROROUT :: 2;
}

#if OS != .WINDOWS {
    CURL_PUSH_OK :: 0;
}

#if OS == .WINDOWS {
    CURL_PUSH_OK :: 0;
}

#if OS != .WINDOWS {
    CURL_READFUNC_ABORT :: 0x10000000;
}

#if OS == .WINDOWS {
    CURL_READFUNC_ABORT :: 0x10000000;
}

#if OS != .WINDOWS {
    CURL_READFUNC_PAUSE :: 0x10000001;
}

#if OS == .WINDOWS {
    CURL_READFUNC_PAUSE :: 0x10000001;
}

#if OS != .WINDOWS {
    CURL_REDIR_GET_ALL :: 0;
}

#if OS == .WINDOWS {
    CURL_REDIR_GET_ALL :: 0;
}

#if OS != .WINDOWS {
    CURL_REDIR_POST_301 :: 1;
}

#if OS == .WINDOWS {
    CURL_REDIR_POST_301 :: 1;
}

#if OS != .WINDOWS {
    CURL_REDIR_POST_302 :: 2;
}

#if OS == .WINDOWS {
    CURL_REDIR_POST_302 :: 2;
}

#if OS != .WINDOWS {
    CURL_REDIR_POST_303 :: 4;
}

#if OS == .WINDOWS {
    CURL_REDIR_POST_303 :: 4;
}

#if OS != .WINDOWS {
    CURL_REDIR_POST_ALL :: CURL_REDIR_POST_301 | CURL_REDIR_POST_302 | CURL_REDIR_POST_303;
}

#if OS == .WINDOWS {
    CURL_REDIR_POST_ALL :: CURL_REDIR_POST_301 | CURL_REDIR_POST_302 | CURL_REDIR_POST_303;
}

#if OS != .WINDOWS {
    CURL_SEEKFUNC_CANTSEEK :: 2;
}

#if OS == .WINDOWS {
    CURL_SEEKFUNC_CANTSEEK :: 2;
}

#if OS != .WINDOWS {
    CURL_SEEKFUNC_FAIL :: 1;
}

#if OS == .WINDOWS {
    CURL_SEEKFUNC_FAIL :: 1;
}

#if OS != .WINDOWS {
    CURL_SEEKFUNC_OK :: 0;
}

#if OS == .WINDOWS {
    CURL_SEEKFUNC_OK :: 0;
}

#if OS != .WINDOWS {
    CURL_SOCKET_BAD :: -1;
}

#if OS != .WINDOWS {
    curl_socket_t :: s32;
}

#if OS == .WINDOWS {
    curl_socket_t :: SOCKET;
}

#if OS != .WINDOWS {
    CURL_SOCKET_TIMEOUT :: CURL_SOCKET_BAD;
}

#if OS != .WINDOWS {
    curl_socklen_t :: socklen_t;
}

#if OS == .WINDOWS {
    curl_socklen_t :: s32;
}

#if OS != .WINDOWS {
    CURL_SOCKOPT_ALREADY_CONNECTED :: 2;
}

#if OS == .WINDOWS {
    CURL_SOCKOPT_ALREADY_CONNECTED :: 2;
}

#if OS != .WINDOWS {
    CURL_SOCKOPT_ERROR :: 1;
}

#if OS == .WINDOWS {
    CURL_SOCKOPT_ERROR :: 1;
}

#if OS != .WINDOWS {
    CURL_SOCKOPT_OK :: 0;
}

#if OS == .WINDOWS {
    CURL_SOCKOPT_OK :: 0;
}

#if OS != .WINDOWS {
    CURL_TRAILERFUNC_ABORT :: 1;
}

#if OS == .WINDOWS {
    CURL_TRAILERFUNC_ABORT :: 1;
}

#if OS != .WINDOWS {
    CURL_TRAILERFUNC_OK :: 0;
}

#if OS == .WINDOWS {
    CURL_TRAILERFUNC_OK :: 0;
}

#if OS == .WINDOWS {
    CURL_TYPEOF_CURL_SOCKLEN_T :: s32;
}

#if OS != .WINDOWS {
    CURL_UPKEEP_INTERVAL_DEFAULT :: 60000;
}

#if OS == .WINDOWS {
    CURL_UPKEEP_INTERVAL_DEFAULT :: 60000;
}

#if OS != .WINDOWS {
    CURL_VERSION_ALTSVC :: 1 << 24;
}

#if OS == .WINDOWS {
    CURL_VERSION_ALTSVC :: 1 << 24;
}

#if OS != .WINDOWS {
    CURL_VERSION_ASYNCHDNS :: 1 << 7;
}

#if OS == .WINDOWS {
    CURL_VERSION_ASYNCHDNS :: 1 << 7;
}

#if OS != .WINDOWS {
    CURL_VERSION_BROTLI :: 1 << 23;
}

#if OS == .WINDOWS {
    CURL_VERSION_BROTLI :: 1 << 23;
}

#if OS != .WINDOWS {
    CURL_VERSION_CONV :: 1 << 12;
}

#if OS == .WINDOWS {
    CURL_VERSION_CONV :: 1 << 12;
}

#if OS != .WINDOWS {
    CURL_VERSION_CURLDEBUG :: 1 << 13;
}

#if OS == .WINDOWS {
    CURL_VERSION_CURLDEBUG :: 1 << 13;
}

#if OS != .WINDOWS {
    CURL_VERSION_DEBUG :: 1 << 6;
}

#if OS == .WINDOWS {
    CURL_VERSION_DEBUG :: 1 << 6;
}

#if OS != .WINDOWS {
    CURL_VERSION_GSASL :: 1 << 29;
}

#if OS == .WINDOWS {
    CURL_VERSION_GSASL :: 1 << 29;
}

#if OS != .WINDOWS {
    CURL_VERSION_GSSAPI :: 1 << 17;
}

#if OS == .WINDOWS {
    CURL_VERSION_GSSAPI :: 1 << 17;
}

#if OS != .WINDOWS {
    CURL_VERSION_GSSNEGOTIATE :: 1 << 5;
}

#if OS == .WINDOWS {
    CURL_VERSION_GSSNEGOTIATE :: 1 << 5;
}

#if OS != .WINDOWS {
    CURL_VERSION_HSTS :: 1 << 28;
}

#if OS == .WINDOWS {
    CURL_VERSION_HSTS :: 1 << 28;
}

#if OS != .WINDOWS {
    CURL_VERSION_HTTP2 :: 1 << 16;
}

#if OS == .WINDOWS {
    CURL_VERSION_HTTP2 :: 1 << 16;
}

#if OS != .WINDOWS {
    CURL_VERSION_HTTP3 :: 1 << 25;
}

#if OS == .WINDOWS {
    CURL_VERSION_HTTP3 :: 1 << 25;
}

#if OS != .WINDOWS {
    CURL_VERSION_HTTPS_PROXY :: 1 << 21;
}

#if OS == .WINDOWS {
    CURL_VERSION_HTTPS_PROXY :: 1 << 21;
}

#if OS != .WINDOWS {
    CURL_VERSION_IDN :: 1 << 10;
}

#if OS == .WINDOWS {
    CURL_VERSION_IDN :: 1 << 10;
}

#if OS != .WINDOWS {
    CURL_VERSION_IPV6 :: 1 << 0;
}

#if OS == .WINDOWS {
    CURL_VERSION_IPV6 :: 1 << 0;
}

#if OS != .WINDOWS {
    CURL_VERSION_KERBEROS4 :: 1 << 1;
}

#if OS == .WINDOWS {
    CURL_VERSION_KERBEROS4 :: 1 << 1;
}

#if OS != .WINDOWS {
    CURL_VERSION_KERBEROS5 :: 1 << 18;
}

#if OS == .WINDOWS {
    CURL_VERSION_KERBEROS5 :: 1 << 18;
}

#if OS != .WINDOWS {
    CURL_VERSION_LARGEFILE :: 1 << 9;
}

#if OS == .WINDOWS {
    CURL_VERSION_LARGEFILE :: 1 << 9;
}

#if OS != .WINDOWS {
    CURL_VERSION_LIBZ :: 1 << 3;
}

#if OS == .WINDOWS {
    CURL_VERSION_LIBZ :: 1 << 3;
}

#if OS != .WINDOWS {
    CURL_VERSION_MULTI_SSL :: 1 << 22;
}

#if OS == .WINDOWS {
    CURL_VERSION_MULTI_SSL :: 1 << 22;
}

#if OS != .WINDOWS {
    CURL_VERSION_NTLM :: 1 << 4;
}

#if OS == .WINDOWS {
    CURL_VERSION_NTLM :: 1 << 4;
}

#if OS != .WINDOWS {
    CURL_VERSION_NTLM_WB :: 1 << 15;
}

#if OS == .WINDOWS {
    CURL_VERSION_NTLM_WB :: 1 << 15;
}

#if OS != .WINDOWS {
    CURL_VERSION_PSL :: 1 << 20;
}

#if OS == .WINDOWS {
    CURL_VERSION_PSL :: 1 << 20;
}

#if OS != .WINDOWS {
    CURL_VERSION_SPNEGO :: 1 << 8;
}

#if OS == .WINDOWS {
    CURL_VERSION_SPNEGO :: 1 << 8;
}

#if OS != .WINDOWS {
    CURL_VERSION_SSL :: 1 << 2;
}

#if OS == .WINDOWS {
    CURL_VERSION_SSL :: 1 << 2;
}

#if OS != .WINDOWS {
    CURL_VERSION_SSPI :: 1 << 11;
}

#if OS == .WINDOWS {
    CURL_VERSION_SSPI :: 1 << 11;
}

#if OS != .WINDOWS {
    CURL_VERSION_THREADSAFE :: 1 << 30;
}

#if OS == .WINDOWS {
    CURL_VERSION_THREADSAFE :: 1 << 30;
}

#if OS != .WINDOWS {
    CURL_VERSION_TLSAUTH_SRP :: 1 << 14;
}

#if OS == .WINDOWS {
    CURL_VERSION_TLSAUTH_SRP :: 1 << 14;
}

#if OS != .WINDOWS {
    CURL_VERSION_UNICODE :: 1 << 27;
}

#if OS == .WINDOWS {
    CURL_VERSION_UNICODE :: 1 << 27;
}

#if OS != .WINDOWS {
    CURL_VERSION_UNIX_SOCKETS :: 1 << 19;
}

#if OS == .WINDOWS {
    CURL_VERSION_UNIX_SOCKETS :: 1 << 19;
}

#if OS != .WINDOWS {
    CURL_VERSION_ZSTD :: 1 << 26;
}

#if OS == .WINDOWS {
    CURL_VERSION_ZSTD :: 1 << 26;
}

#if OS != .WINDOWS {
    CURL_WAIT_POLLIN :: 0x0001;
}

#if OS == .WINDOWS {
    CURL_WAIT_POLLIN :: 0x0001;
}

#if OS != .WINDOWS {
    CURL_WAIT_POLLOUT :: 0x0004;
}

#if OS == .WINDOWS {
    CURL_WAIT_POLLOUT :: 0x0004;
}

#if OS != .WINDOWS {
    CURL_WAIT_POLLPRI :: 0x0002;
}

#if OS == .WINDOWS {
    CURL_WAIT_POLLPRI :: 0x0002;
}

#if OS != .WINDOWS {
    CURL_WRITEFUNC_ERROR :: 0xFFFFFFFF;
}

#if OS == .WINDOWS {
    CURL_WRITEFUNC_ERROR :: 0xFFFFFFFF;
}

#if OS != .WINDOWS {
    CURL_WRITEFUNC_PAUSE :: 0x10000001;
}

#if OS == .WINDOWS {
    CURL_WRITEFUNC_PAUSE :: 0x10000001;
}

#if OS != .WINDOWS {
    CURL_ZERO_TERMINATED :: cast,trunc(u64) -1;
}

#if OS == .WINDOWS {
    CURL_ZERO_TERMINATED :: cast,trunc(u64) -1;
}

#if OS != .WINDOWS {
    CURLALTSVC_H1 :: 1 << 3;
}

#if OS == .WINDOWS {
    CURLALTSVC_H1 :: 1 << 3;
}

#if OS != .WINDOWS {
    CURLALTSVC_H2 :: 1 << 4;
}

#if OS == .WINDOWS {
    CURLALTSVC_H2 :: 1 << 4;
}

#if OS != .WINDOWS {
    CURLALTSVC_H3 :: 1 << 5;
}

#if OS == .WINDOWS {
    CURLALTSVC_H3 :: 1 << 5;
}

#if OS != .WINDOWS {
    CURLALTSVC_READONLYFILE :: 1 << 2;
}

#if OS == .WINDOWS {
    CURLALTSVC_READONLYFILE :: 1 << 2;
}

#if OS != .WINDOWS {
    CURLE_ALREADY_COMPLETE :: 99999;
}

#if OS == .WINDOWS {
    CURLE_ALREADY_COMPLETE :: 99999;
}

#if OS != .WINDOWS {
    CURLFINFOFLAG_KNOWN_FILENAME :: 1 << 0;
}

#if OS == .WINDOWS {
    CURLFINFOFLAG_KNOWN_FILENAME :: 1 << 0;
}

#if OS != .WINDOWS {
    CURLFINFOFLAG_KNOWN_FILETYPE :: 1 << 1;
}

#if OS == .WINDOWS {
    CURLFINFOFLAG_KNOWN_FILETYPE :: 1 << 1;
}

#if OS != .WINDOWS {
    CURLFINFOFLAG_KNOWN_GID :: 1 << 5;
}

#if OS == .WINDOWS {
    CURLFINFOFLAG_KNOWN_GID :: 1 << 5;
}

#if OS != .WINDOWS {
    CURLFINFOFLAG_KNOWN_HLINKCOUNT :: 1 << 7;
}

#if OS == .WINDOWS {
    CURLFINFOFLAG_KNOWN_HLINKCOUNT :: 1 << 7;
}

#if OS != .WINDOWS {
    CURLFINFOFLAG_KNOWN_PERM :: 1 << 3;
}

#if OS == .WINDOWS {
    CURLFINFOFLAG_KNOWN_PERM :: 1 << 3;
}

#if OS != .WINDOWS {
    CURLFINFOFLAG_KNOWN_SIZE :: 1 << 6;
}

#if OS == .WINDOWS {
    CURLFINFOFLAG_KNOWN_SIZE :: 1 << 6;
}

#if OS != .WINDOWS {
    CURLFINFOFLAG_KNOWN_TIME :: 1 << 2;
}

#if OS == .WINDOWS {
    CURLFINFOFLAG_KNOWN_TIME :: 1 << 2;
}

#if OS != .WINDOWS {
    CURLFINFOFLAG_KNOWN_UID :: 1 << 4;
}

#if OS == .WINDOWS {
    CURLFINFOFLAG_KNOWN_UID :: 1 << 4;
}

#if OS != .WINDOWS {
    CURLGSSAPI_DELEGATION_FLAG :: 1 << 1;
}

#if OS == .WINDOWS {
    CURLGSSAPI_DELEGATION_FLAG :: 1 << 1;
}

#if OS != .WINDOWS {
    CURLGSSAPI_DELEGATION_NONE :: 0;
}

#if OS == .WINDOWS {
    CURLGSSAPI_DELEGATION_NONE :: 0;
}

#if OS != .WINDOWS {
    CURLGSSAPI_DELEGATION_POLICY_FLAG :: 1 << 0;
}

#if OS == .WINDOWS {
    CURLGSSAPI_DELEGATION_POLICY_FLAG :: 1 << 0;
}

#if OS != .WINDOWS {
    CURLH_1XX :: 1 << 3;
}

#if OS == .WINDOWS {
    CURLH_1XX :: 1 << 3;
}

#if OS != .WINDOWS {
    CURLH_CONNECT :: 1 << 2;
}

#if OS == .WINDOWS {
    CURLH_CONNECT :: 1 << 2;
}

#if OS != .WINDOWS {
    CURLH_HEADER :: 1 << 0;
}

#if OS == .WINDOWS {
    CURLH_HEADER :: 1 << 0;
}

#if OS != .WINDOWS {
    CURLH_PSEUDO :: 1 << 4;
}

#if OS == .WINDOWS {
    CURLH_PSEUDO :: 1 << 4;
}

#if OS != .WINDOWS {
    CURLH_TRAILER :: 1 << 1;
}

#if OS == .WINDOWS {
    CURLH_TRAILER :: 1 << 1;
}

#if OS != .WINDOWS {
    CURLHEADER_SEPARATE :: 1 << 0;
}

#if OS == .WINDOWS {
    CURLHEADER_SEPARATE :: 1 << 0;
}

#if OS != .WINDOWS {
    CURLHEADER_UNIFIED :: 0;
}

#if OS == .WINDOWS {
    CURLHEADER_UNIFIED :: 0;
}

#if OS != .WINDOWS {
    CURLINFO_DOUBLE :: 0x300000;
}

#if OS == .WINDOWS {
    CURLINFO_DOUBLE :: 0x300000;
}

#if OS != .WINDOWS {
    CURLINFO_LONG :: 0x200000;
}

#if OS == .WINDOWS {
    CURLINFO_LONG :: 0x200000;
}

#if OS != .WINDOWS {
    CURLINFO_MASK :: 0x0fffff;
}

#if OS == .WINDOWS {
    CURLINFO_MASK :: 0x0fffff;
}

#if OS != .WINDOWS {
    CURLINFO_OFF_T :: 0x600000;
}

#if OS == .WINDOWS {
    CURLINFO_OFF_T :: 0x600000;
}

#if OS != .WINDOWS {
    CURLINFO_PTR :: 0x400000;
}

#if OS == .WINDOWS {
    CURLINFO_PTR :: 0x400000;
}

#if OS != .WINDOWS {
    CURLINFO_SLIST :: 0x400000;
}

#if OS == .WINDOWS {
    CURLINFO_SLIST :: 0x400000;
}

#if OS != .WINDOWS {
    CURLINFO_SOCKET :: 0x500000;
}

#if OS == .WINDOWS {
    CURLINFO_SOCKET :: 0x500000;
}

#if OS != .WINDOWS {
    CURLINFO_STRING :: 0x100000;
}

#if OS == .WINDOWS {
    CURLINFO_STRING :: 0x100000;
}

#if OS != .WINDOWS {
    CURLINFO_TYPEMASK :: 0xf00000;
}

#if OS == .WINDOWS {
    CURLINFO_TYPEMASK :: 0xf00000;
}

#if OS != .WINDOWS {
    CURLM :: void;
}

#if OS == .WINDOWS {
    CURLM :: void;
}

#if OS != .WINDOWS {
    CURLMIMEOPT_FORMESCAPE :: 1 << 0;
}

#if OS == .WINDOWS {
    CURLMIMEOPT_FORMESCAPE :: 1 << 0;
}

#if OS != .WINDOWS {
    CURLOPTTYPE_BLOB :: 40000;
}

#if OS == .WINDOWS {
    CURLOPTTYPE_BLOB :: 40000;
}

#if OS != .WINDOWS {
    CURLOPTTYPE_CBPOINT :: CURLOPTTYPE_OBJECTPOINT;
}

#if OS == .WINDOWS {
    CURLOPTTYPE_CBPOINT :: CURLOPTTYPE_OBJECTPOINT;
}

#if OS != .WINDOWS {
    CURLOPTTYPE_FUNCTIONPOINT :: 20000;
}

#if OS == .WINDOWS {
    CURLOPTTYPE_FUNCTIONPOINT :: 20000;
}

#if OS != .WINDOWS {
    CURLOPTTYPE_LONG :: 0;
}

#if OS == .WINDOWS {
    CURLOPTTYPE_LONG :: 0;
}

#if OS != .WINDOWS {
    CURLOPTTYPE_OBJECTPOINT :: 10000;
}

#if OS == .WINDOWS {
    CURLOPTTYPE_OBJECTPOINT :: 10000;
}

#if OS != .WINDOWS {
    CURLOPTTYPE_OFF_T :: 30000;
}

#if OS == .WINDOWS {
    CURLOPTTYPE_OFF_T :: 30000;
}

#if OS != .WINDOWS {
    CURLOPTTYPE_SLISTPOINT :: CURLOPTTYPE_OBJECTPOINT;
}

#if OS == .WINDOWS {
    CURLOPTTYPE_SLISTPOINT :: CURLOPTTYPE_OBJECTPOINT;
}

#if OS != .WINDOWS {
    CURLOPTTYPE_STRINGPOINT :: CURLOPTTYPE_OBJECTPOINT;
}

#if OS == .WINDOWS {
    CURLOPTTYPE_STRINGPOINT :: CURLOPTTYPE_OBJECTPOINT;
}

#if OS != .WINDOWS {
    CURLOPTTYPE_VALUES :: CURLOPTTYPE_LONG;
}

#if OS == .WINDOWS {
    CURLOPTTYPE_VALUES :: CURLOPTTYPE_LONG;
}

#if OS != .WINDOWS {
    CURLOT_FLAG_ALIAS :: 1 << 0;
}

#if OS == .WINDOWS {
    CURLOT_FLAG_ALIAS :: 1 << 0;
}

#if OS != .WINDOWS {
    CURLPAUSE_ALL :: CURLPAUSE_RECV | CURLPAUSE_SEND;
}

#if OS == .WINDOWS {
    CURLPAUSE_ALL :: CURLPAUSE_RECV | CURLPAUSE_SEND;
}

#if OS != .WINDOWS {
    CURLPAUSE_CONT :: CURLPAUSE_RECV_CONT | CURLPAUSE_SEND_CONT;
}

#if OS == .WINDOWS {
    CURLPAUSE_CONT :: CURLPAUSE_RECV_CONT | CURLPAUSE_SEND_CONT;
}

#if OS != .WINDOWS {
    CURLPAUSE_RECV :: 1 << 0;
}

#if OS == .WINDOWS {
    CURLPAUSE_RECV :: 1 << 0;
}

#if OS != .WINDOWS {
    CURLPAUSE_RECV_CONT :: 0;
}

#if OS == .WINDOWS {
    CURLPAUSE_RECV_CONT :: 0;
}

#if OS != .WINDOWS {
    CURLPAUSE_SEND :: 1 << 2;
}

#if OS == .WINDOWS {
    CURLPAUSE_SEND :: 1 << 2;
}

#if OS != .WINDOWS {
    CURLPAUSE_SEND_CONT :: 0;
}

#if OS == .WINDOWS {
    CURLPAUSE_SEND_CONT :: 0;
}

#if OS != .WINDOWS {
    CURLPIPE_HTTP1 :: 1;
}

#if OS == .WINDOWS {
    CURLPIPE_HTTP1 :: 1;
}

#if OS != .WINDOWS {
    CURLPIPE_MULTIPLEX :: 2;
}

#if OS == .WINDOWS {
    CURLPIPE_MULTIPLEX :: 2;
}

#if OS != .WINDOWS {
    CURLPIPE_NOTHING :: 0;
}

#if OS == .WINDOWS {
    CURLPIPE_NOTHING :: 0;
}

#if OS != .WINDOWS {
    CURLPROTO_ALL :: ~0;
}

#if OS == .WINDOWS {
    CURLPROTO_ALL :: ~0;
}

#if OS != .WINDOWS {
    CURLPROTO_DICT :: 1 << 9;
}

#if OS == .WINDOWS {
    CURLPROTO_DICT :: 1 << 9;
}

#if OS != .WINDOWS {
    CURLPROTO_FILE :: 1 << 10;
}

#if OS == .WINDOWS {
    CURLPROTO_FILE :: 1 << 10;
}

#if OS != .WINDOWS {
    CURLPROTO_FTP :: 1 << 2;
}

#if OS == .WINDOWS {
    CURLPROTO_FTP :: 1 << 2;
}

#if OS != .WINDOWS {
    CURLPROTO_FTPS :: 1 << 3;
}

#if OS == .WINDOWS {
    CURLPROTO_FTPS :: 1 << 3;
}

#if OS != .WINDOWS {
    CURLPROTO_GOPHER :: 1 << 25;
}

#if OS == .WINDOWS {
    CURLPROTO_GOPHER :: 1 << 25;
}

#if OS != .WINDOWS {
    CURLPROTO_GOPHERS :: 1 << 29;
}

#if OS == .WINDOWS {
    CURLPROTO_GOPHERS :: 1 << 29;
}

#if OS != .WINDOWS {
    CURLPROTO_HTTP :: 1 << 0;
}

#if OS == .WINDOWS {
    CURLPROTO_HTTP :: 1 << 0;
}

#if OS != .WINDOWS {
    CURLPROTO_HTTPS :: 1 << 1;
}

#if OS == .WINDOWS {
    CURLPROTO_HTTPS :: 1 << 1;
}

#if OS != .WINDOWS {
    CURLPROTO_IMAP :: 1 << 12;
}

#if OS == .WINDOWS {
    CURLPROTO_IMAP :: 1 << 12;
}

#if OS != .WINDOWS {
    CURLPROTO_IMAPS :: 1 << 13;
}

#if OS == .WINDOWS {
    CURLPROTO_IMAPS :: 1 << 13;
}

#if OS != .WINDOWS {
    CURLPROTO_LDAP :: 1 << 7;
}

#if OS == .WINDOWS {
    CURLPROTO_LDAP :: 1 << 7;
}

#if OS != .WINDOWS {
    CURLPROTO_LDAPS :: 1 << 8;
}

#if OS == .WINDOWS {
    CURLPROTO_LDAPS :: 1 << 8;
}

#if OS != .WINDOWS {
    CURLPROTO_MQTT :: 1 << 28;
}

#if OS == .WINDOWS {
    CURLPROTO_MQTT :: 1 << 28;
}

#if OS != .WINDOWS {
    CURLPROTO_POP3 :: 1 << 14;
}

#if OS == .WINDOWS {
    CURLPROTO_POP3 :: 1 << 14;
}

#if OS != .WINDOWS {
    CURLPROTO_POP3S :: 1 << 15;
}

#if OS == .WINDOWS {
    CURLPROTO_POP3S :: 1 << 15;
}

#if OS != .WINDOWS {
    CURLPROTO_RTMP :: 1 << 19;
}

#if OS == .WINDOWS {
    CURLPROTO_RTMP :: 1 << 19;
}

#if OS != .WINDOWS {
    CURLPROTO_RTMPE :: 1 << 21;
}

#if OS == .WINDOWS {
    CURLPROTO_RTMPE :: 1 << 21;
}

#if OS != .WINDOWS {
    CURLPROTO_RTMPS :: 1 << 23;
}

#if OS == .WINDOWS {
    CURLPROTO_RTMPS :: 1 << 23;
}

#if OS != .WINDOWS {
    CURLPROTO_RTMPT :: 1 << 20;
}

#if OS == .WINDOWS {
    CURLPROTO_RTMPT :: 1 << 20;
}

#if OS != .WINDOWS {
    CURLPROTO_RTMPTE :: 1 << 22;
}

#if OS == .WINDOWS {
    CURLPROTO_RTMPTE :: 1 << 22;
}

#if OS != .WINDOWS {
    CURLPROTO_RTMPTS :: 1 << 24;
}

#if OS == .WINDOWS {
    CURLPROTO_RTMPTS :: 1 << 24;
}

#if OS != .WINDOWS {
    CURLPROTO_RTSP :: 1 << 18;
}

#if OS == .WINDOWS {
    CURLPROTO_RTSP :: 1 << 18;
}

#if OS != .WINDOWS {
    CURLPROTO_SCP :: 1 << 4;
}

#if OS == .WINDOWS {
    CURLPROTO_SCP :: 1 << 4;
}

#if OS != .WINDOWS {
    CURLPROTO_SFTP :: 1 << 5;
}

#if OS == .WINDOWS {
    CURLPROTO_SFTP :: 1 << 5;
}

#if OS != .WINDOWS {
    CURLPROTO_SMB :: 1 << 26;
}

#if OS == .WINDOWS {
    CURLPROTO_SMB :: 1 << 26;
}

#if OS != .WINDOWS {
    CURLPROTO_SMBS :: 1 << 27;
}

#if OS == .WINDOWS {
    CURLPROTO_SMBS :: 1 << 27;
}

#if OS != .WINDOWS {
    CURLPROTO_SMTP :: 1 << 16;
}

#if OS == .WINDOWS {
    CURLPROTO_SMTP :: 1 << 16;
}

#if OS != .WINDOWS {
    CURLPROTO_SMTPS :: 1 << 17;
}

#if OS == .WINDOWS {
    CURLPROTO_SMTPS :: 1 << 17;
}

#if OS != .WINDOWS {
    CURLPROTO_TELNET :: 1 << 6;
}

#if OS == .WINDOWS {
    CURLPROTO_TELNET :: 1 << 6;
}

#if OS != .WINDOWS {
    CURLPROTO_TFTP :: 1 << 11;
}

#if OS == .WINDOWS {
    CURLPROTO_TFTP :: 1 << 11;
}

#if OS != .WINDOWS {
    CURLSH :: void;
}

#if OS == .WINDOWS {
    CURLSH :: void;
}

#if OS != .WINDOWS {
    CURLSSH_AUTH_AGENT :: 1 << 4;
}

#if OS == .WINDOWS {
    CURLSSH_AUTH_AGENT :: 1 << 4;
}

#if OS != .WINDOWS {
    CURLSSH_AUTH_ANY :: ~0;
}

#if OS == .WINDOWS {
    CURLSSH_AUTH_ANY :: ~0;
}

#if OS != .WINDOWS {
    CURLSSH_AUTH_DEFAULT :: CURLSSH_AUTH_ANY;
}

#if OS == .WINDOWS {
    CURLSSH_AUTH_DEFAULT :: CURLSSH_AUTH_ANY;
}

#if OS != .WINDOWS {
    CURLSSH_AUTH_GSSAPI :: 1 << 5;
}

#if OS == .WINDOWS {
    CURLSSH_AUTH_GSSAPI :: 1 << 5;
}

#if OS != .WINDOWS {
    CURLSSH_AUTH_HOST :: 1 << 2;
}

#if OS == .WINDOWS {
    CURLSSH_AUTH_HOST :: 1 << 2;
}

#if OS != .WINDOWS {
    CURLSSH_AUTH_KEYBOARD :: 1 << 3;
}

#if OS == .WINDOWS {
    CURLSSH_AUTH_KEYBOARD :: 1 << 3;
}

#if OS != .WINDOWS {
    CURLSSH_AUTH_NONE :: 0;
}

#if OS == .WINDOWS {
    CURLSSH_AUTH_NONE :: 0;
}

#if OS != .WINDOWS {
    CURLSSH_AUTH_PASSWORD :: 1 << 1;
}

#if OS == .WINDOWS {
    CURLSSH_AUTH_PASSWORD :: 1 << 1;
}

#if OS != .WINDOWS {
    CURLSSH_AUTH_PUBLICKEY :: 1 << 0;
}

#if OS == .WINDOWS {
    CURLSSH_AUTH_PUBLICKEY :: 1 << 0;
}

#if OS != .WINDOWS {
    CURLSSLOPT_ALLOW_BEAST :: 1 << 0;
}

#if OS == .WINDOWS {
    CURLSSLOPT_ALLOW_BEAST :: 1 << 0;
}

#if OS != .WINDOWS {
    CURLSSLOPT_AUTO_CLIENT_CERT :: 1 << 5;
}

#if OS == .WINDOWS {
    CURLSSLOPT_AUTO_CLIENT_CERT :: 1 << 5;
}

#if OS != .WINDOWS {
    CURLSSLOPT_NATIVE_CA :: 1 << 4;
}

#if OS == .WINDOWS {
    CURLSSLOPT_NATIVE_CA :: 1 << 4;
}

#if OS != .WINDOWS {
    CURLSSLOPT_NO_PARTIALCHAIN :: 1 << 2;
}

#if OS == .WINDOWS {
    CURLSSLOPT_NO_PARTIALCHAIN :: 1 << 2;
}

#if OS != .WINDOWS {
    CURLSSLOPT_NO_REVOKE :: 1 << 1;
}

#if OS == .WINDOWS {
    CURLSSLOPT_NO_REVOKE :: 1 << 1;
}

#if OS != .WINDOWS {
    CURLSSLOPT_REVOKE_BEST_EFFORT :: 1 << 3;
}

#if OS == .WINDOWS {
    CURLSSLOPT_REVOKE_BEST_EFFORT :: 1 << 3;
}

#if OS != .WINDOWS {
    CURLU :: Curl_URL;
}

#if OS == .WINDOWS {
    CURLU :: Curl_URL;
}

#if OS != .WINDOWS {
    CURLU_ALLOW_SPACE :: 1 << 11;
}

#if OS == .WINDOWS {
    CURLU_ALLOW_SPACE :: 1 << 11;
}

#if OS != .WINDOWS {
    CURLU_APPENDQUERY :: 1 << 8;
}

#if OS == .WINDOWS {
    CURLU_APPENDQUERY :: 1 << 8;
}

#if OS != .WINDOWS {
    CURLU_DEFAULT_PORT :: 1 << 0;
}

#if OS == .WINDOWS {
    CURLU_DEFAULT_PORT :: 1 << 0;
}

#if OS != .WINDOWS {
    CURLU_DEFAULT_SCHEME :: 1 << 2;
}

#if OS == .WINDOWS {
    CURLU_DEFAULT_SCHEME :: 1 << 2;
}

#if OS != .WINDOWS {
    CURLU_DISALLOW_USER :: 1 << 5;
}

#if OS == .WINDOWS {
    CURLU_DISALLOW_USER :: 1 << 5;
}

#if OS != .WINDOWS {
    CURLU_GUESS_SCHEME :: 1 << 9;
}

#if OS == .WINDOWS {
    CURLU_GUESS_SCHEME :: 1 << 9;
}

#if OS != .WINDOWS {
    CURLU_NO_AUTHORITY :: 1 << 10;
}

#if OS == .WINDOWS {
    CURLU_NO_AUTHORITY :: 1 << 10;
}

#if OS != .WINDOWS {
    CURLU_NO_DEFAULT_PORT :: 1 << 1;
}

#if OS == .WINDOWS {
    CURLU_NO_DEFAULT_PORT :: 1 << 1;
}

#if OS != .WINDOWS {
    CURLU_NON_SUPPORT_SCHEME :: 1 << 3;
}

#if OS == .WINDOWS {
    CURLU_NON_SUPPORT_SCHEME :: 1 << 3;
}

#if OS != .WINDOWS {
    CURLU_PATH_AS_IS :: 1 << 4;
}

#if OS == .WINDOWS {
    CURLU_PATH_AS_IS :: 1 << 4;
}

#if OS != .WINDOWS {
    CURLU_PUNY2IDN :: 1 << 13;
}

#if OS == .WINDOWS {
    CURLU_PUNY2IDN :: 1 << 13;
}

#if OS != .WINDOWS {
    CURLU_PUNYCODE :: 1 << 12;
}

#if OS == .WINDOWS {
    CURLU_PUNYCODE :: 1 << 12;
}

#if OS != .WINDOWS {
    CURLU_URLDECODE :: 1 << 6;
}

#if OS == .WINDOWS {
    CURLU_URLDECODE :: 1 << 6;
}

#if OS != .WINDOWS {
    CURLU_URLENCODE :: 1 << 7;
}

#if OS == .WINDOWS {
    CURLU_URLENCODE :: 1 << 7;
}

#if OS != .WINDOWS {
    CURLWS_BINARY :: 1 << 1;
}

#if OS == .WINDOWS {
    CURLWS_BINARY :: 1 << 1;
}

#if OS != .WINDOWS {
    CURLWS_CLOSE :: 1 << 3;
}

#if OS == .WINDOWS {
    CURLWS_CLOSE :: 1 << 3;
}

#if OS != .WINDOWS {
    CURLWS_CONT :: 1 << 2;
}

#if OS == .WINDOWS {
    CURLWS_CONT :: 1 << 2;
}

#if OS != .WINDOWS {
    CURLWS_OFFSET :: 1 << 5;
}

#if OS == .WINDOWS {
    CURLWS_OFFSET :: 1 << 5;
}

#if OS != .WINDOWS {
    CURLWS_PING :: 1 << 4;
}

#if OS == .WINDOWS {
    CURLWS_PING :: 1 << 4;
}

#if OS != .WINDOWS {
    CURLWS_PONG :: 1 << 6;
}

#if OS == .WINDOWS {
    CURLWS_PONG :: 1 << 6;
}

#if OS != .WINDOWS {
    CURLWS_RAW_MODE :: 1 << 0;
}

#if OS == .WINDOWS {
    CURLWS_RAW_MODE :: 1 << 0;
}

#if OS != .WINDOWS {
    CURLWS_TEXT :: 1 << 0;
}

#if OS == .WINDOWS {
    CURLWS_TEXT :: 1 << 0;
}

#if OS != .WINDOWS {
    LIBCURL_COPYRIGHT :: "Daniel Stenberg, <daniel@haxx.se>.";
}

#if OS == .WINDOWS {
    LIBCURL_COPYRIGHT :: "Daniel Stenberg, <daniel@haxx.se>.";
}

#if OS != .WINDOWS {
    LIBCURL_TIMESTAMP :: "2023-12-06";
}

#if OS == .WINDOWS {
    LIBCURL_TIMESTAMP :: "2023-12-06";
}

#if OS != .WINDOWS {
    LIBCURL_VERSION :: "8.5.0";
}

#if OS == .WINDOWS {
    LIBCURL_VERSION :: "8.5.0";
}

#if OS != .WINDOWS {
    LIBCURL_VERSION_MAJOR :: 8;
}

#if OS == .WINDOWS {
    LIBCURL_VERSION_MAJOR :: 8;
}

#if OS != .WINDOWS {
    LIBCURL_VERSION_MINOR :: 5;
}

#if OS == .WINDOWS {
    LIBCURL_VERSION_MINOR :: 5;
}

#if OS != .WINDOWS {
    LIBCURL_VERSION_NUM :: 0x080500;
}

#if OS == .WINDOWS {
    LIBCURL_VERSION_NUM :: 0x080500;
}

#if OS != .WINDOWS {
    LIBCURL_VERSION_PATCH :: 0;
}

#if OS == .WINDOWS {
    LIBCURL_VERSION_PATCH :: 0;
}

// Native entry points

#if OS != .WINDOWS {
    curl_easy_cleanup :: (curl: *CURL) -> void #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_easy_cleanup :: (curl: *CURL) -> void #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_easy_duphandle :: (curl: *CURL) -> *CURL #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_easy_duphandle :: (curl: *CURL) -> *CURL #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_easy_escape :: (handle: *CURL, _string: *u8, length: s32) -> *u8 #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_easy_escape :: (handle: *CURL, _string: *u8, length: s32) -> *u8 #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_easy_getinfo :: (curl: *CURL, info: CURLINFO, __args: ..Any) -> CURLcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_easy_getinfo :: (curl: *CURL, info: CURLINFO, __args: ..Any) -> CURLcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_easy_header :: (easy: *CURL, name: *u8, index: u64, origin: u32, request: s32, hout: **curl_header) -> CURLHcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_easy_header :: (easy: *CURL, name: *u8, index: u64, origin: u32, request: s32, hout: **curl_header) -> CURLHcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_easy_init :: () -> *CURL #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_easy_init :: () -> *CURL #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_easy_nextheader :: (easy: *CURL, origin: u32, request: s32, prev: *curl_header) -> *curl_header #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_easy_nextheader :: (easy: *CURL, origin: u32, request: s32, prev: *curl_header) -> *curl_header #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_easy_option_by_id :: (id: CURLoption) -> *curl_easyoption #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_easy_option_by_id :: (id: CURLoption) -> *curl_easyoption #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_easy_option_by_name :: (name: *u8) -> *curl_easyoption #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_easy_option_by_name :: (name: *u8) -> *curl_easyoption #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_easy_option_next :: (prev: *curl_easyoption) -> *curl_easyoption #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_easy_option_next :: (prev: *curl_easyoption) -> *curl_easyoption #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_easy_pause :: (handle: *CURL, bitmask: s32) -> CURLcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_easy_pause :: (handle: *CURL, bitmask: s32) -> CURLcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_easy_perform :: (curl: *CURL) -> CURLcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_easy_perform :: (curl: *CURL) -> CURLcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_easy_recv :: (curl: *CURL, buffer: *void, buflen: u64, n: *u64) -> CURLcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_easy_recv :: (curl: *CURL, buffer: *void, buflen: u64, n: *u64) -> CURLcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_easy_reset :: (curl: *CURL) -> void #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_easy_reset :: (curl: *CURL) -> void #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_easy_send :: (curl: *CURL, buffer: *void, buflen: u64, n: *u64) -> CURLcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_easy_send :: (curl: *CURL, buffer: *void, buflen: u64, n: *u64) -> CURLcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_easy_setopt :: (curl: *CURL, option: CURLoption, __args: ..Any) -> CURLcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_easy_setopt :: (curl: *CURL, option: CURLoption, __args: ..Any) -> CURLcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_easy_strerror :: (unknown0: CURLcode) -> *u8 #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_easy_strerror :: (unknown0: CURLcode) -> *u8 #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_easy_unescape :: (handle: *CURL, _string: *u8, length: s32, outlength: *s32) -> *u8 #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_easy_unescape :: (handle: *CURL, _string: *u8, length: s32, outlength: *s32) -> *u8 #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_easy_upkeep :: (curl: *CURL) -> CURLcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_easy_upkeep :: (curl: *CURL) -> CURLcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_escape :: (_string: *u8, length: s32) -> *u8 #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_escape :: (_string: *u8, length: s32) -> *u8 #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_formadd :: (httppost: **curl_httppost, last_post: **curl_httppost, __args: ..Any) -> CURLFORMcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_formadd :: (httppost: **curl_httppost, last_post: **curl_httppost, __args: ..Any) -> CURLFORMcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_formfree :: (form: *curl_httppost) -> void #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_formfree :: (form: *curl_httppost) -> void #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_formget :: (form: *curl_httppost, arg: *void, append: curl_formget_callback) -> s32 #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_formget :: (form: *curl_httppost, arg: *void, append: curl_formget_callback) -> s32 #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_free :: (p: *void) -> void #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_free :: (p: *void) -> void #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_getdate :: (p: *u8, unused: *time_t) -> time_t #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_getdate :: (p: *u8, unused: *time_t) -> time_t #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_getenv :: (variable: *u8) -> *u8 #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_getenv :: (variable: *u8) -> *u8 #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_global_cleanup :: () -> void #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_global_cleanup :: () -> void #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_global_init :: (flags: s64) -> CURLcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_global_init :: (flags: s32) -> CURLcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_global_init_mem :: (flags: s64, m: curl_malloc_callback, f: curl_free_callback, r: curl_realloc_callback, s: curl_strdup_callback, c: curl_calloc_callback) -> CURLcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_global_init_mem :: (flags: s32, m: curl_malloc_callback, f: curl_free_callback, r: curl_realloc_callback, s: curl_strdup_callback, c: curl_calloc_callback) -> CURLcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_global_sslset :: (id: curl_sslbackend, name: *u8, avail: ***curl_ssl_backend) -> CURLsslset #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_global_sslset :: (id: curl_sslbackend, name: *u8, avail: ***curl_ssl_backend) -> CURLsslset #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_maprintf_CFormat :: (format: *u8, __args: ..Any) -> *u8 #foreign libcurl "curl_maprintf";
}

#if OS == .WINDOWS {
    curl_maprintf_CFormat :: (format: *u8, __args: ..Any) -> *u8 #foreign libcurl "curl_maprintf";
}

#if OS != .WINDOWS {
    curl_mfprintf_CFormat :: (fd: *FILE, format: *u8, __args: ..Any) -> s32 #foreign libcurl "curl_mfprintf";
}

#if OS == .WINDOWS {
    curl_mfprintf_CFormat :: (fd: *FILE, format: *u8, __args: ..Any) -> s32 #foreign libcurl "curl_mfprintf";
}

#if OS != .WINDOWS {
    curl_mime_addpart :: (mime: *curl_mime) -> *curl_mimepart #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_mime_addpart :: (mime: *curl_mime) -> *curl_mimepart #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_mime_data :: (part: *curl_mimepart, data: *u8, datasize: u64) -> CURLcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_mime_data :: (part: *curl_mimepart, data: *u8, datasize: u64) -> CURLcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_mime_data_cb :: (part: *curl_mimepart, datasize: curl_off_t, readfunc: curl_read_callback, seekfunc: curl_seek_callback, freefunc: curl_free_callback, arg: *void) -> CURLcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_mime_data_cb :: (part: *curl_mimepart, datasize: curl_off_t, readfunc: curl_read_callback, seekfunc: curl_seek_callback, freefunc: curl_free_callback, arg: *void) -> CURLcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_mime_encoder :: (part: *curl_mimepart, encoding: *u8) -> CURLcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_mime_encoder :: (part: *curl_mimepart, encoding: *u8) -> CURLcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_mime_filedata :: (part: *curl_mimepart, filename: *u8) -> CURLcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_mime_filedata :: (part: *curl_mimepart, filename: *u8) -> CURLcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_mime_filename :: (part: *curl_mimepart, filename: *u8) -> CURLcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_mime_filename :: (part: *curl_mimepart, filename: *u8) -> CURLcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_mime_free :: (mime: *curl_mime) -> void #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_mime_free :: (mime: *curl_mime) -> void #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_mime_headers :: (part: *curl_mimepart, headers: *curl_slist, take_ownership: s32) -> CURLcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_mime_headers :: (part: *curl_mimepart, headers: *curl_slist, take_ownership: s32) -> CURLcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_mime_init :: (easy: *CURL) -> *curl_mime #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_mime_init :: (easy: *CURL) -> *curl_mime #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_mime_name :: (part: *curl_mimepart, name: *u8) -> CURLcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_mime_name :: (part: *curl_mimepart, name: *u8) -> CURLcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_mime_subparts :: (part: *curl_mimepart, subparts: *curl_mime) -> CURLcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_mime_subparts :: (part: *curl_mimepart, subparts: *curl_mime) -> CURLcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_mime_type :: (part: *curl_mimepart, mimetype: *u8) -> CURLcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_mime_type :: (part: *curl_mimepart, mimetype: *u8) -> CURLcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_mprintf_CFormat :: (format: *u8, __args: ..Any) -> s32 #foreign libcurl "curl_mprintf";
}

#if OS == .WINDOWS {
    curl_mprintf_CFormat :: (format: *u8, __args: ..Any) -> s32 #foreign libcurl "curl_mprintf";
}

#if OS != .WINDOWS {
    curl_msnprintf_CFormat :: (buffer: *u8, maxlength: u64, format: *u8, __args: ..Any) -> s32 #foreign libcurl "curl_msnprintf";
}

#if OS == .WINDOWS {
    curl_msnprintf_CFormat :: (buffer: *u8, maxlength: u64, format: *u8, __args: ..Any) -> s32 #foreign libcurl "curl_msnprintf";
}

#if OS != .WINDOWS {
    curl_msprintf_CFormat :: (buffer: *u8, format: *u8, __args: ..Any) -> s32 #foreign libcurl "curl_msprintf";
}

#if OS == .WINDOWS {
    curl_msprintf_CFormat :: (buffer: *u8, format: *u8, __args: ..Any) -> s32 #foreign libcurl "curl_msprintf";
}

#if OS != .WINDOWS {
    curl_multi_add_handle :: (multi_handle: *CURLM, curl_handle: *CURL) -> CURLMcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_multi_add_handle :: (multi_handle: *CURLM, curl_handle: *CURL) -> CURLMcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_multi_assign :: (multi_handle: *CURLM, sockfd: curl_socket_t, sockp: *void) -> CURLMcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_multi_assign :: (multi_handle: *CURLM, sockfd: curl_socket_t, sockp: *void) -> CURLMcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_multi_cleanup :: (multi_handle: *CURLM) -> CURLMcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_multi_cleanup :: (multi_handle: *CURLM) -> CURLMcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_multi_fdset :: (multi_handle: *CURLM, read_fd_set: *fd_set, write_fd_set: *fd_set, exc_fd_set: *fd_set, max_fd: *s32) -> CURLMcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_multi_fdset :: (multi_handle: *CURLM, read_fd_set: *fd_set, write_fd_set: *fd_set, exc_fd_set: *fd_set, max_fd: *s32) -> CURLMcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_multi_info_read :: (multi_handle: *CURLM, msgs_in_queue: *s32) -> *CURLMsg #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_multi_info_read :: (multi_handle: *CURLM, msgs_in_queue: *s32) -> *CURLMsg #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_multi_init :: () -> *CURLM #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_multi_init :: () -> *CURLM #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_multi_perform :: (multi_handle: *CURLM, running_handles: *s32) -> CURLMcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_multi_perform :: (multi_handle: *CURLM, running_handles: *s32) -> CURLMcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_multi_poll :: (multi_handle: *CURLM, extra_fds: *curl_waitfd, extra_nfds: u32, timeout_ms: s32, ret: *s32) -> CURLMcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_multi_poll :: (multi_handle: *CURLM, extra_fds: *curl_waitfd, extra_nfds: u32, timeout_ms: s32, ret: *s32) -> CURLMcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_multi_remove_handle :: (multi_handle: *CURLM, curl_handle: *CURL) -> CURLMcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_multi_remove_handle :: (multi_handle: *CURLM, curl_handle: *CURL) -> CURLMcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_multi_setopt :: (multi_handle: *CURLM, option: CURLMoption, __args: ..Any) -> CURLMcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_multi_setopt :: (multi_handle: *CURLM, option: CURLMoption, __args: ..Any) -> CURLMcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_multi_socket :: (multi_handle: *CURLM, s: curl_socket_t, running_handles: *s32) -> CURLMcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_multi_socket :: (multi_handle: *CURLM, s: curl_socket_t, running_handles: *s32) -> CURLMcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_multi_socket_action :: (multi_handle: *CURLM, s: curl_socket_t, ev_bitmask: s32, running_handles: *s32) -> CURLMcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_multi_socket_action :: (multi_handle: *CURLM, s: curl_socket_t, ev_bitmask: s32, running_handles: *s32) -> CURLMcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_multi_socket_all :: (multi_handle: *CURLM, running_handles: *s32) -> CURLMcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_multi_socket_all :: (multi_handle: *CURLM, running_handles: *s32) -> CURLMcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_multi_strerror :: (unknown0: CURLMcode) -> *u8 #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_multi_strerror :: (unknown0: CURLMcode) -> *u8 #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_multi_timeout :: (multi_handle: *CURLM, milliseconds: *s64) -> CURLMcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_multi_timeout :: (multi_handle: *CURLM, milliseconds: *s32) -> CURLMcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_multi_wait :: (multi_handle: *CURLM, extra_fds: *curl_waitfd, extra_nfds: u32, timeout_ms: s32, ret: *s32) -> CURLMcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_multi_wait :: (multi_handle: *CURLM, extra_fds: *curl_waitfd, extra_nfds: u32, timeout_ms: s32, ret: *s32) -> CURLMcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_multi_wakeup :: (multi_handle: *CURLM) -> CURLMcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_multi_wakeup :: (multi_handle: *CURLM) -> CURLMcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_pushheader_byname :: (h: *curl_pushheaders, name: *u8) -> *u8 #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_pushheader_byname :: (h: *curl_pushheaders, name: *u8) -> *u8 #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_pushheader_bynum :: (h: *curl_pushheaders, num: u64) -> *u8 #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_pushheader_bynum :: (h: *curl_pushheaders, num: u64) -> *u8 #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_share_cleanup :: (share: *CURLSH) -> CURLSHcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_share_cleanup :: (share: *CURLSH) -> CURLSHcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_share_init :: () -> *CURLSH #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_share_init :: () -> *CURLSH #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_share_setopt :: (share: *CURLSH, option: CURLSHoption, __args: ..Any) -> CURLSHcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_share_setopt :: (share: *CURLSH, option: CURLSHoption, __args: ..Any) -> CURLSHcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_share_strerror :: (unknown0: CURLSHcode) -> *u8 #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_share_strerror :: (unknown0: CURLSHcode) -> *u8 #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_slist_append :: (list: *curl_slist, data: *u8) -> *curl_slist #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_slist_append :: (list: *curl_slist, data: *u8) -> *curl_slist #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_slist_free_all :: (list: *curl_slist) -> void #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_slist_free_all :: (list: *curl_slist) -> void #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_strequal :: (s1: *u8, s2: *u8) -> s32 #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_strequal :: (s1: *u8, s2: *u8) -> s32 #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_strnequal :: (s1: *u8, s2: *u8, n: u64) -> s32 #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_strnequal :: (s1: *u8, s2: *u8, n: u64) -> s32 #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_unescape :: (_string: *u8, length: s32) -> *u8 #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_unescape :: (_string: *u8, length: s32) -> *u8 #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_url :: () -> *CURLU #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_url :: () -> *CURLU #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_url_cleanup :: (handle: *CURLU) -> void #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_url_cleanup :: (handle: *CURLU) -> void #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_url_dup :: (in: *CURLU) -> *CURLU #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_url_dup :: (in: *CURLU) -> *CURLU #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_url_get :: (handle: *CURLU, what: CURLUPart, part: **u8, flags: u32) -> CURLUcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_url_get :: (handle: *CURLU, what: CURLUPart, part: **u8, flags: u32) -> CURLUcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_url_set :: (handle: *CURLU, what: CURLUPart, part: *u8, flags: u32) -> CURLUcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_url_set :: (handle: *CURLU, what: CURLUPart, part: *u8, flags: u32) -> CURLUcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_url_strerror :: (unknown0: CURLUcode) -> *u8 #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_url_strerror :: (unknown0: CURLUcode) -> *u8 #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_version :: () -> *u8 #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_version :: () -> *u8 #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_version_info :: (unknown0: CURLversion) -> *curl_version_info_data #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_version_info :: (unknown0: CURLversion) -> *curl_version_info_data #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_ws_meta :: (curl: *CURL) -> *curl_ws_frame #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_ws_meta :: (curl: *CURL) -> *curl_ws_frame #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_ws_recv :: (curl: *CURL, buffer: *void, buflen: u64, recv: *u64, metap: **curl_ws_frame) -> CURLcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_ws_recv :: (curl: *CURL, buffer: *void, buflen: u64, recv: *u64, metap: **curl_ws_frame) -> CURLcode #foreign libcurl;
}

#if OS != .WINDOWS {
    curl_ws_send :: (curl: *CURL, buffer: *void, buflen: u64, sent: *u64, fragsize: curl_off_t, flags: u32) -> CURLcode #foreign libcurl;
}

#if OS == .WINDOWS {
    curl_ws_send :: (curl: *CURL, buffer: *void, buflen: u64, sent: *u64, fragsize: curl_off_t, flags: u32) -> CURLcode #foreign libcurl;
}

// Convenience operations

#if OS != .WINDOWS {
    curl_maprintf :: (format: string, __args: ..Any) -> *u8 {
        cformat := Native.terminated_copy(format);
        defer Native.release(cformat);
        fixed: [1] Varargs.Argument = .[.{ kind = 6, value = cast(u64) cformat }];
        return Varargs.invoke_pointer(cast(*void) curl_maprintf_CFormat, fixed, __args);
    }
}

#if OS == .WINDOWS {
    curl_maprintf :: (format: string, __args: ..Any) -> *u8 {
        cformat := Native.terminated_copy(format);
        defer Native.release(cformat);
        fixed: [1] Varargs.Argument = .[.{ kind = 6, value = cast(u64) cformat }];
        return Varargs.invoke_pointer(cast(*void) curl_maprintf_CFormat, fixed, __args);
    }
}

#if OS != .WINDOWS {
    curl_mfprintf :: (fd: *FILE, format: string, __args: ..Any) -> s32 {
        cformat := Native.terminated_copy(format);
        defer Native.release(cformat);
        fixed: [2] Varargs.Argument = .[.{ kind = 6, value = cast(u64) fd }, .{ kind = 6, value = cast(u64) cformat }];
        return Varargs.invoke_int(cast(*void) curl_mfprintf_CFormat, fixed, __args);
    }
}

#if OS == .WINDOWS {
    curl_mfprintf :: (fd: *FILE, format: string, __args: ..Any) -> s32 {
        cformat := Native.terminated_copy(format);
        defer Native.release(cformat);
        fixed: [2] Varargs.Argument = .[.{ kind = 6, value = cast(u64) fd }, .{ kind = 6, value = cast(u64) cformat }];
        return Varargs.invoke_int(cast(*void) curl_mfprintf_CFormat, fixed, __args);
    }
}

#if OS != .WINDOWS {
    curl_mprintf :: (format: string, __args: ..Any) -> s32 {
        cformat := Native.terminated_copy(format);
        defer Native.release(cformat);
        fixed: [1] Varargs.Argument = .[.{ kind = 6, value = cast(u64) cformat }];
        return Varargs.invoke_int(cast(*void) curl_mprintf_CFormat, fixed, __args);
    }
}

#if OS == .WINDOWS {
    curl_mprintf :: (format: string, __args: ..Any) -> s32 {
        cformat := Native.terminated_copy(format);
        defer Native.release(cformat);
        fixed: [1] Varargs.Argument = .[.{ kind = 6, value = cast(u64) cformat }];
        return Varargs.invoke_int(cast(*void) curl_mprintf_CFormat, fixed, __args);
    }
}

#if OS != .WINDOWS {
    curl_msnprintf :: (buffer: *u8, maxlength: u64, format: string, __args: ..Any) -> s32 {
        cformat := Native.terminated_copy(format);
        defer Native.release(cformat);
        fixed: [3] Varargs.Argument = .[.{ kind = 6, value = cast(u64) buffer }, .{ kind = 4, value = maxlength }, .{ kind = 6, value = cast(u64) cformat }];
        return Varargs.invoke_int(cast(*void) curl_msnprintf_CFormat, fixed, __args);
    }
}

#if OS == .WINDOWS {
    curl_msnprintf :: (buffer: *u8, maxlength: u64, format: string, __args: ..Any) -> s32 {
        cformat := Native.terminated_copy(format);
        defer Native.release(cformat);
        fixed: [3] Varargs.Argument = .[.{ kind = 6, value = cast(u64) buffer }, .{ kind = 4, value = maxlength }, .{ kind = 6, value = cast(u64) cformat }];
        return Varargs.invoke_int(cast(*void) curl_msnprintf_CFormat, fixed, __args);
    }
}

#if OS != .WINDOWS {
    curl_msprintf :: (buffer: *u8, format: string, __args: ..Any) -> s32 {
        cformat := Native.terminated_copy(format);
        defer Native.release(cformat);
        fixed: [2] Varargs.Argument = .[.{ kind = 6, value = cast(u64) buffer }, .{ kind = 6, value = cast(u64) cformat }];
        return Varargs.invoke_int(cast(*void) curl_msprintf_CFormat, fixed, __args);
    }
}

#if OS == .WINDOWS {
    curl_msprintf :: (buffer: *u8, format: string, __args: ..Any) -> s32 {
        cformat := Native.terminated_copy(format);
        defer Native.release(cformat);
        fixed: [2] Varargs.Argument = .[.{ kind = 6, value = cast(u64) buffer }, .{ kind = 6, value = cast(u64) cformat }];
        return Varargs.invoke_int(cast(*void) curl_msprintf_CFormat, fixed, __args);
    }
}
