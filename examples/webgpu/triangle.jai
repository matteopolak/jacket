// A spinning triangle with WebGPU. Natively it opens a window (wgpu-native, see
// docs/stdlib/webgpu.md); in the browser playground it draws into the page's canvas.
// Space pauses, a mouse click changes the background, Escape quits.

#import "Basic";
#import "Math";
#import "Input";
#import "Random";
#import "Window_Type";
#import "Extensions/WebGPU";

#if OS != .WASM {
    #import "Window_Creation";
}

SHADER :: #string WGSL
struct Uniforms { angle: f32, aspect: f32 }
@group(0) @binding(0) var<uniform> u: Uniforms;

struct Out {
    @builtin(position) position: vec4f,
    @location(0) color: vec3f,
}

@vertex fn vs(@builtin(vertex_index) i: u32) -> Out {
    let corners = array(vec2f(0.0, 0.6), vec2f(-0.52, -0.3), vec2f(0.52, -0.3));
    let colors = array(vec3f(1.0, 0.3, 0.2), vec3f(0.2, 1.0, 0.4), vec3f(0.3, 0.4, 1.0));
    let c = cos(u.angle);
    let s = sin(u.angle);
    let p = corners[i];
    var out: Out;
    out.position = vec4f((p.x * c - p.y * s) / u.aspect, p.x * s + p.y * c, 0.0, 1.0);
    out.color = colors[i];
    return out;
}

@fragment fn fs(in: Out) -> @location(0) vec4f {
    return vec4f(in.color, 1.0);
}
WGSL

Uniforms :: struct {
    angle: float32;
    aspect: float32;
    _: [2] float32; // uniform buffers are 16-byte aligned
}

