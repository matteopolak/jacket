/* Independent native public socket.h macros; no copied library implementation. */
#include <sys/socket.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

int main(void) {
    uint32_t header[4] = {0x434d5347, 4097, sizeof(struct cmsghdr), sizeof(struct msghdr)};
    if (fwrite(header, sizeof(header), 1, stdout) != 1)
        return 1;
    union {
        struct cmsghdr alignment;
        unsigned char bytes[8192];
    } buffer;
    memset(&buffer, 0, sizeof(buffer));
    for (uint32_t size = 0; size < 4097; ++size) {
        struct msghdr message;
        memset(&message, 0, sizeof(message));
        message.msg_control = &buffer;
        message.msg_controllen = size;
        uint64_t values[4] = {
            CMSG_SPACE(size) - CMSG_SPACE(0), CMSG_LEN(size),
            CMSG_FIRSTHDR(&message) == (struct cmsghdr *)&buffer,
            (unsigned char *)CMSG_DATA((struct cmsghdr *)&buffer) - buffer.bytes
        };
        if (fwrite(values, sizeof(values), 1, stdout) != 1)
            return 2;
    }
    return 0;
}
