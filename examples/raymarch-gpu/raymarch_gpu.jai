// A ray marcher on the GPU, with WebGPU.
//
// The raymarch example marches rays on the CPU, one character at a time. Here a
// fragment shader (WGSL, in the string at the end) marches one ray per
// pixel, every frame: a twisting torus orbiting a blobby sphere. The Jai
// side is the standard WebGPU C API (#import "Extensions/WebGPU"), which the
// playground forwards to your browser's WebGPU; natively the same code
// runs on wgpu-native and draws into a window of its own.
//
// It draws in the Render tab (or the window) for GPU_SECONDS seconds. Click
// it to change the palette, Space pauses, Escape ends it early. Browsers
// without WebGPU (or without JavaScript Promise Integration), and machines
// without a display or a GPU adapter, skip this example.

#import "Extensions/WebGPU";

#if OS != .WASM {
    #import "Input";
    #import "Window_Type";
    #import "Window_Creation";
} else {
    #load "browser_canvas.jai";
}

GPU_SECONDS :: 20.0;

demo_raymarch_gpu :: () {
    if !webgpu_available() {
        print("No WebGPU in this browser, so this example is skipped.\n");
        print("Chrome or Edge (137 or newer) can show it.\n");
        return;
    }

    #if OS != .WASM {
        window: Window_Type;
        window = create_window(960, 600, "Jai: ray marching on the GPU");
        if !window {
            print("No window could be opened (no display?), so this example is skipped.\n");
            return;
        }
    } else {
        if !demo_canvas_available() {
            print("No WebGPU canvas was supplied by this host, so this example is skipped.\n");
            return;
        }
    }

    instance := wgpuCreateInstance(null);
    surface: WGPUSurface;

    #if OS == .WASM {
        // The private host binds this real surface to the supplied page canvas.
        surface = wgpuInstanceCreateSurface(instance, null);
    } else {
        surface = webgpu_create_surface(instance, window);
    }

    if !surface {
        print("No WebGPU surface for the window, so this example is skipped.\n");
        return;
    }

    options: WGPURequestAdapterOptions;
    options.compatibleSurface = surface;
    adapter := webgpu_request_adapter(instance, *options);
    if !adapter {
        print("No GPU adapter here, so this example is skipped.\n");
        return;
    }

    device := webgpu_request_device(instance, adapter);
    if !device return;

    queue := wgpuDeviceGetQueue(device);

    // The shader does its own gamma, so: not an sRGB format.
    format := webgpu_surface_format(surface, adapter);

    if format == .Undefined {
        print("The surface lists no texture formats, so this example is skipped.\n");
        return;
    }

    width, height: s32;

    #if OS == .WASM {
        width, height = demo_canvas_dimensions();
    } else {
        width, height = webgpu_window_size(window);
    }

    gpu_configure(surface, device, format, width, height);

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
    buffer_desc.size = size_of(Gpu_Uniforms);
    buffer_desc.usage = .Uniform | .CopyDst;
    uniforms := wgpuDeviceCreateBuffer(device, *buffer_desc);
    entry: WGPUBindGroupEntry;
    entry.binding = 0;
    entry.buffer = uniforms;
    entry.size = size_of(Gpu_Uniforms);
    bind_desc: WGPUBindGroupDescriptor;
    bind_desc.layout = wgpuRenderPipelineGetBindGroupLayout(pipeline, 0);
    bind_desc.entryCount = 1;
    bind_desc.entries = *entry;
    bind_group := wgpuDeviceCreateBindGroup(device, *bind_desc);

    time, palette: float32;
    paused := false;
    start := demo_gpu_seconds();
    last := start;
    frames := 0;

    while demo_gpu_seconds() - start < GPU_SECONDS {
        quit := false;

        #if OS == .WASM {
            resizes, quit_now, pause, clicks := demo_canvas_input();

            quit = quit_now;
            if pause paused = !paused;

            while clicks > 0 {
                palette += 1;
                clicks -= 1;
            }

            while resizes > 0 {
                width, height = demo_canvas_dimensions();
                gpu_configure(surface, device, format, width, height);
                resizes -= 1;
            }
        } else {
            update_window_events();

            for get_window_resizes() {
                width, height = webgpu_window_size(window);
                gpu_configure(surface, device, format, width, height);
            }

            for events_this_frame {
                if it.type == .QUIT quit = true; // the window was closed
                if it.type != .KEYBOARD || !it.key_pressed continue;
                if it.key_code == .ESCAPE quit = true;
                if it.key_code == .SPACEBAR paused = !paused;
                if it.key_code == .MOUSE_BUTTON_LEFT palette += 1;
            }
        }

        if quit break;

        now := demo_gpu_seconds();
        if !paused time += cast(float32) (now - last);

        last = now;
        aspect := cast(float32) width / cast(float32) max(height, 1);
        u := Gpu_Uniforms.{ time, aspect, palette, 0 };

        wgpuQueueWriteBuffer(queue, uniforms, 0, *u, size_of(Gpu_Uniforms));

        frame: WGPUSurfaceTexture;

        wgpuSurfaceGetCurrentTexture(surface, *frame);

        #if OS != .WASM {
            if frame.status != .SuccessOptimal && frame.status != .SuccessSuboptimal {
                // No frame this time: the window is hidden (macOS says so with
                // WEBGPU_SURFACE_OCCLUDED until it shows again) or its size changed (configure
                // it again). Nap a little and try the next frame.
                if frame.texture wgpuTextureRelease(frame.texture);

                if frame.status != WEBGPU_SURFACE_OCCLUDED {
                    width, height = webgpu_window_size(window);
                    gpu_configure(surface, device, format, width, height);
                }

                sleep_milliseconds(10);
                continue;
            }
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

        // Waits for the browser's next frame (natively, the display's): the scene animates in
        // real time.
        webgpu_present(surface);
        wgpuTextureViewRelease(view);
        wgpuTextureRelease(frame.texture);
        frames += 1;
    }

    if frames == 0 {
        print("The window never showed (it was hidden all along), so nothing was drawn.\n");
    } else {
        print("Drew % frames at %x% pixels.\n", frames, width, height);
    }
}

Gpu_Uniforms :: struct {
    time: float32;
    aspect: float32;
    palette: float32;
    _: float32; // uniforms come in 16-byte blocks
}

gpu_configure :: (surface: WGPUSurface, device: WGPUDevice, format: WGPUTextureFormat, width: s32, height: s32) {
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

#load "scene.jai";

// Browser graphics need the real embedding clock; Basic timing is native-only.
demo_gpu_seconds :: () -> float64 {
    #if OS == .WASM {
        return demo_canvas_seconds();
    } else {
        return seconds_since_init();
    }
}
