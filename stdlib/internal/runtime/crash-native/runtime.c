// Newly written from public POSIX C APIs; see the repository LICENSE.
// Signal/TLS execution must remain valid after an interpreter has ended.
#define _GNU_SOURCE
#include <dlfcn.h>
#include <errno.h>
#include <execinfo.h>
#include <pthread.h>
#include <signal.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdlib.h>
#include <unistd.h>
#if defined(__linux__) && !defined(__ANDROID__)
#include <link.h>
#endif

struct alternate_stack {
    void *buffer;
    size_t size;
};

static pthread_once_t initialization = PTHREAD_ONCE_INIT;
static pthread_key_t stack_key;
static int initialization_error;
static void *lifetime_pin;
static atomic_flag handling = ATOMIC_FLAG_INIT;
static void initialize_global(void);

__attribute__((visibility("default"))) int portable_crash_init(void);

static int emit(const char *data, size_t size) {
    while (size) {
        ssize_t sent = write(STDERR_FILENO, data, size);
        if (sent < 0) {
            if (errno == EINTR) continue;
            return 0;
        }
        if (!sent) return 0;
        data += sent;
        size -= (size_t)sent;
    }
    return 1;
}

#define EMIT_LITERAL(text) emit(text, sizeof(text) - 1)

static int emit_number(unsigned value) {
    char buffer[10];
    size_t count = 0;
    do {
        buffer[sizeof(buffer) - ++count] = (char)('0' + value % 10);
        value /= 10;
    } while (value);
    return emit(buffer + sizeof(buffer) - count, count);
}

static void handle_fatal(int sig, siginfo_t *info, void *secret) {
    (void)info;
    (void)secret;
    if (atomic_flag_test_and_set_explicit(&handling, memory_order_relaxed)) _exit(128 + sig);
    if (!EMIT_LITERAL("crash signal ") || !emit_number((unsigned)sig) ||
        !EMIT_LITERAL("\nnative stack:\n")) _exit(128 + sig);
    void *frames[64];
    int count = backtrace(frames, (int)(sizeof(frames) / sizeof(frames[0])));
    if (count > 0) backtrace_symbols_fd(frames, count, STDERR_FILENO);
    else if (!EMIT_LITERAL("native stack unavailable\n")) _exit(128 + sig);

    struct sigaction action = {0};
    if (sigemptyset(&action.sa_mask) || sigaction(sig, &action, NULL)) _exit(128 + sig);
    sigset_t mask;
    if (sigemptyset(&mask) || sigaddset(&mask, sig) || sigprocmask(SIG_UNBLOCK, &mask, NULL)) _exit(128 + sig);
    if (raise(sig)) _exit(128 + sig);
    _exit(128 + sig);
}

static int disable_owned_stack(struct alternate_stack *owned) {
    stack_t current;
    if (sigaltstack(NULL, &current)) return errno;
    if (current.ss_sp != owned->buffer) return 0;
    stack_t disabled = {0};
    // Darwin validates size even for SS_DISABLE, so retain the real extent.
    disabled.ss_sp = owned->buffer;
    disabled.ss_size = owned->size;
    disabled.ss_flags = SS_DISABLE;
    return sigaltstack(&disabled, NULL) ? errno : 0;
}

static void release_stack(void *data) {
    struct alternate_stack *owned = data;
    if (disable_owned_stack(owned)) {
        if (!EMIT_LITERAL("crash alternate-stack removal failed\n")) _exit(134);
        _exit(134);
    }
    free(owned);
}

#if defined(__linux__) && !defined(__ANDROID__)
static int main_image_contains(struct dl_phdr_info *image, size_t size, void *address) {
    (void)size;
    if (image->dlpi_name == NULL || image->dlpi_name[0] != '\0')
        return 0;
    uintptr_t target = (uintptr_t)address;
    for (ElfW(Half) i = 0; i < image->dlpi_phnum; ++i) {
        const ElfW(Phdr) *segment = &image->dlpi_phdr[i];
        if (segment->p_type != PT_LOAD)
            continue;
        uintptr_t start = image->dlpi_addr + segment->p_vaddr;
        if (target >= start && target - start < segment->p_memsz)
            return 1;
    }
    return 0;
}
#endif

static int pin_provider_image(void) {
    // A private address identifies this provider even with ELF symbol interposition.
    void *address = (void *)(uintptr_t)&initialize_global;
#if defined(__linux__) && !defined(__ANDROID__)
    // Executable-linked handlers already have process lifetime. glibc rejects
    // reopening both PIE and non-PIE executables by pathname with dlopen.
    if (dl_iterate_phdr(main_image_contains, address) == 1)
        return 0;
#endif
    Dl_info image;
    if (!dladdr(address, &image) || !image.dli_fname)
        return EIO;
    // These handlers/TLS destructors outlive an interpreter's library cache.
    lifetime_pin = dlopen(image.dli_fname, RTLD_NOW | RTLD_NODELETE);
    return lifetime_pin != NULL ? 0 : EIO;
}
static void initialize_global(void) {
    initialization_error = pin_provider_image();
    if (initialization_error) return;
    initialization_error = pthread_key_create(&stack_key, release_stack);
    if (initialization_error) return;
    void *warm[1];
    if (backtrace(warm, 1) <= 0 && !EMIT_LITERAL("native backtrace warmup unavailable\n")) {
        initialization_error = EIO;
        return;
    }
    struct sigaction action = {0};
    action.sa_sigaction = handle_fatal;
    action.sa_flags = SA_RESTART | SA_ONSTACK | SA_SIGINFO;
    if (sigemptyset(&action.sa_mask)) {
        initialization_error = errno;
        return;
    }
    const int signals[] = {SIGQUIT, SIGILL, SIGTRAP, SIGABRT, SIGFPE, SIGBUS, SIGSEGV};
    for (size_t i = 0; i < sizeof(signals) / sizeof(signals[0]); ++i) {
        if (sigaction(signals[i], &action, NULL)) {
            initialization_error = errno;
            return;
        }
    }
}

__attribute__((visibility("default"))) int portable_crash_init(void) {
    int error = pthread_once(&initialization, initialize_global);
    if (error) return error;
    if (initialization_error) return initialization_error;
    if (pthread_getspecific(stack_key)) return 0;
    size_t size = 131072;
    long minimum = MINSIGSTKSZ;
    if (minimum < 0) return errno ? errno : EIO;
    if ((size_t)minimum > size) size = (size_t)minimum;
    if (size > SIZE_MAX - sizeof(struct alternate_stack) - 15) return ENOMEM;
    struct alternate_stack *owned = malloc(sizeof(*owned) + size + 15);
    if (!owned) return ENOMEM;
    owned->buffer = (void *)(((uintptr_t)(owned + 1) + 15) & ~(uintptr_t)15);
    owned->size = size;
    stack_t stack = {0};
    stack.ss_sp = owned->buffer;
    stack.ss_size = size;
    if (sigaltstack(&stack, NULL)) {
        error = errno;
        free(owned);
        return error;
    }
    error = pthread_setspecific(stack_key, owned);
    if (error) {
        int cleanup_error = disable_owned_stack(owned);
        if (cleanup_error) return cleanup_error;
        free(owned);
    }
    return error;
}

__attribute__((visibility("default"))) void portable_crash_handle(int sig, void *info, void *secret) {
    handle_fatal(sig, info, secret);
}
