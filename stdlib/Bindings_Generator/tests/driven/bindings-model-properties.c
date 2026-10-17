#include "bindings-model-properties.h"
static int count;
Property_Void inspect_flags(Property_Bool enabled) { count += enabled ? 1 : 3; }
int properties_count(void) { return count; }
