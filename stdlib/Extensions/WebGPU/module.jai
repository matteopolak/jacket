// WebGPU through the standard C API (webgpu.h). generated.jai holds every type and procedure,
// generated from webgpu-headers' webgpu.yml by tools/webgpu_gen.py. Natively the procedures call
// wgpu-native (tools/fetch_wgpu_native.py); in the browser playground the sandbox forwards them to
// the page's WebGPU (crates/jai-wasm/js/webgpu_host.mjs). This file adds what webgpu.h leaves to
// the platform: a surface for a window, waiting for asynchronous requests, and frame pacing.

#import "Basic";
#import "Window_Type";

#load "generated.jai";

#if OS == .WASM {
    #load "wasm.jai";
    #load "generated_wasm.jai";
} else #if OS == .MACOS {
    #load "macos.jai";
} else #if OS == .LINUX {
    #load "linux.jai";
} else #if OS == .WINDOWS {
    #load "windows.jai";
}

// wgpu-native, from artifacts/native-libs (see docs/stdlib/webgpu.md). Unused in the browser.
libwgpu :: #library "libwgpu_native";

// Whether WebGPU can be used here. Natively wgpu-native is linked, so this is true (an adapter
// may still be missing: webgpu_request_adapter returns null then). In the browser it is false
// when the page has no WebGPU or cannot suspend the program while it waits (JSPI); a program
// can then do something else instead of failing at its first wgpu call.
webgpu_available :: () -> bool {
    #if OS == .WASM {
        name :: "jai_webgpu_available";
        if !jai_host_provides(name.data, cast(u64) name.count) return false;
        return jai_webgpu_available() != 0;
    } else {
        return true;
    }
}

// --- Strings ----------------------------------------------------------------------------------

to_string_view :: (s: string) -> WGPUStringView {
    return .{ data = s.data, length = cast(u64) s.count };
}

// A view the implementation wrote (an adapter's name, an error message), as a Jai string that
// borrows its bytes.
to_string :: (view: WGPUStringView) -> string {
    if view.data == null return "";
    if view.length != WGPU_STRLEN return string.{ cast(s64) view.length, view.data };
    count := 0;
    while view.data[count] count += 1;
    return string.{ count, view.data };
}

// --- Waiting ----------------------------------------------------------------------------------

// Process events until `done` is set (by a callback with mode .AllowProcessEvents or
// .AllowSpontaneous). Natively this polls wgpu-native and sleeps a little between polls once
// the wait gets long; in the browser each poll gives the page a turn (and other Jai threads run
// meanwhile, see docs/stdlib/webgpu.md).
webgpu_wait :: (instance: WGPUInstance, done: *bool) {
    polls := 0;
    while !done.* {
        wgpuInstanceProcessEvents(instance);
        #if OS != .WASM {
            polls += 1;
            if !done.* && polls > 100 sleep_milliseconds(1);
        }
    }
}

// Request an adapter and wait for it; null when there is none.
webgpu_request_adapter :: (instance: WGPUInstance, options: *WGPURequestAdapterOptions = null) -> WGPUAdapter {
    state: Request_State;
    info: WGPURequestAdapterCallbackInfo;
    info.mode = .AllowProcessEvents;
    info.callback = (status: WGPURequestAdapterStatus, adapter: WGPUAdapter, message: WGPUStringView, userdata1: *void, userdata2: *void) #c_call {
        state := cast(*Request_State) userdata1;
        state.result = adapter;
        if status != .Success copy_message(*state.message, message);
        state.done = true;
    };
    info.userdata1 = *state;
    wgpuInstanceRequestAdapter(instance, options, info);
    webgpu_wait(instance, *state.done);
    if !state.result log_error("WebGPU: no adapter: %", message_text(*state.message));
    return cast(WGPUAdapter) state.result;
}

// Request a device and wait for it; null on failure. Uncaptured errors are printed unless the
// descriptor sets its own callback.
webgpu_request_device :: (instance: WGPUInstance, adapter: WGPUAdapter, descriptor: *WGPUDeviceDescriptor = null) -> WGPUDevice {
    desc: WGPUDeviceDescriptor;
    if descriptor desc = descriptor.*;
    if !desc.uncapturedErrorCallbackInfo.callback {
        desc.uncapturedErrorCallbackInfo.callback = (device: *WGPUDevice, type: WGPUErrorType, message: WGPUStringView, userdata1: *void, userdata2: *void) #c_call {
            push_context, defer_pop webgpu_callback_context;
            log_error("WebGPU % error: %", type, to_string(message));
        };
    }
    state: Request_State;
    info: WGPURequestDeviceCallbackInfo;
    info.mode = .AllowProcessEvents;
    info.callback = (status: WGPURequestDeviceStatus, device: WGPUDevice, message: WGPUStringView, userdata1: *void, userdata2: *void) #c_call {
        state := cast(*Request_State) userdata1;
        state.result = device;
        if status != .Success copy_message(*state.message, message);
        state.done = true;
    };
    info.userdata1 = *state;
    webgpu_callback_context = context;
    wgpuAdapterRequestDevice(adapter, *desc, info);
    webgpu_wait(instance, *state.done);
    if !state.result log_error("WebGPU: no device: %", message_text(*state.message));
    return cast(WGPUDevice) state.result;
}

