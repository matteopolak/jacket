M :: #import "Metal";
O :: #import "Objective_C";

#assert offset_of(M.MTLRenderPassColorAttachmentDescriptor, "mtlrenderpassattachmentdescriptor") == 0;
#assert offset_of(M.MTLRenderPassDepthAttachmentDescriptor, "mtlrenderpassattachmentdescriptor") == 0;
#assert offset_of(M.MTLRenderPassStencilAttachmentDescriptor, "mtlrenderpassattachmentdescriptor") == 0;
#assert size_of(M.MTLRenderPassColorAttachmentDescriptor) == size_of(M.MTLRenderPassAttachmentDescriptor);
#assert size_of(M.MTLRenderPassDepthAttachmentDescriptor) == size_of(M.MTLRenderPassAttachmentDescriptor);
#assert size_of(M.MTLRenderPassStencilAttachmentDescriptor) == size_of(M.MTLRenderPassAttachmentDescriptor);
#assert type_info(M.MTLRenderPassColorAttachmentDescriptor).name == "MTLRenderPassColorAttachmentDescriptor";

probe :: #library,system "metal_attachment_probe";
create :: () -> *M.MTLRenderPassColorAttachmentDescriptor #foreign probe "binding_attachment_create";
native_texture :: () -> O.id #foreign probe "binding_attachment_texture";
native_depth :: () -> *M.MTLRenderPassDepthAttachmentDescriptor #foreign probe "binding_attachment_depth";
native_stencil :: () -> *M.MTLRenderPassStencilAttachmentDescriptor #foreign probe "binding_attachment_stencil";
native_matches :: (texture: O.id) -> bool #foreign probe "binding_attachment_matches";
finish :: () -> bool #foreign probe "binding_attachment_finish";
libc :: #library,system "c";
abort :: () #foreign libc;

check :: (value: bool) {
    if !value abort();
}

main :: () {
    pool := O.objc_new(O.NSAutoreleasePool);

    check(pool != null);
    defer O.NSAutoreleasePool.drain(pool);
    color := create();

    check(color != null);
    base: *M.MTLRenderPassAttachmentDescriptor = color;

    check(cast(*void) base == cast(*void) color);
    check(*color.mtlrenderpassattachmentdescriptor == base);
    texture := native_texture();

    check(texture != null);
    color.setTexture(color, texture);
    check(color.texture(color) == texture);
    check(native_matches(texture));
    color.setTexture(color, null);
    check(color.texture(color) == null && native_matches(null));
    color.setTexture(color, texture);
    color.setLevel(color, 1);
    check(color.level(color) == 1);
    color.setSlice(color, 0);
    check(color.slice(color) == 0);
    color.setDepthPlane(color, 0);
    check(color.depthPlane(color) == 0);
    color.setResolveTexture(color, null);
    check(color.resolveTexture(color) == null);
    color.setResolveLevel(color, 0);
    check(color.resolveLevel(color) == 0);
    color.setResolveSlice(color, 0);
    check(color.resolveSlice(color) == 0);
    color.setResolveDepthPlane(color, 0);
    check(color.resolveDepthPlane(color) == 0);
    color.setLoadAction(color, .MTLLoadActionClear);
    check(color.loadAction(color) == .MTLLoadActionClear);
    color.setStoreAction(color, .MTLStoreActionStore);
    check(color.storeAction(color) == .MTLStoreActionStore);
    color.setStoreActionOptions(color, .MTLStoreActionOptionNone);
    check(color.storeActionOptions(color) == .MTLStoreActionOptionNone);
    color.setClearColor(color, M.MTLClearColorMake(1, 0, 0, 1));

    // Depth/stencil expose the ordinary field explicitly, without promotion.
    depth := native_depth();
    stencil := native_stencil();

    check(depth != null && stencil != null);
    depth_base := *depth.mtlrenderpassattachmentdescriptor;
    stencil_base := *stencil.mtlrenderpassattachmentdescriptor;

    check(cast(*void) depth_base == cast(*void) depth);
    check(cast(*void) stencil_base == cast(*void) stencil);
    M.MTLRenderPassAttachmentDescriptor.setLevel(depth_base, 3);
    check(M.MTLRenderPassAttachmentDescriptor.level(depth_base) == 3);
    M.MTLRenderPassAttachmentDescriptor.setLevel(stencil_base, 2);
    check(M.MTLRenderPassAttachmentDescriptor.level(stencil_base) == 2);
    M.MTLRenderPassAttachmentDescriptor.setLevel(depth_base, 0);
    M.MTLRenderPassAttachmentDescriptor.setLevel(stencil_base, 0);
    // A real clear and readback at mip level 1 must produce opaque red pixels.
    check(finish());
}
