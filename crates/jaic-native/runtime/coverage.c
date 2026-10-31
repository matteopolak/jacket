// Coverage runtime for `jaic -coverage` (docs/tools/coverage.md): receives SanitizerCoverage's
// trace-pc-guard callbacks and, at exit, writes every block's address and how often it ran.
#define _GNU_SOURCE
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#ifdef __APPLE__
#include <mach-o/dyld.h>
#else
#include <link.h>
#endif

#define MAX_BLOCKS (1u << 25)
#define MAX_MODULES 64

// Counts stop at the largest value. Threads that enter a block together may lose a count: they
// do not synchronize, which would slow every block.
static uint32_t hits[MAX_BLOCKS];
static uint32_t next_id = 1;
static struct { uint32_t first, count; const uintptr_t *pcs; } modules[MAX_MODULES];
static int module_count, registered;
static char directory[1024];

#ifndef __APPLE__
static int first_image(struct dl_phdr_info *info, size_t size, void *data) {
    *(intptr_t *)data = (intptr_t)info->dlpi_addr;
    return 1;
}
#endif

static intptr_t image_slide(void) {
#ifdef __APPLE__
    return _dyld_get_image_vmaddr_slide(0);
#else
    intptr_t slide = 0;
    dl_iterate_phdr(first_image, &slide); // the first entry is the main program
    return slide;
#endif
}

static void executable_path(char *out, size_t size) {
#ifdef __APPLE__
    uint32_t n = (uint32_t)size;
    if (_NSGetExecutablePath(out, &n) != 0) out[0] = 0;
#else
    ssize_t n = readlink("/proc/self/exe", out, size - 1);
    out[n > 0 ? n : 0] = 0;
#endif
}

static void write_report(void) {
    char exe[1024];
    executable_path(exe, sizeof exe);
    const char *name = strrchr(exe, '/');
    name = name ? name + 1 : exe;
    char path[2048];
    const char *file = getenv("JAIC_COVERAGE_FILE");
    if (file && *file) snprintf(path, sizeof path, "%s", file);
    else snprintf(path, sizeof path, "%s/%s.%d.cov", directory, name, (int)getpid());
    FILE *out = fopen(path, "w");
    if (!out) return;
    intptr_t slide = image_slide();
    fprintf(out, "jaic-coverage 1\nexe %s\n", exe);
    for (int m = 0; m < module_count; m++) {
        if (!modules[m].pcs) continue;
        fprintf(out, "module %u\n", modules[m].count);
        for (uint32_t i = 0; i < modules[m].count; i++) {
            uintptr_t pc = modules[m].pcs[2 * i], flags = modules[m].pcs[2 * i + 1];
            fprintf(out, "%lx %d %u\n", (unsigned long)(pc - slide), (int)(flags & 1),
                    hits[modules[m].first + i]);
        }
    }
    fclose(out);
}

void __sanitizer_cov_trace_pc_guard_init(uint32_t *start, uint32_t *stop) {
    if (start == stop || *start) return;
    if (!registered) {
        registered = 1;
        const char *dir = getenv("JAIC_COVERAGE_DIR");
        if (dir && *dir) snprintf(directory, sizeof directory, "%s", dir);
        else if (!getcwd(directory, sizeof directory)) strcpy(directory, ".");
        atexit(write_report);
    }
    if (module_count == MAX_MODULES) return;
    uint32_t count = (uint32_t)(stop - start);
    if (next_id + count > MAX_BLOCKS) return;
    modules[module_count].first = next_id;
    modules[module_count].count = count;
    module_count++;
    for (uint32_t *g = start; g < stop; g++) *g = next_id++;
}

void __sanitizer_cov_trace_pc_guard(uint32_t *guard) {
    uint32_t *count = &hits[*guard];
    if (*count != UINT32_MAX) ++*count;
}

void __sanitizer_cov_pcs_init(const uintptr_t *beg, const uintptr_t *end) {
    // The table follows the guard array of the module registered last.
    if (module_count && !modules[module_count - 1].pcs
        && (uint32_t)((end - beg) / 2) == modules[module_count - 1].count)
        modules[module_count - 1].pcs = beg;
}
