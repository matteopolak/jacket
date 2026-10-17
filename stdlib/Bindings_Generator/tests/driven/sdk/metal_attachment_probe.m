#import <Metal/Metal.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

static id<MTLDevice> device;
static id<MTLCommandQueue> queue;
static id<MTLTexture> texture;
static MTLRenderPassDescriptor *pass;

static void release_objects(void) {
  [pass release];
  pass = nil;
  [texture release];
  texture = nil;
  [queue release];
  queue = nil;
  [device release];
  device = nil;
}

MTLRenderPassColorAttachmentDescriptor *binding_attachment_create(void) {
  if (device)
    abort();
  device = MTLCreateSystemDefaultDevice();
  if (!device)
    goto failure;
  queue = [device newCommandQueue];
  if (!queue)
    goto failure;
  MTLTextureDescriptor *descriptor = [MTLTextureDescriptor
      texture2DDescriptorWithPixelFormat:MTLPixelFormatRGBA8Unorm
                                   width:8
                                  height:8
                               mipmapped:YES];
  descriptor.usage = MTLTextureUsageRenderTarget;
  descriptor.storageMode = MTLStorageModeShared;
  texture = [device newTextureWithDescriptor:descriptor];
  if (!texture)
    goto failure;
  pass = [[MTLRenderPassDescriptor renderPassDescriptor] retain];
  if (!pass)
    goto failure;
  return pass.colorAttachments[0];
failure:
  release_objects();
  return nil;
}

void *binding_attachment_texture(void) { return texture; }
MTLRenderPassDepthAttachmentDescriptor *binding_attachment_depth(void) {
  return pass.depthAttachment;
}
MTLRenderPassStencilAttachmentDescriptor *binding_attachment_stencil(void) {
  return pass.stencilAttachment;
}
bool binding_attachment_matches(void *expected) {
  return pass.colorAttachments[0].texture == expected;
}

bool binding_attachment_finish(void) {
  bool valid = false;
  if (!queue || !pass || !texture)
    goto done;
  id<MTLCommandBuffer> buffer = [queue commandBuffer];
  if (!buffer)
    goto done;
  id<MTLRenderCommandEncoder> encoder =
      [buffer renderCommandEncoderWithDescriptor:pass];
  if (!encoder)
    goto done;
  [encoder endEncoding];
  [buffer commit];
  [buffer waitUntilCompleted];
  if (buffer.status != MTLCommandBufferStatusCompleted || buffer.error)
    goto done;
  struct {
    uint64_t before;
    uint8_t pixels[4 * 4 * 4];
    uint64_t after;
  } output = {.before = UINT64_C(0x91c75a3d06e2b84f),
              .pixels = {0},
              .after = UINT64_C(0x47b8e2063d5ac791)};
  [texture getBytes:output.pixels
        bytesPerRow:4 * 4
         fromRegion:MTLRegionMake2D(0, 0, 4, 4)
        mipmapLevel:1];
  valid = output.before == UINT64_C(0x91c75a3d06e2b84f) &&
          output.after == UINT64_C(0x47b8e2063d5ac791);
  for (unsigned i = 0; i < sizeof(output.pixels); i += 4) {
    if (output.pixels[i] != 255 || output.pixels[i + 1] != 0 ||
        output.pixels[i + 2] != 0 || output.pixels[i + 3] != 255)
      valid = false;
  }
done:
  release_objects();
  return valid;
}
