/* Declarations only: the subset of the C library stdio.h that the corpus libraries use. */
#ifndef CORPUS_LIBC_STDIO_H
#define CORPUS_LIBC_STDIO_H
#include <stddef.h>
#include <stdarg.h>
typedef struct _corpus_FILE FILE;
#define EOF (-1)
#define BUFSIZ 8192
#define SEEK_SET 0
#define SEEK_CUR 1
#define SEEK_END 2
extern FILE *stdin;
extern FILE *stdout;
extern FILE *stderr;
int printf(const char *format, ...);
int fprintf(FILE *stream, const char *format, ...);
int sprintf(char *buffer, const char *format, ...);
int snprintf(char *buffer, size_t size, const char *format, ...);
int vprintf(const char *format, va_list args);
int vfprintf(FILE *stream, const char *format, va_list args);
int vsnprintf(char *buffer, size_t size, const char *format, va_list args);
int puts(const char *s);
int putchar(int c);
int fputc(int c, FILE *stream);
int fflush(FILE *stream);
FILE *fopen(const char *path, const char *mode);
int fclose(FILE *stream);
size_t fread(void *ptr, size_t size, size_t count, FILE *stream);
size_t fwrite(const void *ptr, size_t size, size_t count, FILE *stream);
int fseek(FILE *stream, long offset, int whence);
long ftell(FILE *stream);
int feof(FILE *stream);
int ferror(FILE *stream);
int remove(const char *path);
int rename(const char *from, const char *to);
#endif
