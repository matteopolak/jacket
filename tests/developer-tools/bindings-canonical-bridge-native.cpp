#include "bindings-canonical-bridge.hpp"
extern "C" int original_c_value(void) { return 19; }
int DriverCPP::sum(int value, int extra) { return value + extra; }
DriverCPP::Pair DriverCPP::make_pair(int value) { return {value, value + 1}; }
