#include "imgui.h"
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <type_traits>

using Range = ImGuiTextFilter::ImGuiTextRange;
using Ranges = ImVector<Range>;
static_assert(std::is_standard_layout<ImGuiTextFilter>::value);
static_assert(std::is_standard_layout<Range>::value);
static_assert(std::is_standard_layout<Ranges>::value);

extern "C" uint64_t binding_imgui_size(unsigned record) {
    switch (record) {
        case 0: return sizeof(ImGuiTextFilter);
        case 1: return sizeof(Range);
        case 2: return sizeof(Ranges);
        default: std::abort();
    }
}
extern "C" uint64_t binding_imgui_offset(unsigned field) {
    switch (field) {
        case 0: return offsetof(ImGuiTextFilter, InputBuf);
        case 1: return offsetof(ImGuiTextFilter, Filters);
        case 2: return offsetof(ImGuiTextFilter, CountGrep);
        case 3: return offsetof(Range, e);
        case 4: return offsetof(Ranges, Data);
        default: std::abort();
    }
}
