/* Actual SDK-linked generated C++ adapters, independent of the Jai consumer. */
#include "bindings-cpp-custom.h"
#include <cassert>
extern "C" {
int custom_integer(int a, int b);
double custom_floating(double a, double b);
void make_custom_record(Original_Custom_Record *out, int value);
Original_Custom_Record *reference_custom_record(Original_Custom_Record *value);
}
int main() {
    assert(custom_integer(10, 3) == 103);
    assert(custom_floating(1.25, 0.5) == 13.0);
    Original_Custom_Record value{};
    make_custom_record(&value, 41);
    assert(value.value == 41);
    auto *pointer = reference_custom_record(&value);
    assert(pointer == &value && value.value == 42);
}
