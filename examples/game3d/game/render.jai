// The WebGPU side. A `Renderer` owns the device, the shared mesh buffers, the per-frame instance
// buffer and the scene pipeline. It can draw into a window's surface or into an offscreen texture
// (used by the smoke test, which reads the pixels back).

OFFSCREEN_FORMAT :: WGPUTextureFormat.RGBA8Unorm;
DEPTH_FORMAT :: WGPUTextureFormat.Depth24Plus;
INSTANCE_STRIDE :: size_of(Instance);

Globals :: struct {
    view_projection: [16] float;
    light: [4] float;
    camera: [4] float;
    fog: [4] float;
}

// How many instances of each (mesh, level of detail) a frame drew.
Draw_Stats :: struct {
    visible: s64;
    culled: s64;
    draws: s64;
    triangles: s64;
    per_lod: [LOD_COUNT] s64;
}

Renderer :: struct {
    window: Window_Type;
    instance: WGPUInstance;
    adapter: WGPUAdapter;
    device: WGPUDevice;
    queue: WGPUQueue;
    surface: WGPUSurface;
    format: WGPUTextureFormat;
    width: s32;
    height: s32;

    meshes: Mesh_Set;
    vertex_buffer: WGPUBuffer;
    index_buffer: WGPUBuffer;
    instance_buffer: WGPUBuffer;
    globals_buffer: WGPUBuffer;
    pipeline: WGPURenderPipeline;
    bind_group: WGPUBindGroup;
    depth_view: WGPUTextureView;
    depth_size: [2] s32;

    offscreen: WGPUTexture;
    offscreen_view: WGPUTextureView;

    // Per frame, rebuilt by `pack_instances`.
    packed: [..] Instance;
    buckets: [MESH_KINDS][LOD_COUNT] Bucket;
    stats: Draw_Stats;
    clear: [4] float = .[0.52, 0.64, 0.78, 1];
}

Bucket :: struct {
    first: s64;
    count: s64;
}

// Creates the device. With a window it also makes a surface and configures it; without one the
// renderer can only draw offscreen. Returns false (and says why) when there is no usable GPU.
renderer_init :: (r: *Renderer, window: Window_Type) -> bool {
    r.window = window;
    r.instance = wgpuCreateInstance(null);

    options: WGPURequestAdapterOptions;

    if window {
        r.surface = webgpu_create_surface(r.instance, window);

        if !r.surface {
            print("No WebGPU surface to draw into.\n");

            return false;
        }

        options.compatibleSurface = r.surface;
    }

    r.adapter = webgpu_request_adapter(r.instance, *options);

    if !r.adapter {
        print("No GPU adapter here.\n");

        return false;
    }

    r.device = webgpu_request_device(r.instance, r.adapter);

    if !r.device return false;

    r.queue = wgpuDeviceGetQueue(r.device);
    r.format = OFFSCREEN_FORMAT;

    if window {
        // The shader encodes sRGB itself, so ask for a format that does not.
        r.format = webgpu_surface_format(r.surface, r.adapter);

        if r.format == .Undefined return false;

        r.width, r.height = webgpu_window_size(window);
        configure_surface(r);
    }

    build_meshes(*r.meshes);
    upload_meshes(r);
    create_scene_pipeline(r);

    return true;
}

renderer_shutdown :: (r: *Renderer) {
    free_meshes(*r.meshes);
    array_free(r.packed);
}

adapter_name :: (r: *Renderer) -> string {
    info: WGPUAdapterInfo;

    wgpuAdapterGetInfo(r.adapter, *info);

    name := copy_string(trim(tprint("% % (%)", to_string(info.vendor), to_string(info.device), info.backendType)));

    wgpuAdapterInfoFreeMembers(info);

    return name;
}

configure_surface :: (r: *Renderer) {
    if r.width <= 0 || r.height <= 0 return;

    config: WGPUSurfaceConfiguration;
    config.device = r.device;
    config.format = r.format;
    config.usage = .RenderAttachment;
    config.width = cast(u32) r.width;
    config.height = cast(u32) r.height;
    config.presentMode = .Fifo;
    config.alphaMode = .Auto;
    wgpuSurfaceConfigure(r.surface, *config);
}

