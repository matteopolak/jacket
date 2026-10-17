#include <winsock2.h>
#include <ws2tcpip.h>

_Static_assert(sizeof(INT) == 4, "Winsock INT ABI");
_Static_assert(sizeof(void *) == 8, "Windows 64-bit target required");

int main(void) {
    unsigned char ipv4[4] = {0};
    unsigned char ipv6[16] = {0};
    if (InetPtonA(AF_INET, "127.0.0.1", ipv4) != 1) return 1;
    if (ipv4[0] != 127 || ipv4[1] || ipv4[2] || ipv4[3] != 1) return 2;
    if (inet_pton(AF_INET6, "::1", ipv6) != 1) return 3;
    for (int i = 0; i < 15; ++i) if (ipv6[i]) return 4;
    if (ipv6[15] != 1) return 5;
    if (inet_pton(AF_INET, "invalid-address", ipv4) != 0) return 6;
    return 0;
}
