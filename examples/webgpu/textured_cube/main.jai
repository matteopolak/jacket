// A spinning cube with WebGPU: each corner has its own colour (red, green and blue along the three
// axes), and every face shows a small pixel-art texture over it. The texture is a PNG file shipped
// next to this program: load_asset reads it from disk natively and fetches it in the browser
// playground, and Extensions/Image decodes it the same way in both.
//
// Natively it opens a window (close it or press Escape to quit); in the playground it draws in the
// Render tab for CUBE_SECONDS seconds.
//
//     jaic examples/webgpu/textured_cube/main.jai -exe cube && ./cube

#import "Basic";
#import "Extensions/WebGPU";
#import "Extensions/Assets";
#import "Extensions/Image";

#if OS != .WASM {
    #import "Input";
    #import "Window_Creation";
}

TEXTURE_PATH :: "assets/jacket.png";
CUBE_SECONDS :: 60.0;
INDICES_PER_FACE :: 6;

main :: () {
    if !webgpu_available() {
        print("No WebGPU here (the playground needs Chrome or Edge 137 or newer).\n");
        return;
    }

    #if OS != .WASM {
        // Assets are looked up from the working directory: make it this program's folder, so it
        // runs from anywhere in the repository.
        set_working_directory(#filepath);

        window := create_window(800, 600, "WebGPU textured cube");
        if !window {
            print("No window could be opened (no display?).\n");
            return;
        }
    }

    instance := wgpuCreateInstance(null);

    #if OS == .WASM {
        surface := wgpuInstanceCreateSurface(instance, null); // the playground's canvas
    } else {
        surface := webgpu_create_surface(instance, window);
    }

    if !surface {
        print("No WebGPU surface to draw into.\n");
        return;
    }

    options: WGPURequestAdapterOptions;
    options.compatibleSurface = surface;
    adapter := webgpu_request_adapter(instance, *options);
    if !adapter {
        print("No GPU adapter here.\n");
        return;
    }

    device := webgpu_request_device(instance, adapter);
    if !device return;

    queue := wgpuDeviceGetQueue(device);

    // The texture and vertex colours are already sRGB-encoded, so the surface must not encode them
    // again.
    format := webgpu_surface_format(surface, adapter);

    if format == .Undefined return;

    texture_view := load_texture(device, queue);
    sampler_desc: WGPUSamplerDescriptor;
    sampler_desc.magFilter = .Nearest; // keep the pixels sharp
    sampler_desc.minFilter = .Nearest;
    sampler_desc.maxAnisotropy = 1;
    sampler := wgpuDeviceCreateSampler(device, *sampler_desc);

    vertices, indices := cube_mesh();
    vertex_buffer := make_buffer(device, queue, vertices.data, vertices.count * size_of(Vertex), .Vertex);
    index_buffer := make_buffer(device, queue, indices.data, indices.count * size_of(u16), .Index);

    uniform_desc: WGPUBufferDescriptor;
    uniform_desc.size = size_of(Uniforms);
    uniform_desc.usage = .Uniform | .CopyDst;
    uniform_buffer := wgpuDeviceCreateBuffer(device, *uniform_desc);

    pipeline := make_pipeline(device, format);

    entries: [3] WGPUBindGroupEntry;
    entries[0].binding = 0;
    entries[0].buffer = uniform_buffer;
    entries[0].size = size_of(Uniforms);
    entries[1].binding = 1;
    entries[1].textureView = texture_view;
    entries[2].binding = 2;
    entries[2].sampler = sampler;
    bind_desc: WGPUBindGroupDescriptor;
    bind_desc.layout = wgpuRenderPipelineGetBindGroupLayout(pipeline, 0);
    bind_desc.entryCount = entries.count;
    bind_desc.entries = entries.data;
    bind_group := wgpuDeviceCreateBindGroup(device, *bind_desc);

    width, height: s32;
    depth_view: WGPUTextureView;
    start := seconds_since_init();

    while true {
        #if OS == .WASM {
            if seconds_since_init() - start > CUBE_SECONDS break;
            new_width, new_height := webgpu_canvas_size();
        } else {
            update_window_events();
            quit := false;

            for events_this_frame {
                if it.type == .QUIT quit = true;
                if it.type == .KEYBOARD && it.key_pressed && it.key_code == .ESCAPE quit = true;
            }

            if quit break;

            new_width, new_height := webgpu_window_size(window);
        }

        // Configure the surface (and make a depth buffer of the same size) whenever it changes.
        if new_width != width || new_height != height {
            width, height = new_width, new_height;
            if width <= 0 || height <= 0 {
                sleep_milliseconds(10);
                continue;
            }

            configure(surface, device, format, width, height);
            if depth_view wgpuTextureViewRelease(depth_view);

            depth_view = make_depth_view(device, width, height);
        }

        frame: WGPUSurfaceTexture;

        wgpuSurfaceGetCurrentTexture(surface, *frame);

        if frame.status != .SuccessOptimal && frame.status != .SuccessSuboptimal {
            // Hidden or resized: try again next frame.
            if frame.texture wgpuTextureRelease(frame.texture);

            width = 0;
            sleep_milliseconds(10);
            continue;
        }

        u := Uniforms.{ time = cast(float32) (seconds_since_init() - start), aspect = cast(float32) width / cast(float32) height };

        wgpuQueueWriteBuffer(queue, uniform_buffer, 0, *u, size_of(Uniforms));

        view := wgpuTextureCreateView(frame.texture, null);
        color: WGPURenderPassColorAttachment;
        color.view = view;
        color.loadOp = .Clear;
        color.storeOp = .Store;
        color.clearValue = .{ 0.08, 0.08, 0.1, 1 };
        depth: WGPURenderPassDepthStencilAttachment;
        depth.view = depth_view;
        depth.depthLoadOp = .Clear;
        depth.depthStoreOp = .Discard;
        depth.depthClearValue = 1;
        pass_desc: WGPURenderPassDescriptor;
        pass_desc.colorAttachmentCount = 1;
        pass_desc.colorAttachments = *color;
        pass_desc.depthStencilAttachment = *depth;

        encoder := wgpuDeviceCreateCommandEncoder(device, null);
        pass := wgpuCommandEncoderBeginRenderPass(encoder, *pass_desc);

        wgpuRenderPassEncoderSetPipeline(pass, pipeline);
        wgpuRenderPassEncoderSetBindGroup(pass, 0, bind_group, 0, null);
        wgpuRenderPassEncoderSetVertexBuffer(pass, 0, vertex_buffer, 0, WGPU_WHOLE_SIZE);
        wgpuRenderPassEncoderSetIndexBuffer(pass, index_buffer, .Uint16, 0, WGPU_WHOLE_SIZE);
        wgpuRenderPassEncoderDrawIndexed(pass, cast(u32) indices.count, 1, 0, 0, 0);
        wgpuRenderPassEncoderEnd(pass);
        wgpuRenderPassEncoderRelease(pass);
        commands := wgpuCommandEncoderFinish(encoder, null);

        wgpuQueueSubmit(queue, 1, *commands);
        wgpuCommandBufferRelease(commands);
        wgpuCommandEncoderRelease(encoder);

        webgpu_present(surface); // waits for the next frame
        wgpuTextureViewRelease(view);
        wgpuTextureRelease(frame.texture);
    }
}

