#ifndef PORTABLE_CLIPBOARD_DATA_H
#define PORTABLE_CLIPBOARD_DATA_H
#include <stddef.h>
#include <stdint.h>
int pcb_utf8(const unsigned char *text, size_t size);
unsigned char *pcb_bitmap(int32_t width, int32_t height,
                          const unsigned char *pixels, int32_t channels,
                          int flip, size_t *size);
#endif
