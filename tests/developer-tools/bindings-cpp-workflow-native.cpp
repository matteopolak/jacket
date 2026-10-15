#include "bindings-cpp-workflow.h"
int original_cpp_workflow_sum(int left, int right) { return left + right; }
double original_cpp_workflow_scale(double value, double factor) { return value * factor; }
void original_cpp_workflow_bump(int *value) { *value += 1; }
long original_cpp_workflow_long(long value) { return value + 1; }
