#include "bindings-function-wrappers.h"
int original_pointer_pair(struct OriginalWrapperPair *pair) { return pair->a + pair->b; }
int original_format_adapter(long long count, const char *data) {
    int result = 0;
    for (long long i = 0; i < count; ++i) result += (unsigned char)data[i];
    return result;
}
