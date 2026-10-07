// macOS: a CAMetalLayer as the backing layer of a window's content view, for
// WGPUSurfaceSourceMetalLayer. Plain objc_msgSend calls keep this independent of the AppKit bindings.

#scope_module

webgpu_metal_layer :: (window: Window_Type) -> *void {
    if !window return null;
    view := msg_ptr(window, sel("contentView"));
    if !view return null;
    layer_class := objc_getClass("CAMetalLayer");
    if !layer_class {
        log_error("WebGPU: CAMetalLayer is not available (QuartzCore is loaded with wgpu-native; create the instance first)");
        return null;
    }
    layer := msg_ptr(layer_class, sel("layer"));
    msg_set_bool(view, sel("setWantsLayer:"), 1);
    msg_set_ptr(view, sel("setLayer:"), layer);
    msg_set_f64(layer, sel("setContentsScale:"), msg_f64(window, sel("backingScaleFactor")));
    return layer;
}

// The content view's size in pixels (points times the backing scale).
webgpu_macos_drawable_size :: (window: Window_Type) -> width: s32, height: s32 {
    WC :: #import "Window_Creation";
    _, _, w, h := WC.get_dimensions(window, false);
    scale := msg_f64(window, sel("backingScaleFactor"));
    return cast(s32) (w * scale), cast(s32) (h * scale);
}

sel :: (name: string) -> *void {
    buffer: [128] u8;
    assert(name.count < buffer.count);
    memcpy(buffer.data, name.data, name.count);
    return sel_registerName(buffer.data);
}

objc_getClass :: (name: string) -> *void {
    buffer: [128] u8;
    assert(name.count < buffer.count);
    memcpy(buffer.data, name.data, name.count);
    return objc_getClass_c(buffer.data);
}

objc :: #system_library "libobjc";

sel_registerName :: (name: *u8) -> *void #foreign objc;
objc_getClass_c :: (name: *u8) -> *void #foreign objc "objc_getClass";
msg_ptr :: (receiver: *void, selector: *void) -> *void #foreign objc "objc_msgSend";
msg_f64 :: (receiver: *void, selector: *void) -> float64 #foreign objc "objc_msgSend";
msg_set_ptr :: (receiver: *void, selector: *void, value: *void) #foreign objc "objc_msgSend";
msg_set_bool :: (receiver: *void, selector: *void, value: s8) #foreign objc "objc_msgSend";
msg_set_f64 :: (receiver: *void, selector: *void, value: float64) #foreign objc "objc_msgSend";

// What the static libwgpu_native.a needs when `jaic build` links it.
wgpu_metal :: #system_library,link_always "Metal";
wgpu_quartz_core :: #system_library,link_always "QuartzCore";
wgpu_foundation :: #system_library,link_always "Foundation";
wgpu_core_foundation :: #system_library,link_always "CoreFoundation";
wgpu_iconv :: #system_library,link_always "iconv";
