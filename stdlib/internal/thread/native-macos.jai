// Minimal private Mach synchronization ABI from Apple's public SDK headers:
// mach/port.h, mach/mach_types.h, mach/mach_time.h, mach/semaphore.h,
// mach/kern_return.h and mach/sync_policy.h. No public macOS facade is replaced.
task_t :: u32;
semaphore_t :: u32;
kern_return_t :: s32;

mach_timespec_t :: struct {
    tv_sec: u32;
    tv_nsec: s32;
}

SYNC_POLICY_FIFO :: 0;
KERN_ABORTED :: 14;
KERN_OPERATION_TIMED_OUT :: 49;
semaphore_create :: (task: task_t, semaphore: *semaphore_t, policy: s32, value: s32) -> kern_return_t #foreign libc;
semaphore_destroy :: (task: task_t, semaphore: semaphore_t) -> kern_return_t #foreign libc;
semaphore_signal :: (semaphore: semaphore_t) -> kern_return_t #foreign libc;
semaphore_wait :: (semaphore: semaphore_t) -> kern_return_t #foreign libc;
semaphore_timedwait :: (semaphore: semaphore_t, interval: mach_timespec_t) -> kern_return_t #foreign libc;
#scope_file
libc :: #system_library "libc";
