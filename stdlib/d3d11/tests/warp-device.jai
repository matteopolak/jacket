// test: only(.WINDOWS) timeout(60)
// Runs on the WARP software adapter: device, buffer upload and readback through a staging
// buffer, and a render-target clear read back through a staging texture.
#import "Basic";
D3D11 :: #import "d3d11";
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
    device: *D3D11.ID3D11Device;
    immediate: *D3D11.ID3D11DeviceContext;
    level: D3D11.D3D_FEATURE_LEVEL;

    succeeded(D3D11.D3D11CreateDevice(null, .WARP, null, .BGRA_SUPPORT, null, 0, D3D11.D3D11_SDK_VERSION, *device, *level, *immediate), "D3D11CreateDevice");
    check(device != null && immediate != null, "device and immediate context are not null");
    check(cast(s32) level >= cast(s32) D3D11.D3D_FEATURE_LEVEL._11_0, "WARP supports feature level 11_0");
    print("device created\n");

    // Upload 256 bytes through the initial data, copy to a staging buffer and compare.
    SIZE :: 256;
    source: [SIZE] u8;

    for 0..SIZE - 1 source[it] = cast(u8) (255 - it);

    buffer_desc: D3D11.D3D11_BUFFER_DESC;
    buffer_desc.ByteWidth = SIZE;
    buffer_desc.Usage = .DEFAULT;
    buffer_desc.BindFlags = .VERTEX_BUFFER;
    initial: D3D11.D3D11_SUBRESOURCE_DATA;
    initial.pSysMem = source.data;

    gpu: *D3D11.ID3D11Buffer;

    succeeded(D3D11.ID3D11Device_CreateBuffer(device, *buffer_desc, *initial, *gpu), "CreateBuffer (default)");

    staging_desc := buffer_desc;
    staging_desc.Usage = .STAGING;
    staging_desc.BindFlags = 0;
    staging_desc.CPUAccessFlags = .READ;
    staging: *D3D11.ID3D11Buffer;

    succeeded(D3D11.ID3D11Device_CreateBuffer(device, *staging_desc, null, *staging), "CreateBuffer (staging)");

    D3D11.ID3D11DeviceContext_CopyResource(immediate, cast(*D3D11.ID3D11Resource) staging, cast(*D3D11.ID3D11Resource) gpu);
    mapped: D3D11.D3D11_MAPPED_SUBRESOURCE;

    succeeded(D3D11.ID3D11DeviceContext_Map(immediate, cast(*D3D11.ID3D11Resource) staging, 0, .READ, 0, *mapped), "Map staging buffer");
    bytes := cast(*u8) mapped.pData;
    mismatches := 0;

    for 0..SIZE - 1 if bytes[it] != source[it] mismatches += 1;

    D3D11.ID3D11DeviceContext_Unmap(immediate, cast(*D3D11.ID3D11Resource) staging, 0);
    print("buffer mismatches: %\n", mismatches);

    // Clear a 4x4 render target to opaque red and read the first pixel back.
    texture_desc: D3D11.D3D11_TEXTURE2D_DESC;
    texture_desc.Width = 4;
    texture_desc.Height = 4;
    texture_desc.MipLevels = 1;
    texture_desc.ArraySize = 1;
    texture_desc.Format = .R8G8B8A8_UNORM;
    texture_desc.SampleDesc.Count = 1;
    texture_desc.Usage = .DEFAULT;
    texture_desc.BindFlags = .RENDER_TARGET;
    target: *D3D11.ID3D11Texture2D;

    succeeded(D3D11.ID3D11Device_CreateTexture2D(device, *texture_desc, null, *target), "CreateTexture2D (target)");

    view: *D3D11.ID3D11RenderTargetView;

    succeeded(D3D11.ID3D11Device_CreateRenderTargetView(device, cast(*D3D11.ID3D11Resource) target, null, *view), "CreateRenderTargetView");
    red := float.[1.0, 0.0, 0.0, 1.0];

    D3D11.ID3D11DeviceContext_ClearRenderTargetView(immediate, view, *red);

    readback_desc := texture_desc;
    readback_desc.Usage = .STAGING;
    readback_desc.BindFlags = 0;
    readback_desc.CPUAccessFlags = .READ;
    readback: *D3D11.ID3D11Texture2D;

    succeeded(D3D11.ID3D11Device_CreateTexture2D(device, *readback_desc, null, *readback), "CreateTexture2D (staging)");
    D3D11.ID3D11DeviceContext_CopyResource(immediate, cast(*D3D11.ID3D11Resource) readback, cast(*D3D11.ID3D11Resource) target);
    succeeded(D3D11.ID3D11DeviceContext_Map(immediate, cast(*D3D11.ID3D11Resource) readback, 0, .READ, 0, *mapped), "Map staging texture");
    pixel := cast(*u8) mapped.pData;

    print("pixel: % % % %\n", pixel[0], pixel[1], pixel[2], pixel[3]);
    check(mapped.RowPitch >= 16, "row pitch covers four pixels");
    D3D11.ID3D11DeviceContext_Unmap(immediate, cast(*D3D11.ID3D11Resource) readback, 0);

    release(readback);
    release(view);
    release(target);
    release(staging);
    release(gpu);
    release(immediate);
    release(device);
}