main :: () {
    window: Window_Type;
    #if OS != .WASM {
        window = create_window(800, 600, "WebGPU triangle");
    }

    instance := wgpuCreateInstance(null);
    surface := webgpu_create_surface(instance, window);
    options: WGPURequestAdapterOptions;
    options.compatibleSurface = surface;
    adapter := webgpu_request_adapter(instance, *options);
    if !adapter exit(1);
    device := webgpu_request_device(instance, adapter);
    if !device exit(1);
    queue := wgpuDeviceGetQueue(device);

    info: WGPUAdapterInfo;
    wgpuAdapterGetInfo(adapter, *info);
    print("Adapter: % (%)\n", to_string(info.description), info.backendType);
    wgpuAdapterInfoFreeMembers(info);

    // Not an sRGB format, so the colors below look the same natively and in the browser.
    format := webgpu_surface_format(surface, adapter);
    print("Surface format: %\n", format);

    width, height := webgpu_window_size(window);
    configure(surface, device, format, width, height);

    wgsl: WGPUShaderSourceWGSL;
    wgsl.code = to_string_view(SHADER);
    shader_desc: WGPUShaderModuleDescriptor;
    shader_desc.nextInChain = *wgsl.chain;
    shader := wgpuDeviceCreateShaderModule(device, *shader_desc);

    target: WGPUColorTargetState;
    target.format = format;
    target.writeMask = .All;
    fragment: WGPUFragmentState;
    fragment.module = shader;
    fragment.entryPoint = to_string_view("fs");
    fragment.targetCount = 1;
    fragment.targets = *target;
    pipeline_desc: WGPURenderPipelineDescriptor;
    pipeline_desc.label = to_string_view("triangle");
    pipeline_desc.vertex.module = shader;
    pipeline_desc.vertex.entryPoint = to_string_view("vs");
    pipeline_desc.primitive.topology = .TriangleList;
    pipeline_desc.multisample.count = 1;
    pipeline_desc.multisample.mask = 0xFFFF_FFFF;
    pipeline_desc.fragment = *fragment;
    pipeline := wgpuDeviceCreateRenderPipeline(device, *pipeline_desc);

    buffer_desc: WGPUBufferDescriptor;
    buffer_desc.size = size_of(Uniforms);
    buffer_desc.usage = .Uniform | .CopyDst;
    uniforms := wgpuDeviceCreateBuffer(device, *buffer_desc);
    entry: WGPUBindGroupEntry;
    entry.binding = 0;
    entry.buffer = uniforms;
    entry.size = size_of(Uniforms);
    bind_desc: WGPUBindGroupDescriptor;
    bind_desc.layout = wgpuRenderPipelineGetBindGroupLayout(pipeline, 0);
    bind_desc.entryCount = 1;
    bind_desc.entries = *entry;
    bind_group := wgpuDeviceCreateBindGroup(device, *bind_desc);

    background := Vector3.{ 0.08, 0.08, 0.12 };
    angle: float32;
    paused := false;
    frames := 0;
    last := seconds_since_init();
    while true {
        update_window_events();
        for get_window_resizes() {
            width, height = webgpu_window_size(window);
            configure(surface, device, format, width, height);
        }
        quit := false;
        for events_this_frame {
            if it.type == .QUIT quit = true;
            if it.type != .KEYBOARD || !it.key_pressed continue;
            if it.key_code == .ESCAPE quit = true;
            if it.key_code == .SPACEBAR paused = !paused;
            if it.key_code == .MOUSE_BUTTON_LEFT {
                background = .{ random_get_zero_to_one() * 0.4, random_get_zero_to_one() * 0.4, random_get_zero_to_one() * 0.4 };
                print("Background %\n", background);
            }
        }
        if quit break;

        now := seconds_since_init();
        if !paused angle += cast(float32) (now - last);
        last = now;
        u := Uniforms.{ angle = angle, aspect = cast(float32) width / cast(float32) max(height, 1) };
        wgpuQueueWriteBuffer(queue, uniforms, 0, *u, size_of(Uniforms));

        frame: WGPUSurfaceTexture;
        wgpuSurfaceGetCurrentTexture(surface, *frame);
        if frame.status == WEBGPU_SURFACE_OCCLUDED {
            // A hidden window on macOS: nothing to draw into until it shows again.
            sleep_milliseconds(16);
            continue;
        }
        if frame.status != .SuccessOptimal && frame.status != .SuccessSuboptimal {
            width, height = webgpu_window_size(window);
            configure(surface, device, format, width, height);
            continue;
        }
        view := wgpuTextureCreateView(frame.texture, null);

        color: WGPURenderPassColorAttachment;
        color.view = view;
        color.loadOp = .Clear;
        color.storeOp = .Store;
        color.clearValue = .{ background.x, background.y, background.z, 1 };
        pass_desc: WGPURenderPassDescriptor;
        pass_desc.colorAttachmentCount = 1;
        pass_desc.colorAttachments = *color;

        encoder := wgpuDeviceCreateCommandEncoder(device, null);
        pass := wgpuCommandEncoderBeginRenderPass(encoder, *pass_desc);
        wgpuRenderPassEncoderSetPipeline(pass, pipeline);
        wgpuRenderPassEncoderSetBindGroup(pass, 0, bind_group, 0, null);
        wgpuRenderPassEncoderDraw(pass, 3, 1, 0, 0);
        wgpuRenderPassEncoderEnd(pass);
        wgpuRenderPassEncoderRelease(pass);
        commands := wgpuCommandEncoderFinish(encoder, null);
        wgpuQueueSubmit(queue, 1, *commands);
        wgpuCommandBufferRelease(commands);
        wgpuCommandEncoderRelease(encoder);

        webgpu_present(surface);
        wgpuTextureViewRelease(view);
        wgpuTextureRelease(frame.texture);

        frames += 1;
        if frames == 3 print("Drew % frames\n", frames);
    }
    print("Done after % frames\n", frames);
}

configure :: (surface: WGPUSurface, device: WGPUDevice, format: WGPUTextureFormat, width: s32, height: s32) {
    if width <= 0 || height <= 0 return;
    config: WGPUSurfaceConfiguration;
    config.device = device;
    config.format = format;
    config.usage = .RenderAttachment;
    config.width = cast(u32) width;
    config.height = cast(u32) height;
    config.presentMode = .Fifo;
    config.alphaMode = .Auto;
    wgpuSurfaceConfigure(surface, *config);
}
