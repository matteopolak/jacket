/* Declarations only: the time types the corpus libraries name. */
#ifndef CORPUS_LIBC_TIME_H
#define CORPUS_LIBC_TIME_H
#include <stddef.h>
typedef long time_t;
typedef long clock_t;
struct tm { int tm_sec; int tm_min; int tm_hour; int tm_mday; int tm_mon; int tm_year; int tm_wday; int tm_yday; int tm_isdst; };
time_t time(time_t *out);
#endif
