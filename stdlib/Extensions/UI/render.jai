// The WebGPU renderer: turns a frame's display list into one instanced draw per clip rectangle.
//
// One pipeline draws everything. Each display list entry becomes one instance of a quad; rounded
// rectangles are evaluated as a signed distance in the fragment shader and glyphs sample the atlas
// texture. The same renderer draws into a window's surface or, for tests and screenshots, into an
// offscreen texture it reads back.

// A renderer bound to one window or to an offscreen target.
Renderer :: struct {
    instance: WGPUInstance;
    adapter: WGPUAdapter;
    device: WGPUDevice;
    queue: WGPUQueue;
    surface: WGPUSurface; // null when rendering offscreen
    format: WGPUTextureFormat;
    pipeline: WGPURenderPipeline;
    uniforms: WGPUBuffer;
    instances: WGPUBuffer;
    instance_capacity: int;
    bind_group: WGPUBindGroup;
    atlas: WGPUTexture;
    atlas_view: WGPUTextureView;
    sampler: WGPUSampler;
    atlas_version: int;
    width: int; // target size in pixels
    height: int;
    target: WGPUTexture; // offscreen only
    target_view: WGPUTextureView;
    staged: [..] Instance_Data;
    batches: [..] Batch;
}

// One entry of the instance buffer, as the shader reads it.
Instance_Data :: struct {
    rect: Vector4; // x, y, width, height in pixels
    uv: Vector4;
    color: Vector4;
    border_color: Vector4;
    params: Vector4; // radius, border width, 1 for a glyph, unused
}

// A run of instances drawn with one scissor rectangle.
Batch :: struct {
    first: int;
    count: int;
    x: int;
    y: int;
    w: int;
    h: int;
}

// Pixels read back from an offscreen render: rows top to bottom, 4 bytes per pixel (R, G, B, A).
Rendered_Image :: struct {
    width: int;
    height: int;
    pixels: [..] u8;
}

// Create a renderer drawing into `window`'s surface. Returns null if no GPU or surface is available.
//
// - window: the window to draw into.
// - width: the drawable width in pixels.
// - height: the drawable height in pixels.
create_window_renderer :: (window: Window_Type, width: int, height: int) -> *Renderer {
    r := New(Renderer);
    r.instance = wgpuCreateInstance(null);
    if !r.instance {
        free(r);
        return null;
    }

    r.surface = webgpu_create_surface(r.instance, window);
    if !r.surface {
        free(r);
        return null;
    }

    options: WGPURequestAdapterOptions;
    options.compatibleSurface = r.surface;
    r.adapter = webgpu_request_adapter(r.instance, *options);

    if !r.adapter || !start_device(r) {
        free(r);
        return null;
    }

    r.format = webgpu_surface_format(r.surface, r.adapter);
    resize_target(r, width, height);
    create_pipeline(r);

    return r;
}

// Create a renderer drawing into an offscreen texture of `width` by `height` pixels, for tests and
// screenshots. Returns null if no adapter is available.
//
// - width: the target width in pixels.
// - height: the target height in pixels.
create_offscreen_renderer :: (width: int, height: int) -> *Renderer {
    r := New(Renderer);
    r.instance = wgpuCreateInstance(null);
    if !r.instance {
        free(r);
        return null;
    }

    r.adapter = webgpu_request_adapter(r.instance, null);

    if !r.adapter || !start_device(r) {
        free(r);
        return null;
    }

    r.format = .RGBA8Unorm;
    resize_target(r, width, height);
    create_pipeline(r);

    return r;
}

// Release the renderer and its GPU objects.
//
// - r: the renderer to release.
destroy_renderer :: (r: *Renderer) {
    if r.target_view wgpuTextureViewRelease(r.target_view);
    if r.target wgpuTextureRelease(r.target);
    if r.bind_group wgpuBindGroupRelease(r.bind_group);
    if r.instances wgpuBufferRelease(r.instances);
    if r.uniforms wgpuBufferRelease(r.uniforms);
    if r.atlas_view wgpuTextureViewRelease(r.atlas_view);
    if r.atlas wgpuTextureRelease(r.atlas);
    if r.sampler wgpuSamplerRelease(r.sampler);
    if r.pipeline wgpuRenderPipelineRelease(r.pipeline);
    if r.queue wgpuQueueRelease(r.queue);
    if r.surface wgpuSurfaceRelease(r.surface);
    if r.device wgpuDeviceRelease(r.device);
    if r.adapter wgpuAdapterRelease(r.adapter);
    if r.instance wgpuInstanceRelease(r.instance);

    array_free(r.staged);
    array_free(r.batches);
    free(r);
}

