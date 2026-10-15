#include "bindings-cpp-workflow.h"
int original_cpp_workflow_sum(int left, int right) { return left + right; }
double original_cpp_workflow_scale(double value, double factor) { return value * factor; }
void original_cpp_workflow_bump(int *value) { *value += 1; }
long original_cpp_workflow_long(long value) { return value + 1; }
Original_Cpp_Pair original_cpp_workflow_pair(int number, double scale) { return {number, scale}; }
double original_cpp_workflow_pair_sum(Original_Cpp_Pair value) { return value.number + value.scale; }
Original_Cpp_Pair &original_cpp_workflow_reference(Original_Cpp_Pair &value) { value.number += 1; return value; }
const Original_Cpp_Pair &original_cpp_workflow_const_reference(const Original_Cpp_Pair &value) { return value; }
Original_Cpp_Pair original_cpp_workflow_empty() { return {42, 2.5}; }
Original_Cpp_Pair original_cpp_workflow_collision(int out_result_0) { return {out_result_0, 3.5}; }
Original_Cpp_Small original_cpp_workflow_small(Original_Cpp_Small value) { return {value.right, value.left}; }
Original_Cpp_Large original_cpp_workflow_large(Original_Cpp_Large value) { return {value.third, value.first, value.second}; }
