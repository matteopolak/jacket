#include "developer-tools-canonical-bridge.cpp"
int main() {
    if (original_c_value() != 19) return 3;
    if (DriverCPP_sum(40) != 42 || DriverCPP_sum(30, 12) != 42) return 1;
    DriverCPP::Pair value;
    DriverCPP_make_pair(&value, 20);
    if (value.first != 20 || value.second != 21) return 2;
}
