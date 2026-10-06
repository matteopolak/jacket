// C side of blocking.jai and deadlock.jai: a thread of C's own that calls a #c_call procedure,
// a call back on the calling thread, and a long call that waits for a flag Jai sets.
#ifdef _WIN32
#include <windows.h>
#define EXPORT __declspec(dllexport)
#else
#include <pthread.h>
#include <unistd.h>
#define EXPORT
#endif

typedef int (*Callback)(int);

static void nap(void) {
#ifdef _WIN32
    Sleep(1);
#else
    usleep(1000);
#endif
}

static Callback caller_callback;
static int caller_argument;
static int caller_result;

#ifdef _WIN32
static HANDLE caller;
static DWORD WINAPI caller_main(LPVOID unused) {
#else
static pthread_t caller;
static void *caller_main(void *unused) {
#endif
    (void)unused;
    caller_result = caller_callback(caller_argument);
    return 0;
}

// Starts a thread that calls `cb(x)`, and returns at once.
EXPORT void start_caller(Callback cb, int x) {
    caller_callback = cb;
    caller_argument = x;
#ifdef _WIN32
    caller = CreateThread(NULL, 0, caller_main, NULL, 0, NULL);
#else
    pthread_create(&caller, NULL, caller_main, NULL);
#endif
}

// Waits for the thread `start_caller` started and returns what the callback returned.
EXPORT int finish_caller(void) {
#ifdef _WIN32
    WaitForSingleObject(caller, INFINITE);
    CloseHandle(caller);
#else
    pthread_join(caller, NULL);
#endif
    return caller_result;
}

EXPORT int call_back(Callback cb, int x) { return cb(x); }

// Returns once `*flag` is nonzero: a long call that only another thread can end.
EXPORT int wait_for_flag(volatile int *flag) {
    int naps = 0;
    while (!*flag) {
        nap();
        naps++;
    }
    return naps > 0;
}
