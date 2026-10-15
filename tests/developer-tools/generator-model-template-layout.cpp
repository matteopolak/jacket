#include <cstddef>
#include <cstdio>
#include "generator-model-templates.hpp"
#include "generator-model-literals.hpp"
using OriginalConcrete = OriginalGeneric<int, double>;
static_assert(sizeof(OriginalWideEnum) == 16);
static_assert(static_cast<unsigned __int128>(OriginalWideEnum::Large) > ~static_cast<unsigned long long>(0));
int main() {
    std::printf("Native_Template_Size :: %zu;\nNative_Template_Alignment :: %zu;\n",
                sizeof(OriginalConcrete), alignof(OriginalConcrete));
    std::printf("Native_Char16_Size :: %zu;\nNative_Char32_Size :: %zu;\nNative_Char16_Value :: %u;\nNative_Char32_Value :: %u;\n",
                sizeof(char16_t), sizeof(char32_t),
                static_cast<unsigned>(original_char16), static_cast<unsigned>(original_char32));
    std::printf("Native_WChar_Size :: %zu;\nNative_LongDouble_Size :: %zu;\n", sizeof(wchar_t), sizeof(long double));
    return 0;
}
