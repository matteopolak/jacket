#include "codeview-strings-native.h"
int original_codeview_strings(int value) {
    return original_codeview_header_transform(value) + 7;
}
