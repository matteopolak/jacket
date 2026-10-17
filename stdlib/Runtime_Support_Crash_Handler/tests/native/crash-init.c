/* Independent native startup/lifetime control; never deliberately faults. */
#define _GNU_SOURCE
#include <dlfcn.h>
#include <pthread.h>
#include <signal.h>
#include <stdio.h>
#include <string.h>

typedef int (*init_function)(void);
static init_function initialize;

#if !defined(CRASH_SHARED) || defined(CRASH_INTERPOSE)
extern int portable_crash_init(void);
#endif

static int check_initialization(void) {
    int error = initialize();
    if (error != 0) {
        const char *detail = dlerror();
        fprintf(stderr, "portable_crash_init=%d; loader=%s\n", error,
                detail == NULL ? "no loader error" : detail);
        return 1;
    }
    stack_t first, second;
    if (sigaltstack(NULL, &first) != 0 || (first.ss_flags & SS_DISABLE) != 0 ||
        first.ss_sp == NULL || first.ss_size < (size_t)MINSIGSTKSZ)
        return 2;
    error = initialize();
    if (error != 0 || sigaltstack(NULL, &second) != 0 || first.ss_sp != second.ss_sp ||
        first.ss_size != second.ss_size)
        return 3;
    struct sigaction action;
    if (sigaction(SIGSEGV, NULL, &action) != 0 ||
        (action.sa_flags & (SA_SIGINFO | SA_ONSTACK)) != (SA_SIGINFO | SA_ONSTACK) ||
        action.sa_sigaction == NULL)
        return 4;
    return 0;
}

static void *worker(void *argument) {
    int *result = argument;
    *result = check_initialization();
    return NULL;
}

int main(int argc, char **argv) {
#ifdef CRASH_INTERPOSE
    initialize = portable_crash_init;
    int initial_result = check_initialization();
    if (initial_result != 0)
        return initial_result;
#endif
#ifdef CRASH_SHARED
    if (argc != 2)
        return 10;
    void *library = dlopen(argv[1], RTLD_NOW | RTLD_LOCAL);
    if (library == NULL) {
        fprintf(stderr, "dlopen failed: %s\n", dlerror());
        return 11;
    }
    void *symbol = dlsym(library, "portable_crash_init");
    if (symbol == NULL)
        return 12;
#ifdef CRASH_INTERPOSE
    if (symbol == (void *)portable_crash_init)
        return 16;
#endif
    _Static_assert(sizeof(symbol) == sizeof(initialize), "POSIX function pointer ABI");
    memcpy(&initialize, &symbol, sizeof(initialize));
#else
    (void)argc;
    (void)argv;
    initialize = portable_crash_init;
#endif
    int result = check_initialization();
    if (result != 0)
        return result;
#ifdef CRASH_SHARED
    /* The provider must remain valid after its caller drops the load handle. */
    if (dlclose(library) != 0)
        return 13;
    result = check_initialization();
    if (result != 0)
        return result;
#endif
    pthread_t thread;
    int thread_result = -1;
    int error = pthread_create(&thread, NULL, worker, &thread_result);
    if (error != 0)
        return 14;
    error = pthread_join(thread, NULL);
    if (error != 0 || thread_result != 0)
        return 15;
    puts("ok");
    return 0;
}
