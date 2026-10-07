// WebGPU in a real window: open a Window_Creation window, make a surface for it, configure it and
// present a number of frames, checking every frame's surface status and that nothing raised a
// WebGPU error. Where the surface can be copied from, the last frame is also read back before
// it is presented: the middle must be the triangle's color and the corners the clear color. Run
// by CI on Linux under Xvfb with Mesa's lavapipe and on Windows with D3D12's WARP adapter
// (docs/stdlib/webgpu.md#tests):
//
//     jaic run tests/webgpu/window.jai -- frames 30
//     jaic build tests/webgpu/window.jai -o window && ./window frames 30
//
// Arguments: `frames N` (default 60) and `fallback` (ask for the fallback, software, adapter).
// Exit codes: 0 passed, 1 a check failed, 2 no window (no display), 77 no adapter here.

#import "Basic";
#import "Input";
#import "String";
#import "Window_Creation";
#import "Extensions/WebGPU";

WIDTH :: 320;
HEIGHT :: 240;

SHADER :: #string WGSL
@vertex fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4f {
    let p = array(vec2f(0.0, 0.8), vec2f(-0.8, -0.8), vec2f(0.8, -0.8));
    return vec4f(p[i], 0.0, 1.0);
}
@fragment fn fs() -> @location(0) vec4f {
    return vec4f(1.0, 0.5, 0.0, 1.0);
}
WGSL

failures := 0;

// Errors no error scope caught, counted by the device's callback.
uncaptured_errors: s32;

check :: (ok: bool, what: string, args: ..Any) {
    if ok return;
    failures += 1;
    print("FAIL: ");
    print(what, ..args);
    print("\n");
}