// Fills `instances` (one per entity, from `cull_job`) into the buffer layout the GPU draws from:
// every (mesh, level of detail) group is contiguous, so each group is one instanced draw. The
// floor goes last.
pack_instances :: (r: *Renderer, world: *World, instances: [] Instance, lods: [] u8) {
    array_reset_keeping_memory(*r.packed);
    r.stats = .{};

    // Count, assign group offsets, then place.
    for kind: 0..MESH_KINDS - 1 for lod : 0..LOD_COUNT - 1 r.buckets[kind][lod].count = 0;

    for i: 0..world.count - 1 {
        if lods[i] == NOT_VISIBLE {
            r.stats.culled += 1;

            continue;
        }

        r.buckets[world.entities.kind[i]][lods[i]].count += 1;
    }

    offset := 0;

    for kind: 0..MESH_KINDS - 1 {
        for lod: 0..LOD_COUNT - 1 {
            r.buckets[kind][lod].first = offset;
            offset += r.buckets[kind][lod].count;
            r.buckets[kind][lod].count = 0;
        }
    }

    array_resize(*r.packed, offset + 1);

    for i: 0..world.count - 1 {
        if lods[i] == NOT_VISIBLE continue;

        bucket := *r.buckets[world.entities.kind[i]][lods[i]];

        r.packed[bucket.first + bucket.count] = instances[i];
        bucket.count += 1;
        r.stats.per_lod[lods[i]] += 1;
    }

    r.stats.visible = offset;

    floor := scale(Matrix4_Identity, xyz(120, 1, 120));

    r.packed[offset] = make_instance(floor, .{ 0.42, 0.46, 0.44 }, 1);
    r.buckets[Mesh_Kind.FLOOR][0] = .{ first = offset, count = 1 };
}

// Draws one frame into `target` with the packed instances. The caller submits nothing else.
render_scene :: (r: *Renderer, target: WGPUTextureView, width: s32, height: s32, camera: *Camera) {
    ensure_depth(r, width, height);

    view_projection := camera_view_projection(camera, cast(float) width / cast(float) height);
    globals: Globals;

    globals.view_projection = transpose(view_projection).floats;
    globals.light = .[0.45, 0.8, 0.35, 0];
    globals.camera = .[camera.position.x, camera.position.y, camera.position.z, 0];
    globals.fog = .[r.clear[0], r.clear[1], r.clear[2], 0.008];
    wgpuQueueWriteBuffer(r.queue, r.globals_buffer, 0, *globals, size_of(Globals));
    wgpuQueueWriteBuffer(r.queue, r.instance_buffer, 0, r.packed.data, cast(u64) (r.packed.count * INSTANCE_STRIDE));

    color: WGPURenderPassColorAttachment;
    color.view = target;
    color.loadOp = .Clear;
    color.storeOp = .Store;
    color.clearValue = .{ r.clear[0], r.clear[1], r.clear[2], 1 };
    depth: WGPURenderPassDepthStencilAttachment;
    depth.view = r.depth_view;
    depth.depthLoadOp = .Clear;
    depth.depthStoreOp = .Discard;
    depth.depthClearValue = 1;
    pass_desc: WGPURenderPassDescriptor;
    pass_desc.colorAttachmentCount = 1;
    pass_desc.colorAttachments = *color;
    pass_desc.depthStencilAttachment = *depth;

    encoder := wgpuDeviceCreateCommandEncoder(r.device, null);
    pass := wgpuCommandEncoderBeginRenderPass(encoder, *pass_desc);

    wgpuRenderPassEncoderSetPipeline(pass, r.pipeline);
    wgpuRenderPassEncoderSetBindGroup(pass, 0, r.bind_group, 0, null);
    wgpuRenderPassEncoderSetVertexBuffer(pass, 0, r.vertex_buffer, 0, WGPU_WHOLE_SIZE);
    wgpuRenderPassEncoderSetVertexBuffer(pass, 1, r.instance_buffer, 0, WGPU_WHOLE_SIZE);
    wgpuRenderPassEncoderSetIndexBuffer(pass, r.index_buffer, .Uint32, 0, WGPU_WHOLE_SIZE);

    for kind: 0..MESH_KINDS - 1 {
        for lod: 0..LOD_COUNT - 1 {
            bucket := r.buckets[kind][lod];

            if bucket.count == 0 continue;

            range := r.meshes.ranges[kind][lod];

            wgpuRenderPassEncoderDrawIndexed(pass, range.index_count, cast(u32) bucket.count, range.first_index, range.base_vertex, cast(u32) bucket.first);
            r.stats.draws += 1;
            r.stats.triangles += cast(s64) (range.index_count / 3) * bucket.count;
        }
    }

    wgpuRenderPassEncoderEnd(pass);
    wgpuRenderPassEncoderRelease(pass);
    submit(r, encoder);
}

// Draws a frame into the window's surface. Returns false when there was nothing to draw into
// (a hidden or resizing window); try again next frame.
render_to_window :: (r: *Renderer, camera: *Camera) -> bool {
    width, height := webgpu_window_size(r.window);

    if width != r.width || height != r.height {
        r.width, r.height = width, height;
        configure_surface(r);
    }

    if width <= 0 || height <= 0 return false;

    frame: WGPUSurfaceTexture;

    wgpuSurfaceGetCurrentTexture(r.surface, *frame);

    if frame.status != .SuccessOptimal && frame.status != .SuccessSuboptimal {
        if frame.texture wgpuTextureRelease(frame.texture);

        r.width = 0; // reconfigure next time

        return false;
    }

    view := wgpuTextureCreateView(frame.texture, null);

    zone_begin(.DRAW);
    render_scene(r, view, width, height, camera);
    zone_end();

    zone_begin(.PRESENT);
    present_frame(r);
    zone_end();
    wgpuTextureViewRelease(view);
    wgpuTextureRelease(frame.texture);

    return true;
}

