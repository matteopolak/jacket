#include "imgui.h"
#include <type_traits>
static_assert(IMGUI_VERSION_NUM == 19040);
#ifndef IMGUI_HAS_DOCK
#error This fixture requires the pinned docking variant.
#endif
static_assert(sizeof(ImVec2) == 8 && alignof(ImVec2) == 4);
static_assert(std::is_same_v<decltype(&ImDrawList::AddRectFilled), void(ImDrawList::*)(const ImVec2&,const ImVec2&,ImU32,float,ImDrawFlags)>);
extern "C" void* jai_imgui_prepare() {
    ImGui::CreateContext();
    auto& io = ImGui::GetIO();
    io.IniFilename = nullptr;
    io.LogFilename = nullptr;
    io.DisplaySize = ImVec2(640,480);
    io.DeltaTime = 1.0f / 60.0f;
    io.ConfigFlags |= ImGuiConfigFlags_DockingEnable;
    unsigned char* pixels; int width, height;
    io.Fonts->GetTexDataAsRGBA32(&pixels,&width,&height);
    ImGui::NewFrame();
    ImGui::DockSpaceOverViewport();
    return ImGui::GetForegroundDrawList();
}
extern "C" int jai_imgui_finish() {
    ImGui::Render();
    const auto* data = ImGui::GetDrawData();
    const bool valid = data && data->Valid && data->TotalVtxCount == 12 && data->TotalIdxCount == 18;
    ImGui::DestroyContext();
    return valid ? 42 : 1;
}
