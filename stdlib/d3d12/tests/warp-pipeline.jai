// test: only(.WINDOWS) timeout(60)
// Runs on the WARP software adapter: device, RTV descriptor heap, committed resources,
// command list, fence signal and wait, and a buffer upload and readback.
#import "Basic";
D3D12 :: #import "d3d12";
DXGI :: #import "dxgi";
W :: #import "Windows";

check :: (condition: bool, what: string) {
    if !condition {
        print("FAIL: %\n", what);
        exit(1);
    }
}

succeeded :: (hr: W.HRESULT, what: string) {
    if hr < 0 {
        print("FAIL: % (hr=0x%)\n", what, formatInt(cast,trunc(u32) hr, base = 16));
        exit(1);
    }
}

release :: (object: *void) {
    if object W.IUnknown_Release(cast(*W.IUnknown) object);
}

main :: () {
    factory: *DXGI.IDXGIFactory4;
    factory_iid := W.uid(DXGI.IDXGIFactory4_UUID);

    succeeded(DXGI.CreateDXGIFactory2(0, factory_iid, cast(**void) *factory), "CreateDXGIFactory2");

    adapter: *DXGI.IDXGIAdapter1;
    adapter_iid := W.uid(DXGI.IDXGIAdapter1_UUID);

    succeeded(DXGI.IDXGIFactory4_EnumWarpAdapter(factory, *adapter_iid, cast(**void) *adapter), "EnumWarpAdapter");

    device: *D3D12.ID3D12Device;
    device_iid := W.uid(D3D12.ID3D12Device_UUID);

    succeeded(D3D12.D3D12CreateDevice(cast(*W.IUnknown) adapter, D3D12.D3D_FEATURE_LEVEL._11_0, device_iid, cast(**void) *device), "D3D12CreateDevice");
    check(device != null, "device is not null");
    print("device created\n");

    // Descriptor heap and a render-target view over a committed texture.
    heap_desc: D3D12.D3D12_DESCRIPTOR_HEAP_DESC;
    heap_desc.Type = .RTV;
    heap_desc.NumDescriptors = 2;
    heap_desc.Flags = .NONE;

    heap: *D3D12.ID3D12DescriptorHeap;
    heap_iid := W.uid(D3D12.ID3D12DescriptorHeap_UUID);

    succeeded(D3D12.ID3D12Device_CreateDescriptorHeap(device, *heap_desc, *heap_iid, cast(**void) *heap), "CreateDescriptorHeap");

    start := D3D12.ID3D12DescriptorHeap_GetCPUDescriptorHandleForHeapStart(heap);

    check(start.ptr != 0, "heap start handle is not null");
    check(start.ptr != cast(u64) heap, "heap start handle is not the heap object");
    increment := D3D12.ID3D12Device_GetDescriptorHandleIncrementSize(device, .RTV);

    check(increment > 0, "RTV increment size is positive");
    print("rtv heap: ok\n");

    default_props: D3D12.D3D12_HEAP_PROPERTIES;
    default_props.Type = .DEFAULT;
    default_props.CreationNodeMask = 1;
    default_props.VisibleNodeMask = 1;

    texture_desc: D3D12.D3D12_RESOURCE_DESC;
    texture_desc.Dimension = .TEXTURE2D;
    texture_desc.Width = 64;
    texture_desc.Height = 64;
    texture_desc.DepthOrArraySize = 1;
    texture_desc.MipLevels = 1;
    texture_desc.Format = .R8G8B8A8_UNORM;
    texture_desc.SampleDesc.Count = 1;
    texture_desc.Layout = .UNKNOWN;
    texture_desc.Flags = .ALLOW_RENDER_TARGET;

    clear_value: D3D12.D3D12_CLEAR_VALUE;
    clear_value.Format = .R8G8B8A8_UNORM;
    clear_value.Color = .[0.0, 0.0, 0.0, 1.0];

    texture: *D3D12.ID3D12Resource;
    resource_iid := W.uid(D3D12.ID3D12Resource_UUID);

    succeeded(D3D12.ID3D12Device_CreateCommittedResource(device, *default_props, .NONE, *texture_desc,
        .RENDER_TARGET, *clear_value, *resource_iid, cast(**void) *texture), "CreateCommittedResource (texture)");

    D3D12.ID3D12Device_CreateRenderTargetView(device, texture, null, start);
    second := D3D12.D3D12_CPU_DESCRIPTOR_HANDLE.{ ptr = start.ptr + increment };

    D3D12.ID3D12Device_CreateRenderTargetView(device, texture, null, second);
    print("rtv created: ok\n");

    back := D3D12.ID3D12Resource_GetDesc(texture);

    check(back.Width == 64 && back.Height == 64 && back.Format == .R8G8B8A8_UNORM, "texture desc round trips");

    // Command queue, allocator, list and fence.
    queue_desc: D3D12.D3D12_COMMAND_QUEUE_DESC;
    queue_desc.Type = .DIRECT;
    queue: *D3D12.ID3D12CommandQueue;
    queue_iid := W.uid(D3D12.ID3D12CommandQueue_UUID);

    succeeded(D3D12.ID3D12Device_CreateCommandQueue(device, *queue_desc, *queue_iid, cast(**void) *queue), "CreateCommandQueue");

    allocator: *D3D12.ID3D12CommandAllocator;
    allocator_iid := W.uid(D3D12.ID3D12CommandAllocator_UUID);

    succeeded(D3D12.ID3D12Device_CreateCommandAllocator(device, .DIRECT, *allocator_iid, cast(**void) *allocator), "CreateCommandAllocator");

    list: *D3D12.ID3D12GraphicsCommandList;
    list_iid := W.uid(D3D12.ID3D12GraphicsCommandList_UUID);

    succeeded(D3D12.ID3D12Device_CreateCommandList(device, 0, .DIRECT, allocator, null, *list_iid, cast(**void) *list), "CreateCommandList");

    fence: *D3D12.ID3D12Fence;
    fence_iid := W.uid(D3D12.ID3D12Fence_UUID);

    succeeded(D3D12.ID3D12Device_CreateFence(device, 0, .NONE, *fence_iid, cast(**void) *fence), "CreateFence");
    check(D3D12.ID3D12Fence_GetCompletedValue(fence) == 0, "fence starts at 0");

    // Buffer upload and readback: bytes 0..255 go to an upload buffer, are copied on the GPU
    // timeline to a readback buffer, and are compared.
    SIZE :: 256;
    buffer_desc: D3D12.D3D12_RESOURCE_DESC;
    buffer_desc.Dimension = .BUFFER;
    buffer_desc.Width = SIZE;
    buffer_desc.Height = 1;
    buffer_desc.DepthOrArraySize = 1;
    buffer_desc.MipLevels = 1;
    buffer_desc.Format = .UNKNOWN;
    buffer_desc.SampleDesc.Count = 1;
    buffer_desc.Layout = .ROW_MAJOR;

    upload_props: D3D12.D3D12_HEAP_PROPERTIES;
    upload_props.Type = .UPLOAD;
    upload_props.CreationNodeMask = 1;
    upload_props.VisibleNodeMask = 1;
    readback_props := upload_props;
    readback_props.Type = .READBACK;

    upload: *D3D12.ID3D12Resource;

    succeeded(D3D12.ID3D12Device_CreateCommittedResource(device, *upload_props, .NONE, *buffer_desc,
        .GENERIC_READ, null, *resource_iid, cast(**void) *upload), "CreateCommittedResource (upload)");
    readback: *D3D12.ID3D12Resource;

    succeeded(D3D12.ID3D12Device_CreateCommittedResource(device, *readback_props, .NONE, *buffer_desc,
        .COPY_DEST, null, *resource_iid, cast(**void) *readback), "CreateCommittedResource (readback)");

    mapped: *u8;
    no_read: D3D12.D3D12_RANGE;

    succeeded(D3D12.ID3D12Resource_Map(upload, 0, *no_read, cast(**void) *mapped), "Map upload");
    for 0..SIZE - 1 mapped[it] = cast(u8) it;

    written := D3D12.D3D12_RANGE.{ Begin = 0, End = SIZE };

    D3D12.ID3D12Resource_Unmap(upload, 0, *written);

    D3D12.ID3D12GraphicsCommandList_CopyBufferRegion(list, readback, 0, upload, 0, SIZE);
    succeeded(D3D12.ID3D12GraphicsCommandList_Close(list), "Close command list");

    lists: [1] *D3D12.ID3D12CommandList;
    lists[0] = cast(*D3D12.ID3D12CommandList) list;
    D3D12.ID3D12CommandQueue_ExecuteCommandLists(queue, 1, lists.data);
    succeeded(D3D12.ID3D12CommandQueue_Signal(queue, fence, 7), "Signal");

    event := W.CreateEventW(null, 0, 0, null);

    check(event != null, "CreateEventW");
    succeeded(D3D12.ID3D12Fence_SetEventOnCompletion(fence, 7, event), "SetEventOnCompletion");
    check(W.WaitForSingleObject(event, 20000) == W.WAIT_OBJECT_0, "fence signalled within 20 s");
    check(D3D12.ID3D12Fence_GetCompletedValue(fence) == 7, "fence reached 7");
    print("fence: ok\n");

    data: *u8;
    everything := D3D12.D3D12_RANGE.{ Begin = 0, End = SIZE };

    succeeded(D3D12.ID3D12Resource_Map(readback, 0, *everything, cast(**void) *data), "Map readback");
    mismatches := 0;
    for 0..SIZE - 1 if data[it] != cast(u8) it mismatches += 1;

    no_write := D3D12.D3D12_RANGE.{ Begin = 0, End = 0 };

    D3D12.ID3D12Resource_Unmap(readback, 0, *no_write);
    print("readback mismatches: %\n", mismatches);

    W.CloseHandle(event);
    release(readback);
    release(upload);
    release(fence);
    release(list);
    release(allocator);
    release(queue);
    release(texture);
    release(heap);
    release(device);
    release(adapter);
    release(factory);
}