main :: () {
    frame_count := 60;
    fallback := false;
    args := get_command_line_arguments();
    for args {
        if it == "fallback" fallback = true;
        if it == "frames" && it_index + 1 < args.count {
            value, ok := parse_int(*args[it_index + 1]);
            if ok && value > 0 frame_count = value;
        }
    }

    window := create_window(WIDTH, HEIGHT, "WebGPU window test");
    if !window {
        print("no window (is there a display?)\n");
        exit(2);
    }
    width, height := webgpu_window_size(window);
    print("window: % x %\n", width, height);
    check(width > 0 && height > 0, "the window is % x %", width, height);

    instance := wgpuCreateInstance(null);
    if !instance {
        print("no WebGPU instance\n");
        exit(77);
    }
    surface := webgpu_create_surface(instance, window);
    if !surface {
        print("FAIL: no surface for the window\n");
        exit(1);
    }
    options: WGPURequestAdapterOptions;
    options.compatibleSurface = surface;
    options.forceFallbackAdapter = xx fallback;
    adapter := webgpu_request_adapter(instance, *options);
    if !adapter {
        print("no adapter for the surface\n");
        exit(77);
    }
    info: WGPUAdapterInfo;
    wgpuAdapterGetInfo(adapter, *info);
    print("adapter: % (%, %)\n", to_string(info.device), info.backendType, info.adapterType);
    wgpuAdapterInfoFreeMembers(info);

    device_desc: WGPUDeviceDescriptor;
    device_desc.uncapturedErrorCallbackInfo.callback = (device: *WGPUDevice, type: WGPUErrorType, message: WGPUStringView, userdata1: *void, userdata2: *void) #c_call {
        uncaptured_errors += 1;
    };
    device := webgpu_request_device(instance, adapter, *device_desc);
    if !device {
        print("FAIL: no device\n");
        exit(1);
    }
    queue := wgpuDeviceGetQueue(device);

    caps: WGPUSurfaceCapabilities;
    status := wgpuSurfaceGetCapabilities(surface, adapter, *caps);
    check(status == .Success, "surface capabilities: %", status);
    usages := caps.usages;
    print("surface usages: %, % formats, % present modes\n", usages, caps.formatCount, caps.presentModeCount);
    wgpuSurfaceCapabilitiesFreeMembers(caps);

    format := webgpu_surface_format(surface, adapter);
    print("surface format: %\n", format);
    check(format != .Undefined, "the surface lists no formats");
    // The last frame is read back where the surface allows copying from it, in a format whose
    // bytes are known.
    readback := (usages & .CopySrc) != 0 && (format == .BGRA8Unorm || format == .RGBA8Unorm);

    config: WGPUSurfaceConfiguration;
    config.device = device;
    config.format = format;
    config.usage = .RenderAttachment;
    if readback config.usage |= .CopySrc;
    config.width = cast(u32) width;
    config.height = cast(u32) height;
    config.presentMode = .Fifo;
    config.alphaMode = .Auto;
    wgpuSurfaceConfigure(surface, *config);

    shader := create_shader(device, SHADER);
    target: WGPUColorTargetState;
    target.format = format;
    target.writeMask = .All;
    fragment: WGPUFragmentState;
    fragment.module = shader;
    fragment.entryPoint = to_string_view("fs");
    fragment.targetCount = 1;
    fragment.targets = *target;
    pipeline_desc: WGPURenderPipelineDescriptor;
    pipeline_desc.vertex.module = shader;
    pipeline_desc.vertex.entryPoint = to_string_view("vs");
    pipeline_desc.primitive.topology = .TriangleList;
    pipeline_desc.multisample.count = 1;
    pipeline_desc.multisample.mask = 0xFFFF_FFFF;
    pipeline_desc.fragment = *fragment;
    pipeline := wgpuDeviceCreateRenderPipeline(device, *pipeline_desc);

    // Rows of a texture-to-buffer copy are a multiple of 256 bytes.
    row_bytes := (cast(u32) width * 4 + 255) & ~cast(u32) 255;
    buffer: WGPUBuffer;
    if readback buffer = create_buffer(device, cast(u64) row_bytes * cast(u64) height, .MapRead | .CopyDst);

    wgpuDevicePushErrorScope(device, .Validation);
    wgpuDevicePushErrorScope(device, .OutOfMemory);
    // (wgpu-native panics on an .Internal scope; internal errors reach the uncaptured count.)

    presented := 0;
    retries := 0;
    occluded := 0;
    start := seconds_since_init();
    while presented < frame_count {
        update_window_events();
        for events_this_frame if it.type == .QUIT {
            check(false, "the window was closed after % frames", presented);
            break;
        }

        frame: WGPUSurfaceTexture;
        wgpuSurfaceGetCurrentTexture(surface, *frame);
        if frame.status == WEBGPU_SURFACE_OCCLUDED {
            // macOS until the window is on screen: no texture, and no frame this time.
            occluded += 1;
            if occluded > 500 {
                check(false, "the window stayed occluded (frame %)", presented);
                break;
            }
            sleep_milliseconds(10);
            continue;
        }
        if frame.status == .Outdated || frame.status == .Timeout {
            // Once in a while (a resize, or a slow compositor); more often is a failure.
            retries += 1;
            if frame.texture wgpuTextureRelease(frame.texture);
            if retries > 10 {
                check(false, "the surface kept returning % (frame %)", frame.status, presented);
                break;
            }
            width, height = webgpu_window_size(window);
            config.width = cast(u32) width;
            config.height = cast(u32) height;
            wgpuSurfaceConfigure(surface, *config);
            continue;
        }
        if frame.status != .SuccessOptimal && frame.status != .SuccessSuboptimal {
            check(false, "frame %: the surface texture's status is %", presented, frame.status);
            break;
        }
        texture_width := wgpuTextureGetWidth(frame.texture);
        texture_height := wgpuTextureGetHeight(frame.texture);
        if presented == 0 {
            print("first frame: % (% x %)\n", frame.status, texture_width, texture_height);
            check(texture_width == config.width && texture_height == config.height,
                "the surface texture is % x %, configured % x %", texture_width, texture_height, config.width, config.height);
        }
        last := presented == frame_count - 1;
        view := wgpuTextureCreateView(frame.texture, null);

        color: WGPURenderPassColorAttachment;
        color.view = view;
        color.depthSlice = WGPU_DEPTH_SLICE_UNDEFINED;
        color.loadOp = .Clear;
        color.storeOp = .Store;
        color.clearValue = .{ 0, 0, 1, 1 };
        pass_desc: WGPURenderPassDescriptor;
        pass_desc.colorAttachmentCount = 1;
        pass_desc.colorAttachments = *color;

        encoder := wgpuDeviceCreateCommandEncoder(device, null);
        pass := wgpuCommandEncoderBeginRenderPass(encoder, *pass_desc);
        wgpuRenderPassEncoderSetPipeline(pass, pipeline);
        wgpuRenderPassEncoderDraw(pass, 3, 1, 0, 0);
        wgpuRenderPassEncoderEnd(pass);
        wgpuRenderPassEncoderRelease(pass);
        if last && readback copy_texture_to_buffer(encoder, frame.texture, buffer, texture_width, texture_height, row_bytes);
        commands := wgpuCommandEncoderFinish(encoder, null);
        wgpuQueueSubmit(queue, 1, *commands);
        wgpuCommandBufferRelease(commands);
        wgpuCommandEncoderRelease(encoder);

        if last && readback check_pixels(instance, buffer, format, texture_width, texture_height, row_bytes);

        present_status := wgpuSurfacePresent(surface);
        check(present_status == .Success, "frame %: presenting gave %", presented, present_status);
        wgpuTextureViewRelease(view);
        wgpuTextureRelease(frame.texture);
        presented += 1;
    }
    elapsed := seconds_since_init() - start;
    print("presented % frames in % s (% occluded tries, % reconfigurations)\n", presented, formatFloat(elapsed, trailing_width = 2), occluded, retries);

    for filter: WGPUErrorFilter.[.OutOfMemory, .Validation] {
        type, message := webgpu_pop_error_scope(instance, device);
        check(type == .NoError, "% error while rendering: %", filter, message);
    }
    wgpuInstanceProcessEvents(instance);
    check(uncaptured_errors == 0, "% uncaptured WebGPU errors", uncaptured_errors);
    check(presented == frame_count, "presented % of % frames", presented, frame_count);

    wgpuSurfaceUnconfigure(surface);
    if buffer wgpuBufferRelease(buffer);
    wgpuRenderPipelineRelease(pipeline);
    wgpuShaderModuleRelease(shader);
    wgpuQueueRelease(queue);
    wgpuDeviceRelease(device);
    wgpuAdapterRelease(adapter);
    wgpuSurfaceRelease(surface);
    wgpuInstanceRelease(instance);
    if failures {
        print("% checks failed\n", failures);
        exit(1);
    }
    print("webgpu window: ok\n");
}