// Map `size` bytes of `buffer` at `offset` for reading (.Read) or writing (.Write) and wait
// until they are; then wgpuBufferGetConstMappedRange / wgpuBufferGetMappedRange give the bytes
// and wgpuBufferUnmap ends it. False (and the reason logged) when mapping failed.
webgpu_map_buffer :: (instance: WGPUInstance, buffer: WGPUBuffer, mode: WGPUMapMode, offset: u64 = 0, size: u64 = WGPU_WHOLE_MAP_SIZE) -> bool {
    state: Map_State;
    info: WGPUBufferMapCallbackInfo;
    info.mode = .AllowProcessEvents;
    info.callback = (status: WGPUMapAsyncStatus, message: WGPUStringView, userdata1: *void, userdata2: *void) #c_call {
        state := cast(*Map_State) userdata1;
        state.status = status;
        copy_message(*state.message, message);
        state.done = true;
    };
    info.userdata1 = *state;
    // wgpu-native rejects WGPU_WHOLE_MAP_SIZE ("must be multiple of 4"): pass the rest's size.
    if size == WGPU_WHOLE_MAP_SIZE size = wgpuBufferGetSize(buffer) - offset;
    wgpuBufferMapAsync(buffer, mode, offset, size, info);
    webgpu_wait(instance, *state.done);
    if state.status != .Success {
        log_error("WebGPU: mapping a buffer failed (%): %", state.status, message_text(*state.message));
        return false;
    }
    return true;
}

// Pop the device's innermost error scope (wgpuDevicePushErrorScope) and wait for it: the type of
// the first error it caught (.NoError for none), and its message (temporary storage).
webgpu_pop_error_scope :: (instance: WGPUInstance, device: WGPUDevice) -> WGPUErrorType, message: string {
    state: Error_Scope_State;
    info: WGPUPopErrorScopeCallbackInfo;
    info.mode = .AllowProcessEvents;
    info.callback = (status: WGPUPopErrorScopeStatus, type: WGPUErrorType, message: WGPUStringView, userdata1: *void, userdata2: *void) #c_call {
        state := cast(*Error_Scope_State) userdata1;
        state.type = ifx status == .Success then type else .Unknown;
        copy_message(*state.message, message);
        state.done = true;
    };
    info.userdata1 = *state;
    wgpuDevicePopErrorScope(device, info);
    webgpu_wait(instance, *state.done);
    return state.type, copy_temporary_string(message_text(*state.message));
}

// --- Surfaces and frames ------------------------------------------------------------------------

// A surface that draws into `window` (a Window_Creation window). In the browser the window is
// the page's canvas and `window` is ignored.
webgpu_create_surface :: (instance: WGPUInstance, window: Window_Type) -> WGPUSurface {
    desc: WGPUSurfaceDescriptor;
    #if OS == .MACOS {
        source: WGPUSurfaceSourceMetalLayer;
        source.layer = webgpu_metal_layer(window);
        if !source.layer return null;
        desc.nextInChain = *source.chain;
    } else #if OS == .WINDOWS {
        source: WGPUSurfaceSourceWindowsHWND;
        source.hinstance = webgpu_module_instance();
        source.hwnd = window;
        desc.nextInChain = *source.chain;
    } else #if OS == .LINUX {
        source: WGPUSurfaceSourceXlibWindow;
        source.display = webgpu_x11_display();
        if !source.display return null;
        source.window = cast(u64) window;
        desc.nextInChain = *source.chain;
    }
    return wgpuInstanceCreateSurface(instance, *desc);
}

