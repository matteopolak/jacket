/* Glue between stdlib/Extensions/Tls and mbedTLS: what is easier to write in C than to bind. It
 * allocates the library's opaque objects (the Jai side never needs their sizes), starts the library
 * once, reads the system's root certificates and reads fields of structures that are private to
 * the library. Everything else in the module calls mbedTLS directly. */
#include "mbedtls/ssl.h"
#include "mbedtls/x509_crt.h"
#include "mbedtls/pk.h"
#include "mbedtls/psa_util.h"
#include "mbedtls/threading.h"
#include "psa/crypto.h"

#include <stdlib.h>
#include <string.h>

#if defined(_WIN32)
#include <windows.h>
#else
#include <pthread.h>
#endif

#if defined(MBEDTLS_THREADING_ALT)
static void jai_mutex_init(mbedtls_threading_mutex_t *mutex) {
    InitializeCriticalSection(&mutex->section);
    mutex->is_valid = 1;
}

static void jai_mutex_free(mbedtls_threading_mutex_t *mutex) {
    if (mutex->is_valid) {
        DeleteCriticalSection(&mutex->section);
        mutex->is_valid = 0;
    }
}

static int jai_mutex_lock(mbedtls_threading_mutex_t *mutex) {
    if (!mutex->is_valid) return MBEDTLS_ERR_THREADING_BAD_INPUT_DATA;
    EnterCriticalSection(&mutex->section);
    return 0;
}

static int jai_mutex_unlock(mbedtls_threading_mutex_t *mutex) {
    if (!mutex->is_valid) return MBEDTLS_ERR_THREADING_BAD_INPUT_DATA;
    LeaveCriticalSection(&mutex->section);
    return 0;
}
#endif

/* Runs `function` once per process, from whichever thread gets here first; the others wait. */
#if defined(_WIN32)
typedef INIT_ONCE jai_once_t;
#define JAI_ONCE_INIT INIT_ONCE_STATIC_INIT
static BOOL CALLBACK jai_once_trampoline(PINIT_ONCE once, PVOID parameter, PVOID *context) {
    ((void (*)(void)) parameter)();
    return TRUE;
}
static void jai_once(jai_once_t *once, void (*function)(void)) {
    InitOnceExecuteOnce(once, jai_once_trampoline, (PVOID) function, NULL);
}
#else
typedef pthread_once_t jai_once_t;
#define JAI_ONCE_INIT PTHREAD_ONCE_INIT
static void jai_once(jai_once_t *once, void (*function)(void)) {
    pthread_once(once, function);
}
#endif

static jai_once_t library_once = JAI_ONCE_INIT;
static int library_status = -1;

static void library_start(void) {
#if defined(MBEDTLS_THREADING_ALT)
    mbedtls_threading_set_alt(jai_mutex_init, jai_mutex_free, jai_mutex_lock, jai_mutex_unlock);
#endif
    library_status = (int) psa_crypto_init();
}

/* Starts mbedTLS (and PSA, which TLS 1.3 needs). Safe from any thread, any number of times: 0 once
 * it is ready, the PSA status otherwise. */
int jai_tls_init(void) {
    jai_once(&library_once, library_start);
    return library_status;
}

/* The random generator every connection and key parse uses: PSA's, which is locked inside. */
int jai_tls_random(void *state, unsigned char *output, size_t length) {
    return mbedtls_psa_get_random(state, output, length);
}

void *jai_tls_ssl_new(void) {
    mbedtls_ssl_context *ssl = calloc(1, sizeof *ssl);
    if (ssl) mbedtls_ssl_init(ssl);
    return ssl;
}

void jai_tls_ssl_free(void *ssl) {
    if (!ssl) return;
    mbedtls_ssl_free(ssl);
    free(ssl);
}

/* A configuration for a client or a server with the library's defaults, the shared random
 * generator and the protocol versions (0x0303 is TLS 1.2, 0x0304 TLS 1.3). */
void *jai_tls_conf_new(int is_client, int min_version, int max_version, int *status) {
    mbedtls_ssl_config *conf = calloc(1, sizeof *conf);
    if (!conf) {
        *status = MBEDTLS_ERR_SSL_ALLOC_FAILED;
        return NULL;
    }
    mbedtls_ssl_config_init(conf);
    *status = mbedtls_ssl_config_defaults(conf, is_client ? MBEDTLS_SSL_IS_CLIENT : MBEDTLS_SSL_IS_SERVER,
                                          MBEDTLS_SSL_TRANSPORT_STREAM, MBEDTLS_SSL_PRESET_DEFAULT);
    if (*status != 0) {
        mbedtls_ssl_config_free(conf);
        free(conf);
        return NULL;
    }
    mbedtls_ssl_conf_rng(conf, jai_tls_random, NULL);
    mbedtls_ssl_conf_min_tls_version(conf, (mbedtls_ssl_protocol_version) min_version);
    mbedtls_ssl_conf_max_tls_version(conf, (mbedtls_ssl_protocol_version) max_version);
    return conf;
}

