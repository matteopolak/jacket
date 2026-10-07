// WebGPU without a window: render to a texture, run a compute shader, upload a texture, read
// each back through a mapped buffer, and check that an invalid shader is a validation error in
// an error scope. Exercises adapter and device requests, buffer mapping and error scopes, all of
// which finish through callbacks. Run by CI on every platform with a (software) adapter
// (docs/stdlib/webgpu.md#tests):
//
//     jaic run tests/webgpu/headless.jai                   # interpreted
//     jaic build tests/webgpu/headless.jai -o headless && ./headless
//
// Exit codes: 0 passed, 1 a check failed, 77 no adapter here (CI decides whether that is fine).
// `fallback` as an argument (`jaic run headless.jai -- fallback`) asks for the fallback (software)
// adapter.

#import "Basic";
#import "Extensions/WebGPU";

SIZE :: 64; // the render target is SIZE x SIZE
ROW_BYTES :: SIZE * 4; // rgba8, already a multiple of 256 as copies need

failures := 0;

check :: (ok: bool, what: string, args: ..Any) {
    if ok return;
    failures += 1;
    print("FAIL: ");
    print(what, ..args);
    print("\n");
}

main :: () {
    args := get_command_line_arguments();
    fallback := false;
    for args if it == "fallback" fallback = true;

    instance := wgpuCreateInstance(null);
    if !instance {
        print("no WebGPU instance\n");
        exit(77);
    }
    options: WGPURequestAdapterOptions;
    options.forceFallbackAdapter = xx fallback;
    adapter := webgpu_request_adapter(instance, *options);
    if !adapter {
        print("no adapter\n");
        exit(77);
    }
    info: WGPUAdapterInfo;
    wgpuAdapterGetInfo(adapter, *info);
    print("adapter: % (%, %)\n", to_string(info.device), info.backendType, info.adapterType);
    wgpuAdapterInfoFreeMembers(info);

    device := webgpu_request_device(instance, adapter);
    if !device {
        print("FAIL: no device\n");
        exit(1);
    }
    queue := wgpuDeviceGetQueue(device);

    render_to_texture(instance, device, queue);
    compute_squares(instance, device, queue);
    upload_texture(instance, device, queue);
    error_scopes(instance, device);

    wgpuQueueRelease(queue);
    wgpuDeviceRelease(device);
    wgpuAdapterRelease(adapter);
    wgpuInstanceRelease(instance);
    if failures {
        print("% checks failed\n", failures);
        exit(1);
    }
    print("webgpu headless: ok\n");
}

