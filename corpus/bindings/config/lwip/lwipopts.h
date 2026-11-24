/* Corpus configuration for lwIP: no OS layer, TCP, UDP and DHCP. */
#ifndef LWIPOPTS_H
#define LWIPOPTS_H
#define NO_SYS 1
#define LWIP_TCP 1
#define LWIP_UDP 1
#define LWIP_DHCP 1
#define LWIP_DNS 1
#define LWIP_ICMP 1
#define MEM_ALIGNMENT 4
#define MEM_SIZE 4096
#define TCP_WND 2048
#define TCP_MSS 536
#endif
