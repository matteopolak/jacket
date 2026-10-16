// Actual native C SDK writes must remain inside the public binding object.
#import "Basic";
P :: #import "POSIX";

sdk :: #library "thread_sdk_identity";
thread_sdk_size :: (kind: u32) -> u64 #foreign sdk;
thread_sdk_alignment :: (kind: u32) -> u64 #foreign sdk;
thread_sdk_mutex_init :: (storage: *void) -> s32 #foreign sdk;
thread_sdk_mutex_destroy :: (storage: *void) -> s32 #foreign sdk;
thread_sdk_creation_control :: () -> s32 #foreign sdk;

Guarded_Mutex :: struct {
    mutex: P.pthread_mutex_t;
    after: [16] u8;
}

main :: () {
    assert(thread_sdk_size(0) == size_of(P.pthread_t));
    assert(thread_sdk_size(1) == size_of(P.pthread_mutex_t));
    assert(thread_sdk_size(2) == size_of(P.pthread_mutexattr_t));
    assert(thread_sdk_size(3) == size_of(P.pthread_cond_t));
    assert(thread_sdk_size(4) == size_of(P.sem_t));
    assert(thread_sdk_alignment(0) == align_of(P.pthread_t));
    assert(thread_sdk_alignment(1) == align_of(P.pthread_mutex_t));
    assert(thread_sdk_alignment(2) == align_of(P.pthread_mutexattr_t));
    assert(thread_sdk_alignment(3) == align_of(P.pthread_cond_t));
    assert(thread_sdk_alignment(4) == align_of(P.sem_t));
    guarded: Guarded_Mutex;
    for *byte: guarded.after byte.* = 0xa7;

    assert(thread_sdk_mutex_init(*guarded.mutex) == 0);
    for byte: guarded.after assert(byte == 0xa7);

    assert(thread_sdk_mutex_destroy(*guarded.mutex) == 0);
    for byte: guarded.after assert(byte == 0xa7);

    assert(thread_sdk_creation_control() == 0);
    print("ok\n");
}