// Match the target to a new size in pixels (a window resize).
//
// - r: the renderer to resize.
// - width: the new width in pixels. Zero or less does nothing.
// - height: the new height in pixels. Zero or less does nothing.
resize_target :: (r: *Renderer, width: int, height: int) {
    if width <= 0 || height <= 0 return;

    r.width = width;
    r.height = height;

    if r.surface {
        config: WGPUSurfaceConfiguration;
        config.device = r.device;
        config.format = r.format;
        config.usage = .RenderAttachment;
        config.width = cast(u32) width;
        config.height = cast(u32) height;
        config.presentMode = .Fifo;
        config.alphaMode = .Auto;
        wgpuSurfaceConfigure(r.surface, *config);
    } else {
        if r.target_view wgpuTextureViewRelease(r.target_view);
        if r.target wgpuTextureRelease(r.target);

        desc: WGPUTextureDescriptor;
        desc.usage = .RenderAttachment | .CopySrc;
        desc.dimension = ._2D;
        desc.size = .{ cast(u32) width, cast(u32) height, 1 };
        desc.format = r.format;
        r.target = wgpuDeviceCreateTexture(r.device, *desc);
        r.target_view = wgpuTextureCreateView(r.target, null);
    }
}

// Draw the UI's display list into the window. Returns false when there was nothing to draw into
// (the window is hidden) and the frame should be tried again.
//
// - r: the window renderer.
// - ui: the UI whose display list to draw.
// - clear: the colour the window is cleared to first.
present :: (r: *Renderer, ui: *UI_State, clear: Color) -> bool {
    frame: WGPUSurfaceTexture;

    wgpuSurfaceGetCurrentTexture(r.surface, *frame);

    if frame.status == WEBGPU_SURFACE_OCCLUDED return false;

    if frame.status != .SuccessOptimal && frame.status != .SuccessSuboptimal {
        resize_target(r, r.width, r.height);
        return false;
    }

    view := wgpuTextureCreateView(frame.texture, null);

    encode(r, ui, view, clear);
    webgpu_present(r.surface);
    wgpuTextureViewRelease(view);
    wgpuTextureRelease(frame.texture);

    return true;
}

// Draw the UI's display list offscreen and read the pixels back. Free `pixels` with `array_free`.
//
// - r: the offscreen renderer.
// - ui: the UI whose display list to draw.
// - clear: the colour the target is cleared to first.
render_to_image :: (r: *Renderer, ui: *UI_State, clear: Color) -> Rendered_Image {
    encode(r, ui, r.target_view, clear);

    image: Rendered_Image;
    image.width = r.width;
    image.height = r.height;

    row_bytes := (r.width * 4 + 255) / 256 * 256;
    size := cast(u64) (row_bytes * r.height);

    desc: WGPUBufferDescriptor;
    desc.size = size;
    desc.usage = .MapRead | .CopyDst;
    readback := wgpuDeviceCreateBuffer(r.device, *desc);

    encoder := wgpuDeviceCreateCommandEncoder(r.device, null);
    source: WGPUTexelCopyTextureInfo;
    source.texture = r.target;
    source.aspect = .All;
    destination: WGPUTexelCopyBufferInfo;
    destination.buffer = readback;
    destination.layout.bytesPerRow = cast(u32) row_bytes;
    destination.layout.rowsPerImage = cast(u32) r.height;
    extent := WGPUExtent3D.{ cast(u32) r.width, cast(u32) r.height, 1 };

    wgpuCommandEncoderCopyTextureToBuffer(encoder, *source, *destination, *extent);
    commands := wgpuCommandEncoderFinish(encoder, null);

    wgpuQueueSubmit(r.queue, 1, *commands);
    wgpuCommandBufferRelease(commands);
    wgpuCommandEncoderRelease(encoder);

    if webgpu_map_buffer(r.instance, readback, .Read, 0, size) {
        data := cast(*u8) wgpuBufferGetConstMappedRange(readback, 0, size);

        array_resize(*image.pixels, r.width * r.height * 4);

        for y: 0..r.height - 1 {
            memcpy(image.pixels.data + y * r.width * 4, data + y * row_bytes, r.width * 4);
        }

        wgpuBufferUnmap(readback);
    }

    wgpuBufferRelease(readback);

    return image;
}

