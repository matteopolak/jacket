/* Compile against the explicitly supplied official bionic bits/tcphdr.h. */
#include PORTABLE_BIONIC_TCP_HEADER
#include "socket-tcp-header-check.h"

int main(void) {
    return check_tcp_header();
}
