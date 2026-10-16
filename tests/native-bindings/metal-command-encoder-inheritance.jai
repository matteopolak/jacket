M :: #import "Metal";
O :: #import "Objective_C";

#assert offset_of(M.MTLRenderCommandEncoder, "mtlcommandencder") == 0;
#assert size_of(M.MTLRenderCommandEncoder) == size_of(M.MTLCommandEncoder);

probe :: #library,system "appkit_metal_contract_probe";
create :: () -> *M.MTLRenderCommandEncoder #foreign probe "binding_encoder_create";
native_device :: () -> O.id #foreign probe "binding_encoder_device";
native_label :: () -> *O.NSString #foreign probe "binding_encoder_label";
label_matches :: (text: *O.NSString) -> bool #foreign probe "binding_encoder_label_matches";
finish :: () -> bool #foreign probe "binding_encoder_finish";
libc :: #library,system "c";
abort :: () #foreign libc;

check :: (value: bool) {
    if !value abort();
}

main :: () {
    pool := O.objc_new(O.NSAutoreleasePool);

    check(pool != null);
    defer O.NSAutoreleasePool.drain(pool);
    encoder := create();

    check(encoder != null);
    base: *M.MTLCommandEncoder = encoder;

    check(cast(*void) base == cast(*void) encoder);
    check(*encoder.mtlcommandencder == base);
    check(encoder.device(encoder) == native_device());
    label := native_label();

    encoder.setLabel(encoder, label);
    check(label_matches(encoder.label(encoder)));
    encoder.pushDebugGroup(encoder, label);
    encoder.insertDebugSignpost(encoder, label);
    encoder.popDebugGroup(encoder);
    encoder.endEncoding(encoder);
    // Commit a real offscreen render pass; the SDK must report completion.
    check(finish());
}
