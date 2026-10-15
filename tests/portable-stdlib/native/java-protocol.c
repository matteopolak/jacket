/* Independent launcher-status fixture, never presented as a JVM implementation. */
#include <stdio.h>
#include <string.h>
int main(int argc, char **argv) {
    if (argc != 2 || strcmp(argv[1], "-version")) return 91;
    fputs("independent launcher fixture\n", stderr);
    return strstr(argv[0], "failing") ? 7 : 0;
}