// Draw the UI's last frame offscreen and write it to `path` as a PNG. Needs a GPU; build the UI
// with `system_fonts` for readable text. Returns false when there is no adapter or the file
// cannot be written.
//
// - ui: the UI whose last frame to draw.
// - path: where to write the PNG.
write_screenshot :: (ui: *UI_State, path: string) -> bool {
    width := cast(int) ceil(ui.size.x * ui.text.scale);
    height := cast(int) ceil(ui.size.y * ui.text.scale);
    r := create_offscreen_renderer(width, height);

    if !r return false;

    defer destroy_renderer(r);

    image := render_to_image(r, ui, ui.theme.background);

    defer array_free(image.pixels);
    if !image.pixels.count return false;

    // Offscreen targets are premultiplied over an opaque clear colour, so alpha is already 255.
    picture := Image.{ cast(s32) image.width, cast(s32) image.height, .RGBA8, image.pixels };
    png := encode_png(picture);

    defer array_free(png);

    return write_entire_file(path, string.{ png.count, png.data });
}

#scope_module

SHADER :: #string WGSL
struct Globals { size: vec2f, pad: vec2f }
struct Inst { rect: vec4f, uv: vec4f, color: vec4f, border_color: vec4f, params: vec4f }

@group(0) @binding(0) var<uniform> g: Globals;
@group(0) @binding(1) var<storage, read> insts: array<Inst>;
@group(0) @binding(2) var atlas: texture_2d<f32>;
@group(0) @binding(3) var samp: sampler;

struct VOut {
    @builtin(position) pos: vec4f,
    @location(0) local: vec2f,
    @location(1) uv: vec2f,
    @location(2) @interpolate(flat) idx: u32,
}

