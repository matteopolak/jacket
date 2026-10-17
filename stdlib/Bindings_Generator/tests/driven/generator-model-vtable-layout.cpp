#include <cstdio>
#include <cstring>
#include "generator-model-vtables.hpp"
int main() {
    OriginalVirtualBase base;
    OriginalVirtualDerived derived;
    const unsigned char *table = nullptr;
    std::memcpy(&table, static_cast<const void *>(&derived), sizeof(table));
    using Entry = int (*)(void *, int);
    Entry entries[4];
    std::memcpy(entries, table, sizeof(entries));
    if (entries[0](&derived, 5) != derived.first(5) ||
        entries[1](&derived, 5) != static_cast<OriginalVirtualBase *>(&derived)->hidden(5) ||
        entries[2](&derived, 5) != static_cast<const OriginalVirtualDerived &>(derived).hidden(5) ||
        entries[3](&derived, 5) != derived.last(5)) return 1;
    std::printf("Native_Virtual_Base_Size :: %zu;\nNative_Virtual_Derived_Size :: %zu;\nNative_Virtual_Alignment :: %zu;\n",
                sizeof(base), sizeof(derived), alignof(OriginalVirtualDerived));
    std::printf("Native_Virtual_Value_Offset :: %td;\nNative_Virtual_Other_Offset :: %td;\n",
                reinterpret_cast<unsigned char *>(&derived.value) - reinterpret_cast<unsigned char *>(&derived),
                reinterpret_cast<unsigned char *>(&derived.other) - reinterpret_cast<unsigned char *>(&derived));
    OriginalVirtualAligned aligned;
    std::printf("Native_Empty_Record_Size :: %zu;\n", sizeof(OriginalEmptyRecord));
    std::printf("Native_Aligned_Size :: %zu;\nNative_Aligned_Alignment :: %zu;\nNative_Aligned_Value_Offset :: %td;\n",
                sizeof(aligned), alignof(OriginalVirtualAligned),
                reinterpret_cast<unsigned char *>(&aligned.value) - reinterpret_cast<unsigned char *>(&aligned));
}
