// The intro clip: an MPEG-1 video decoded with pl_mpeg, one frame at a time, into a texture that a
// full-screen quad shows (letterboxed to the clip's aspect ratio).

Intro :: struct {
    plm: *P.Plm;
    bytes: [] u8;
    width: s32;
    height: s32;
    frame_period: float64;
    clock: float64;
    shown: s64;
    done: bool;
    rgba: [] u8;

    texture: WGPUTexture;
    view: WGPUTextureView;
    sampler: WGPUSampler;
    pipeline: WGPURenderPipeline;
    bind_group: WGPUBindGroup;
    params: WGPUBuffer;
}

Intro_Params :: struct {
    scale: [2] float;
    padding: [2] float;
}

// Opens the clip from the assets. Returns false when it is missing or is not MPEG.
intro_open :: (intro: *Intro, path: string) -> bool {
    bytes, found := load_asset(path);

    if !found return false;

    intro.bytes = bytes;
    intro.plm = P.create_with_memory(bytes, false);

    if !intro.plm || !P.has_headers(intro.plm) {
        intro_close(intro);

        return false;
    }

    P.set_audio_enabled(intro.plm, false);
    intro.width, intro.height = P.get_size(intro.plm);
    intro.frame_period = 1.0 / P.get_framerate(intro.plm);
    intro.rgba = NewArray(intro.width * intro.height * 4, u8);

    return true;
}

intro_close :: (intro: *Intro) {
    if intro.plm P.plm_destroy(intro.plm);

    array_free(intro.bytes);
    array_free(intro.rgba);
    intro.* = .{};
}

// Makes the texture and pipeline the clip is drawn with.
intro_prepare :: (intro: *Intro, r: *Renderer) {
    desc: WGPUTextureDescriptor;
    desc.usage = .TextureBinding | .CopyDst;
    desc.dimension = ._2D;
    desc.size = .{ cast(u32) intro.width, cast(u32) intro.height, 1 };
    desc.format = .RGBA8Unorm;
    desc.mipLevelCount = 1;
    desc.sampleCount = 1;
    intro.texture = wgpuDeviceCreateTexture(r.device, *desc);
    intro.view = wgpuTextureCreateView(intro.texture, null);

    sampler_desc: WGPUSamplerDescriptor;
    sampler_desc.magFilter = .Linear;
    sampler_desc.minFilter = .Linear;
    sampler_desc.maxAnisotropy = 1;
    intro.sampler = wgpuDeviceCreateSampler(r.device, *sampler_desc);
    intro.params = create_buffer(r, size_of(Intro_Params), .Uniform | .CopyDst);

    wgsl: WGPUShaderSourceWGSL;
    wgsl.code = to_string_view(INTRO_SHADER);
    shader_desc: WGPUShaderModuleDescriptor;
    shader_desc.nextInChain = *wgsl.chain;
    shader := wgpuDeviceCreateShaderModule(r.device, *shader_desc);

    target: WGPUColorTargetState;
    target.format = r.format;
    target.writeMask = .All;
    fragment: WGPUFragmentState;
    fragment.module = shader;
    fragment.entryPoint = to_string_view("fs");
    fragment.targetCount = 1;
    fragment.targets = *target;
    pipeline_desc: WGPURenderPipelineDescriptor;
    pipeline_desc.vertex.module = shader;
    pipeline_desc.vertex.entryPoint = to_string_view("vs");
    pipeline_desc.primitive.topology = .TriangleStrip;
    pipeline_desc.multisample.count = 1;
    pipeline_desc.multisample.mask = 0xFFFF_FFFF;
    pipeline_desc.fragment = *fragment;
    intro.pipeline = wgpuDeviceCreateRenderPipeline(r.device, *pipeline_desc);

    entries: [3] WGPUBindGroupEntry;

    entries[0].binding = 0;
    entries[0].buffer = intro.params;
    entries[0].size = size_of(Intro_Params);
    entries[1].binding = 1;
    entries[1].textureView = intro.view;
    entries[2].binding = 2;
    entries[2].sampler = intro.sampler;
    bind_desc: WGPUBindGroupDescriptor;
    bind_desc.layout = wgpuRenderPipelineGetBindGroupLayout(intro.pipeline, 0);
    bind_desc.entryCount = entries.count;
    bind_desc.entries = entries.data;
    intro.bind_group = wgpuDeviceCreateBindGroup(r.device, *bind_desc);
}

