/* Apple public SDK layouts and real local database operations. */
#include <arpa/inet.h>
#include <netdb.h>
#include <stddef.h>
#include <stdio.h>
#include <string.h>

int main(void) {
    printf("hostent %zu %zu %zu %zu %zu %zu\n", sizeof(struct hostent),
           offsetof(struct hostent, h_name), offsetof(struct hostent, h_aliases),
           offsetof(struct hostent, h_addrtype), offsetof(struct hostent, h_length),
           offsetof(struct hostent, h_addr_list));
    printf("netent %zu %zu %zu %zu %zu\n", sizeof(struct netent),
           offsetof(struct netent, n_name), offsetof(struct netent, n_aliases),
           offsetof(struct netent, n_addrtype), offsetof(struct netent, n_net));
    printf("protoent %zu %zu %zu %zu\n", sizeof(struct protoent),
           offsetof(struct protoent, p_name), offsetof(struct protoent, p_aliases),
           offsetof(struct protoent, p_proto));
    printf("servent %zu %zu %zu %zu %zu\n", sizeof(struct servent),
           offsetof(struct servent, s_name), offsetof(struct servent, s_aliases),
           offsetof(struct servent, s_port), offsetof(struct servent, s_proto));
    printf("rpcent %zu %zu %zu %zu\n", sizeof(struct rpcent),
           offsetof(struct rpcent, r_name), offsetof(struct rpcent, r_aliases),
           offsetof(struct rpcent, r_number));
    struct hostent *host = gethostbyname("127.0.0.1");
    if (!host || host->h_addrtype != AF_INET || host->h_length != 4 ||
        !host->h_addr_list || !host->h_addr_list[0]) return 1;
    const unsigned char expected[] = {127, 0, 0, 1};
    if (memcmp(host->h_addr_list[0], expected, sizeof(expected))) return 2;
    struct protoent *protocol = getprotobyname("tcp");
    if (!protocol || protocol->p_proto != IPPROTO_TCP) return 3;
    struct servent *service = getservbyname("http", "tcp");
    if (!service || ntohs(service->s_port) != 80 || strcmp(service->s_proto, "tcp")) return 4;
    return fflush(stdout) == 0 ? 0 : 5;
}
