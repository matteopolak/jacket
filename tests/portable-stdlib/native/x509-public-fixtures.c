/* Independent public DER fixtures. Ephemeral new keys stay in memory only. */
#include <openssl/crypto.h>
#include <openssl/evp.h>
#include <openssl/rsa.h>
#include <openssl/x509.h>
#include <openssl/x509v3.h>
#include <stdint.h>
#include <stdio.h>

static int word(uint32_t value) {
    unsigned char data[4] = {(unsigned char)value, (unsigned char)(value >> 8),
        (unsigned char)(value >> 16), (unsigned char)(value >> 24)};
    return fwrite(data, 1, sizeof(data), stdout) == sizeof(data);
}

static EVP_PKEY *new_key(int algorithm) {
    EVP_PKEY_CTX *context = EVP_PKEY_CTX_new_id(algorithm, NULL);
    EVP_PKEY *key = NULL;
    if (!context || EVP_PKEY_keygen_init(context) <= 0 ||
        (algorithm == EVP_PKEY_RSA && EVP_PKEY_CTX_set_rsa_keygen_bits(context, 2048) <= 0) ||
        EVP_PKEY_keygen(context, &key) <= 0) {
        EVP_PKEY_free(key);
        key = NULL;
    }
    EVP_PKEY_CTX_free(context);
    return key;
}

static int certificate(EVP_PKEY *key, int version, int extension) {
    X509 *cert = X509_new();
    unsigned char *der = NULL, *spki = NULL;
    BIGNUM *serial_bn = NULL;
    ASN1_INTEGER *serial = NULL;
    BASIC_CONSTRAINTS *constraints = NULL;
    X509_EXTENSION *ext = NULL;
    int ok = 0;
    const unsigned char large_serial[20] = {0x7f, 1, 2, 3, 4, 5, 6, 7, 8, 9,
        10, 11, 12, 13, 14, 15, 16, 17, 18, 19};
    if (!cert || !X509_set_version(cert, version)) goto cleanup;
    serial_bn = BN_bin2bn(large_serial, sizeof(large_serial), NULL);
    if (!serial_bn || !(serial = BN_to_ASN1_INTEGER(serial_bn, NULL)) ||
        !X509_set_serialNumber(cert, serial) || !X509_set_pubkey(cert, key)) goto cleanup;
    X509_NAME *subject = X509_get_subject_name(cert);
    if (!subject || !X509_NAME_add_entry_by_txt(subject, "CN", MBSTRING_ASC,
        (const unsigned char *)"Independent public fixture", -1, -1, 0) ||
        !X509_set_issuer_name(cert, subject) ||
        !ASN1_TIME_set_string(X509_getm_notBefore(cert), "250101000000Z") ||
        !ASN1_TIME_set_string(X509_getm_notAfter(cert), "300101000000Z")) goto cleanup;
    if (extension) {
        constraints = BASIC_CONSTRAINTS_new();
        if (!constraints) goto cleanup;
        constraints->ca = 0xff;
        ext = X509V3_EXT_i2d(NID_basic_constraints, 1, constraints);
        if (!ext || !X509_add_ext(cert, ext, -1)) goto cleanup;
    }
    const EVP_MD *digest = EVP_PKEY_base_id(key) == EVP_PKEY_ED25519 ? NULL : EVP_sha256();
    if (X509_sign(cert, key, digest) <= 0 || X509_verify(cert, key) != 1) goto cleanup;
    int der_length = i2d_X509(cert, &der);
    int spki_length = i2d_X509_PUBKEY(X509_get_X509_PUBKEY(cert), &spki);
    if (der_length <= 0 || spki_length <= 0 || !word((uint32_t)der_length) ||
        fwrite(der, 1, (size_t)der_length, stdout) != (size_t)der_length ||
        !word((uint32_t)spki_length) || fwrite(spki, 1, (size_t)spki_length, stdout) != (size_t)spki_length) goto cleanup;
    ok = 1;
cleanup:
    X509_EXTENSION_free(ext);
    BASIC_CONSTRAINTS_free(constraints);
    ASN1_INTEGER_free(serial);
    BN_free(serial_bn);
    OPENSSL_free(der);
    OPENSSL_free(spki);
    X509_free(cert);
    return ok;
}

int main(void) {
    if (!OPENSSL_init_crypto(OPENSSL_INIT_NO_LOAD_CONFIG, NULL)) return 1;
    EVP_PKEY *edwards = new_key(EVP_PKEY_ED25519);
    EVP_PKEY *rsa = new_key(EVP_PKEY_RSA);
    int ok = edwards && rsa && word(4);
    for (int version = 0; ok && version < 3; ++version) ok = certificate(edwards, version, version == 2);
    if (ok) ok = certificate(rsa, 2, 1);
    EVP_PKEY_free(edwards);
    EVP_PKEY_free(rsa);
    if (!ok) fputs("native public certificate fixture construction failed\n", stderr);
    return ok && fflush(stdout) == 0 ? 0 : 2;
}
