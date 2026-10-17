// Native SDK ABI constants, independently checked against glibc public headers.
#if OS != .WINDOWS P :: #import "POSIX";
T :: #import "Thread";

#if OS == .LINUX {
    #assert size_of(P.pthread_t) == 8;
    #assert align_of(P.pthread_t) == 8;
    #assert size_of(P.sem_t) == 32;
    #assert align_of(P.sem_t) == 8;
    #assert size_of(P.pthread_cond_t) == 48;
    #assert align_of(P.pthread_cond_t) == 8;

    #if CPU == .ARM64 {
        #assert size_of(P.pthread_mutex_t) == 48;
        #assert size_of(P.pthread_mutexattr_t) == 8;
        #assert align_of(P.pthread_mutexattr_t) == 4;
    } else {
        #assert size_of(P.pthread_mutex_t) == 40;
        #assert size_of(P.pthread_mutexattr_t) == 4;
        #assert align_of(P.pthread_mutexattr_t) == 4;
    }

    #assert align_of(P.pthread_mutex_t) == 8;
    #assert size_of(T.Mutex) >= size_of(P.pthread_mutex_t);
}

main :: () {}
