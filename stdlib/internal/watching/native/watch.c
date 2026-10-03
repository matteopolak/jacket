// Independently authored native FSEvents lifecycle and queue implementation.
// Public SDK headers provide native ABI layouts; no external implementation copied.
#include <CoreServices/CoreServices.h>
#include <dispatch/dispatch.h>
#include <errno.h>
#include <math.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <time.h>

#define API __attribute__((visibility("default")))

typedef struct Event {
    struct Event *next;
    char *path;
    uint64_t bytes;
    uint32_t flags;
    double time;
} Event;

typedef struct Watch {
    FSEventStreamRef stream;
    dispatch_queue_t queue;
    pthread_mutex_t mutex;
    Event *head, *tail;
    char *root;
    size_t queued_bytes, limit;
    _Atomic int error;
    int overflow, closed;
} Watch;

API double portable_watch_time(void) {
    struct timespec time;
    if (clock_gettime(CLOCK_MONOTONIC, &time) != 0) return -1;
    return (double)time.tv_sec + (double)time.tv_nsec * 1e-9;
}

static void receive_events(ConstFSEventStreamRef stream, void *info, size_t count,
                           void *paths, const FSEventStreamEventFlags flags[],
                           const FSEventStreamEventId ids[]) {
    (void)stream;
    (void)ids;
    Watch *watch = info;
    char **names = paths;
    double time = portable_watch_time();
    if (time < 0) {
        atomic_store(&watch->error, errno ? errno : EIO);
        return;
    }
    int rc = pthread_mutex_lock(&watch->mutex);
    if (rc) {
        atomic_store(&watch->error, rc);
        return;
    }
    for (size_t i = 0; i < count; ++i) {
        size_t bytes = strlen(names[i]);
        if (bytes > SIZE_MAX - sizeof(Event) - 1 ||
            bytes + sizeof(Event) + 1 > watch->limit - watch->queued_bytes) {
            watch->overflow = 1;
            continue;
        }
        Event *event = calloc(1, sizeof(*event));
        char *path = strdup(names[i]);
        if (!event || !path) {
            free(event);
            free(path);
            atomic_store(&watch->error, ENOMEM);
            break;
        }
        event->path = path;
        event->bytes = bytes;
        event->flags = flags[i];
        event->time = time;
        if (watch->tail) watch->tail->next = event;
        else watch->head = event;
        watch->tail = event;
        watch->queued_bytes += bytes + sizeof(Event) + 1;
    }
    rc = pthread_mutex_unlock(&watch->mutex);
    if (rc) atomic_store(&watch->error, rc);
}

API void *portable_watch_create(const char *path, double latency, uint64_t limit) {
    if (!path || !isfinite(latency) || latency < 0 || limit == 0 || limit > SIZE_MAX) {
        errno = EINVAL;
        return NULL;
    }
    Watch *watch = calloc(1, sizeof(*watch));
    if (!watch) return NULL;
    atomic_init(&watch->error, 0);
    watch->root = realpath(path, NULL);
    if (!watch->root) {
        free(watch);
        return NULL;
    }
    struct stat metadata;
    int status = stat(watch->root, &metadata);
    if (status != 0 || !S_ISDIR(metadata.st_mode)) {
        int error = status != 0 ? errno : ENOTDIR;
        free(watch->root);
        free(watch);
        errno = error;
        return NULL;
    }
    int rc = pthread_mutex_init(&watch->mutex, NULL);
    if (rc) {
        free(watch->root);
        free(watch);
        errno = rc;
        return NULL;
    }
    watch->limit = (size_t)limit;
    CFStringRef name = CFStringCreateWithCString(NULL, watch->root, kCFStringEncodingUTF8);
    CFArrayRef names = name ? CFArrayCreate(NULL, (const void **)&name, 1, &kCFTypeArrayCallBacks) : NULL;
    watch->queue = dispatch_queue_create("portable.stdlib.file-watcher", DISPATCH_QUEUE_SERIAL);
    if (names && watch->queue) {
        FSEventStreamContext context = {.version = 0, .info = watch};
        watch->stream = FSEventStreamCreate(NULL, receive_events, &context, names,
            kFSEventStreamEventIdSinceNow, latency,
            kFSEventStreamCreateFlagFileEvents | kFSEventStreamCreateFlagWatchRoot |
            kFSEventStreamCreateFlagNoDefer);
    }
    if (names) CFRelease(names);
    if (name) CFRelease(name);
    if (watch->stream) {
        FSEventStreamSetDispatchQueue(watch->stream, watch->queue);
        if (FSEventStreamStart(watch->stream)) return watch;
        FSEventStreamInvalidate(watch->stream);
        FSEventStreamRelease(watch->stream);
        dispatch_sync(watch->queue, ^{});
    }
    if (watch->queue) dispatch_release(watch->queue);
    rc = pthread_mutex_destroy(&watch->mutex);
    // No caller can recover a failed constructor; a mutex lifecycle defect is fatal.
    if (rc) abort();
    Event *event = watch->head;
    while (event) {
        Event *next = event->next;
        free(event->path);
        free(event);
        event = next;
    }
    free(watch->root);
    free(watch);
    errno = EIO;
    return NULL;
}

