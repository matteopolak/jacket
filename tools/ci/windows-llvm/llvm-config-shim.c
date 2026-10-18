/*
 * llvm-config-23.exe: a front for the official Windows LLVM release's llvm-config.exe.
 *
 * The release's `llvm-config --system-libs` names its static zstd by the absolute path it had on
 * LLVM's build machine (`S:/llvm/.../zstd_static.lib`). llvm-sys hands every entry to Cargo as
 * `rustc-link-lib=<name>`, and rustc reads `S:/...` as "rename library S", which fails the build.
 * This shim runs the real llvm-config next to it and, for `--system-libs` only, rewrites each
 * path-qualified `*.lib` entry to its bare file name, which the linker then finds through the
 * LLVM `lib/` search path (tools/windows-llvm/prepare.sh puts a zstd_static.lib there).
 *
 * llvm-sys 231 (LLVM 23.1) tries `llvm-config-23.exe` in `$LLVM_SYS_231_PREFIX/bin` before `llvm-config.exe`,
 * so installing the shim under that name is enough; nothing else changes.
 */
#include <fcntl.h>
#include <io.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <windows.h>

static int wants_system_libs(int argc, char **argv) {
    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--system-libs") == 0) return 1;
    }
    return 0;
}

/* Writes `word` (length `n`), dropping any directory part when it is a path to a .lib file. */
static void emit_word(const char *word, size_t n, int strip) {
    if (strip && n > 4 && _strnicmp(word + n - 4, ".lib", 4) == 0) {
        size_t start = 0;
        for (size_t i = 0; i < n; i++) {
            if (word[i] == '/' || word[i] == '\\') start = i + 1;
        }
        word += start;
        n -= start;
    }
    fwrite(word, 1, n, stdout);
}

int main(int argc, char **argv) {
    char self[MAX_PATH];
    DWORD len = GetModuleFileNameA(NULL, self, MAX_PATH);
    if (len == 0 || len >= MAX_PATH) {
        fprintf(stderr, "llvm-config shim: cannot locate itself\n");
        return 1;
    }
    char *slash = strrchr(self, '\\');
    if (!slash) slash = strrchr(self, '/');
    if (!slash) {
        fprintf(stderr, "llvm-config shim: unexpected path %s\n", self);
        return 1;
    }
    slash[1] = 0;

    /* cmd.exe strips the outermost pair of quotes, so the whole line gets an extra pair. */
    size_t cap = strlen(self) + 64;
    for (int i = 1; i < argc; i++) cap += strlen(argv[i]) + 3;
    char *line = malloc(cap);
    if (!line) return 1;
    snprintf(line, cap, "\"\"%sllvm-config.exe\"", self);
    for (int i = 1; i < argc; i++) {
        strcat(line, " \"");
        strcat(line, argv[i]);
        strcat(line, "\"");
    }
    strcat(line, "\"");

    FILE *child = _popen(line, "rb");
    if (!child) {
        fprintf(stderr, "llvm-config shim: cannot run %s\n", line);
        return 1;
    }

    /* Pass the child's bytes through unchanged: no \n -> \r\n translation on top of its own. */
    _setmode(_fileno(stdout), _O_BINARY);
    int strip = wants_system_libs(argc, argv);
    char word[4096];
    size_t n = 0;
    int c;
    while ((c = fgetc(child)) != EOF) {
        if (c == ' ' || c == '\t' || c == '\r' || c == '\n') {
            emit_word(word, n, strip);
            n = 0;
            fputc(c, stdout);
        } else if (n < sizeof word) {
            word[n++] = (char)c;
        }
    }
    emit_word(word, n, strip);
    fflush(stdout);

    int status = _pclose(child);
    free(line);
    return status;
}
