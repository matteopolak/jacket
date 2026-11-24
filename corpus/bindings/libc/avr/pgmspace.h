/* Declarations only: the program-memory macros of avr-libc that nanopb reads on AVR. */
#ifndef CORPUS_LIBC_AVR_PGMSPACE_H
#define CORPUS_LIBC_AVR_PGMSPACE_H
#include <stdint.h>
#define PROGMEM __attribute__((__progmem__))
#define PSTR(s) (s)
#define pgm_read_byte(addr) (*(const unsigned char *)(addr))
#define pgm_read_word(addr) (*(const unsigned short *)(addr))
#define pgm_read_dword(addr) (*(const unsigned long *)(addr))
#define pgm_read_ptr(addr) (*(const void *const *)(addr))
#endif
