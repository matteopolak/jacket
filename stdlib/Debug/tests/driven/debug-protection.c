#include <sys/mman.h>
#include <unistd.h>

int developer_debug_readable(const void *pointer);

int main(void) {
  long size = sysconf(_SC_PAGESIZE);
  if (size <= 0)
    return 1;
  void *page = mmap(NULL, (size_t)size, PROT_READ | PROT_WRITE,
                    MAP_PRIVATE | MAP_ANON, -1, 0);
  if (page == MAP_FAILED)
    return 2;
  if (!developer_debug_readable(page))
    return 3;
  if (mprotect(page, (size_t)size, PROT_NONE) != 0)
    return 4;
  if (developer_debug_readable(page))
    return 5;
  if (mprotect(page, (size_t)size, PROT_READ) != 0)
    return 6;
  if (!developer_debug_readable(page))
    return 7;
  if (munmap(page, (size_t)size) != 0)
    return 8;
  if (developer_debug_readable(page))
    return 9;
  return 0;
}
