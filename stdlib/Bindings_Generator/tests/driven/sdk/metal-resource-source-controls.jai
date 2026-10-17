// Independent source controls. These test-local records assert no production
// Texture/Buffer layout and are never passed to native SDK procedures.
M :: #import "Metal";

Using_Resource_Control :: struct {
    using M.MTLResource;
}

Ordinary_Resource_Control :: struct {
    resource: M.MTLResource;
}

canonical_setPurgeableState :: M.MTLResource.setPurgeableState;
promoted_setPurgeableState :: Using_Resource_Control.setPurgeableState;
#assert type_of(promoted_setPurgeableState) == type_of(canonical_setPurgeableState);
canonical_makeAliasable :: M.MTLResource.makeAliasable;
promoted_makeAliasable :: Using_Resource_Control.makeAliasable;
#assert type_of(promoted_makeAliasable) == type_of(canonical_makeAliasable);
canonical_isAliasable :: M.MTLResource.isAliasable;
promoted_isAliasable :: Using_Resource_Control.isAliasable;
#assert type_of(promoted_isAliasable) == type_of(canonical_isAliasable);
canonical_label :: M.MTLResource.label;
promoted_label :: Using_Resource_Control.label;
#assert type_of(promoted_label) == type_of(canonical_label);
canonical_setLabel :: M.MTLResource.setLabel;
promoted_setLabel :: Using_Resource_Control.setLabel;
#assert type_of(promoted_setLabel) == type_of(canonical_setLabel);
canonical_device :: M.MTLResource.device;
promoted_device :: Using_Resource_Control.device;
#assert type_of(promoted_device) == type_of(canonical_device);
canonical_cpuCacheMode :: M.MTLResource.cpuCacheMode;
promoted_cpuCacheMode :: Using_Resource_Control.cpuCacheMode;
#assert type_of(promoted_cpuCacheMode) == type_of(canonical_cpuCacheMode);
canonical_storageMode :: M.MTLResource.storageMode;
promoted_storageMode :: Using_Resource_Control.storageMode;
#assert type_of(promoted_storageMode) == type_of(canonical_storageMode);
canonical_heap :: M.MTLResource.heap;
promoted_heap :: Using_Resource_Control.heap;
#assert type_of(promoted_heap) == type_of(canonical_heap);
canonical_allocatedSize :: M.MTLResource.allocatedSize;
promoted_allocatedSize :: Using_Resource_Control.allocatedSize;
#assert type_of(promoted_allocatedSize) == type_of(canonical_allocatedSize);
#assert offset_of(Ordinary_Resource_Control, "resource") == 0;

#assert size_of(Using_Resource_Control) == size_of(M.MTLResource);

ordinary_access :: (control: *Ordinary_Resource_Control) -> *M.MTLResource {
    return *control.resource;
}

texture_setPurgeableState :: M.MTLTexture.setPurgeableState;
#assert type_of(texture_setPurgeableState) == type_of(canonical_setPurgeableState);
texture_makeAliasable :: M.MTLTexture.makeAliasable;
#assert type_of(texture_makeAliasable) == type_of(canonical_makeAliasable);
texture_isAliasable :: M.MTLTexture.isAliasable;
#assert type_of(texture_isAliasable) == type_of(canonical_isAliasable);
texture_label :: M.MTLTexture.label;
#assert type_of(texture_label) == type_of(canonical_label);
texture_setLabel :: M.MTLTexture.setLabel;
#assert type_of(texture_setLabel) == type_of(canonical_setLabel);
texture_device :: M.MTLTexture.device;
#assert type_of(texture_device) == type_of(canonical_device);
texture_cpuCacheMode :: M.MTLTexture.cpuCacheMode;
#assert type_of(texture_cpuCacheMode) == type_of(canonical_cpuCacheMode);
texture_storageMode :: M.MTLTexture.storageMode;
#assert type_of(texture_storageMode) == type_of(canonical_storageMode);
texture_heap :: M.MTLTexture.heap;
#assert type_of(texture_heap) == type_of(canonical_heap);
texture_allocatedSize :: M.MTLTexture.allocatedSize;
#assert type_of(texture_allocatedSize) == type_of(canonical_allocatedSize);

#assert size_of(M.MTLTexture) == size_of(M.MTLResource);
#assert type_info(M.MTLTexture).name == "MTLTexture";

main :: () {}
