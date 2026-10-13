// The settings of a TLS endpoint: who it is (certificate and key), who it trusts (roots), and which
// protocol versions and ALPN protocols it speaks. A config is built once with `tls_config_init`
// and then shared by every connection and every scheduler thread; nothing changes in it afterwards.

Tls_Version :: enum u8 {
    TLS_1_2;
    TLS_1_3;
}

// Decides whether to trust a peer's certificate chain, instead of mbedTLS's own verification. `chain`
// is the peer's certificates in DER, its own first; `server_name` is what the client asked for (empty
// on the server). Returns "" to accept, or why the chain is rejected. It runs on the fiber that is
// doing the handshake, after the handshake itself succeeded, and may wait.
Tls_Verify_Proc :: #type (chain: [][] u8, server_name: string, data: *void) -> string;

// Zero-initialised, a config is a client that verifies servers against the system's roots. Fill in
// the fields you need, then call `tls_config_init`.
Tls_Config :: struct {
    // Server (and, for a client that must prove who it is, client): the certificate chain as PEM,
    // the endpoint's own certificate first, and its private key as PEM (PKCS#8, PKCS#1 or SEC1,
    // not encrypted).
    certificate_chain_pem: string;
    private_key_pem: string;

    // The certificate authorities to trust, as PEM (any number of certificates). Empty: the system's
    // (see docs/stdlib/tls.md for where each OS keeps them).
    roots_pem: string;

    // The protocols to offer (client) or accept (server) through ALPN, most preferred first:
    // `.["h2", "http/1.1"]`. Empty: no ALPN extension. A server that has a list and a client that
    // offers protocols but none in common fails the handshake.
    alpn: [] string;

    // Client: check the server's certificate chain and name. Turning it off makes the connection
    // open to anyone in the middle; it exists for tests and for talking to devices on a private
    // network, and is never the default.
    verify := true;

    // Server: ask the client for a certificate and check it against `roots_pem` (or the system's
    // roots on Linux). Clients without a valid certificate fail the handshake.
    client_auth := false;

    min_version := Tls_Version.TLS_1_2;
    max_version := Tls_Version.TLS_1_3;

    // Replaces mbedTLS's check of the peer's chain (`roots_pem` and the system roots are then not
    // used for verification). The system verifiers of macOS and Windows are this mechanism too.
    verify_proc: Tls_Verify_Proc;
    verify_data: *void;

    // Set by `tls_config_init` (its type is internal to the module): do not touch it, and do not
    // copy a config after that.
    native: *Tls_Native;
}

#scope_module

// What a config builds: one mbedTLS configuration for each side, and the objects they point to.
Tls_Native :: struct {
    client: *void;
    server: *void;
    // Why there is no `client` (no system roots): `tls_connect` returns it.
    client_error: Tls_Error;

    own_chain: *void;
    own_key: *void;
    roots: *void;

    // The ALPN names as C strings and the list mbedTLS holds on to: names, then a null.
    alpn_names: [..] *u8;
    alpn_list: [..] *u8;

    // For a client: checks the chain after the handshake. Null: mbedTLS has checked it (or no check).
    external_verify: Tls_Verify_Proc;
    external_data: *void;
}

#scope_export

// Parses the certificates and keys and builds the mbedTLS configurations. Call it once before the
// first connection; the config must stay where it is and outlive every stream made from it.
// Returns `.CONFIG` with a reason for a certificate or key that does not parse, a version range
// that is empty, or a server certificate without its key. A config with no certificate is a
// client; one with a certificate and key can also accept.
tls_config_init :: (config: *Tls_Config) -> Tls_Error {
    if config.native return .{};

    status := jai_tls_init();

    if status != 0 return make_error(.CONFIG, "mbedTLS could not start", status);

    native := New(Tls_Native);
    error := build_native(config, native);

    if error.kind != .NONE {
        release_native(native);

        return error;
    }

    config.native = native;

    return .{};
}

// Releases what `tls_config_init` built. Streams made from the config must be closed first.
tls_config_free :: (config: *Tls_Config) {
    if !config.native return;

    release_native(config.native);

    config.native = null;
}

// Whether `tls_accept` works with this config: it was set up, and has a certificate and key.
can_accept :: (config: *Tls_Config) -> bool {
    return config.native != null && config.native.server != null;
}

#scope_module

SYSTEM_VERIFIER :: OS == .MACOS || OS == .WINDOWS;

version_number :: (version: Tls_Version) -> s32 {
    return ifx version == .TLS_1_3 then VERSION_TLS_1_3 else VERSION_TLS_1_2;
}

parse_certificates :: (text: string, what: string) -> *void, Tls_Error {
    crt := jai_tls_crt_new();

    if !crt return null, make_error(.CONFIG, "out of memory");

    status := jai_tls_crt_parse(crt, text.data, cast(u64) text.count);

    if status != 0 {
        jai_tls_crt_free(crt);

        if status > 0 return null, make_error(.CONFIG, tprint("% certificates in the % could not be parsed", status, what));

        return null, make_error(.CONFIG, tprint("the %: %", what, library_message(status)), status);
    }

    return crt, .{};
}