present_frame :: (r: *Renderer) {
    webgpu_present(r.surface);
}

// Draws into the offscreen texture of the given size and returns its pixels (RGBA, rows packed),
// which the caller frees.
render_offscreen :: (r: *Renderer, camera: *Camera, width: s32, height: s32) -> [] u8 {
    if !r.offscreen || r.width != width || r.height != height {
        if r.offscreen {
            wgpuTextureViewRelease(r.offscreen_view);
            wgpuTextureRelease(r.offscreen);
        }

        desc: WGPUTextureDescriptor;
        desc.usage = .RenderAttachment | .CopySrc | .TextureBinding;
        desc.dimension = ._2D;
        desc.size = .{ cast(u32) width, cast(u32) height, 1 };
        desc.format = OFFSCREEN_FORMAT;
        desc.mipLevelCount = 1;
        desc.sampleCount = 1;
        r.offscreen = wgpuDeviceCreateTexture(r.device, *desc);
        r.offscreen_view = wgpuTextureCreateView(r.offscreen, null);
        r.width, r.height = width, height;
    }

    render_scene(r, r.offscreen_view, width, height, camera);

    return webgpu_read_texture(r.instance, r.device, r.queue, r.offscreen, width, height);
}

submit :: (r: *Renderer, encoder: WGPUCommandEncoder) {
    commands := wgpuCommandEncoderFinish(encoder, null);

    wgpuQueueSubmit(r.queue, 1, *commands);
    wgpuCommandBufferRelease(commands);
    wgpuCommandEncoderRelease(encoder);
}

create_buffer :: (r: *Renderer, size: u64, usage: WGPUBufferUsage) -> WGPUBuffer {
    desc: WGPUBufferDescriptor;
    desc.size = size;
    desc.usage = usage;

    return wgpuDeviceCreateBuffer(r.device, *desc);
}

#scope_file

upload_meshes :: (r: *Renderer) {
    vertex_bytes := cast(u64) align_forward(r.meshes.vertices.count * size_of(Vertex), 4);
    index_bytes := cast(u64) align_forward(r.meshes.indices.count * size_of(u32), 4);

    r.vertex_buffer = create_buffer(r, vertex_bytes, .Vertex | .CopyDst);
    r.index_buffer = create_buffer(r, index_bytes, .Index | .CopyDst);
    r.instance_buffer = create_buffer(r, cast(u64) ((MAX_ENTITIES + 1) * INSTANCE_STRIDE), .Vertex | .CopyDst);
    r.globals_buffer = create_buffer(r, size_of(Globals), .Uniform | .CopyDst);

    wgpuQueueWriteBuffer(r.queue, r.vertex_buffer, 0, r.meshes.vertices.data, cast(u64) (r.meshes.vertices.count * size_of(Vertex)));
    wgpuQueueWriteBuffer(r.queue, r.index_buffer, 0, r.meshes.indices.data, cast(u64) (r.meshes.indices.count * size_of(u32)));
}

ensure_depth :: (r: *Renderer, width: s32, height: s32) {
    if r.depth_view && r.depth_size[0] == width && r.depth_size[1] == height return;

    if r.depth_view wgpuTextureViewRelease(r.depth_view);

    desc: WGPUTextureDescriptor;
    desc.usage = .RenderAttachment;
    desc.dimension = ._2D;
    desc.size = .{ cast(u32) width, cast(u32) height, 1 };
    desc.format = DEPTH_FORMAT;
    desc.mipLevelCount = 1;
    desc.sampleCount = 1;
    texture := wgpuDeviceCreateTexture(r.device, *desc);

    r.depth_view = wgpuTextureCreateView(texture, null);
    r.depth_size = .[width, height];
    wgpuTextureRelease(texture); // the view keeps it alive
}