void jai_tls_conf_free(void *conf) {
    if (!conf) return;
    mbedtls_ssl_config_free(conf);
    free(conf);
}

void *jai_tls_crt_new(void) {
    mbedtls_x509_crt *crt = calloc(1, sizeof *crt);
    if (crt) mbedtls_x509_crt_init(crt);
    return crt;
}

void jai_tls_crt_free(void *crt) {
    if (!crt) return;
    mbedtls_x509_crt_free(crt);
    free(crt);
}

void *jai_tls_pk_new(void) {
    mbedtls_pk_context *pk = calloc(1, sizeof *pk);
    if (pk) mbedtls_pk_init(pk);
    return pk;
}

void jai_tls_pk_free(void *pk) {
    if (!pk) return;
    mbedtls_pk_free(pk);
    free(pk);
}

/* The PEM or DER parsers want PEM text to end in a NUL that is counted: this adds one. For a
 * chain it returns 0 when every certificate parsed, else a negative error (nothing parsed and the
 * first failure) or the count of certificates that did not (the rest are in `crt`). */
int jai_tls_crt_parse(void *crt, const unsigned char *data, size_t length) {
    unsigned char *copy = malloc(length + 1);
    if (!copy) return MBEDTLS_ERR_X509_ALLOC_FAILED;
    memcpy(copy, data, length);
    copy[length] = 0;
    int status = mbedtls_x509_crt_parse(crt, copy, length + 1);
    free(copy);
    return status;
}

int jai_tls_pk_parse(void *pk, const unsigned char *data, size_t length) {
    unsigned char *copy = malloc(length + 1);
    if (!copy) return MBEDTLS_ERR_PK_ALLOC_FAILED;
    memcpy(copy, data, length);
    copy[length] = 0;
    int status = mbedtls_pk_parse_key(pk, copy, length + 1, NULL, 0, jai_tls_random, NULL);
    mbedtls_platform_zeroize(copy, length + 1);
    free(copy);
    return status;
}

/* The DER of a certificate and the next one of a chain (NULL at the end). */
const unsigned char *jai_tls_crt_der(const void *crt, size_t *length) {
    const mbedtls_x509_crt *c = crt;
    *length = c->raw.len;
    return c->raw.p;
}

const void *jai_tls_crt_next(const void *crt) {
    return ((const mbedtls_x509_crt *) crt)->next;
}

/* System root certificates, where mbedTLS verifies against them (not on macOS and Windows, where
 * the operating system's verifier does; see stdlib/Extensions/Tls/system_trust.jai). */
#if !defined(_WIN32) && !defined(__APPLE__)
static const char *const bundle_files[] = {
    "/etc/ssl/certs/ca-certificates.crt",
    "/etc/pki/tls/certs/ca-bundle.crt",
    "/etc/ssl/ca-bundle.pem",
    "/etc/pki/tls/cacert.pem",
    "/etc/pki/ca-trust/extracted/pem/tls-ca-bundle.pem",
    "/etc/ssl/cert.pem",
};

static jai_once_t roots_once = JAI_ONCE_INIT;
static mbedtls_x509_crt system_roots;

static int count_roots(void) {
    int count = 0;
    for (const mbedtls_x509_crt *c = &system_roots; c && c->version != 0; c = c->next) count++;
    return count;
}

/* Adds the certificates of a bundle file. A certificate that does not parse is skipped (the
 * library keeps the others); true when the file added any. */
static int add_file(const char *path) {
    int before = count_roots();
    mbedtls_x509_crt_parse_file(&system_roots, path);
    return count_roots() > before;
}

static void add_directory(const char *path) {
    mbedtls_x509_crt_parse_path(&system_roots, path);
}

static void roots_load(void) {
    mbedtls_x509_crt_init(&system_roots);
    const char *file = getenv("SSL_CERT_FILE");
    int loaded = 0;
    if (file && *file) loaded = add_file(file);
    if (!loaded) {
        for (size_t i = 0; i < sizeof bundle_files / sizeof *bundle_files && !loaded; i++) {
            loaded = add_file(bundle_files[i]);
        }
    }
    /* SSL_CERT_DIR is a colon-separated list of directories of certificate files. */
    const char *directories = getenv("SSL_CERT_DIR");
    while (directories && *directories) {
        const char *end = strchr(directories, ':');
        size_t length = end ? (size_t) (end - directories) : strlen(directories);
        char path[4096];
        if (length > 0 && length < sizeof path) {
            memcpy(path, directories, length);
            path[length] = 0;
            add_directory(path);
        }
        directories += length + (end ? 1 : 0);
    }
}

/* The parsed system roots, loaded on first use and shared (read-only) afterwards; NULL when the
 * system has none. */
const void *jai_tls_system_roots(void) {
    jai_once(&roots_once, roots_load);
    return count_roots() > 0 ? &system_roots : NULL;
}
#else
const void *jai_tls_system_roots(void) {
    return NULL;
}
#endif
