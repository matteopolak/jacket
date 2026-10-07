// Windows: HWND surfaces for Window_Creation's windows (wgpu-native presents through D3D12,
// Vulkan or GL), and the system libraries the static wgpu_native.lib needs when `jaic build`
// links it. The library carries import stubs for most DLLs it calls; these are the rest.

#scope_module

Win32 :: #import "Windows";

// The executable's module handle, which WGPUSurfaceSourceWindowsHWND wants with the window.
webgpu_module_instance :: () -> *void {
    return Win32.GetModuleHandleW(null);
}

wgpu_d3dcompiler :: #system_library,link_always "d3dcompiler";
wgpu_ws2_32 :: #system_library,link_always "ws2_32";
wgpu_userenv :: #system_library,link_always "userenv";
wgpu_bcrypt :: #system_library,link_always "bcrypt";
wgpu_ntdll :: #system_library,link_always "ntdll";
wgpu_opengl32 :: #system_library,link_always "opengl32";
wgpu_propsys :: #system_library,link_always "propsys";
wgpu_runtimeobject :: #system_library,link_always "runtimeobject";
wgpu_user32 :: #system_library,link_always "user32";
wgpu_gdi32 :: #system_library,link_always "gdi32";
wgpu_advapi32 :: #system_library,link_always "advapi32";
wgpu_ole32 :: #system_library,link_always "ole32";
wgpu_oleaut32 :: #system_library,link_always "oleaut32";