API void *portable_watch_stream(void *handle) {
    return handle ? ((Watch *)handle)->stream : NULL;
}

API const char *portable_watch_root(void *handle, uint64_t *bytes) {
    if (!handle || !bytes) return NULL;
    Watch *watch = handle;
    *bytes = strlen(watch->root);
    return watch->root;
}

API int portable_watch_pop(void *handle, char **path, uint64_t *bytes,
                           uint32_t *flags, double *time) {
    if (!handle || !path || !bytes || !flags || !time) return -1;
    Watch *watch = handle;
    if (watch->closed || atomic_load(&watch->error)) return -1;
    int rc = pthread_mutex_lock(&watch->mutex);
    if (rc) {
        atomic_store(&watch->error, rc);
        return -1;
    }
    int result = 0;
    Event *event = watch->head;
    if (event) {
        watch->head = event->next;
        if (!watch->head) watch->tail = NULL;
        watch->queued_bytes -= event->bytes + sizeof(Event) + 1;
        *path = event->path;
        *bytes = event->bytes;
        *flags = event->flags;
        *time = event->time;
        free(event);
        result = 1;
    } else if (watch->overflow) {
        *path = strdup(watch->root);
        *time = portable_watch_time();
        if (!*path || *time < 0) {
            free(*path);
            *path = NULL;
            atomic_store(&watch->error, errno ? errno : ENOMEM);
            result = -1;
        } else {
            *bytes = strlen(watch->root);
            *flags = kFSEventStreamEventFlagMustScanSubDirs;
            watch->overflow = 0;
            result = 1;
        }
    }
    rc = pthread_mutex_unlock(&watch->mutex);
    if (rc) {
        atomic_store(&watch->error, rc);
        if (result == 1) {
            free(*path);
            *path = NULL;
        }
        result = -1;
    }
    return result;
}

API void portable_watch_free(void *pointer) { free(pointer); }

API int portable_watch_error(void *handle) {
    return handle ? atomic_load(&((Watch *)handle)->error) : EINVAL;
}

// Private high-level event protocol. Public Jai enum values are checked there.
API uint32_t portable_watch_classify(uint32_t flags) {
    uint32_t result = 0;
    if (flags & kFSEventStreamEventFlagItemCreated) result |= 1;
#if __MAC_OS_X_VERSION_MAX_ALLOWED >= 101300
    if (flags & kFSEventStreamEventFlagItemCloned) result |= 1;
#endif
    if (flags & kFSEventStreamEventFlagItemModified) result |= 2;
    if (flags & kFSEventStreamEventFlagItemRenamed) result |= 4;
    if (flags & (kFSEventStreamEventFlagItemInodeMetaMod |
                 kFSEventStreamEventFlagItemFinderInfoMod |
                 kFSEventStreamEventFlagItemChangeOwner |
                 kFSEventStreamEventFlagItemXattrMod)) result |= 32;
    if (flags & kFSEventStreamEventFlagItemRemoved) result |= 64;
    if (flags & (kFSEventStreamEventFlagMustScanSubDirs |
                 kFSEventStreamEventFlagRootChanged |
                 kFSEventStreamEventFlagEventIdsWrapped |
                 kFSEventStreamEventFlagUserDropped |
                 kFSEventStreamEventFlagKernelDropped |
                 kFSEventStreamEventFlagMount |
                 kFSEventStreamEventFlagUnmount)) result |= 128 | 2;
    if (flags == 0) result |= 128 | 2;
    return result;
}

API int portable_watch_close_v2(void *handle, int *released) {
    if (!released) {
        errno = EINVAL;
        return EINVAL;
    }
    *released = 0;
    if (!handle) {
        *released = 1;
        return 0;
    }
    Watch *watch = handle;
    if (!watch->closed) {
        FSEventStreamStop(watch->stream);
        FSEventStreamInvalidate(watch->stream);
        FSEventStreamRelease(watch->stream);
        dispatch_sync(watch->queue, ^{});
        dispatch_release(watch->queue);
        watch->closed = 1;
    }
    int error = atomic_load(&watch->error);
    int rc = pthread_mutex_destroy(&watch->mutex);
    if (rc) {
        errno = rc;
        return rc;
    }
    Event *event = watch->head;
    while (event) {
        Event *next = event->next;
        free(event->path);
        free(event);
        event = next;
    }
    free(watch->root);
    free(watch);
    *released = 1;
    if (error) {
        errno = error;
        return error;
    }
    return 0;
}
