// test: only(.WINDOWS) timeout(30)
// Factory creation, adapter enumeration to the end of the list, the WARP adapter's identity,
// interface queries, and the ABI sizes of the descriptor types.
#import "Basic";
DXGI :: #import "dxgi";
W :: #import "Windows";

#assert size_of(DXGI.DXGI_ADAPTER_DESC1) == 312;
#assert size_of(DXGI.DXGI_SAMPLE_DESC) == 8;
#assert cast(s32) DXGI.DXGI_FORMAT.UNKNOWN == 0;
#assert cast(s32) DXGI.DXGI_FORMAT.R8G8B8A8_UNORM == 28;
#assert cast(s32) DXGI.DXGI_FORMAT.B8G8R8A8_UNORM == 87;
#assert cast(s32) DXGI.DXGI_FORMAT.D32_FLOAT == 40;

DXGI_ERROR_NOT_FOUND : s32 : cast,trunc(s32) 0x887A0002;

check :: (condition: bool, what: string) {
    if !condition {
        print("FAIL: %\n", what);
        exit(1);
    }
}

main :: () {
    factory: *DXGI.IDXGIFactory1;
    factory_iid := W.uid(DXGI.IDXGIFactory1_UUID);
    check(DXGI.CreateDXGIFactory1(factory_iid, cast(**void) *factory) >= 0, "CreateDXGIFactory1");
    check(factory != null, "factory is not null");

    // Enumeration ends with DXGI_ERROR_NOT_FOUND and visits indices in order.
    count := 0;
    while true {
        adapter: *DXGI.IDXGIAdapter1;
        hr := DXGI.IDXGIFactory1_EnumAdapters1(factory, cast(u32) count, *adapter);
        if hr == DXGI_ERROR_NOT_FOUND break;
        check(hr >= 0 && adapter != null, "EnumAdapters1 returns an adapter or NOT_FOUND");
        desc: DXGI.DXGI_ADAPTER_DESC1;
        check(DXGI.IDXGIAdapter1_GetDesc1(adapter, *desc) >= 0, "GetDesc1");
        check(desc.Description[0] != 0, "adapter has a description");
        W.IUnknown_Release(cast(*W.IUnknown) adapter);
        count += 1;
        check(count < 64, "adapter list ends");
    }
    print("enumeration ended with NOT_FOUND\n");

    // The WARP adapter is always present and identifies as the Microsoft software renderer.
    factory4: *DXGI.IDXGIFactory4;
    iid4 := W.uid(DXGI.IDXGIFactory4_UUID);
    check(W.IUnknown_QueryInterface(cast(*W.IUnknown) factory, *iid4, cast(**void) *factory4) >= 0, "QueryInterface to IDXGIFactory4");
    warp: *DXGI.IDXGIAdapter1;
    adapter_iid := W.uid(DXGI.IDXGIAdapter1_UUID);
    check(DXGI.IDXGIFactory4_EnumWarpAdapter(factory4, *adapter_iid, cast(**void) *warp) >= 0, "EnumWarpAdapter");
    desc: DXGI.DXGI_ADAPTER_DESC1;
    check(DXGI.IDXGIAdapter1_GetDesc1(warp, *desc) >= 0, "GetDesc1 of WARP");
    print("warp vendor: 0x%\n", formatInt(desc.VendorId, base = 16));
    print("warp is software: %\n", (cast(u32) desc.Flags & cast(u32) DXGI.DXGI_ADAPTER_FLAG.SOFTWARE) != 0);
    print("warp dedicated video memory: %\n", desc.DedicatedVideoMemory);

    W.IUnknown_Release(cast(*W.IUnknown) warp);
    W.IUnknown_Release(cast(*W.IUnknown) factory4);
    W.IUnknown_Release(cast(*W.IUnknown) factory);
}
