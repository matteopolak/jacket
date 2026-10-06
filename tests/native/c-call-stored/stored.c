// C side of stored.jai: calls #c_call procedures it finds in memory (struct fields, a global
// the Jai program owns) rather than receives as arguments, on the calling thread and on
// threads of its own.
#include <stdlib.h>
#ifdef _WIN32
#include <windows.h>
#define EXPORT __declspec(dllexport)
#else
#include <pthread.h>
#include <unistd.h>
#define EXPORT
#endif

typedef int (*Unary)(int);
typedef int (*Compare)(const void *, const void *);

typedef struct {
    int bias;
    Unary apply;
} Holder;

typedef struct {
    Compare compare;
} Sorter;

EXPORT int call_holder(const Holder *h, int x) { return h->apply(x) + h->bias; }

EXPORT int call_stored(Unary *slot, int x) { return (*slot)(x); }

EXPORT void sort_ints(int *xs, int n, const Sorter *s) { qsort(xs, (size_t)n, sizeof(int), s->compare); }

typedef struct {
    const Holder *holder;
    int x;
    volatile int result;
    volatile int done;
} Job;

#ifdef _WIN32
static DWORD WINAPI job_main(LPVOID p) {
#else
static void *job_main(void *p) {
#endif
    Job *job = (Job *)p;
    job->result = call_holder(job->holder, job->x);
    job->done = 1;
    return 0;
}

// Calls the holder on a new thread and waits for it.
EXPORT int call_holder_on_thread(const Holder *h, int x) {
    Job job = {h, x, 0, 0};
#ifdef _WIN32
    HANDLE t = CreateThread(NULL, 0, job_main, &job, 0, NULL);
    WaitForSingleObject(t, INFINITE);
    CloseHandle(t);
#else
    pthread_t t;
    pthread_create(&t, NULL, job_main, &job);
    pthread_join(t, NULL);
#endif
    return job.result;
}

static Job background;

// Calls the holder on a new thread and returns at once; poll with `background_result`.
EXPORT void start_background(const Holder *h, int x) {
    background.holder = h;
    background.x = x;
    background.done = 0;
#ifdef _WIN32
    CloseHandle(CreateThread(NULL, 0, job_main, &background, 0, NULL));
#else
    pthread_t t;
    pthread_create(&t, NULL, job_main, &background);
    pthread_detach(t);
#endif
}

// -1 until the background call has finished; waits a millisecond first.
EXPORT int background_result(void) {
#ifdef _WIN32
    Sleep(1);
#else
    usleep(1000);
#endif
    return background.done ? background.result : -1;
}
