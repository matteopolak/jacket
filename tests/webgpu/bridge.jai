// Runs in the browser engine against tools/test_webgpu_host.mjs's mock of the WebGPU JavaScript
// API: requests that settle later (each suspends the module through JSPI), a mapped buffer, an
// error scope, a few frames, and a thread that must keep running while the program waits.

#import "Basic";
#import "Thread";
#import "WebGPU";

ticks: s64;
stop: bool;

main :: () {
    print("available: %\n", webgpu_available());
    instance := wgpuCreateInstance(null);
    adapter := webgpu_request_adapter(instance);
    info: WGPUAdapterInfo;
    wgpuAdapterGetInfo(adapter, *info);
    print("adapter: %\n", to_string(info.description));
    wgpuAdapterInfoFreeMembers(info);
    device := webgpu_request_device(instance, adapter);
    print("device: %\n", device != null);

    // A thread that counts while the main one waits for the page.
    counter: Thread;
    thread_init(*counter, (thread: *Thread) -> s64 {
        while !stop {
            ticks += 1;
            sleep_milliseconds(1);
        }
        return 0;
    });
    thread_start(*counter);

    desc: WGPUBufferDescriptor;
    desc.label = to_string_view("readback");
    desc.size = 16;
    desc.usage = .MapRead | .CopyDst;
    buffer := wgpuDeviceCreateBuffer(device, *desc);
    ok := webgpu_map_buffer(instance, buffer, .Read, 0, 16);
    print("thread ran while mapping: %\n", ticks > 0);
    data := cast(*u8) wgpuBufferGetConstMappedRange(buffer, 0, 16);
    print("mapped: % % %\n", ok, data[0], data[15]);
    wgpuBufferUnmap(buffer);

    before := ticks;
    for 1..3 webgpu_present(null);
    print("thread ran during frames: %\n", ticks > before);
    stop = true;
    thread_deinit(*counter);

    wgpuDevicePushErrorScope(device, .Validation);
    wgsl: WGPUShaderSourceWGSL;
    wgsl.code = to_string_view("not wgsl");
    shader_desc: WGPUShaderModuleDescriptor;
    shader_desc.nextInChain = *wgsl.chain;
    wgpuDeviceCreateShaderModule(device, *shader_desc);
    type, message := webgpu_pop_error_scope(instance, device);
    print("error scope: % (%)\n", type, message);
    print("done\n");
}
