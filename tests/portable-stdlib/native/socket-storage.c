#include <netdb.h>
#include <netinet/in.h>

void *portable_h_errno_address(void) { return &h_errno; }
const void *portable_any_address(void) { return &in6addr_any; }
const void *portable_loopback_address(void) { return &in6addr_loopback; }
int portable_h_errno_value(void) { return h_errno; }