// A triangle over the middle of a cleared target: the middle is the triangle's color, the
// corners the clear color.
render_to_texture :: (instance: WGPUInstance, device: WGPUDevice, queue: WGPUQueue) {
    SHADER :: #string WGSL
@vertex fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4f {
    let p = array(vec2f(0.0, 0.8), vec2f(-0.8, -0.8), vec2f(0.8, -0.8));
    return vec4f(p[i], 0.0, 1.0);
}
@fragment fn fs() -> @location(0) vec4f {
    return vec4f(1.0, 0.5, 0.0, 1.0);
}
WGSL
    shader := create_shader(device, SHADER);

    texture_desc: WGPUTextureDescriptor;
    texture_desc.usage = .RenderAttachment | .CopySrc;
    texture_desc.dimension = ._2D;
    texture_desc.size = .{ SIZE, SIZE, 1 };
    texture_desc.format = .RGBA8Unorm;
    target := wgpuDeviceCreateTexture(device, *texture_desc);
    view := wgpuTextureCreateView(target, null);

    color_target: WGPUColorTargetState;
    color_target.format = .RGBA8Unorm;
    color_target.writeMask = .All;
    fragment: WGPUFragmentState;
    fragment.module = shader;
    fragment.entryPoint = to_string_view("fs");
    fragment.targetCount = 1;
    fragment.targets = *color_target;
    pipeline_desc: WGPURenderPipelineDescriptor;
    pipeline_desc.vertex.module = shader;
    pipeline_desc.vertex.entryPoint = to_string_view("vs");
    pipeline_desc.primitive.topology = .TriangleList;
    pipeline_desc.multisample.count = 1;
    pipeline_desc.multisample.mask = 0xFFFF_FFFF;
    pipeline_desc.fragment = *fragment;
    pipeline := wgpuDeviceCreateRenderPipeline(device, *pipeline_desc);

    readback := create_buffer(device, ROW_BYTES * SIZE, .MapRead | .CopyDst);

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
    copy_texture_to_buffer(encoder, target, readback, SIZE, SIZE, ROW_BYTES);
    submit(queue, encoder);

    pixels := map_read(instance, readback, ROW_BYTES * SIZE);
    if pixels {
        pixel :: (x: s64, y: s64) -> u32 #expand {
            return (cast(*u32) (pixels + y * ROW_BYTES + x * 4)).*;
        }
        // Little-endian rgba8: 0xAABBGGRR.
        check(pixel(SIZE / 2, SIZE / 2) == 0xFF00_80FF || pixel(SIZE / 2, SIZE / 2) == 0xFF00_7FFF,
            "render: middle pixel is 0x%, not orange", formatInt(pixel(SIZE / 2, SIZE / 2), base = 16));
        check(pixel(0, 0) == 0xFFFF_0000, "render: corner pixel is 0x%, not blue", formatInt(pixel(0, 0), base = 16));
        check(pixel(SIZE - 1, 0) == 0xFFFF_0000, "render: corner pixel is 0x%, not blue", formatInt(pixel(SIZE - 1, 0), base = 16));
        print("render to texture: checked\n");
        wgpuBufferUnmap(readback);
    }

    wgpuBufferRelease(readback);
    wgpuRenderPipelineRelease(pipeline);
    wgpuTextureViewRelease(view);
    wgpuTextureRelease(target);
    wgpuShaderModuleRelease(shader);
}

// values[i] = i * i + 1 in a storage buffer, copied to a mappable one.
compute_squares :: (instance: WGPUInstance, device: WGPUDevice, queue: WGPUQueue) {
    COUNT :: 256;
    SHADER :: #string WGSL
@group(0) @binding(0) var<storage, read_write> values: array<u32>;
@compute @workgroup_size(64) fn main(@builtin(global_invocation_id) id: vec3u) {
    values[id.x] = id.x * id.x + 1u;
}
WGSL
    shader := create_shader(device, SHADER);
    pipeline_desc: WGPUComputePipelineDescriptor;
    pipeline_desc.compute.module = shader;
    pipeline_desc.compute.entryPoint = to_string_view("main");
    pipeline := wgpuDeviceCreateComputePipeline(device, *pipeline_desc);

    storage := create_buffer(device, COUNT * 4, .Storage | .CopySrc);
    readback := create_buffer(device, COUNT * 4, .MapRead | .CopyDst);
    entry: WGPUBindGroupEntry;
    entry.binding = 0;
    entry.buffer = storage;
    entry.size = COUNT * 4;
    bind_desc: WGPUBindGroupDescriptor;
    bind_desc.layout = wgpuComputePipelineGetBindGroupLayout(pipeline, 0);
    bind_desc.entryCount = 1;
    bind_desc.entries = *entry;
    bind_group := wgpuDeviceCreateBindGroup(device, *bind_desc);

    encoder := wgpuDeviceCreateCommandEncoder(device, null);
    pass := wgpuCommandEncoderBeginComputePass(encoder, null);
    wgpuComputePassEncoderSetPipeline(pass, pipeline);
    wgpuComputePassEncoderSetBindGroup(pass, 0, bind_group, 0, null);
    wgpuComputePassEncoderDispatchWorkgroups(pass, COUNT / 64, 1, 1);
    wgpuComputePassEncoderEnd(pass);
    wgpuComputePassEncoderRelease(pass);
    wgpuCommandEncoderCopyBufferToBuffer(encoder, storage, 0, readback, 0, COUNT * 4);
    submit(queue, encoder);

    data := map_read(instance, readback, COUNT * 4);
    if data {
        values := cast(*u32) data;
        wrong := 0;
        for 0..COUNT - 1 if values[it] != cast(u32) (it * it + 1) wrong += 1;
        check(wrong == 0, "compute: % of % values are wrong (values[3] = %)", wrong, COUNT, values[3]);
        print("compute shader: checked\n");
        wgpuBufferUnmap(readback);
    }

    wgpuBindGroupRelease(bind_group);
    wgpuBindGroupLayoutRelease(bind_desc.layout);
    wgpuBufferRelease(readback);
    wgpuBufferRelease(storage);
    wgpuComputePipelineRelease(pipeline);
    wgpuShaderModuleRelease(shader);
}