// The middle of the frame is the triangle (orange), the corners the clear color (blue).
check_pixels :: (instance: WGPUInstance, buffer: WGPUBuffer, format: WGPUTextureFormat, width: u32, height: u32, row_bytes: u32) {
    size := cast(u64) row_bytes * cast(u64) height;
    if !webgpu_map_buffer(instance, buffer, .Read, 0, size) {
        check(false, "mapping the readback buffer failed");
        return;
    }
    pixels := cast(*u8) wgpuBufferGetConstMappedRange(buffer, 0, size);
    check(pixels != null, "the mapped range is null");
    if pixels {
        rgb :: (x: u32, y: u32) -> r: u8, g: u8, b: u8 #expand {
            p := pixels + y * row_bytes + x * 4;
            if format == .BGRA8Unorm return p[2], p[1], p[0];
            return p[0], p[1], p[2];
        }
        r, g, b := rgb(width / 2, height / 2);
        check(r == 255 && (g == 127 || g == 128) && b == 0, "readback: the middle pixel is (%, %, %), not orange", r, g, b);
        r, g, b = rgb(1, 1);
        check(r == 0 && g == 0 && b == 255, "readback: the top-left pixel is (%, %, %), not blue", r, g, b);
        r, g, b = rgb(width - 2, 1);
        check(r == 0 && g == 0 && b == 255, "readback: the top-right pixel is (%, %, %), not blue", r, g, b);
        print("readback of the last frame: checked\n");
    }
    wgpuBufferUnmap(buffer);
}

create_shader :: (device: WGPUDevice, code: string) -> WGPUShaderModule {
    wgsl: WGPUShaderSourceWGSL;
    wgsl.code = to_string_view(code);
    desc: WGPUShaderModuleDescriptor;
    desc.nextInChain = *wgsl.chain;
    return wgpuDeviceCreateShaderModule(device, *desc);
}

create_buffer :: (device: WGPUDevice, size: u64, usage: WGPUBufferUsage) -> WGPUBuffer {
    desc: WGPUBufferDescriptor;
    desc.size = size;
    desc.usage = usage;
    return wgpuDeviceCreateBuffer(device, *desc);
}

copy_texture_to_buffer :: (encoder: WGPUCommandEncoder, texture: WGPUTexture, buffer: WGPUBuffer, width: u32, height: u32, row_bytes: u32) {
    source: WGPUTexelCopyTextureInfo;
    source.texture = texture;
    source.aspect = .All;
    destination: WGPUTexelCopyBufferInfo;
    destination.buffer = buffer;
    destination.layout.bytesPerRow = row_bytes;
    destination.layout.rowsPerImage = height;
    extent := WGPUExtent3D.{ width, height, 1 };
    wgpuCommandEncoderCopyTextureToBuffer(encoder, *source, *destination, *extent);
}
