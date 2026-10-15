struct Original_Bridge_Pair { int left; int right; };
int original_bridge_add(int left, int right);
Original_Bridge_Pair original_bridge_pair(int seed);
int &original_bridge_reference();
int original_bridge_read_reference(const int &value);
void original_bridge_write_reference(int &value, int replacement);
extern "C" int original_bridge_c_linkage(int value);
[[deprecated("original omitted declaration")]] int original_bridge_deprecated();
struct Original_Bridge_Methods { int unsupported_method(); };
