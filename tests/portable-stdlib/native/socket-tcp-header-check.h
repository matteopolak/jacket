/* Independent whole-header ABI checks shared by official native SDK probes. */
#include <stdint.h>
#include <string.h>

_Static_assert(sizeof(struct tcphdr) == 20, "TCP header ABI");

static int check_tcp_header(void) {
    for (uint32_t value = 0; value <= UINT16_MAX; ++value) {
        struct tcphdr packet;
        unsigned char *bytes = (unsigned char *)&packet;
        memset(&packet, 0xa5, sizeof(packet));
        packet.th_x2 = value;
        packet.th_off = value >> 4;
        if (bytes[12] != (value & 255) || bytes[13] != 0xa5)
            return 1;
        packet.res1 = value;
        packet.doff = value >> 4;
        packet.fin = value >> 8;
        packet.syn = value >> 9;
        packet.rst = value >> 10;
        packet.psh = value >> 11;
        packet.ack = value >> 12;
        packet.urg = value >> 13;
        packet.res2 = value >> 14;
        uint16_t word;
        memcpy(&word, bytes + 12, sizeof(word));
        if (word != value)
            return 2;
        for (size_t i = 0; i < sizeof(packet); ++i)
            if (i != 12 && i != 13 && bytes[i] != 0xa5)
                return 3;
    }
    return 0;
}
