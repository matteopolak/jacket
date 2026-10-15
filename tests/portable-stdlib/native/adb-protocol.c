/* Independent SDK CLI protocol fixture. No device, server, keys or auth access. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static int arguments(int argc, char **argv, int count, const char **expected) {
    if (argc != count) return 0;
    for (int i = 0; i < count; ++i) if (strcmp(argv[i], expected[i])) return 0;
    return 1;
}

int main(int argc, char **argv) {
    int first = 1;
    if (argc > 2 && !strcmp(argv[1], "-s")) first = 3;
    if (first >= argc) return 90;
    const char *command = argv[first];
    const char *log_path = getenv("JAI_ADB_PROTOCOL_LOG");
    if (log_path) {
        FILE *log = fopen(log_path, "a");
        if (!log) return 99;
        fprintf(log, "%s\n", command);
        if (fclose(log)) return 99;
    }
    if (!strcmp(command, "start-server")) return strstr(argv[0], "start failure") ? 6 : 0;
    if (!strcmp(command, "echo-args") || (!strcmp(command, "shell") && argc > first + 1 && !strcmp(argv[first + 1], "echo-args"))) {
        for (int i = 1; i < argc; ++i) puts(argv[i]);
        return 0;
    }
    if (!strcmp(command, "fail")) {
        puts("partial");
        fputs("SDK failure\n", stderr);
        return 7;
    }
    if (!strcmp(command, "version")) {
        if (strstr(argv[0], "malformed")) { puts("not an SDK version"); return 0; }
        puts("Android Debug Bridge version 1.0.41\nVersion 35.0.2-test\nInstalled as /explicit/sdk/platform-tools/adb");
        return 0;
    }
    if (!strcmp(command, "devices")) {
        if (strstr(argv[0], "malformed")) { puts("List of devices attached\nbad device transport_id:not-a-number"); return 0; }
        fputs("List of devices attached\r\nready-1 device product:husky model:Pixel_8 device:husky transport_id:12\r\nunauth-2 unauthorized transport_id:13\r\noffline-3 offline transport_id:14\r\n", stdout);
        return 0;
    }
    if (!strcmp(command, "uninstall")) {
        const char *expected[] = {"-s", "selected serial", "uninstall", "org.example.app"};
        return arguments(argc - 1, argv + 1, 4, expected) ? 0 : 96;
    }
    if (!strcmp(command, "push")) {
        const char *expected[] = {"-s", "selected serial", "push", "--sync", "-n", "-z", "zstd", "local space", "/remote space"};
        return arguments(argc - 1, argv + 1, 9, expected) ? 0 : 91;
    }
    if (!strcmp(command, "pull")) {
        const char *expected[] = {"-s", "selected serial", "pull", "-a", "-Z", "/remote space", "local space"};
        return arguments(argc - 1, argv + 1, 7, expected) ? 0 : 92;
    }
    if (!strcmp(command, "shell") && argc > first + 1) {
        if (!strcmp(argv[first + 1], "mkdir")) {
            const char *expected[] = {"-s", "selected serial", "shell", "mkdir", "-p", "'/remote/a b'\"'\"'c'"};
            return arguments(argc - 1, argv + 1, 6, expected) ? 0 : 93;
        }
        if (!strcmp(argv[first + 1], "pidof")) {
            if (argc != first + 3) return 94;
            if (!strcmp(argv[first + 2], "'org.example.absent'")) return 1;
            if (!strcmp(argv[first + 2], "'org.example.offline'")) { fputs("device offline\n", stderr); return 1; }
            if (!strcmp(argv[first + 2], "'org.example.badpid'")) { puts("0x1234"); return 0; }
            if (!strcmp(argv[first + 2], "'org.example.overflow'")) { puts("2147483648"); return 0; }
            if (!strcmp(argv[first + 2], "'org.example.zero'")) { puts("0"); return 0; }
            if (!strcmp(argv[first + 2], "'org.example.badsecond'")) { puts("1234 nope"); return 0; }
            if (strcmp(argv[first + 2], "'org.example.app'")) return 94;
            puts("1234 5678");
            return 0;
        }
        if (!strcmp(argv[first + 1], "am") && argc == first + 4 && !strcmp(argv[first + 2], "force-stop")) {
            if (!strcmp(argv[first + 3], "'org.example.offline'")) { fputs("device offline\n", stderr); return 1; }
            return strcmp(argv[first + 3], "'org.example.app'") ? 97 : 0;
        }
        if (!strcmp(argv[first + 1], "pm") && argc == first + 4 && !strcmp(argv[first + 2], "path")) {
            if (!strcmp(argv[first + 3], "'org.example.absent'")) return 0;
            if (!strcmp(argv[first + 3], "'org.example.offline'")) { fputs("device offline\n", stderr); return 1; }
            if (!strcmp(argv[first + 3], "'org.example.badpath'")) { puts("unexpected response"); return 0; }
            if (strcmp(argv[first + 3], "'org.example.app'")) return 98;
            puts("package:/data/app/base.apk\npackage:/data/app/split_config.apk");
            return 0;
        }
    }
    return 95;
}
