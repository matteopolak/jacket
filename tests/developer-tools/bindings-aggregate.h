#ifndef ORIGINAL_AGGREGATE_H
#define ORIGINAL_AGGREGATE_H
typedef struct Packet { int first; int second; } Packet;
#ifdef __cplusplus
extern "C" {
#endif
int original_packet(Packet packet);
#ifdef __cplusplus
}
namespace Native { using Packet = ::Packet; }
#endif
#endif
