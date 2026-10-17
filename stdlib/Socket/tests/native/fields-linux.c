/* Independent ABI checks against the actual native glibc public header. */
#define _DEFAULT_SOURCE 1
#include <netinet/tcp.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>

#include "tcp-header-check.h"
_Static_assert(offsetof(struct tcp_info, tcpi_rto) == 8, "TCP scale byte ABI");
_Static_assert(offsetof(struct tcp_info, tcpi_total_retrans) == 100,
               "Reviewed TCP info prefix ABI");

int main(void) {
    int status = check_tcp_header();
    if (status) return status;
    for (uint32_t value = 0; value <= UINT16_MAX; ++value) {
        struct tcp_info info;
        memset(&info, 0xa5, sizeof(info));
        info.tcpi_snd_wscale = value;
        info.tcpi_rcv_wscale = value >> 4;
        if (((unsigned char *)&info)[6] != (value & 255))
            return 4;
        for (size_t i = 0; i < sizeof(info); ++i)
            if (i != 6 && ((unsigned char *)&info)[i] != 0xa5)
                return 5;
    }
    return 0;
}
