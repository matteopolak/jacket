#include <errno.h>
#include <stdio.h>
#include <sys/mman.h>
#include <unistd.h>

int main(void) {
    long page = sysconf(_SC_PAGESIZE);
    if (page <= 0) return 2;
    void *mapping = mmap(NULL, (size_t)page, PROT_READ | PROT_WRITE,
                         MAP_PRIVATE | MAP_ANON, -1, 0);
    if (mapping == MAP_FAILED) return 3;
    if (mprotect(mapping, 1, PROT_READ | PROT_WRITE) != 0) return 4;
    if (munmap(mapping, (size_t)page) != 0) return 5;
    char residency = 0;
    int resident_status = mincore(mapping, 1, &residency);
    printf("unmapped mincore status=%d residency=%d\n", resident_status, residency);
    errno = 0;
    if (mprotect(mapping, 1, PROT_READ | PROT_WRITE) == 0 || errno != ENOMEM) return 6;
    puts("actual SDK unmapped mprotect rejects ENOMEM");
    return 0;
}
