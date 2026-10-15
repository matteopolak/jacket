// Exercise the exact owned backend with allocation tracking and sanitizers.
#include <CoreServices/CoreServices.h>
#include <dispatch/dispatch.h>
#include <assert.h>
#include <errno.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <time.h>

static _Atomic long live;
static void *tracked_calloc(size_t count, size_t bytes) {
    void *result = calloc(count, bytes);
    if (result) atomic_fetch_add(&live, 1);
    return result;
}
static char *tracked_strdup(const char *input) {
    char *result = strdup(input);
    if (result) atomic_fetch_add(&live, 1);
    return result;
}
static char *tracked_realpath(const char *input, char *output) {
    char *result = realpath(input, output);
    if (result && !output) atomic_fetch_add(&live, 1);
    return result;
}
static void tracked_free(void *pointer) {
    if (pointer) {
        long previous = atomic_fetch_sub(&live, 1);
        assert(previous > 0);
    }
    free(pointer);
}

#define calloc tracked_calloc
#define strdup tracked_strdup
#define realpath tracked_realpath
#define free tracked_free
#include "../../../stdlib/internal/watching/native/watch.c"
#undef calloc
#undef strdup
#undef realpath
#undef free

static void write_fixture(const char *filename) {
    FILE *file = fopen(filename, "wb");
    assert(file);
    assert(fputs("original native watcher test\n", file) >= 0);
    assert(fclose(file) == 0);
}

static void wait_event(void *watch, const char *wanted, uint32_t mask) {
    for (int i = 0; i < 150; ++i) {
        char *path = NULL;
        uint64_t bytes = 0;
        uint32_t flags = 0;
        double time = 0;
        int result;
        while ((result = portable_watch_pop(watch, &path, &bytes, &flags, &time)) == 1) {
            assert(bytes == strlen(path) && time > 0);
            int found = strcmp(path, wanted) == 0 && (flags & mask) != 0;
            portable_watch_free(path);
            if (found) return;
        }
        assert(result == 0);
        struct timespec delay = {.tv_sec = 0, .tv_nsec = 20000000};
        while (nanosleep(&delay, &delay) != 0) assert(errno == EINTR);
    }
    assert(!"native FSEvents notification did not arrive");
}

int main(int argc, char **argv) {
    assert(argc == 2);
    char filename[4096];
    int count = snprintf(filename, sizeof(filename), "%s/native.txt", argv[1]);
    assert(count > 0 && (size_t)count < sizeof(filename));
    void *watch = portable_watch_create(argv[1], 0.01, 8000);
    assert(watch && portable_watch_stream(watch));
    write_fixture(filename);
    wait_event(watch, filename, kFSEventStreamEventFlagItemCreated);
    assert(portable_watch_close(watch) == 0 && atomic_load(&live) == 0);
    watch = portable_watch_create(argv[1], 0.01, 1);
    assert(watch);
    write_fixture(filename);
    wait_event(watch, argv[1], kFSEventStreamEventFlagMustScanSubDirs);
    assert(portable_watch_close(watch) == 0 && atomic_load(&live) == 0);
    assert(!portable_watch_create(filename, 0.01, 8000));
    assert(atomic_load(&live) == 0);
    assert(puts("native watcher queue/overflow/lifetime checks passed") >= 0);
    return 0;
}
