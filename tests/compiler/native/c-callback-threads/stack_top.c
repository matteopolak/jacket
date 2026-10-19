// C side of stack_top.jai: calls a callback from the top of a stack with an unmapped page just
// above it, as a thread's first function does, so a callee that reads past its own arguments
// faults.
#define _XOPEN_SOURCE 600
#ifdef __APPLE__
#define _DARWIN_C_SOURCE
#endif
#include <sys/mman.h>
#include <ucontext.h>
#include <unistd.h>

typedef int (*Callback)(int);

static ucontext_t caller, top;
static Callback callback;
static int argument, result;

static void at_top(void) { result = callback(argument); }

// Returns `cb(x)`, called on a stack of its own whose top is followed by an unmapped page.
int call_at_stack_top(Callback cb, int x) {
    size_t page = (size_t)sysconf(_SC_PAGESIZE);
    size_t size = 16u << 20;
    char *memory = mmap(NULL, size + page, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANON, -1, 0);
    if (memory == MAP_FAILED || mprotect(memory + size, page, PROT_NONE)) return -1;
    callback = cb;
    argument = x;
    getcontext(&top);
    top.uc_stack.ss_sp = memory;
    top.uc_stack.ss_size = size;
    top.uc_link = &caller;
    makecontext(&top, at_top, 0);
    swapcontext(&caller, &top);
    munmap(memory, size + page);
    return result;
}
