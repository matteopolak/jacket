// A compute shader with WebGPU: the GPU runs the Collatz sequence from each of 1..COUNT and
// writes how many steps it took; the program maps the results and prints the longest. No window:
// it runs natively (wgpu-native) and in the browser playground alike.

#import "Basic";
#import "WebGPU";

COUNT :: 100_000;

SHADER :: #string WGSL
@group(0) @binding(0) var<storage, read_write> steps: array<u32>;

@compute @workgroup_size(64) fn main(@builtin(global_invocation_id) id: vec3u) {
    if id.x >= arrayLength(&steps) { return; }
    var n = id.x + 1u;
    var count = 0u;
    while n != 1u {
        if n % 2u == 0u { n = n / 2u; } else { n = 3u * n + 1u; }
        count += 1u;
    }
    steps[id.x] = count;
}
WGSL

main :: () {
    if !webgpu_available() {
        print("This browser has no WebGPU.\n");
        return;
    }
    instance := wgpuCreateInstance(null);
    adapter := webgpu_request_adapter(instance);
    if !adapter exit(1);
    device := webgpu_request_device(instance, adapter);
    if !device exit(1);
    queue := wgpuDeviceGetQueue(device);

    wgsl: WGPUShaderSourceWGSL;
    wgsl.code = to_string_view(SHADER);
    shader_desc: WGPUShaderModuleDescriptor;
    shader_desc.nextInChain = *wgsl.chain;
    shader := wgpuDeviceCreateShaderModule(device, *shader_desc);
    pipeline_desc: WGPUComputePipelineDescriptor;
    pipeline_desc.compute.module = shader;
    pipeline_desc.compute.entryPoint = to_string_view("main");
    pipeline := wgpuDeviceCreateComputePipeline(device, *pipeline_desc);

    // The shader writes `storage`; a copy goes to `readback`, which the CPU can map.
    SIZE :: COUNT * size_of(u32);
    buffer_desc: WGPUBufferDescriptor;
    buffer_desc.size = SIZE;
    buffer_desc.usage = .Storage | .CopySrc;
    storage := wgpuDeviceCreateBuffer(device, *buffer_desc);
    buffer_desc.usage = .MapRead | .CopyDst;
    readback := wgpuDeviceCreateBuffer(device, *buffer_desc);

    entry: WGPUBindGroupEntry;
    entry.binding = 0;
    entry.buffer = storage;
    entry.size = SIZE;
    bind_desc: WGPUBindGroupDescriptor;
    bind_desc.layout = wgpuComputePipelineGetBindGroupLayout(pipeline, 0);
    bind_desc.entryCount = 1;
    bind_desc.entries = *entry;
    bind_group := wgpuDeviceCreateBindGroup(device, *bind_desc);

    encoder := wgpuDeviceCreateCommandEncoder(device, null);
    pass := wgpuCommandEncoderBeginComputePass(encoder, null);
    wgpuComputePassEncoderSetPipeline(pass, pipeline);
    wgpuComputePassEncoderSetBindGroup(pass, 0, bind_group, 0, null);
    wgpuComputePassEncoderDispatchWorkgroups(pass, (COUNT + 63) / 64, 1, 1);
    wgpuComputePassEncoderEnd(pass);
    wgpuComputePassEncoderRelease(pass);
    wgpuCommandEncoderCopyBufferToBuffer(encoder, storage, 0, readback, 0, SIZE);
    commands := wgpuCommandEncoderFinish(encoder, null);
    wgpuQueueSubmit(queue, 1, *commands);

    // Waits for the GPU (and, in the browser, for the page) until the copy can be read.
    if !webgpu_map_buffer(instance, readback, .Read) exit(1);
    steps := cast(*u32) wgpuBufferGetConstMappedRange(readback, 0, SIZE);
    longest := 0;
    for 1..COUNT - 1 if steps[it] > steps[longest] longest = it;
    print("Below %, % takes the most Collatz steps: %.\n", COUNT + 1, longest + 1, steps[longest]);
    print("27 takes %.\n", steps[26]);
    wgpuBufferUnmap(readback);

    wgpuCommandBufferRelease(commands);
    wgpuCommandEncoderRelease(encoder);
    wgpuBindGroupRelease(bind_group);
    wgpuBindGroupLayoutRelease(bind_desc.layout);
    wgpuBufferRelease(readback);
    wgpuBufferRelease(storage);
    wgpuComputePipelineRelease(pipeline);
    wgpuShaderModuleRelease(shader);
    wgpuQueueRelease(queue);
    wgpuDeviceRelease(device);
    wgpuAdapterRelease(adapter);
    wgpuInstanceRelease(instance);
}
