// Who to trust on macOS: the operating system. The system's root certificates are not available
// to read (`SecTrustCopyAnchorCertificates` leaves out most of them, and the keychain also holds
// per-user and per-domain trust settings), so the connection is made with mbedTLS's own check off
// and the server's chain is handed to Security.framework after the handshake: the same policy
// Safari uses (`SecPolicyCreateSSL`), which checks the chain, the roots, the name and the dates.
// This is what Go does too. It needs macOS 10.14 for `SecTrustEvaluateWithError`.
//
// The evaluation may fetch missing intermediates over the network and so can block the scheduler
// thread for a moment (there is no way to run it asynchronously).

security :: #system_library "Security";
core_foundation :: #system_library "CoreFoundation";

CFTypeRef :: *void;

CF_STRING_ENCODING_UTF8 :: 0x08000100;

CFDataCreate :: (allocator: *void, bytes: *u8, length: s64) -> CFTypeRef #foreign core_foundation;
CFArrayCreate :: (allocator: *void, values: *CFTypeRef, count: s64, callbacks: *void) -> CFTypeRef #foreign core_foundation;
CFStringCreateWithBytes :: (allocator: *void, bytes: *u8, count: s64, encoding: u32, external: u8) -> CFTypeRef #foreign core_foundation;
CFStringGetCString :: (text: CFTypeRef, buffer: *u8, size: s64, encoding: u32) -> u8 #foreign core_foundation;
CFErrorCopyDescription :: (error: CFTypeRef) -> CFTypeRef #foreign core_foundation;
CFRelease :: (value: CFTypeRef) #foreign core_foundation;

SecCertificateCreateWithData :: (allocator: *void, data: CFTypeRef) -> CFTypeRef #foreign security;
SecPolicyCreateSSL :: (server: u8, hostname: CFTypeRef) -> CFTypeRef #foreign security;
SecTrustCreateWithCertificates :: (certificates: CFTypeRef, policies: CFTypeRef, trust: *CFTypeRef) -> s32 #foreign security;
SecTrustEvaluateWithError :: (trust: CFTypeRef, error: *CFTypeRef) -> u8 #foreign security;

// Asks Security.framework whether `chain` (DER, the server's certificate first) is a valid chain
// for `server_name`. "" if it is, else why not.
system_verify :: (chain: [][] u8, server_name: string, data: *void) -> string {
    certificates: [..] CFTypeRef;
    owned: [..] CFTypeRef;

    defer {
        for owned CFRelease(it);

        array_free(certificates);
        array_free(owned);
    }

    for der: chain {
        bytes := CFDataCreate(null, der.data, der.count);

        if !bytes return "out of memory";

        array_add(*owned, bytes);

        certificate := SecCertificateCreateWithData(null, bytes);

        if !certificate return "the server sent a certificate Security.framework cannot read";

        array_add(*owned, certificate);
        array_add(*certificates, certificate);
    }

    list := CFArrayCreate(null, certificates.data, certificates.count, null);

    if !list return "out of memory";

    array_add(*owned, list);

    name: CFTypeRef;

    if server_name.count > 0 {
        name = CFStringCreateWithBytes(null, server_name.data, server_name.count, CF_STRING_ENCODING_UTF8, 0);

        if !name return "out of memory";

        array_add(*owned, name);
    }

    policy := SecPolicyCreateSSL(1, name);

    if !policy return "out of memory";

    array_add(*owned, policy);

    trust: CFTypeRef;
    status := SecTrustCreateWithCertificates(list, policy, *trust);

    if status != 0 || !trust return tprint("Security.framework could not set up the check (OSStatus %)", status);

    array_add(*owned, trust);

    failure: CFTypeRef;

    if SecTrustEvaluateWithError(trust, *failure) != 0 return "";

    if !failure return "the system does not trust the server's certificate";

    // The error is ours to release; its description is a string we read once.
    description := CFErrorCopyDescription(failure);

    defer CFRelease(failure);

    if !description return "the system does not trust the server's certificate";

    defer CFRelease(description);

    buffer: [160] u8;

    if CFStringGetCString(description, buffer.data, buffer.count, CF_STRING_ENCODING_UTF8) == 0 {
        return "the system does not trust the server's certificate";
    }

    return copy_temporary_string(c_text(buffer.data));
}
