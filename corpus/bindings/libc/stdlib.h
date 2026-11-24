/* Declarations only: the subset of the C library stdlib.h that the corpus libraries use. */
#ifndef CORPUS_LIBC_STDLIB_H
#define CORPUS_LIBC_STDLIB_H
#include <stddef.h>
#define NULL ((void *)0)
#define EXIT_SUCCESS 0
#define EXIT_FAILURE 1
void *malloc(size_t size);
void *calloc(size_t count, size_t size);
void *realloc(void *ptr, size_t size);
void free(void *ptr);
void abort(void);
void exit(int status);
int abs(int value);
long labs(long value);
int atoi(const char *s);
long strtol(const char *s, char **end, int base);
unsigned long strtoul(const char *s, char **end, int base);
void qsort(void *base, size_t count, size_t size, int (*compare)(const void *, const void *));
#endif