Vertex :: struct {
    position: [3] float32;
    color: [3] float32;
    uv: [2] float32;
}

Uniforms :: struct {
    time: float32;
    aspect: float32;
    _: [2] float32; // uniforms come in 16-byte blocks
}

// Four corners per face (so each face has its own texture coordinates) and two triangles each.
cube_mesh :: () -> [] Vertex, [] u16 {
    // Each face: the axis it faces along, and the two axes across it.
    faces :: [3] float32.[
        .[1, 0, 0], .[0, 0, -1], .[0, 1, 0],
        .[-1, 0, 0], .[0, 0, 1], .[0, 1, 0],
        .[0, 1, 0], .[1, 0, 0], .[0, 0, -1],
        .[0, -1, 0], .[1, 0, 0], .[0, 0, 1],
        .[0, 0, 1], .[1, 0, 0], .[0, 1, 0],
        .[0, 0, -1], .[-1, 0, 0], .[0, 1, 0],
    ];

    vertices := NewArray(24, Vertex);
    indices := NewArray(36, u16);

    for face: 0..5 {
        normal, across, up := faces[face * 3], faces[face * 3 + 1], faces[face * 3 + 2];

        for corner: 0..3 {
            s := cast(float32) (corner & 1);
            t := cast(float32) (corner >> 1);
            v := *vertices[face * 4 + corner];

            for 0..2 {
                v.position[it] = normal[it] + (s * 2 - 1) * across[it] + (1 - t * 2) * up[it];
                v.color[it] = v.position[it] * 0.5 + 0.5; // the corner at (1, 0, 0) is red
            }

            v.uv = .[s, t];
        }

        base := cast(u16) (face * 4);
        for u16.[0, 2, 1, 1, 2, 3] indices[face * INDICES_PER_FACE + it_index] = base + it;
    }

    return vertices, indices;
}

