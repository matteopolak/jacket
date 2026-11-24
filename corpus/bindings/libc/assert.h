/* Declarations only: the assert macro the corpus libraries use. */
#ifndef CORPUS_LIBC_ASSERT_H
#define CORPUS_LIBC_ASSERT_H
void __assert_fail(const char *expression, const char *file, unsigned line, const char *function);
#define assert(e) ((e) ? (void)0 : __assert_fail(#e, __FILE__, __LINE__, __func__))
#endif
