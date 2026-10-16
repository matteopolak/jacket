extern "C" int original_c_value(void);
namespace DriverCPP {
struct Pair { int first; int second; };
int sum(int value, int extra = 2);
Pair make_pair(int value);
}
