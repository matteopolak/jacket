// Who to trust on Windows: the operating system. The root store is not read (a root the system
// would fetch on demand through Windows Update is not in it yet), so the connection is made with
// mbedTLS's own check off, and after the handshake the server's chain goes to CryptoAPI: build the
// chain with `CertGetCertificateChain` and check it with `CertVerifyCertificateChainPolicy` and
// the SSL policy for the server name. This is what Go does too. Revocation is not checked.
//
// Windows may download missing intermediates or roots while it builds the chain, so this can block
// the scheduler thread for a moment (there is no way to run it asynchronously).

crypt32 :: #system_library "crypt32";

X509_ASN_ENCODING :: 0x00000001;
CERT_STORE_PROV_MEMORY :: 2;
CERT_STORE_ADD_ALWAYS :: 4;
CERT_CHAIN_CACHE_END_CERT :: 0x00000001;
CERT_CHAIN_POLICY_SSL :: 4;
USAGE_MATCH_TYPE_AND :: 0;
AUTHTYPE_SERVER :: 2;

Cert_Enhkey_Usage :: struct {
    count: u32;
    identifiers: **u8;
}

Cert_Usage_Match :: struct {
    type: u32;
    usage: Cert_Enhkey_Usage;
}

// CERT_CHAIN_PARA up to `pftCacheResync`, the part every Windows version since Vista reads.
Cert_Chain_Para :: struct {
    size: u32;
    requested_usage: Cert_Usage_Match;
    requested_issuance_policy: Cert_Usage_Match;
    url_retrieval_timeout: u32;
    check_revocation_freshness_time: s32;
    revocation_freshness_time: u32;
    cache_resync: *void;
}

Cert_Chain_Policy_Para :: struct {
    size: u32;
    flags: u32;
    extra: *void;
}

// HTTPSPolicyCallbackData
Https_Policy_Data :: struct {
    size: u32;
    auth_type: u32;
    checks: u32;
    server_name: *u16;
}

Cert_Chain_Policy_Status :: struct {
    size: u32;
    error: u32;
    chain_index: s32;
    element_index: s32;
    extra: *void;
}

CertOpenStore :: (provider: *u8, encoding: u32, crypto_provider: *void, flags: u32, parameter: *void) -> *void #foreign crypt32;
CertCloseStore :: (store: *void, flags: u32) -> s32 #foreign crypt32;
CertCreateCertificateContext :: (encoding: u32, encoded: *u8, length: u32) -> *void #foreign crypt32;
CertAddCertificateContextToStore :: (store: *void, certificate: *void, disposition: u32, result: **void) -> s32 #foreign crypt32;
CertFreeCertificateContext :: (certificate: *void) -> s32 #foreign crypt32;
CertGetCertificateChain :: (engine: *void, certificate: *void, time: *void, store: *void, parameters: *Cert_Chain_Para, flags: u32, reserved: *void, chain: **void) -> s32 #foreign crypt32;
CertFreeCertificateChain :: (chain: *void) #foreign crypt32;
CertVerifyCertificateChainPolicy :: (policy: *u8, chain: *void, parameters: *Cert_Chain_Policy_Para, status: *Cert_Chain_Policy_Status) -> s32 #foreign crypt32;

// Asks CryptoAPI whether `chain` (DER, the server's certificate first) is a valid chain for
// `server_name`. "" if it is, else why not.
system_verify :: (chain: [][] u8, server_name: string, data: *void) -> string {
    store := CertOpenStore(cast(*u8) CERT_STORE_PROV_MEMORY, 0, null, 0, null);

    if !store return "CertOpenStore failed";

    defer CertCloseStore(store, 0);

    leaf: *void;

    defer if leaf CertFreeCertificateContext(leaf);

    // Every certificate the server sent goes in the store, so the intermediates are found.
    for der, index: chain {
        certificate := CertCreateCertificateContext(X509_ASN_ENCODING, der.data, cast(u32) der.count);

        if !certificate return "the server sent a certificate Windows cannot read";

        if index == 0 {
            leaf = certificate;
        } else {
            CertAddCertificateContextToStore(store, certificate, CERT_STORE_ADD_ALWAYS, null);
            CertFreeCertificateContext(certificate);
        }
    }

    // The server's identity, as the TLS server authentication usage.
    server_auth := "1.3.6.1.5.5.7.3.1\0";
    usages: [1] *u8;

    usages[0] = server_auth.data;

    parameters: Cert_Chain_Para;

    parameters.size = size_of(Cert_Chain_Para);
    parameters.requested_usage.type = USAGE_MATCH_TYPE_AND;
    parameters.requested_usage.usage.count = 1;
    parameters.requested_usage.usage.identifiers = usages.data;

    chain_context: *void;

    if CertGetCertificateChain(null, leaf, null, store, *parameters, CERT_CHAIN_CACHE_END_CERT, null, *chain_context) == 0 {
        return "Windows could not build a certificate chain";
    }

    defer CertFreeCertificateChain(chain_context);

    // The name is UTF-16 and ends in a zero. Host names are ASCII (internationalised ones arrive
    // as punycode).
    // Preserve temporary-storage ownership through Basic's public scratch API. The
    // buffer is only borrowed by the synchronous policy check below.
    if server_name.count < 0 || server_name.count > 0x7fffffffffffffff / size_of(u16) - 1 {
        return "the TLS server name is too long";
    }

    name := cast(*u16) talloc((server_name.count + 1) * size_of(u16));

    if !name return "could not allocate the TLS server name";

    for 0..server_name.count - 1 name[it] = server_name[it];

    name[server_name.count] = 0;

    https: Https_Policy_Data;

    https.size = size_of(Https_Policy_Data);
    https.auth_type = AUTHTYPE_SERVER;
    https.server_name = ifx server_name.count > 0 then name else null;

    policy: Cert_Chain_Policy_Para;

    policy.size = size_of(Cert_Chain_Policy_Para);
    policy.extra = *https;

    status: Cert_Chain_Policy_Status;

    status.size = size_of(Cert_Chain_Policy_Status);

    if CertVerifyCertificateChainPolicy(cast(*u8) CERT_CHAIN_POLICY_SSL, chain_context, *policy, *status) == 0 {
        return "Windows could not check the certificate chain";
    }

    if status.error == 0 return "";

    return windows_verify_message(status.error);
}

windows_verify_message :: (code: u32) -> string {
    if code == {
        case 0x800B0101; return "the certificate has expired or is not yet valid";
        case 0x800B0109; return "the certificate chain ends in a root that is not trusted";
        case 0x800B010A; return "the certificate chain could not be built to a trusted root";
        case 0x800B010F; return "the certificate is not valid for the server name";
        case 0x800B0110; return "the certificate is not for server authentication";
        case 0x800B0106; return "the certificate is not for server authentication";
        case 0x800B010C; return "the certificate has been revoked";
        case 0x80092010; return "the certificate has been revoked";
        case 0x80096004; return "the certificate's signature is not valid";
    }

    return tprint("Windows rejected the certificate chain (0x%)", FormatInt.{ value = code, base = 16 });
}
