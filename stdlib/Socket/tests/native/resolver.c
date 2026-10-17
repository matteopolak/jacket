/* Official host netdb/socket ABI and real numeric resolver, no DNS query. */
#include <arpa/inet.h>
#include <netdb.h>
#include <stddef.h>
#include <stdio.h>
#include <string.h>
int main(void) {
    struct addrinfo hints;
    memset(&hints, 0, sizeof(hints));
    hints.ai_flags = AI_NUMERICHOST | AI_NUMERICSERV;
    hints.ai_family = AF_INET;
    hints.ai_socktype = SOCK_STREAM;
    hints.ai_protocol = IPPROTO_TCP;
    struct addrinfo *result = NULL;
    int status = getaddrinfo("127.0.0.1", "32123", &hints, &result);
    if (status || !result) { fprintf(stderr, "numeric resolver failed: %d\n", status); return 1; }
    const struct sockaddr_in *address = (const struct sockaddr_in *)result->ai_addr;
    int valid = result->ai_family == AF_INET && result->ai_socktype == SOCK_STREAM &&
        result->ai_protocol == IPPROTO_TCP && result->ai_addrlen == sizeof(*address) &&
        address && ntohs(address->sin_port) == 32123 && ntohl(address->sin_addr.s_addr) == 0x7f000001u;
    printf("%zu %zu %zu %zu %zu %zu %zu %zu %zu %zu %u %u\n", sizeof(struct addrinfo),
        _Alignof(struct addrinfo), offsetof(struct addrinfo, ai_flags), offsetof(struct addrinfo, ai_family),
        offsetof(struct addrinfo, ai_socktype), offsetof(struct addrinfo, ai_protocol),
        offsetof(struct addrinfo, ai_addrlen), offsetof(struct addrinfo, ai_canonname),
        offsetof(struct addrinfo, ai_addr), offsetof(struct addrinfo, ai_next),
        address ? ntohs(address->sin_port) : 0, address ? ntohl(address->sin_addr.s_addr) : 0);
    freeaddrinfo(result);
    return valid && fflush(stdout) == 0 ? 0 : 2;
}
