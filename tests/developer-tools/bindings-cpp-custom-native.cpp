#include "bindings-cpp-custom.h"
int original_cpp_custom(int left, int right) { return left * 10 + right; }
double original_cpp_custom(double left, double right) { return left * 10.0 + right; }
Original_Custom_Record original_cpp_record(int value) { return {value}; }
Original_Custom_Record &original_cpp_reference(Original_Custom_Record &value) { value.value += 1; return value; }
int original_cpp_omitted(int value) { return value + 99; }
