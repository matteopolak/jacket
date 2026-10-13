/* mbedTLS 3.6 configuration on top of the library's defaults (MBEDTLS_USER_CONFIG_FILE), for
 * stdlib/Extensions/Tls. The defaults already give TLS 1.2 and 1.3, ALPN, SNI, PSA, X25519, P-256,
 * P-384, ECDSA, RSA (PKCS#1 v1.5 and PSS), AES-GCM, ChaCha20-Poly1305, SHA-256 and SHA-384, PEM and
 * X.509. This file only adds threading and drops what the module does not use. */
#ifndef JAI_MBEDTLS_CONFIG_H
#define JAI_MBEDTLS_CONFIG_H

/* PSA (which TLS 1.3 needs), the ticket keys and the shared random generator are used from every
 * scheduler thread: each connection has a context of its own, but the library's global state is
 * locked. Windows has no pthreads: tools/native/mbedtls/threading_alt.h and the glue provide the
 * mutex. */
#define MBEDTLS_THREADING_C
#if defined(_WIN32)
#define MBEDTLS_THREADING_ALT
#else
#define MBEDTLS_THREADING_PTHREAD
#endif

/* The record buffers are the memory of an idle connection: the input buffer stays at the full 16 KiB
 * a peer may send, the output buffer holds one record we send (and one whole handshake message,
 * which mbedTLS does not fragment: a certificate chain larger than this cannot be sent). */
#define MBEDTLS_SSL_OUT_CONTENT_LEN 8192

/* No persistent PSA keys, so no storage for them. */
#undef MBEDTLS_PSA_CRYPTO_STORAGE_C
#undef MBEDTLS_PSA_ITS_FILE_C

/* Sockets and timers are the Io module's. */
#undef MBEDTLS_NET_C
#undef MBEDTLS_TIMING_C

/* The ARM crypto instruction paths need compiler support the Windows toolchains do not all have. */
#if defined(_WIN32) && (defined(_M_ARM64) || defined(__aarch64__))
#undef MBEDTLS_AESCE_C
#undef MBEDTLS_SHA256_USE_A64_CRYPTO_IF_PRESENT
#undef MBEDTLS_SHA512_USE_A64_CRYPTO_IF_PRESENT
#endif

#endif
