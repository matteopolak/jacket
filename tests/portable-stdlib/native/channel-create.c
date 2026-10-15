/* Original SDK diagnostic: never open/read an already existing shared object. */
#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include <unistd.h>

static int probe(const char *name) {
    int fd = shm_open(name, O_RDWR | O_CREAT | O_EXCL, 0600);
    if (fd < 0) {
        printf("exclusive create %s: errno=%d\n", name, errno);
        return 1;
    }
    int status = 0;
    struct stat info;
    if (ftruncate(fd, 8192) != 0 || fstat(fd, &info) != 0) {
        printf("extent setup: errno=%d\n", errno);
        status = 1;
    } else {
        void *mapping = mmap(NULL, 8192, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);
        if (mapping == MAP_FAILED) {
            printf("mapping: errno=%d\n", errno);
            status = 1;
        } else {
            printf("created %s: bytes=%lld\n", name, (long long)info.st_size);
            if (munmap(mapping, 8192) != 0) status = 1;
        }
    }
    /* Only this probe's successfully acquired exclusive ownership is released. */
    int closed = close(fd);
    int unlinked = shm_unlink(name);
    if (closed != 0 || unlinked != 0) status = 1;
    return status;
}

int main(void) {
    char name[31];
    int count = snprintf(name, sizeof(name), "/cleanroom-ci-%ld", (long)getpid());
    if (count < 0 || count >= (int)sizeof(name)) return 2;
    int unique = probe(name);
    int canonical = probe("/Jai Memory Visualization");
    return unique || canonical;
}
