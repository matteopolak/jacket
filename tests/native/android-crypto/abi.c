/* Public OpenSSL headers only: check the exact C ABI used by the Jai binding. */
#include <stdint.h>
#include <stddef.h>
#include <openssl/evp.h>
#include <openssl/x509.h>
#include <openssl/err.h>

_Static_assert(sizeof(size_t) == sizeof(uint64_t), "requires 64-bit size_t");
_Static_assert(sizeof(int) == sizeof(int32_t), "requires 32-bit int");
#ifdef _WIN32
_Static_assert(sizeof(long) == sizeof(int32_t), "Windows C long");
#else
_Static_assert(sizeof(long) == sizeof(int64_t), "POSIX C long");
#endif

static int callback(const char *bytes, size_t size, void *user) {
    (void)bytes;
    (void)size;
    (void)user;
    return 1;
}

int main(void) {
    EVP_PKEY *(*private_key)(EVP_PKEY **, const unsigned char **, long) = d2i_AutoPrivateKey;
    EVP_PKEY *(*public_key)(EVP_PKEY **, const unsigned char **, long) = d2i_PUBKEY;
    int (*key_is_a)(const EVP_PKEY *, const char *) = EVP_PKEY_is_a;
    int (*sign_init)(EVP_MD_CTX *, EVP_PKEY_CTX **, const EVP_MD *, ENGINE *, EVP_PKEY *) = EVP_DigestSignInit;
    int (*sign_update)(EVP_MD_CTX *, const void *, size_t) = EVP_DigestSignUpdate;
    int (*sign_final)(EVP_MD_CTX *, unsigned char *, size_t *) = EVP_DigestSignFinal;
    int (*verify_init)(EVP_MD_CTX *, EVP_PKEY_CTX **, const EVP_MD *, ENGINE *, EVP_PKEY *) = EVP_DigestVerifyInit;
    int (*verify_update)(EVP_MD_CTX *, const void *, size_t) = EVP_DigestVerifyUpdate;
    int (*verify_final)(EVP_MD_CTX *, const unsigned char *, size_t) = EVP_DigestVerifyFinal;
    void (*drain)(int (*)(const char *, size_t, void *), void *) = ERR_print_errors_cb;
    (void)private_key;
    (void)public_key;
    (void)key_is_a;
    (void)sign_init;
    (void)sign_update;
    (void)sign_final;
    (void)verify_init;
    (void)verify_update;
    (void)verify_final;
    EVP_MD_CTX *ctx = EVP_MD_CTX_new();
    if (!ctx || !EVP_sha256()) return 1;
    EVP_MD_CTX_free(ctx);
    EVP_PKEY_free(NULL);
    drain(callback, NULL);
    return 0;
}
