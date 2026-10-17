#include "bindings-bridge.h"
static int original_bridge_storage = 13;
int original_bridge_add(int left, int right) { return left + right; }
Original_Bridge_Pair original_bridge_pair(int seed) { return {seed, seed + 1}; }
int &original_bridge_reference() { return original_bridge_storage; }
int original_bridge_read_reference(const int &value) { return value + 7; }
void original_bridge_write_reference(int &value, int replacement) { value = replacement; }
extern "C" int original_bridge_c_linkage(int value) { return value * 3; }
#include "bindings-bridge-overload.h"
int original_bridge_collision(int value) { return value; }
double original_bridge_collision(double value) { return value; }
