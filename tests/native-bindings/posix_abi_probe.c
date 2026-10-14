// This oracle uses only the host's official public C SDK headers.
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <time.h>
#include <sys/time.h>
#include <sys/uio.h>
#include <sys/ipc.h>
#include <poll.h>
#include <signal.h>

uint64_t binding_posix_size(unsigned record) {
    switch (record) {
        case 0: return sizeof(struct timeval);
        case 1: return sizeof(struct timespec);
        case 2: return sizeof(struct itimerval);
        case 3: return sizeof(struct iovec);
        case 4: return sizeof(struct pollfd);
        case 5: return sizeof(struct ipc_perm);
        case 6: return sizeof(stack_t);
        default: abort();
    }
}

uint64_t binding_posix_offset(unsigned field) {
    switch (field) {
        case 0: return offsetof(struct timeval, tv_usec);
        case 1: return offsetof(struct timespec, tv_nsec);
        case 2: return offsetof(struct itimerval, it_value);
        case 3: return offsetof(struct iovec, iov_len);
        case 4: return offsetof(struct pollfd, revents);
        case 5: return offsetof(struct ipc_perm, mode);
        case 6: return offsetof(stack_t, ss_flags);
        default: abort();
    }
}
