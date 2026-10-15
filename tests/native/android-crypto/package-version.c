/* Public API runtime identity: verify the packaged archive and dylib agree. */
#include <stdio.h>
#include <openssl/crypto.h>

int main(void) {
    if (OPENSSL_version_major() != 3) return 1;
    puts(OpenSSL_version(OPENSSL_VERSION));
    return 0;
}
