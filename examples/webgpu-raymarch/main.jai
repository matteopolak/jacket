// A raymarched scene drawn by one full-screen fragment shader with WebGPU:
// a twisting torus orbiting a blobby sphere over a checkered floor, with soft shadows.
// Click to change the palette, Space pauses, Escape quits. It opens a window (see
// docs/extensions/webgpu.md), so it is native only; the raymarch-gpu example
// is the same scene for the browser playground.

#import "Basic";
#import "Input";
#import "Extensions/WebGPU";

#import "Window_Creation";

#load "../raymarch-gpu/scene.jai";

Uniforms :: struct {
    time: float32;
    aspect: float32;
    palette: float32;
    _: float32;
}

main :: () {
    window := create_window(960, 600, "WebGPU raymarch");

    instance := wgpuCreateInstance(null);
    surface := webgpu_create_surface(instance, window);
    options: WGPURequestAdapterOptions;
    options.compatibleSurface = surface;
    adapter := webgpu_request_adapter(instance, *options);
    if !adapter exit(1);

    device := webgpu_request_device(instance, adapter);
    if !device exit(1);

    queue := wgpuDeviceGetQueue(device);

    // The shader applies gamma itself (the pow at the end of fs), so it wants a surface format
    // that is not sRGB: natively the first one usually is, and would brighten it twice.
    format := webgpu_surface_format(surface, adapter);

    width, height := webgpu_window_size(window);

    configure(surface, device, format, width, height);

    wgsl: WGPUShaderSourceWGSL;
    wgsl.code = to_string_view(GPU_SCENE);
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

    time: float32;
    palette: float32;
    paused := false;
    last := seconds_since_init();
    frames := 0;

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
            if it.key_code == .MOUSE_BUTTON_LEFT palette += 1;
        }

        if quit break;

        now := seconds_since_init();
        if !paused time += cast(float32) (now - last);

        last = now;
        u := Uniforms.{ time = time, aspect = cast(float32) width / cast(float32) max(height, 1), palette = palette };

        wgpuQueueWriteBuffer(queue, uniforms, 0, *u, size_of(Uniforms));

        frame: WGPUSurfaceTexture;

        wgpuSurfaceGetCurrentTexture(surface, *frame);

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
