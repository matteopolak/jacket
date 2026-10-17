#import <AppKit/AppKit.h>
#import <Metal/Metal.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

_Static_assert(sizeof(NSWindowCollectionBehavior) == sizeof(NSUInteger),
               "collection behavior uses NSUInteger");
_Static_assert(NSWindowCollectionBehaviorFullScreenPrimary == 128,
               "fullscreen primary flag");
_Static_assert(NSWindowCollectionBehaviorManaged == 4, "managed flag");

// Compile-only controls: do not message a nil substitute or create an encoder.
// Each derived protocol must expose every declared MTLCommandEncoder operation.
#define CHECK_ENCODER(NAME)                                                    \
  void binding_check_##NAME(id<NAME> encoder, NSString *text) {                \
    [encoder insertDebugSignpost:text];                                        \
    [encoder pushDebugGroup:text];                                             \
    [encoder popDebugGroup];                                                   \
    id<MTLDevice> device = [encoder device];                                   \
    NSString *label = [encoder label];                                         \
    [encoder setLabel:text];                                                   \
    [encoder endEncoding];                                                     \
    (void)device;                                                              \
    (void)label;                                                               \
  }

CHECK_ENCODER(MTLRenderCommandEncoder)
CHECK_ENCODER(MTLBlitCommandEncoder)
CHECK_ENCODER(MTLComputeCommandEncoder)
CHECK_ENCODER(MTLParallelRenderCommandEncoder)

uint64_t binding_collection_value(uint32_t index) {
  const NSWindowCollectionBehavior values[] = {
      NSWindowCollectionBehaviorDefault,
      NSWindowCollectionBehaviorCanJoinAllSpaces,
      NSWindowCollectionBehaviorMoveToActiveSpace,
      NSWindowCollectionBehaviorManaged,
      NSWindowCollectionBehaviorTransient,
      NSWindowCollectionBehaviorStationary,
      NSWindowCollectionBehaviorParticipatesInCycle,
      NSWindowCollectionBehaviorIgnoresCycle,
      NSWindowCollectionBehaviorFullScreenPrimary,
      NSWindowCollectionBehaviorFullScreenAuxiliary,
      NSWindowCollectionBehaviorFullScreenNone,
      NSWindowCollectionBehaviorFullScreenAllowsTiling,
      NSWindowCollectionBehaviorFullScreenDisallowsTiling};
  if (index >= sizeof(values) / sizeof(values[0]))
    abort();
  return values[index];
}

static id<MTLDevice> device;
static id<MTLCommandQueue> queue;
static id<MTLCommandBuffer> buffer;
static id<MTLTexture> texture;
static id<MTLRenderCommandEncoder> encoder;

static void release_objects(void) {
  [encoder release];
  encoder = nil;
  [texture release];
  texture = nil;
  [buffer release];
  buffer = nil;
  [queue release];
  queue = nil;
  [device release];
  device = nil;
}

void *binding_encoder_create(void) {
  if (device)
    abort();
  device = MTLCreateSystemDefaultDevice();
  if (!device)
    goto failure;
  queue = [device newCommandQueue];
  if (!queue)
    goto failure;
  buffer = [[queue commandBuffer] retain];
  if (!buffer)
    goto failure;
  MTLTextureDescriptor *descriptor = [MTLTextureDescriptor
      texture2DDescriptorWithPixelFormat:MTLPixelFormatRGBA8Unorm
                                   width:4
                                  height:4
                               mipmapped:NO];
  descriptor.usage = MTLTextureUsageRenderTarget;
  texture = [device newTextureWithDescriptor:descriptor];
  if (!texture)
    goto failure;
  MTLRenderPassDescriptor *pass =
      [MTLRenderPassDescriptor renderPassDescriptor];
  pass.colorAttachments[0].texture = texture;
  pass.colorAttachments[0].loadAction = MTLLoadActionClear;
  pass.colorAttachments[0].storeAction = MTLStoreActionDontCare;
  encoder = [[buffer renderCommandEncoderWithDescriptor:pass] retain];
  if (!encoder)
    goto failure;
  return encoder;
failure:
  release_objects();
  return NULL;
}

void *binding_encoder_device(void) { return device; }
NSString *binding_encoder_label(void) { return @"bindings inherited encoder"; }
bool binding_encoder_label_matches(NSString *text) {
  return [text isEqualToString:binding_encoder_label()];
}

bool binding_encoder_finish(void) {
  if (!encoder || !buffer)
    return false;
  [buffer commit];
  [buffer waitUntilCompleted];
  bool completed =
      buffer.status == MTLCommandBufferStatusCompleted && !buffer.error;
  release_objects();
  return completed;
}
