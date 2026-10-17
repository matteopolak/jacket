// Real ImGui and libffi execution; this does not replace any SDK function.
#include "imgui.h"
#include <cassert>
#include <cstdint>
#include <string>

struct binding_argument { uint32_t kind, reserved; uint64_t value; };
static_assert(sizeof(binding_argument) == 16);
extern "C" int binding_ffi_call(void *, int, uint32_t, binding_argument *, uint32_t, uint64_t *);

static std::string clipboard;
static void set_clipboard(void *, const char *text) { clipboard = text; }
static uint64_t pointer(const void *value) { return reinterpret_cast<uint64_t>(value); }

int main() {
    ImGui::CreateContext();
    ImGuiIO &io = ImGui::GetIO();
    io.IniFilename = nullptr;
    io.DisplaySize = ImVec2(640, 480);
    io.DeltaTime = 1.0f / 60.0f;
    io.SetClipboardTextFn = set_clipboard;
    unsigned char *pixels;
    int width, height;
    io.Fonts->GetTexDataAsRGBA32(&pixels, &width, &height);
    assert(pixels && width > 0 && height > 0);
    ImGui::NewFrame();
    ImGui::SetNextWindowSize(ImVec2(400, 300));
    ImGui::SetNextWindowPos(ImVec2(0, 0));
    ImGui::Begin("native bridge");
    ImGui::LogToClipboard();
    binding_argument arguments[] = {
        {6, 0, pointer("%d:%s")},
        {1, 0, static_cast<uint64_t>(-17)},
        {6, 0, pointer("hello")},
    };
    uint64_t result = 0;
    assert(binding_ffi_call(reinterpret_cast<void *>(&ImGui::Text), 2, 1,
                            arguments, 3, &result) == 0);
    ImVec4 color(0.2f, 0.4f, 0.6f, 1.0f);
    binding_argument colored[] = {
        {6, 0, pointer(&color)}, {6, 0, pointer("color:%d")}, {1, 0, 23},
    };
    assert(binding_ffi_call(reinterpret_cast<void *>(&ImGui::TextColored), 2, 2,
                            colored, 3, &result) == 0);
    ImGui::LogFinish();
    assert(clipboard.find("-17:hello") != std::string::npos);
    assert(clipboard.find("color:23") != std::string::npos);

    using TreeNode = bool (*)(const char *, const char *, ...);
    auto tree = static_cast<TreeNode>(&ImGui::TreeNode);
    binding_argument node[] = {
        {6, 0, pointer("open")}, {6, 0, pointer("node:%d")}, {1, 0, 31},
    };
    ImGui::SetNextItemOpen(true, ImGuiCond_Always);
    assert(binding_ffi_call(reinterpret_cast<void *>(tree), 3, 2, node, 3, &result) == 0);
    assert(result == 1);
    ImGui::TreePop();
    node[0].value = pointer("closed");
    ImGui::SetNextItemOpen(false, ImGuiCond_Always);
    assert(binding_ffi_call(reinterpret_cast<void *>(tree), 3, 2, node, 3, &result) == 0);
    assert(result == 0);
    ImGui::End();
    ImGui::Render();
    auto data = ImGui::GetDrawData();
    assert(data && data->TotalVtxCount > 0 && data->TotalIdxCount > 0);
    ImGui::DestroyContext();
}
