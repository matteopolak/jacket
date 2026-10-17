#include "bindings-aggregate.h"
extern "C" int original_packet(Packet packet) { return packet.first + packet.second; }