// The texture format to configure `surface` with: one the surface supports that is not sRGB
// (unless `srgb`), so colors mean the same everywhere. Natively a surface usually lists an sRGB
// format first (bgra8unorm-srgb), where the GPU converts the shader's output to sRGB on write;
// the browser's canvas has only plain formats. A shader that writes already-encoded colors (or
// applies gamma itself) wants `srgb = false`; one that writes linear colors wants `srgb = true`,
// and must then encode them itself where the surface has no sRGB format (check with
// webgpu_format_is_srgb). .Undefined when the surface reports no formats.
webgpu_surface_format :: (surface: WGPUSurface, adapter: WGPUAdapter, srgb := false) -> WGPUTextureFormat {
    caps: WGPUSurfaceCapabilities;
    if wgpuSurfaceGetCapabilities(surface, adapter, *caps) != .Success || caps.formatCount == 0 return .Undefined;
    defer wgpuSurfaceCapabilitiesFreeMembers(caps);
    for 0..cast(s64) caps.formatCount - 1 {
        format := caps.formats[it];
        if webgpu_format_is_srgb(format) == srgb return format;
    }
    // None of the wanted kind: the surface's preferred one.
    return caps.formats[0];
}

// Whether writes to `format` are sRGB-encoded by the GPU.
webgpu_format_is_srgb :: (format: WGPUTextureFormat) -> bool {
    if format == {
        case .RGBA8UnormSrgb; #through;
        case .BGRA8UnormSrgb; #through;
        case .BC1RGBAUnormSrgb; #through;
        case .BC2RGBAUnormSrgb; #through;
        case .BC3RGBAUnormSrgb; #through;
        case .BC7RGBAUnormSrgb; #through;
        case .ETC2RGB8UnormSrgb; #through;
        case .ETC2RGB8A1UnormSrgb; #through;
        case .ETC2RGBA8UnormSrgb;
            return true;
    }
    // The ASTC sRGB formats are the odd values from ASTC4x4UnormSrgb on.
    if format >= .ASTC4x4UnormSrgb && format <= .ASTC12x12UnormSrgb return (cast(u32) format - cast(u32) WGPUTextureFormat.ASTC4x4UnormSrgb) % 2 == 0;
    return false;
}

// The size of the surface `window` shows, in pixels (the canvas in the browser).
webgpu_window_size :: (window: Window_Type) -> width: s32, height: s32 {
    #if OS == .WASM {
        w, h: u32;
        jai_canvas_size(*w, *h);
        return cast(s32) w, cast(s32) h;
    } else #if OS == .MACOS {
        return webgpu_macos_drawable_size(window);
    } else {
        WC :: #import "Window_Creation";
        _, _, w, h := WC.get_dimensions(window, false);
        return cast(s32) w, cast(s32) h;
    }
}

// wgpu-native's own wgpuSurfaceGetCurrentTexture status (wgpu.h, not webgpu.h): the window is not
// visible, so there is no texture this frame. Only its Metal backend reports it, instead of
// blocking for up to a second; skip the frame (handle events, wait a little) and try again.
WEBGPU_SURFACE_OCCLUDED :: cast(WGPUSurfaceGetCurrentTextureStatus) 0x0003_0001;

// Show the frame drawn into the surface's current texture and wait until the next one is due.
// Natively that is wgpuSurfacePresent (a FIFO surface blocks for vsync). The browser presents on
// its own; there this suspends the program until the page's next animation frame (other Jai
// threads run meanwhile) and then runs the callbacks with mode .AllowSpontaneous that finished.
webgpu_present :: (surface: WGPUSurface) {
    #if OS == .WASM {
        webgpu_wasm_next_frame();
    } else {
        wgpuSurfacePresent(surface);
    }
}

#scope_module

Request_State :: struct {
    done: bool;
    result: *void;
    message: Message_Copy;
}

// A callback's message lives only during the callback: callbacks copy it here.
Message_Copy :: struct {
    bytes: [512] u8;
    count: s64;
}

copy_message :: (copy: *Message_Copy, message: WGPUStringView) #c_call {
    text := to_string_c(message);
    copy.count = text.count;
    if copy.count > copy.bytes.count copy.count = copy.bytes.count;
    if copy.count memcpy(copy.bytes.data, text.data, copy.count);
}

message_text :: (copy: *Message_Copy) -> string {
    return string.{ copy.count, copy.bytes.data };
}

// to_string, callable from a #c_call callback.
to_string_c :: (view: WGPUStringView) -> string #c_call {
    if view.data == null return "";
    if view.length != WGPU_STRLEN return string.{ cast(s64) view.length, view.data };
    count := 0;
    while view.data[count] count += 1;
    return string.{ count, view.data };
}

Map_State :: struct {
    done: bool;
    status: WGPUMapAsyncStatus;
    message: Message_Copy;
}

Error_Scope_State :: struct {
    done: bool;
    type: WGPUErrorType;
    message: Message_Copy;
}

webgpu_callback_context: #Context;
