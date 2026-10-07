// Linux: Xlib surfaces for Window_Creation's windows (wgpu-native presents through Vulkan, or
// GLES through EGL), and what the static libwgpu_native.a needs when `jaic build` links it.

#scope_module

X11 :: #import "X11";

// The display Window_Creation opened its windows on.
webgpu_x11_display :: () -> *void {
    if !X11.x_global_display log_error("WebGPU: no X display (create the window first)");
    return X11.x_global_display;
}

wgpu_libm :: #system_library,link_always "libm";
wgpu_libdl :: #system_library,link_always "libdl";
wgpu_libpthread :: #system_library,link_always "libpthread";
wgpu_libgcc_s :: #system_library,link_always "libgcc_s";
