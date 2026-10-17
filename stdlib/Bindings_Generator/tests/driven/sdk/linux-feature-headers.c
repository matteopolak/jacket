/* Original compile-only Linux header contract. No advice values are invented.
 */
#include <stddef.h>
#include <stdint.h>
#include <pthread.h>
#include <sys/mman.h>

_Static_assert(MADV_FREE != MADV_DONTNEED,
               "retained-page policies must stay distinct");
int binding_linux_advice_values[] = {POSIX_MADV_DONTNEED, MADV_DONTNEED,
                                     MADV_FREE};
int (*binding_linux_madvise)(void *, size_t, int) = madvise;
int (*binding_linux_posix_madvise)(void *, size_t, int) = posix_madvise;
int (*binding_linux_key_create)(pthread_key_t *,
                                void (*)(void *)) = pthread_key_create;
int (*binding_linux_setspecific)(pthread_key_t,
                                 const void *) = pthread_setspecific;