// The texture as a view to bind: the PNG decoded to RGBA, or one white pixel if it is missing.
load_texture :: (device: WGPUDevice, queue: WGPUQueue) -> WGPUTextureView {
    image: Image;
    bytes, found := load_asset(TEXTURE_PATH);

    if found {
        decoded, ok, error := decode_image_as(bytes, .RGBA8);
        if ok image = decoded;
        else print("Could not decode %: %\n", TEXTURE_PATH, error);
        array_free(bytes);
    } else {
        print("Could not load %; the faces are left plain.\n", TEXTURE_PATH);
    }

    if !image.pixels {
        image = make_image(1, 1, .RGBA8);
        memset(image.pixels.data, 0, 4); // fully transparent: the corner colours show alone
    }

    defer deinit(*image);

    size := WGPUExtent3D.{ cast(u32) image.width, cast(u32) image.height, 1 };
    desc: WGPUTextureDescriptor;
    desc.usage = .TextureBinding | .CopyDst;
    desc.dimension = ._2D;
    desc.size = size;
    desc.format = .RGBA8Unorm;
    desc.mipLevelCount = 1;
    desc.sampleCount = 1;
    texture := wgpuDeviceCreateTexture(device, *desc);

    destination: WGPUTexelCopyTextureInfo;
    destination.texture = texture;
    layout: WGPUTexelCopyBufferLayout;
    layout.bytesPerRow = cast(u32) image.width * 4;
    layout.rowsPerImage = cast(u32) image.height;
    wgpuQueueWriteTexture(queue, *destination, image.pixels.data, cast(u64) image.pixels.count, *layout, *size);

    return wgpuTextureCreateView(texture, null);
}

make_buffer :: (device: WGPUDevice, queue: WGPUQueue, data: *void, size: s64, usage: WGPUBufferUsage) -> WGPUBuffer {
    desc: WGPUBufferDescriptor;
    desc.size = cast(u64) align_forward(size, 4);
    desc.usage = usage | .CopyDst;
    buffer := wgpuDeviceCreateBuffer(device, *desc);

    wgpuQueueWriteBuffer(queue, buffer, 0, data, desc.size);

    return buffer;
}

