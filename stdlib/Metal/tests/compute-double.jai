// test: only(.MACOS)
// A compute kernel compiled from MSL source doubles 16 uint32 values in a buffer, and the GPU results equal 2 * input.
#import "Basic";
#import "Metal";
#import "Objective_C";

KERNEL_SOURCE :: #string END
#include <metal_stdlib>
using namespace metal;

kernel void double_values(device uint *values [[buffer(0)]],
                          uint index [[thread_position_in_grid]]) {
    values[index] = values[index] * 2u;
}
END

main :: () {
    init_metal();

    device := cast(*MTLDevice) MTLCreateSystemDefaultDevice();

    assert(device != null);

    error: *NSError;
    library := MTLDevice.newLibraryWithSource_options_error(device, NSString.getTempString(KERNEL_SOURCE), null, *error);

    assert(library != null);
    assert(error == null);

    function := MTLLibrary.newFunctionWithName(library, NSString.getTempString("double_values"));

    assert(function != null);

    state := MTLDevice.newComputePipelineStateWithFunction_error(device, function, *error);

    assert(state != null);

    input: [16] u32;
    for i: 0..15 input[i] = cast(u32) (i * 3 + 1000);

    buffer := MTLDevice.newBufferWithBytes_length_options(device, cast(*void) *input[0], 64, MTLResourceStorageModeShared);

    assert(buffer != null);

    queue := MTLDevice.newCommandQueue(device);
    command := MTLCommandQueue.commandBuffer(queue);
    encoder := MTLCommandBuffer.computeCommandEncoder(command);

    MTLComputeCommandEncoder.setComputePipelineState(encoder, cast(id) state);
    MTLComputeCommandEncoder.setBuffer_offset_atIndex(encoder, cast(id) buffer, 0, 0);
    MTLComputeCommandEncoder.dispatchThreads_threadsPerThreadgroup(encoder, MTLSizeMake(16, 1, 1), MTLSizeMake(16, 1, 1));
    MTLCommandEncoder.endEncoding(cast(*MTLCommandEncoder) encoder);
    MTLCommandBuffer.commit(command);
    MTLCommandBuffer.waitUntilCompleted(command);

    assert(MTLCommandBuffer.status(command) == MTLCommandBufferStatusCompleted);

    output := cast(*u32) MTLBuffer.contents(buffer);
    for i: 0..15 print("%\n", output[i]);
}
