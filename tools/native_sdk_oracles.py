"""Independently authored SDK probes; no downloaded build script executes."""

IMGUI = r'''
#include "imgui.h"
#include <cstddef>
#include <cstdio>
#include <cstring>
#include <type_traits>
static_assert(IMGUI_VERSION_NUM == 19040);
#ifndef IMGUI_HAS_DOCK
#error The selected Vk-Engine bindings require the docking source variant
#endif
static_assert(sizeof(void*) == 8 && sizeof(ImDrawIdx) == 2);
static_assert(std::is_same_v<decltype(&ImGui::CreateContext), ImGuiContext*(*)(ImFontAtlas*)>);
static_assert(std::is_same_v<decltype(&ImGui::DestroyContext), void(*)(ImGuiContext*)>);
static_assert(std::is_same_v<decltype(&ImGui::GetVersion), const char*(*)()>);
static_assert(std::is_same_v<decltype(&ImGui::NewFrame), void(*)()>);
static_assert(std::is_same_v<decltype(&ImGui::Render), void(*)()>);
static_assert(std::is_same_v<decltype(&ImGui::GetDrawData), ImDrawData*(*)()>);
static_assert(std::is_same_v<decltype(&ImGui::DockSpaceOverViewport), ImGuiID(*)(const ImGuiViewport*,ImGuiDockNodeFlags,const ImGuiWindowClass*)>);
static_assert(std::is_same_v<decltype(&ImDrawList::AddRectFilled), void(ImDrawList::*)(const ImVec2&,const ImVec2&,ImU32,float,ImDrawFlags)>);
int main() {
    if (std::strcmp(ImGui::GetVersion(), "1.90.4")) return 11;
    if (!IMGUI_CHECKVERSION()) return 12;
    auto* context=ImGui::CreateContext();
    if (!context || ImGui::GetCurrentContext()!=context) return 13;
    auto& io=ImGui::GetIO();
    io.IniFilename=nullptr; io.LogFilename=nullptr;
    io.DisplaySize=ImVec2(640,480); io.DeltaTime=1.0f/60.0f;
    io.ConfigFlags |= ImGuiConfigFlags_DockingEnable;
    unsigned char* pixels=nullptr; int width=0,height=0;
    io.Fonts->GetTexDataAsRGBA32(&pixels,&width,&height);
    if (!pixels || width<=0 || height<=0) return 14;
    ImGui::NewFrame();
    ImGui::DockSpaceOverViewport();
    auto* list=ImGui::GetForegroundDrawList();
    list->AddRectFilled(ImVec2(10,10),ImVec2(52,52),IM_COL32(255,255,255,255));
    ImGui::Render();
    auto* draw=ImGui::GetDrawData();
    if (!draw || !draw->Valid || draw->TotalVtxCount<4 || draw->TotalIdxCount<6) return 15;
    std::printf("ImVec2 %zu %zu %zu %zu\n",sizeof(ImVec2),alignof(ImVec2),offsetof(ImVec2,x),offsetof(ImVec2,y));
    std::printf("ImDrawVert %zu %zu %zu %zu %zu\n",sizeof(ImDrawVert),alignof(ImDrawVert),offsetof(ImDrawVert,pos),offsetof(ImDrawVert,uv),offsetof(ImDrawVert,col));
    std::printf("ImVectorPointer %zu %zu %zu %zu %zu\n",sizeof(ImVector<void*>),alignof(ImVector<void*>),offsetof(ImVector<void*>,Size),offsetof(ImVector<void*>,Capacity),offsetof(ImVector<void*>,Data));
    std::printf("ImGuiIO %zu %zu\n",sizeof(ImGuiIO),alignof(ImGuiIO));
    std::printf("ImGuiStyle %zu %zu\n",sizeof(ImGuiStyle),alignof(ImGuiStyle));
    std::printf("cpu_draw_counts %d %d\n",draw->TotalVtxCount,draw->TotalIdxCount);
    ImGui::DestroyContext(context);
    if (ImGui::GetCurrentContext()) return 16;
}
'''

FOCUS = r'''
#import "LightweightRenderingView.h"
#import <objc/runtime.h>
#include <cstdio>
#include <cstddef>
#include <cstring>
int main() {
    @autoreleasepool {
        Class base=[LightweightRenderingView class];
        Class view_class=[LightweightOpenGLView class];
        if (class_getSuperclass(view_class)!=base || class_getSuperclass(base)!=[NSView class]) return 11;
        Method setter=class_getInstanceMethod(view_class,@selector(setGlContext:));
        Method getter=class_getInstanceMethod(view_class,@selector(glContext));
        Method swap_method=class_getInstanceMethod(view_class,@selector(swapBuffers));
        if (!setter || !getter || method_getNumberOfArguments(setter)!=3 || method_getNumberOfArguments(getter)!=2) return 12;
        if (!swap_method || method_getNumberOfArguments(swap_method)!=2) return 20;
        char type[64];
        method_getReturnType(setter,type,sizeof(type)); if(std::strcmp(type,"v")) return 13;
        method_getArgumentType(setter,2,type,sizeof(type)); if(std::strcmp(type,"@")) return 14;
        method_getReturnType(getter,type,sizeof(type)); if(std::strcmp(type,"@")) return 15;
        method_getReturnType(swap_method,type,sizeof(type)); if(std::strcmp(type,"v")) return 21;
        Ivar context=class_getInstanceVariable(view_class,"gl_context");
        if (!context || ivar_getOffset(context)<class_getInstanceSize(base)) return 16;
        // Raw runtime storage avoids initializing a window or graphics context.
        id view=class_createInstance(view_class,0);
        auto get=reinterpret_cast<id(*)(id,SEL)>(method_getImplementation(getter));
        auto set=reinterpret_cast<void(*)(id,SEL,id)>(method_getImplementation(setter));
        if(get(view,@selector(glContext))!=nil) return 17;
        NSObject* token=[[NSObject alloc] init];
        set(view,@selector(setGlContext:),token);
        if(get(view,@selector(glContext))!=token) return 18;
        set(view,@selector(setGlContext:),nil);
        if(get(view,@selector(glContext))!=nil) return 19;
        [token release];
        auto swap=reinterpret_cast<void(*)(id,SEL)>(method_getImplementation(swap_method));
        swap(view,@selector(swapBuffers));
        object_dispose(view);
        std::printf("NSRect %zu %zu %zu %zu\n",sizeof(NSRect),alignof(NSRect),offsetof(NSRect,origin),offsetof(NSRect,size));
        std::printf("class_storage %zu %zu %zu\n",class_getInstanceSize(base),class_getInstanceSize(view_class),static_cast<size_t>(ivar_getOffset(context)));
        std::puts("selectors object-getter object-setter void-swap");
    }
}
'''