// Advances the clip's clock and decodes the frames that are due, uploading the newest. Sets
// `done` once the stream ends.
intro_advance :: (intro: *Intro, r: *Renderer, seconds: float64) {
    intro.clock += seconds;

    while !intro.done && cast(float64) intro.shown * intro.frame_period <= intro.clock {
        if !intro_next_frame(intro) {
            intro.done = true;

            break;
        }

        intro_upload(intro, r);
    }
}

// Decodes the next frame into `intro.rgba`. False at the end of the clip.
intro_next_frame :: (intro: *Intro) -> bool {
    frame := P.decode_video(intro.plm);

    if !frame return false;

    P.frame_to_rgba(frame, intro.rgba.data, intro.width * 4);
    intro.shown += 1;

    return true;
}

intro_upload :: (intro: *Intro, r: *Renderer) {
    destination: WGPUTexelCopyTextureInfo;
    destination.texture = intro.texture;
    layout: WGPUTexelCopyBufferLayout;
    layout.bytesPerRow = cast(u32) intro.width * 4;
    layout.rowsPerImage = cast(u32) intro.height;
    size := WGPUExtent3D.{ cast(u32) intro.width, cast(u32) intro.height, 1 };

    wgpuQueueWriteTexture(r.queue, *destination, intro.rgba.data, cast(u64) intro.rgba.count, *layout, *size);
}

// Draws the newest frame letterboxed into `target`, which is `width` by `height` pixels.
intro_draw :: (intro: *Intro, r: *Renderer, target: WGPUTextureView, width: s32, height: s32) {
    clip_aspect := cast(float) intro.width / cast(float) intro.height;
    window_aspect := cast(float) width / cast(float) height;
    params: Intro_Params;

    params.scale = ifx window_aspect > clip_aspect then float.[clip_aspect / window_aspect, 1] else float.[1, window_aspect / clip_aspect];
    wgpuQueueWriteBuffer(r.queue, intro.params, 0, *params, size_of(Intro_Params));

    color: WGPURenderPassColorAttachment;
    color.view = target;
    color.loadOp = .Clear;
    color.storeOp = .Store;
    color.clearValue = .{ 0, 0, 0, 1 };
    pass_desc: WGPURenderPassDescriptor;
    pass_desc.colorAttachmentCount = 1;
    pass_desc.colorAttachments = *color;

    encoder := wgpuDeviceCreateCommandEncoder(r.device, null);
    pass := wgpuCommandEncoderBeginRenderPass(encoder, *pass_desc);

    wgpuRenderPassEncoderSetPipeline(pass, intro.pipeline);
    wgpuRenderPassEncoderSetBindGroup(pass, 0, intro.bind_group, 0, null);
    wgpuRenderPassEncoderDraw(pass, 4, 1, 0, 0);
    wgpuRenderPassEncoderEnd(pass);
    wgpuRenderPassEncoderRelease(pass);
    submit(r, encoder);
}

INTRO_SHADER :: #string WGSL
struct Params { scale: vec2f, padding: vec2f }
@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var clip: texture_2d<f32>;
@group(0) @binding(2) var clip_sampler: sampler;

struct Out { @builtin(position) position: vec4f, @location(0) uv: vec2f }

@vertex fn vs(@builtin(vertex_index) i: u32) -> Out {
    let x = f32(i & 1u) * 2.0 - 1.0;
    let y = f32(i >> 1u) * 2.0 - 1.0;
    var out: Out;
    out.position = vec4f(x * p.scale.x, y * p.scale.y, 0.0, 1.0);
    out.uv = vec2f((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

@fragment fn fs(v: Out) -> @location(0) vec4f {
    return textureSample(clip, clip_sampler, v.uv);
}
WGSL