build_native :: (config: *Tls_Config, native: *Tls_Native) -> Tls_Error {
    if config.min_version > config.max_version {
        return make_error(.CONFIG, "min_version is above max_version");
    }

    if (config.certificate_chain_pem.count > 0) != (config.private_key_pem.count > 0) {
        return make_error(.CONFIG, "certificate_chain_pem and private_key_pem go together");
    }

    error: Tls_Error;

    if config.certificate_chain_pem.count > 0 {
        native.own_chain, error = parse_certificates(config.certificate_chain_pem, "certificate chain");

        if error.kind != .NONE return error;

        native.own_key = jai_tls_pk_new();

        if !native.own_key return make_error(.CONFIG, "out of memory");

        status := jai_tls_pk_parse(native.own_key, config.private_key_pem.data, cast(u64) config.private_key_pem.count);

        if status != 0 return make_error(.CONFIG, tprint("the private key: %", library_message(status)), status);
    }

    if config.roots_pem.count > 0 {
        native.roots, error = parse_certificates(config.roots_pem, "roots");

        if error.kind != .NONE return error;
    }

    for name: config.alpn {
        if name.count == 0 || name.count > 255 return make_error(.CONFIG, "an ALPN name is empty or longer than 255 bytes");

        array_add(*native.alpn_names, to_c_string(name));
        array_add(*native.alpn_list, native.alpn_names[native.alpn_names.count - 1]);
    }

    if native.alpn_list.count > 0 array_add(*native.alpn_list, null);

    min := version_number(config.min_version);
    max := version_number(config.max_version);

    error = build_client(config, native, min, max);

    if error.kind != .NONE return error;

    if native.own_chain {
        error = build_server(config, native, min, max);

        if error.kind != .NONE return error;
    }

    return .{};
}

build_client :: (config: *Tls_Config, native: *Tls_Native, min: s32, max: s32) -> Tls_Error {
    status: s32;

    conf := jai_tls_conf_new(1, min, max, *status);

    if !conf return make_error(.CONFIG, "the mbedTLS client configuration failed", status);

    if native.own_chain {
        status = mbedtls_ssl_conf_own_cert(conf, native.own_chain, native.own_key);

        if status != 0 {
            jai_tls_conf_free(conf);

            return from_library(.CONFIG, status);
        }
    }

    if !config.verify {
        mbedtls_ssl_conf_authmode(conf, SSL_VERIFY_NONE);
    } else if config.verify_proc {
        mbedtls_ssl_conf_authmode(conf, SSL_VERIFY_OPTIONAL);

        native.external_verify = config.verify_proc;
        native.external_data = config.verify_data;
    } else if native.roots {
        mbedtls_ssl_conf_authmode(conf, SSL_VERIFY_REQUIRED);
        mbedtls_ssl_conf_ca_chain(conf, native.roots, null);
    } else #if SYSTEM_VERIFIER {
        // The operating system's verifier has the last word (see trust_*.jai).
        mbedtls_ssl_conf_authmode(conf, SSL_VERIFY_OPTIONAL);

        native.external_verify = system_verify;
    } else {
        system_roots := jai_tls_system_roots();

        if !system_roots {
            jai_tls_conf_free(conf);

            native.client_error = make_error(.CONFIG, "no system root certificates were found (set SSL_CERT_FILE or SSL_CERT_DIR, or give roots_pem)");

            return .{};
        }

        mbedtls_ssl_conf_authmode(conf, SSL_VERIFY_REQUIRED);
        mbedtls_ssl_conf_ca_chain(conf, system_roots, null);
    }

    if native.alpn_list.count > 0 {
        status = mbedtls_ssl_conf_alpn_protocols(conf, native.alpn_list.data);

        if status != 0 {
            jai_tls_conf_free(conf);

            return from_library(.CONFIG, status);
        }
    }

    native.client = conf;

    return .{};
}

build_server :: (config: *Tls_Config, native: *Tls_Native, min: s32, max: s32) -> Tls_Error {
    status: s32;

    conf := jai_tls_conf_new(0, min, max, *status);

    if !conf return make_error(.CONFIG, "the mbedTLS server configuration failed", status);

    status = mbedtls_ssl_conf_own_cert(conf, native.own_chain, native.own_key);

    if status != 0 {
        jai_tls_conf_free(conf);

        return from_library(.CONFIG, status);
    }

    if config.client_auth {
        trust := native.roots;

        #if !SYSTEM_VERIFIER {
            if !trust trust = jai_tls_system_roots();
        }

        if !trust {
            jai_tls_conf_free(conf);

            return make_error(.CONFIG, "client_auth needs roots_pem to check the clients' certificates against");
        }

        mbedtls_ssl_conf_authmode(conf, SSL_VERIFY_REQUIRED);
        mbedtls_ssl_conf_ca_chain(conf, trust, null);
    } else {
        mbedtls_ssl_conf_authmode(conf, SSL_VERIFY_NONE);
    }

    if native.alpn_list.count > 0 {
        status = mbedtls_ssl_conf_alpn_protocols(conf, native.alpn_list.data);

        if status != 0 {
            jai_tls_conf_free(conf);

            return from_library(.CONFIG, status);
        }
    }

    mbedtls_ssl_conf_sni(conf, sni_callback, null);

    native.server = conf;

    return .{};
}

release_native :: (native: *Tls_Native) {
    if native.client jai_tls_conf_free(native.client);
    if native.server jai_tls_conf_free(native.server);
    if native.own_chain jai_tls_crt_free(native.own_chain);
    if native.own_key jai_tls_pk_free(native.own_key);
    if native.roots jai_tls_crt_free(native.roots);

    for native.alpn_names free(it);

    array_free(native.alpn_names);
    array_free(native.alpn_list);

    free(native);
}
