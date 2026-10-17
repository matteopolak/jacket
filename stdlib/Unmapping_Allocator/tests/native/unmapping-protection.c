/* Defined SDK error injection on a page owned only by this test process. */
#include <errno.h>
#include <mach/mach.h>
#include <mach/mach_vm.h>
#include <stdint.h>
#include <sys/mman.h>
#include <unistd.h>

int test_unmapping_deny_read(void *memory) {
    long page = sysconf(_SC_PAGESIZE);
    if (page <= 0) return EINVAL;
    uintptr_t base = (uintptr_t)memory & ~((uintptr_t)page - 1);
    return mach_vm_protect(mach_task_self(), (mach_vm_address_t)base,
                           (mach_vm_size_t)page, TRUE, VM_PROT_NONE);
}

int test_unmapping_restore(void *memory) {
    long page = sysconf(_SC_PAGESIZE);
    if (page <= 0) return EINVAL;
    uintptr_t base = (uintptr_t)memory & ~((uintptr_t)page - 1);
    /* The reservation is still ours: replace exactly that controlled page. */
    void *restored = mmap((void *)base, (size_t)page, PROT_READ | PROT_WRITE,
                          MAP_FIXED | MAP_PRIVATE | MAP_ANON, -1, 0);
    return restored == MAP_FAILED ? errno : restored == (void *)base ? 0 : EINVAL;
}