create_scene_pipeline :: (r: *Renderer) {
    wgsl: WGPUShaderSourceWGSL;
    wgsl.code = to_string_view(SCENE_SHADER);
    shader_desc: WGPUShaderModuleDescriptor;
    shader_desc.nextInChain = *wgsl.chain;
    shader := wgpuDeviceCreateShaderModule(r.device, *shader_desc);

    vertex_attributes: [2] WGPUVertexAttribute;
    vertex_attributes[0] = .{ format = .Float32x3, offset = 0, shaderLocation = 0 };
    vertex_attributes[1] = .{ format = .Float32x3, offset = 12, shaderLocation = 1 };

    // One mat4 as four vec4 columns, then the colour.
    instance_attributes: [5] WGPUVertexAttribute;

    for 0..4 instance_attributes[it] = .{ format = .Float32x4, offset = cast(u64) (it * 16), shaderLocation = cast(u32) (2 + it) };

    layouts: [2] WGPUVertexBufferLayout;

    layouts[0].stepMode = .Vertex;
    layouts[0].arrayStride = size_of(Vertex);
    layouts[0].attributeCount = vertex_attributes.count;
    layouts[0].attributes = vertex_attributes.data;
    layouts[1].stepMode = .Instance;
    layouts[1].arrayStride = INSTANCE_STRIDE;
    layouts[1].attributeCount = instance_attributes.count;
    layouts[1].attributes = instance_attributes.data;

    target: WGPUColorTargetState;
    target.format = r.format;
    target.writeMask = .All;
    fragment: WGPUFragmentState;
    fragment.module = shader;
    fragment.entryPoint = to_string_view("fs");
    fragment.targetCount = 1;
    fragment.targets = *target;

    depth: WGPUDepthStencilState;
    depth.format = DEPTH_FORMAT;
    depth.depthWriteEnabled = .True;
    depth.depthCompare = .Less;

    desc: WGPURenderPipelineDescriptor;
    desc.vertex.module = shader;
    desc.vertex.entryPoint = to_string_view("vs");
    desc.vertex.bufferCount = layouts.count;
    desc.vertex.buffers = layouts.data;
    desc.primitive.topology = .TriangleList;
    desc.primitive.cullMode = .Back;
    desc.depthStencil = *depth;
    desc.multisample.count = 1;
    desc.multisample.mask = 0xFFFF_FFFF;
    desc.fragment = *fragment;
    r.pipeline = wgpuDeviceCreateRenderPipeline(r.device, *desc);

    entry: WGPUBindGroupEntry;

    entry.binding = 0;
    entry.buffer = r.globals_buffer;
    entry.size = size_of(Globals);
    bind_desc: WGPUBindGroupDescriptor;
    bind_desc.layout = wgpuRenderPipelineGetBindGroupLayout(r.pipeline, 0);
    bind_desc.entryCount = 1;
    bind_desc.entries = *entry;
    r.bind_group = wgpuDeviceCreateBindGroup(r.device, *bind_desc);
}

SCENE_SHADER :: #string WGSL
struct Globals {
    view_projection: mat4x4f,
    light: vec4f,
    camera: vec4f,
    fog: vec4f,
}
@group(0) @binding(0) var<uniform> g: Globals;

struct In {
    @location(0) position: vec3f,
    @location(1) normal: vec3f,
    @location(2) m0: vec4f,
    @location(3) m1: vec4f,
    @location(4) m2: vec4f,
    @location(5) m3: vec4f,
    @location(6) color: vec4f,
}

struct Out {
    @builtin(position) position: vec4f,
    @location(0) world: vec3f,
    @location(1) normal: vec3f,
    @location(2) color: vec4f,
}

@vertex fn vs(v: In) -> Out {
    let model = mat4x4f(v.m0, v.m1, v.m2, v.m3);
    let world = model * vec4f(v.position, 1.0);
    var out: Out;
    out.position = g.view_projection * world;
    out.world = world.xyz;
    out.normal = mat3x3f(v.m0.xyz, v.m1.xyz, v.m2.xyz) * v.normal;
    out.color = v.color;
    return out;
}

@fragment fn fs(v: Out) -> @location(0) vec4f {
    let n = normalize(v.normal);
    let l = normalize(g.light.xyz);
    let eye = g.camera.xyz - v.world;
    let eye_dir = normalize(eye);
    var base = v.color.rgb;

    if v.color.a > 0.5 {
        let tile = floor(v.world.x * 0.125) + floor(v.world.z * 0.125);
        base = mix(base, base * 0.6, abs(tile - 2.0 * floor(tile * 0.5)));
    }

    let sky = mix(vec3f(0.16, 0.14, 0.12), vec3f(0.28, 0.34, 0.46), n.y * 0.5 + 0.5);
    let diffuse = max(dot(n, l), 0.0);
    let half_way = normalize(l + eye_dir);
    let shine = pow(max(dot(n, half_way), 0.0), 40.0) * 0.3 * (1.0 - v.color.a);
    var lit = base * (sky + vec3f(1.0, 0.95, 0.85) * diffuse) + vec3f(shine);
    let fog_amount = 1.0 - exp(-length(eye) * g.fog.w);
    lit = mix(lit, g.fog.rgb, fog_amount);
    return vec4f(pow(lit, vec3f(1.0 / 2.2)), 1.0);
}
WGSL