@vertex fn vs(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VOut {
    let corner = vec2f(f32(vi & 1u), f32((vi >> 1u) & 1u));
    let inst = insts[ii];
    let px = inst.rect.xy + corner * inst.rect.zw;
    var out: VOut;
    out.pos = vec4f(px.x / g.size.x * 2.0 - 1.0, 1.0 - px.y / g.size.y * 2.0, 0.0, 1.0);
    out.local = corner * inst.rect.zw;
    out.uv = mix(inst.uv.xy, inst.uv.zw, corner);
    out.idx = ii;
    return out;
}

@fragment fn fs(in: VOut) -> @location(0) vec4f {
    let inst = insts[in.idx];

    if (inst.params.z > 0.5) {
        let a = textureSampleLevel(atlas, samp, in.uv, 0.0).r * inst.color.a;
        return vec4f(inst.color.rgb * a, a);
    }

    let half = inst.rect.zw * 0.5;
    let p = in.local - half;
    let r = min(inst.params.x, min(half.x, half.y));
    let q = abs(p) - (half - vec2f(r));
    let d = length(max(q, vec2f(0.0))) + min(max(q.x, q.y), 0.0) - r;
    let outer = clamp(0.5 - d, 0.0, 1.0);
    var rgb = inst.color.rgb;
    var a = inst.color.a;

    if (inst.params.y > 0.0) {
        let inner = clamp(0.5 - (d + inst.params.y), 0.0, 1.0);
        rgb = mix(inst.border_color.rgb, inst.color.rgb, inner);
        a = mix(inst.border_color.a, inst.color.a, inner);
    }

    let alpha = a * outer;
    return vec4f(rgb * alpha, alpha);
}
WGSL

start_device :: (r: *Renderer) -> bool {
    r.device = webgpu_request_device(r.instance, r.adapter);
    if !r.device return false;

    r.queue = wgpuDeviceGetQueue(r.device);
    return true;
}

create_pipeline :: (r: *Renderer) {
    wgsl: WGPUShaderSourceWGSL;
    wgsl.code = to_string_view(SHADER);
    shader_desc: WGPUShaderModuleDescriptor;
    shader_desc.nextInChain = *wgsl.chain;
    shader := wgpuDeviceCreateShaderModule(r.device, *shader_desc);

    blend: WGPUBlendState;
    blend.color = .{ .Add, .One, .OneMinusSrcAlpha };
    blend.alpha = .{ .Add, .One, .OneMinusSrcAlpha };

    target: WGPUColorTargetState;
    target.format = r.format;
    target.writeMask = .All;
    target.blend = *blend;

    fragment: WGPUFragmentState;
    fragment.module = shader;
    fragment.entryPoint = to_string_view("fs");
    fragment.targetCount = 1;
    fragment.targets = *target;

    desc: WGPURenderPipelineDescriptor;
    desc.label = to_string_view("ui");
    desc.vertex.module = shader;
    desc.vertex.entryPoint = to_string_view("vs");
    desc.primitive.topology = .TriangleStrip;
    desc.multisample.count = 1;
    desc.multisample.mask = 0xFFFF_FFFF;
    desc.fragment = *fragment;
    r.pipeline = wgpuDeviceCreateRenderPipeline(r.device, *desc);
    wgpuShaderModuleRelease(shader);

    buffer_desc: WGPUBufferDescriptor;
    buffer_desc.size = 16;
    buffer_desc.usage = .Uniform | .CopyDst;
    r.uniforms = wgpuDeviceCreateBuffer(r.device, *buffer_desc);

    sampler_desc: WGPUSamplerDescriptor;
    sampler_desc.addressModeU = .ClampToEdge;
    sampler_desc.addressModeV = .ClampToEdge;
    sampler_desc.addressModeW = .ClampToEdge;
    sampler_desc.magFilter = .Nearest;
    sampler_desc.minFilter = .Nearest;
    r.sampler = wgpuDeviceCreateSampler(r.device, *sampler_desc);

    atlas_desc: WGPUTextureDescriptor;
    atlas_desc.usage = .TextureBinding | .CopyDst;
    atlas_desc.dimension = ._2D;
    atlas_desc.size = .{ ATLAS_SIZE, ATLAS_SIZE, 1 };
    atlas_desc.format = .R8Unorm;
    r.atlas = wgpuDeviceCreateTexture(r.device, *atlas_desc);
    r.atlas_view = wgpuTextureCreateView(r.atlas, null);
    r.atlas_version = -1;
}

ATLAS_SIZE :: 1024;

// Make sure the instance buffer holds `count` instances, which invalidates the bind group.
ensure_instance_capacity :: (r: *Renderer, count: int) {
    if count <= r.instance_capacity && r.bind_group return;

    capacity := max(r.instance_capacity, 256);
    while capacity < count capacity *= 2;

    if capacity != r.instance_capacity || !r.instances {
        if r.instances wgpuBufferRelease(r.instances);

        desc: WGPUBufferDescriptor;
        desc.size = cast(u64) (capacity * size_of(Instance_Data));
        desc.usage = .Storage | .CopyDst;
        r.instances = wgpuDeviceCreateBuffer(r.device, *desc);
        r.instance_capacity = capacity;
    }

    if r.bind_group wgpuBindGroupRelease(r.bind_group);

    entries: [4] WGPUBindGroupEntry;
    entries[0].binding = 0;
    entries[0].buffer = r.uniforms;
    entries[0].size = 16;
    entries[1].binding = 1;
    entries[1].buffer = r.instances;
    entries[1].size = cast(u64) (r.instance_capacity * size_of(Instance_Data));
    entries[2].binding = 2;
    entries[2].textureView = r.atlas_view;
    entries[3].binding = 3;
    entries[3].sampler = r.sampler;

    desc: WGPUBindGroupDescriptor;
    desc.layout = wgpuRenderPipelineGetBindGroupLayout(r.pipeline, 0);
    desc.entryCount = 4;
    desc.entries = entries.data;
    r.bind_group = wgpuDeviceCreateBindGroup(r.device, *desc);
    wgpuBindGroupLayoutRelease(desc.layout);
}

// Turn the display list into instances and batches, upload them, and draw.
encode :: (r: *Renderer, ui: *UI_State, view: WGPUTextureView, clear: Color) {
    scale := ui.text.scale;

    array_reset(*r.staged);
    array_reset(*r.batches);

    current := Batch.{ 0, 0, 0, 0, r.width, r.height };

    for cmd: ui.draw.cmds {
        if cmd.kind == .CLIP {
            if current.count > 0 array_add(*r.batches, current);

            x0 := clamp(cast(int) floor(cmd.rect.x * scale), 0, r.width);
            y0 := clamp(cast(int) floor(cmd.rect.y * scale), 0, r.height);
            x1 := clamp(cast(int) ceil((cmd.rect.x + cmd.rect.w) * scale), 0, r.width);
            y1 := clamp(cast(int) ceil((cmd.rect.y + cmd.rect.h) * scale), 0, r.height);

            current = .{ r.staged.count, 0, x0, y0, x1 - x0, y1 - y0 };
            continue;
        }

        instance: Instance_Data;
        instance.rect = .{ cmd.rect.x * scale, cmd.rect.y * scale, cmd.rect.w * scale, cmd.rect.h * scale };
        instance.color = cmd.color;
        instance.border_color = cmd.border_color;

        if cmd.kind == .GLYPH {
            instance.uv = cmd.uv;
            instance.params = .{ 0, 0, 1, 0 };
        } else {
            instance.params = .{ cmd.radius * scale, cmd.border * scale, 0, 0 };
        }

        array_add(*r.staged, instance);
        current.count += 1;
    }

    if current.count > 0 array_add(*r.batches, current);

    ensure_instance_capacity(r, r.staged.count);

    if ui.text.atlas.version != r.atlas_version && ui.text.atlas.pixels.count == ATLAS_SIZE * ATLAS_SIZE {
        destination: WGPUTexelCopyTextureInfo;
        destination.texture = r.atlas;
        destination.aspect = .All;
        layout: WGPUTexelCopyBufferLayout;
        layout.bytesPerRow = ATLAS_SIZE;
        layout.rowsPerImage = ATLAS_SIZE;
        extent := WGPUExtent3D.{ ATLAS_SIZE, ATLAS_SIZE, 1 };

        wgpuQueueWriteTexture(r.queue, *destination, ui.text.atlas.pixels.data, cast(u64) ui.text.atlas.pixels.count, *layout, *extent);
        r.atlas_version = ui.text.atlas.version;
    }

    globals := Vector4.{ cast(float) r.width, cast(float) r.height, 0, 0 };

    wgpuQueueWriteBuffer(r.queue, r.uniforms, 0, *globals, 16);

    if r.staged.count {
        wgpuQueueWriteBuffer(r.queue, r.instances, 0, r.staged.data, cast(u64) (r.staged.count * size_of(Instance_Data)));
    }

    color: WGPURenderPassColorAttachment;
    color.view = view;
    color.depthSlice = WGPU_DEPTH_SLICE_UNDEFINED;
    color.loadOp = .Clear;
    color.storeOp = .Store;
    color.clearValue = .{ clear.x, clear.y, clear.z, clear.w };
    pass_desc: WGPURenderPassDescriptor;
    pass_desc.colorAttachmentCount = 1;
    pass_desc.colorAttachments = *color;

    encoder := wgpuDeviceCreateCommandEncoder(r.device, null);
    pass := wgpuCommandEncoderBeginRenderPass(encoder, *pass_desc);

    wgpuRenderPassEncoderSetPipeline(pass, r.pipeline);
    wgpuRenderPassEncoderSetBindGroup(pass, 0, r.bind_group, 0, null);

    for batch: r.batches {
        if batch.w <= 0 || batch.h <= 0 continue;

        wgpuRenderPassEncoderSetScissorRect(pass, cast(u32) batch.x, cast(u32) batch.y, cast(u32) batch.w, cast(u32) batch.h);
        wgpuRenderPassEncoderDraw(pass, 4, cast(u32) batch.count, 0, cast(u32) batch.first);
    }

    wgpuRenderPassEncoderEnd(pass);
    wgpuRenderPassEncoderRelease(pass);
    commands := wgpuCommandEncoderFinish(encoder, null);

    wgpuQueueSubmit(r.queue, 1, *commands);
    wgpuCommandBufferRelease(commands);
    wgpuCommandEncoderRelease(encoder);
}
