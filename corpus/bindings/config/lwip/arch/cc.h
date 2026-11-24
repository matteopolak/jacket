/* Corpus compiler and platform hooks for lwIP's arch port. */
#ifndef LWIP_ARCH_CC_H
#define LWIP_ARCH_CC_H
#include <stdint.h>
#include <stdio.h>
#define LWIP_NO_STDDEF_H 0
#define LWIP_NO_INTTYPES_H 0
#define LWIP_NO_LIMITS_H 0
#define LWIP_PLATFORM_DIAG(x) do { (void)0; } while (0)
#define LWIP_PLATFORM_ASSERT(x) do { (void)0; } while (0)
#define LWIP_RAND() 0u
#define BYTE_ORDER LITTLE_ENDIAN
#define PACK_STRUCT_FIELD(x) x
#define PACK_STRUCT_BEGIN
#define PACK_STRUCT_END
#endif