// wgpuQueueWriteTexture into a 4x4 texture (rows of 16 bytes: no 256-byte alignment needed for
// writes), then back through a buffer (rows padded to 256 bytes).
upload_texture :: (instance: WGPUInstance, device: WGPUDevice, queue: WGPUQueue) {
    N :: 4;
    texels: [N * N] u32;
    for *texels it.* = cast(u32) (0x1020_3040 + it_index * 0x0101_0101);

    texture_desc: WGPUTextureDescriptor;
    texture_desc.usage = .CopyDst | .CopySrc;
    texture_desc.dimension = ._2D;
    texture_desc.size = .{ N, N, 1 };
    texture_desc.format = .RGBA8Unorm;
    texture := wgpuDeviceCreateTexture(device, *texture_desc);

    destination: WGPUTexelCopyTextureInfo;
    destination.texture = texture;
    destination.aspect = .All;
    layout: WGPUTexelCopyBufferLayout;
    layout.bytesPerRow = N * 4;
    layout.rowsPerImage = N;
    extent := WGPUExtent3D.{ N, N, 1 };
    wgpuQueueWriteTexture(queue, *destination, texels.data, size_of(type_of(texels)), *layout, *extent);

    readback := create_buffer(device, 256 * N, .MapRead | .CopyDst);
    encoder := wgpuDeviceCreateCommandEncoder(device, null);
    copy_texture_to_buffer(encoder, texture, readback, N, N, 256);
    submit(queue, encoder);

    data := map_read(instance, readback, 256 * N);
    if data {
        wrong := 0;
        for y: 0..N - 1 for x : 0..N - 1 {
            if (cast(*u32) (data + y * 256 + x * 4)).* != texels[y * N + x] wrong += 1;
        }
        check(wrong == 0, "texture upload: % of % texels are wrong", wrong, N * N);
        print("texture upload: checked\n");
        wgpuBufferUnmap(readback);
    }
    wgpuBufferRelease(readback);
    wgpuTextureRelease(texture);
}

// An invalid shader is a validation error the scope catches (not a crash); an empty scope
// catches nothing.
error_scopes :: (instance: WGPUInstance, device: WGPUDevice) {
    wgpuDevicePushErrorScope(device, .Validation);
    bad := create_shader(device, "fn broken( { this is not WGSL");
    type, message := webgpu_pop_error_scope(instance, device);
    check(type == .Validation, "error scope: an invalid shader gave %, not a validation error", type);
    check(message.count > 0, "error scope: the validation error has no message");
    if bad wgpuShaderModuleRelease(bad);

    wgpuDevicePushErrorScope(device, .Validation);
    type = webgpu_pop_error_scope(instance, device);
    check(type == .NoError, "error scope: an empty scope gave %", type);
    print("error scopes: checked\n");
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

submit :: (queue: WGPUQueue, encoder: WGPUCommandEncoder) {
    commands := wgpuCommandEncoderFinish(encoder, null);
    wgpuQueueSubmit(queue, 1, *commands);
    wgpuCommandBufferRelease(commands);
    wgpuCommandEncoderRelease(encoder);
}

// Map `buffer` for reading; its bytes, or null (and a failure) when mapping failed.
map_read :: (instance: WGPUInstance, buffer: WGPUBuffer, size: u64) -> *u8 {
    if !webgpu_map_buffer(instance, buffer, .Read, 0, size) {
        check(false, "mapping a buffer failed");
        return null;
    }
    data := cast(*u8) wgpuBufferGetConstMappedRange(buffer, 0, size);
    check(data != null, "the mapped range is null");
    return data;
}
