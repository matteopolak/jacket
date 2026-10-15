// Official Apple SDK is the native layout/lifetime oracle. This public-layout
// mirror is independently written from the reviewed specification.
#include <CoreServices/CoreServices.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

_Static_assert(sizeof(FSEventStreamContext) == 40, "native context size");
_Static_assert(offsetof(FSEventStreamContext, version) == 0, "version");
_Static_assert(offsetof(FSEventStreamContext, info) == 8, "info");
_Static_assert(offsetof(FSEventStreamContext, retain) == 16, "retain");
_Static_assert(offsetof(FSEventStreamContext, release) == 24, "release");
_Static_assert(offsetof(FSEventStreamContext, copyDescription) == 32, "description");

typedef struct {
    CFAllocatorCopyDescriptionCallBack copyDescription;
    void *info;
    CFAllocatorRetainCallBack retain;
    CFAllocatorReleaseCallBack release;
    CFIndex version;
} PublicContext;
typedef ConstFSEventStreamRef (*PublicCreate)(CFAllocatorRef, FSEventStreamCallback,
    PublicContext *, CFArrayRef, FSEventStreamEventId, CFTimeInterval, FSEventStreamCreateFlags);

static unsigned retains, releases, descriptions;
static size_t events;
static int source_info;
#define require(value) do { if (!(value)) { \
    if (fprintf(stderr, "FSEvents assertion failed at line %d: %s\n", __LINE__, #value) < 0) abort(); \
    abort(); } } while (0)
static const void *retain_info(const void *info) {
    require(info == &source_info);
    ++retains;
    return &source_info;
}
static void release_info(const void *info) {
    require(info == &source_info);
    ++releases;
}
static CFStringRef describe_info(const void *info) {
    require(info == &source_info);
    ++descriptions;
    return CFStringCreateCopy(NULL, CFSTR("bindings retained info"));
}
static void event_callback(ConstFSEventStreamRef stream, void *info, size_t count,
                           void *paths, const FSEventStreamEventFlags *flags,
                           const FSEventStreamEventId *ids) {
    require(stream != NULL && count > 0 && paths != NULL && flags != NULL && ids != NULL);
    require(info == &source_info);
    events += count;
}

static ConstFSEventStreamRef native_control(CFAllocatorRef allocator, FSEventStreamCallback callback,
        PublicContext *source, CFArrayRef paths, FSEventStreamEventId since,
        CFTimeInterval latency, FSEventStreamCreateFlags flags) {
    FSEventStreamContext native;
    if (source) native = (FSEventStreamContext){source->version, source->info,
        source->retain, source->release, source->copyDescription};
    return FSEventStreamCreate(allocator, callback, source ? &native : NULL, paths, since, latency, flags);
}

static void run_lifetime(PublicCreate create) {
    retains = releases = descriptions = 0;
    struct {
        uint64_t before;
        PublicContext value;
        uint64_t after;
    } guarded = {UINT64_C(0x1234567887654321),
        {describe_info, &source_info, retain_info, release_info, 0},
        UINT64_C(0x89abcdef76543210)};
    unsigned char original[sizeof guarded];
    memcpy(original, &guarded, sizeof guarded);
    CFStringRef path = CFSTR("/tmp");
    CFArrayRef paths = CFArrayCreate(NULL, (const void **)&path, 1, &kCFTypeArrayCallBacks);
    require(paths != NULL);
    ConstFSEventStreamRef stream = create(NULL, event_callback, &guarded.value, paths,
        kFSEventStreamEventIdSinceNow, 0.01, kFSEventStreamCreateFlagFileEvents);
    require(stream != NULL && retains == 1 && releases == 0);
    require(memcmp(original, &guarded, sizeof guarded) == 0);
    // SDK must own a copied context after the wrapper's native stack expires.
    memset(&guarded.value, 0xa5, sizeof guarded.value);
    CFStringRef description = FSEventStreamCopyDescription(stream);
    require(description != NULL);
    CFRelease(description);
    FSEventStreamRetain((FSEventStreamRef)stream);
    FSEventStreamRelease((FSEventStreamRef)stream);
    require(releases == 0);
    FSEventStreamRelease((FSEventStreamRef)stream);
    require(releases == 1);
    // Null context has the SDK's documented zero-context semantics.
    stream = create(NULL, event_callback, NULL, paths,
        kFSEventStreamEventIdSinceNow, 0.01, 0);
    require(stream != NULL);
    FSEventStreamRelease((FSEventStreamRef)stream);
    require(retains == 1 && releases == 1);
    CFRelease(paths);
}

void binding_fsevents_lifetime(PublicCreate create) {
    run_lifetime(native_control);
    run_lifetime(create);
}

void binding_fsevents_delivery(PublicCreate create) {
    retains = releases = descriptions = 0;
    events = 0;
    const char *temporary = getenv("TMPDIR");
    require(temporary != NULL);
    char directory[4096], file[4096];
    int length = snprintf(directory, sizeof directory, "%s/bindings-fsevents-XXXXXX", temporary);
    require(length > 0 && (size_t)length < sizeof directory);
    require(mkdtemp(directory) != NULL);
    CFStringRef path = CFStringCreateWithCString(NULL, directory, kCFStringEncodingUTF8);
    require(path != NULL);
    CFArrayRef paths = CFArrayCreate(NULL, (const void **)&path, 1, &kCFTypeArrayCallBacks);
    require(paths != NULL);
    PublicContext context = {describe_info, &source_info, retain_info, release_info, 0};
    FSEventStreamRef stream = (FSEventStreamRef)create(NULL, event_callback, &context, paths,
        kFSEventStreamEventIdSinceNow, 0.01, kFSEventStreamCreateFlagFileEvents);
    require(stream != NULL);
    memset(&context, 0xa5, sizeof context);
    FSEventStreamScheduleWithRunLoop(stream, CFRunLoopGetCurrent(), kCFRunLoopDefaultMode);
    require(FSEventStreamStart(stream));
    length = snprintf(file, sizeof file, "%s/actual-event", directory);
    require(length > 0 && (size_t)length < sizeof file);
    FILE *output = fopen(file, "wb");
    require(output != NULL);
    require(fputs("native callback delivery", output) >= 0);
    require(fclose(output) == 0);
    CFAbsoluteTime deadline = CFAbsoluteTimeGetCurrent() + 10.0;
    while (!events && CFAbsoluteTimeGetCurrent() < deadline)
        CFRunLoopRunInMode(kCFRunLoopDefaultMode, 0.1, false);
    FSEventStreamStop(stream);
    FSEventStreamInvalidate(stream);
    FSEventStreamRelease(stream);
    require(retains == 1 && releases == 1);
    CFRelease(paths);
    CFRelease(path);
    require(unlink(file) == 0);
    require(rmdir(directory) == 0);
    require(events > 0);
}
