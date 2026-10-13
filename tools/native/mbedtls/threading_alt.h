/* The mutex type of MBEDTLS_THREADING_ALT on Windows (see jai_mbedtls.c for the functions). */
#ifndef JAI_MBEDTLS_THREADING_ALT_H
#define JAI_MBEDTLS_THREADING_ALT_H

#include <windows.h>

typedef struct mbedtls_threading_mutex_t {
    CRITICAL_SECTION section;
    char is_valid;
} mbedtls_threading_mutex_t;

#endif
