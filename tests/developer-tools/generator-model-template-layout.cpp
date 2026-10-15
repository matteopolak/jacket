#include <cstddef>
#include <cstdio>
#include "generator-model-templates.hpp"
using OriginalConcrete = OriginalGeneric<int, double>;
int main() {
    std::printf("Native_Template_Size :: %zu;\nNative_Template_Alignment :: %zu;\n",
                sizeof(OriginalConcrete), alignof(OriginalConcrete));
    return 0;
}
