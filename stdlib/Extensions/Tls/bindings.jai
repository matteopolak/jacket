// The part of mbedTLS 3.6 (and the glue in tools/native/mbedtls/jai_mbedtls.c) this module calls,
// written by hand: every object is an opaque pointer, so no layout of the library's structures is
// needed here. None of this is exported; users see `Tls_Config` and `Tls_Stream`.

libmbedtls :: #system_library "mbedtls";

#if OS == .WINDOWS {
    // mbedTLS's entropy source on Windows (BCryptGenRandom).
    #library,system,link_always "bcrypt";
    // x509 name matching parses addresses with inet_pton.
    #library,system,link_always "ws2_32";
}

// What the library returns, by name (all negative).
ERR_SSL_WANT_READ :: -0x6900;
ERR_SSL_WANT_WRITE :: -0x6880;
ERR_SSL_PEER_CLOSE_NOTIFY :: -0x7880;
ERR_SSL_CONN_EOF :: -0x7280;
ERR_SSL_RECEIVED_NEW_SESSION_TICKET :: -0x7B00;
ERR_SSL_ALLOC_FAILED :: -0x7F00;
ERR_X509_CERT_VERIFY_FAILED :: -0x2700;
// The library's own network codes (its sockets are not compiled in): what the BIO callbacks return
// when the Io call under them failed. The stream keeps the `Io_Error`.
ERR_NET_SEND_FAILED :: -0x004E;
ERR_NET_RECV_FAILED :: -0x004C;

SSL_VERIFY_NONE :: 0;
SSL_VERIFY_OPTIONAL :: 1;
SSL_VERIFY_REQUIRED :: 2;

// mbedtls_x509_crt_verify's result bits.
X509_BADCERT_EXPIRED :: 0x01;
X509_BADCERT_REVOKED :: 0x02;
X509_BADCERT_CN_MISMATCH :: 0x04;
X509_BADCERT_NOT_TRUSTED :: 0x08;
X509_BADCERT_FUTURE :: 0x200;

VERSION_TLS_1_2 :: 0x0303;
VERSION_TLS_1_3 :: 0x0304;

Bio_Send :: #type (context_data: *void, buffer: *u8, length: u64) -> s32 #c_call;
Bio_Receive :: #type (context_data: *void, buffer: *u8, length: u64) -> s32 #c_call;
Sni_Callback :: #type (data: *void, ssl: *void, name: *u8, length: u64) -> s32 #c_call;

jai_tls_init :: () -> s32 #foreign libmbedtls;
jai_tls_ssl_new :: () -> *void #foreign libmbedtls;
jai_tls_ssl_free :: (ssl: *void) #foreign libmbedtls;
jai_tls_ssl_set_user :: (ssl: *void, user: *void) #foreign libmbedtls;
jai_tls_ssl_user :: (ssl: *void) -> *void #foreign libmbedtls;
jai_tls_conf_new :: (is_client: s32, min_version: s32, max_version: s32, status: *s32) -> *void #foreign libmbedtls;
jai_tls_conf_free :: (conf: *void) #foreign libmbedtls;
jai_tls_crt_new :: () -> *void #foreign libmbedtls;
jai_tls_crt_free :: (crt: *void) #foreign libmbedtls;
jai_tls_crt_parse :: (crt: *void, data: *u8, length: u64) -> s32 #foreign libmbedtls;
jai_tls_crt_der :: (crt: *void, length: *u64) -> *u8 #foreign libmbedtls;
jai_tls_crt_next :: (crt: *void) -> *void #foreign libmbedtls;
jai_tls_pk_new :: () -> *void #foreign libmbedtls;
jai_tls_pk_free :: (pk: *void) #foreign libmbedtls;
jai_tls_pk_parse :: (pk: *void, data: *u8, length: u64) -> s32 #foreign libmbedtls;
jai_tls_system_roots :: () -> *void #foreign libmbedtls;

mbedtls_ssl_conf_own_cert :: (conf: *void, own_cert: *void, pk_key: *void) -> s32 #foreign libmbedtls;
mbedtls_ssl_conf_ca_chain :: (conf: *void, ca_chain: *void, ca_crl: *void) #foreign libmbedtls;
mbedtls_ssl_conf_authmode :: (conf: *void, authmode: s32) #foreign libmbedtls;
mbedtls_ssl_conf_alpn_protocols :: (conf: *void, protocols: **u8) -> s32 #foreign libmbedtls;
mbedtls_ssl_conf_sni :: (conf: *void, callback: Sni_Callback, data: *void) #foreign libmbedtls;

mbedtls_ssl_setup :: (ssl: *void, conf: *void) -> s32 #foreign libmbedtls;
mbedtls_ssl_set_bio :: (ssl: *void, context_data: *void, send: Bio_Send, receive: Bio_Receive, receive_timeout: *void) #foreign libmbedtls;
mbedtls_ssl_set_hostname :: (ssl: *void, hostname: *u8) -> s32 #foreign libmbedtls;
mbedtls_ssl_handshake :: (ssl: *void) -> s32 #foreign libmbedtls;
mbedtls_ssl_read :: (ssl: *void, buffer: *u8, length: u64) -> s32 #foreign libmbedtls;
mbedtls_ssl_write :: (ssl: *void, buffer: *u8, length: u64) -> s32 #foreign libmbedtls;
mbedtls_ssl_close_notify :: (ssl: *void) -> s32 #foreign libmbedtls;
mbedtls_ssl_check_pending :: (ssl: *void) -> s32 #foreign libmbedtls;
mbedtls_ssl_get_bytes_avail :: (ssl: *void) -> u64 #foreign libmbedtls;
mbedtls_ssl_get_alpn_protocol :: (ssl: *void) -> *u8 #foreign libmbedtls;
mbedtls_ssl_get_version :: (ssl: *void) -> *u8 #foreign libmbedtls;
mbedtls_ssl_get_ciphersuite :: (ssl: *void) -> *u8 #foreign libmbedtls;
mbedtls_ssl_get_verify_result :: (ssl: *void) -> u32 #foreign libmbedtls;
mbedtls_ssl_get_peer_cert :: (ssl: *void) -> *void #foreign libmbedtls;

mbedtls_x509_crt_verify_info :: (buffer: *u8, size: u64, prefix: *u8, flags: u32) -> s32 #foreign libmbedtls;
mbedtls_strerror :: (code: s32, buffer: *u8, size: u64) #foreign libmbedtls;
