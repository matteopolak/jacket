// Independent SDK ownership/label oracle. No Jai wrapper field is assumed.
#import <Metal/Metal.h>
#include <stdbool.h>
#include <stdint.h>

static id<MTLDevice> device;
static id<MTLTexture> shared_texture;
static id<MTLHeap> heap;
static id<MTLTexture> heap_texture;

void binding_resource_finish(void) {
  [heap_texture release];
  heap_texture = nil;
  [shared_texture release];
  shared_texture = nil;
  [heap release];
  heap = nil;
  [device release];
  device = nil;
}

bool binding_resource_init(void) {
  if (device)
    return false;
  device = MTLCreateSystemDefaultDevice();
  if (!device)
    goto failure;
  MTLTextureDescriptor *descriptor = [MTLTextureDescriptor
      texture2DDescriptorWithPixelFormat:MTLPixelFormatRGBA8Unorm
                                   width:4
                                  height:4
                               mipmapped:NO];
  descriptor.storageMode = MTLStorageModeShared;
  descriptor.usage = MTLTextureUsageShaderRead | MTLTextureUsageRenderTarget;
  shared_texture = [device newTextureWithDescriptor:descriptor];
  if (!shared_texture)
    goto failure;
  descriptor.storageMode = MTLStorageModePrivate;
  MTLSizeAndAlign requirement =
      [device heapTextureSizeAndAlignWithDescriptor:descriptor];
  if (!requirement.size || !requirement.align)
    goto failure;
  MTLHeapDescriptor *heap_descriptor = [[MTLHeapDescriptor alloc] init];
  if (!heap_descriptor)
    goto failure;
  heap_descriptor.storageMode = MTLStorageModePrivate;
  heap_descriptor.size = requirement.size;
  heap = [device newHeapWithDescriptor:heap_descriptor];
  [heap_descriptor release];
  if (!heap)
    goto failure;
  heap_texture = [heap newTextureWithDescriptor:descriptor];
  if (!heap_texture)
    goto failure;
  return true;
failure:
  binding_resource_finish();
  return false;
}

void *binding_resource_texture(bool heap_backed) {
  return heap_backed ? heap_texture : shared_texture;
}
NSString *binding_resource_label(void) {
  return @"clean bindings texture resource";
}
NSString *binding_resource_alternate_label(void) {
  return @"SDK setter texture label";
}
void binding_resource_sdk_set_label(void *pointer, NSString *label) {
  [(id<MTLTexture>)pointer setLabel:label];
}
bool binding_resource_label_matches(void *pointer, NSString *expected) {
  NSString *actual = [(id<MTLTexture>)pointer label];
  return expected ? [actual isEqualToString:expected] : actual == nil;
}
void *binding_resource_device(void) { return device; }
void *binding_resource_heap(void) { return heap; }
uint64_t binding_resource_allocated_size(void *pointer) {
  return [(id<MTLResource>)pointer allocatedSize];
}
uint64_t binding_resource_cpu_cache_mode(void *pointer) {
  return [(id<MTLResource>)pointer cpuCacheMode];
}
uint64_t binding_resource_storage_mode(void *pointer) {
  return [(id<MTLResource>)pointer storageMode];
}
uint64_t binding_resource_purgeability(void *pointer) {
  return
      [(id<MTLResource>)pointer setPurgeableState:MTLPurgeableStateKeepCurrent];
}
bool binding_resource_is_aliasable(void *pointer) {
  return [(id<MTLResource>)pointer isAliasable];
}