make_pipeline :: (device: WGPUDevice, format: WGPUTextureFormat) -> WGPURenderPipeline {
    wgsl: WGPUShaderSourceWGSL;
    wgsl.code = to_string_view(SHADER);
    shader_desc: WGPUShaderModuleDescriptor;
    shader_desc.nextInChain = *wgsl.chain;
    shader := wgpuDeviceCreateShaderModule(device, *shader_desc);

    attributes: [3] WGPUVertexAttribute;
    attributes[0] = .{ format = .Float32x3, offset = 0, shaderLocation = 0 };
    attributes[1] = .{ format = .Float32x3, offset = 12, shaderLocation = 1 };
    attributes[2] = .{ format = .Float32x2, offset = 24, shaderLocation = 2 };
    vertex_layout: WGPUVertexBufferLayout;
    vertex_layout.stepMode = .Vertex;
    vertex_layout.arrayStride = size_of(Vertex);
    vertex_layout.attributeCount = attributes.count;
    vertex_layout.attributes = attributes.data;

    target: WGPUColorTargetState;
    target.format = format;
    target.writeMask = .All;
    fragment: WGPUFragmentState;
    fragment.module = shader;
    fragment.entryPoint = to_string_view("fs");
    fragment.targetCount = 1;
    fragment.targets = *target;

    depth: WGPUDepthStencilState;
    depth.format = .Depth24Plus;
    depth.depthWriteEnabled = .True;
    depth.depthCompare = .Less;

    desc: WGPURenderPipelineDescriptor;
    desc.vertex.module = shader;
    desc.vertex.entryPoint = to_string_view("vs");
    desc.vertex.bufferCount = 1;
    desc.vertex.buffers = *vertex_layout;
    desc.primitive.topology = .TriangleList;
    desc.depthStencil = *depth;
    desc.multisample.count = 1;
    desc.multisample.mask = 0xFFFF_FFFF;
    desc.fragment = *fragment;

    return wgpuDeviceCreateRenderPipeline(device, *desc);
}

make_depth_view :: (device: WGPUDevice, width: s32, height: s32) -> WGPUTextureView {
    desc: WGPUTextureDescriptor;
    desc.usage = .RenderAttachment;
    desc.dimension = ._2D;
    desc.size = .{ cast(u32) width, cast(u32) height, 1 };
    desc.format = .Depth24Plus;
    desc.mipLevelCount = 1;
    desc.sampleCount = 1;
    texture := wgpuDeviceCreateTexture(device, *desc);
    view := wgpuTextureCreateView(texture, null);

    wgpuTextureRelease(texture); // the view keeps it alive

    return view;
}

configure :: (surface: WGPUSurface, device: WGPUDevice, format: WGPUTextureFormat, width: s32, height: s32) {
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

SHADER :: #string WGSL
struct Uniforms { time: f32, aspect: f32, _pad: vec2f }
@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var picture: texture_2d<f32>;
@group(0) @binding(2) var pixels: sampler;

struct In { @location(0) position: vec3f, @location(1) color: vec3f, @location(2) uv: vec2f }
struct Out { @builtin(position) position: vec4f, @location(0) color: vec3f, @location(1) uv: vec2f }

@vertex fn vs(v: In) -> Out {
    // Turn the cube about y, then about x, at different speeds.
    let a = u.time * 0.9;
    let b = u.time * 0.6;
    let spin = mat3x3f(cos(a), 0.0, -sin(a), 0.0, 1.0, 0.0, sin(a), 0.0, cos(a));
    let tilt = mat3x3f(1.0, 0.0, 0.0, 0.0, cos(b), sin(b), 0.0, -sin(b), cos(b));
    let p = tilt * spin * v.position;

    // Perspective from a camera 4.5 units away on +z: w is the distance in front of it, and depth
    // runs from 0 at the near plane to 1 at the far one.
    let distance = 4.5 - p.z;
    let focal = 1.8; // 1 / tan(half the vertical field of view)
    let near = 0.1;
    let far = 100.0;
    var out: Out;
    out.position = vec4f(p.x * focal / u.aspect, p.y * focal, far * (distance - near) / (far - near), distance);
    out.color = v.color;
    out.uv = v.uv;
    return out;
}

@fragment fn fs(v: Out) -> @location(0) vec4f {
    let texel = textureSample(picture, pixels, v.uv);
    // The texture over the corner colours, where it is opaque.
    return vec4f(mix(v.color, texel.rgb, texel.a), 1.0);
}
WGSL
