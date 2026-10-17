#import <AppKit/AppKit.h>
#include <spawn.h>
#include <sys/wait.h>
#include <stdio.h>
#include <errno.h>
#include <stdint.h>
#include <stdlib.h>
#include <signal.h>
#include <time.h>
#include <unistd.h>

extern char **environ;

static int64_t milliseconds(void) {
    struct timespec now;
    if (clock_gettime(CLOCK_MONOTONIC, &now) != 0) return -1;
    return (int64_t)now.tv_sec * 1000 + now.tv_nsec / 1000000;
}

static int terminate_child(pid_t pid, int *status) {
    if (kill(pid, SIGKILL) < 0 && errno != ESRCH) return -1;
    pid_t result;
    do result = waitpid(pid, status, 0); while (result < 0 && errno == EINTR);
    return result == pid ? 0 : -1;
}

// Preserve every eagerly materialized item/type, including images, without
// printing clipboard data. Child failure (including abort) still restores it.
int main(int argc, char **argv) {
    if (argc < 2) return 2;
    long long timeout_ms = 120000;
    const char *configured_timeout = getenv("JAIC_TEST_CHILD_TIMEOUT_MS");
    if (configured_timeout) {
        char *end;
        errno = 0;
        timeout_ms = strtoll(configured_timeout, &end, 10);
        if (errno || *end || timeout_ms <= 0 || timeout_ms > 86400000) return 2;
    }
    @autoreleasepool {
        NSPasteboard *board = [NSPasteboard generalPasteboard];
        if (!board) {
            fprintf(stderr, "native clipboard unavailable; child not started\n");
            return 3;
        }
        NSMutableArray<NSPasteboardItem *> *saved = [NSMutableArray array];
        for (NSPasteboardItem *item in board.pasteboardItems) {
            NSPasteboardItem *copy = [[NSPasteboardItem alloc] init];
            for (NSPasteboardType type in item.types) {
                NSData *data = [item dataForType:type];
                if (!data || ![copy setData:data forType:type]) {
                    fprintf(stderr, "clipboard snapshot unavailable; child not started\n");
                    return 3;
                }
            }
            [saved addObject:copy];
        }
        pid_t pid;
        int64_t started = milliseconds();
        if (started < 0) return 5;
        int error = posix_spawn(&pid, argv[1], NULL, NULL, argv + 1, environ);
        if (error) return 4;
        int status = 0;
        int wait_error = 0;
        int timed_out = 0;
        for (;;) {
            pid_t waited = waitpid(pid, &status, WNOHANG);
            if (waited == pid) break;
            if (waited < 0) {
                if (errno == EINTR) continue;
                wait_error = 5;
                break;
            }
            int64_t now = milliseconds();
            if (now < 0 || now - started >= timeout_ms) {
                timed_out = now >= 0;
                if (terminate_child(pid, &status) != 0 || now < 0) wait_error = 5;
                if (timed_out) fprintf(stderr, "clipboard child timed out after %lld ms\n", timeout_ms);
                break;
            }
            if (usleep(10000) < 0 && errno != EINTR) {
                if (terminate_child(pid, &status) != 0) fprintf(stderr, "clipboard child reap failed\n");
                wait_error = 5;
                break;
            }
        }
        [board clearContents];
        if (saved.count && ![board writeObjects:saved]) return 6;
        NSArray<NSPasteboardItem *> *restored = board.pasteboardItems;
        if (restored.count != saved.count) return 7;
        for (NSUInteger i = 0; i < saved.count; ++i) {
            for (NSPasteboardType type in saved[i].types) {
                if (![[restored[i] dataForType:type] isEqualToData:[saved[i] dataForType:type]]) return 8;
            }
        }
        if (wait_error) return wait_error;
        if (timed_out) return 124;
        return WIFEXITED(status) ? WEXITSTATUS(status) : 128 + WTERMSIG(status);
    }
}
