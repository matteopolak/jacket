// test: only(.MACOS)
// A shared MTLBuffer keeps the length and bytes it was created with, and bytes copied in are not affected by later writes to the source.
#import "Basic";
#import "Metal";
#import "Objective_C";

main :: () {
    init_metal();

    device := cast(*MTLDevice) MTLCreateSystemDefaultDevice();

    assert(device != null);
    assert(NSString.length(MTLDevice.name(device)) > 0);

    buffer := MTLDevice.newBufferWithLength_options(device, 16, MTLResourceStorageModeShared);

    assert(buffer != null);
    print("length: %\n", MTLBuffer.length(buffer));

    bytes := cast(*u8) MTLBuffer.contents(buffer);
    source: [16] u8;

    for i: 0..15 {
        source[i] = cast(u8) (i * 16 + 1);
        bytes[i] = source[i];
    }

    same := true;
    for i: 0..15 if bytes[i] != source[i] same = false;

    print("write matches: %\n", same);

    copied := MTLDevice.newBufferWithBytes_length_options(device, cast(*void) *source[0], 16, MTLResourceStorageModeShared);

    assert(copied != null);
    source[0] = 0;

    copied_bytes := cast(*u8) MTLBuffer.contents(copied);

    print("copy length: %\n", MTLBuffer.length(copied));
    print("copy first byte: %\n", copied_bytes[0]);
    print("copy last byte: %\n", copied_bytes[15]);
}
