M :: #import "Metal";
O :: #import "Objective_C";

libc :: #library,system "c";
abort :: () #foreign libc;
probe :: #library "../../.work/native/apple_abi_probe";
native_size :: (record: u32) -> u64 #foreign probe "binding_apple_size";
native_offset :: (member: u32) -> u64 #foreign probe "binding_apple_offset";

check :: (value: bool) {
    if !value abort();
}

main :: () {
    check(size_of(M.MTLOrigin) == native_size(0));
    check(size_of(M.MTLSize) == native_size(1));
    check(size_of(M.MTLRegion) == native_size(2));
    check(size_of(M.MTLViewport) == native_size(3));
    check(size_of(M.MTLClearColor) == native_size(4));
    check(size_of(O.NSOperatingSystemVersion) == native_size(5));
    origin: M.MTLOrigin;
    size: M.MTLSize;
    region_layout: M.MTLRegion;
    viewport: M.MTLViewport;
    clear: M.MTLClearColor;
    version: O.NSOperatingSystemVersion;

    check(cast(u64) *origin.z - cast(u64) *origin == native_offset(0));
    check(cast(u64) *size.depth - cast(u64) *size == native_offset(1));
    check(cast(u64) *region_layout.size - cast(u64) *region_layout == native_offset(2));
    check(cast(u64) *viewport.zfar - cast(u64) *viewport == native_offset(3));
    check(cast(u64) *clear.alpha - cast(u64) *clear == native_offset(4));
    check(cast(u64) *version.patchVersion - cast(u64) *version == native_offset(5));
    M.init_metal();
    pool := O.objc_new(O.NSAutoreleasePool);

    check(pool != null);
    defer O.NSAutoreleasePool.drain(pool);
    descriptor := M.MTLTextureDescriptor.texture2DDescriptorWithPixelFormat_width_height_mipmapped(
        .MTLPixelFormatRGBA8Unorm, 64, 32, O.NO);

    check(descriptor != null);
    check(M.MTLTextureDescriptor.width(descriptor) == 64);
    check(M.MTLTextureDescriptor.height(descriptor) == 32);
    check(M.MTLTextureDescriptor.depth(descriptor) == 1);
    check(M.MTLTextureDescriptor.pixelFormat(descriptor) == .MTLPixelFormatRGBA8Unorm);
    M.MTLTextureDescriptor.setWidth(descriptor, 17);
    check(M.MTLTextureDescriptor.width(descriptor) == 17);
    region := M.MTLRegionMake3D(1, 2, 3, 4, 5, 6);

    check(region.origin.x == 1 && region.origin.y == 2 && region.origin.z == 3);
    check(region.size.width == 4 && region.size.height == 5 && region.size.depth == 6);
}
