M :: #import "Metal";
O :: #import "Objective_C";

probe :: #library,system "metal_resource_probe";
initialize :: () -> bool #foreign probe "binding_resource_init";
finish :: () #foreign probe "binding_resource_finish";
texture :: (heap_backed: bool) -> *M.MTLTexture #foreign probe "binding_resource_texture";
label :: () -> *O.NSString #foreign probe "binding_resource_label";
alternate_label :: () -> *O.NSString #foreign probe "binding_resource_alternate_label";
sdk_set_label :: (pointer: *M.MTLTexture, value: *O.NSString) #foreign probe "binding_resource_sdk_set_label";
matches :: (pointer: *M.MTLTexture, value: *O.NSString) -> bool #foreign probe "binding_resource_label_matches";
device :: () -> O.id #foreign probe "binding_resource_device";
heap :: () -> O.id #foreign probe "binding_resource_heap";
allocated_size :: (pointer: *M.MTLTexture) -> u64 #foreign probe "binding_resource_allocated_size";
cpu_cache_mode :: (pointer: *M.MTLTexture) -> u64 #foreign probe "binding_resource_cpu_cache_mode";
storage_mode :: (pointer: *M.MTLTexture) -> u64 #foreign probe "binding_resource_storage_mode";
purgeability :: (pointer: *M.MTLTexture) -> u64 #foreign probe "binding_resource_purgeability";
aliasable :: (pointer: *M.MTLTexture) -> bool #foreign probe "binding_resource_is_aliasable";
libc :: #library,system "c";
abort :: () #foreign libc;

check :: (value: bool) {
    if !value abort();
}

main :: () {
    check(cast(*void) M.MTLTexture.setPurgeableState == cast(*void) M.MTLResource.setPurgeableState);
    check(cast(*void) M.MTLTexture.makeAliasable == cast(*void) M.MTLResource.makeAliasable);
    check(cast(*void) M.MTLTexture.isAliasable == cast(*void) M.MTLResource.isAliasable);
    check(cast(*void) M.MTLTexture.label == cast(*void) M.MTLResource.label);
    check(cast(*void) M.MTLTexture.setLabel == cast(*void) M.MTLResource.setLabel);
    check(cast(*void) M.MTLTexture.device == cast(*void) M.MTLResource.device);
    check(cast(*void) M.MTLTexture.cpuCacheMode == cast(*void) M.MTLResource.cpuCacheMode);
    check(cast(*void) M.MTLTexture.storageMode == cast(*void) M.MTLResource.storageMode);
    check(cast(*void) M.MTLTexture.heap == cast(*void) M.MTLResource.heap);
    check(cast(*void) M.MTLTexture.allocatedSize == cast(*void) M.MTLResource.allocatedSize);
    pool := O.objc_new(O.NSAutoreleasePool);

    check(pool != null);
    defer O.NSAutoreleasePool.drain(pool);
    check(initialize());
    defer finish();

    for heap_backed: bool.[false, true] {
        object := texture(heap_backed);

        check(object != null);
        // Unnamed using promotes names, retaining the canonical Resource receiver.
        base := cast(*M.MTLResource) object;

        check(cast(*void) base == cast(*void) object);
        M.MTLTexture.setLabel(base, label());
        check(matches(object, label()));
        check(M.MTLTexture.label(base) != null);
        M.MTLTexture.setLabel(base, null);
        check(matches(object, null));
        check(M.MTLTexture.label(base) == null);
        sdk_set_label(object, alternate_label());
        check(M.MTLTexture.label(base) == alternate_label());
        check(matches(object, alternate_label()));
        check(M.MTLTexture.device(base) == device());
        check(cast(u64) M.MTLTexture.allocatedSize(base) == allocated_size(object));
        check(cast(u64) M.MTLTexture.cpuCacheMode(base) == cpu_cache_mode(object));
        check(cast(u64) M.MTLTexture.storageMode(base) == storage_mode(object));

        if heap_backed {
            check(M.MTLTexture.heap(base) == heap());
            check(!aliasable(object));
            check(M.MTLTexture.isAliasable(base) == 0);
            // The SDK permits makeAliasable only on heap allocations. No later use.
            M.MTLTexture.makeAliasable(base);
            check(aliasable(object));
            check(M.MTLTexture.isAliasable(base) != 0);
        } else {
            check(M.MTLTexture.heap(base) == null);
            check(cast(u64) M.MTLTexture.setPurgeableState(base, .MTLPurgeableStateKeepCurrent) == purgeability(object));
        }
    }
}
