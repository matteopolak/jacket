// Cross-checks against the C implementations in stb_image (zlib decoder) and stb_image_write
// (zlib compressor): our streams decode there, theirs decode here. Native services only.
//
//     jaic stdlib/Extensions/Deflate/tests/stb.jai -interp

#load "support/helpers.jai";
#import "stb_image";
#import "stb_image_write";

// The stb functions allocate with the C library.
libc :: #system_library "libc";
c_free :: (memory: *void) #foreign libc "free";

stb_compress :: (data: [] u8, quality: s32) -> [] u8 {
    length: s32;
    packed := stbi_zlib_compress(data.data, cast(s32) data.count, *length, quality);

    result: [] u8;
    result.count = length;
    result.data = alloc(max(length, 1));

    for 0..length - 1 result[it] = packed[it];

    c_free(packed);

    return result;
}

main :: () {
    inputs: [5][] u8;
    inputs[0] = text_bytes(30000, 41);
    inputs[1] = random_bytes(5000, 42);
    inputs[2] = image_bytes(40000, 43);
    inputs[3] = run_bytes(20000);
    inputs[4] = all_byte_values();

    for data: inputs {
        label := tprint("input of % bytes", data.count);

        // stb's streams decode here.
        for quality: s32.[1, 5, 8, 9] {
            packed := stb_compress(data, quality);
            output, ok, error := inflate(packed, .ZLIB);

            check(ok, tprint("%: stb quality % stream failed: %", label, quality, error));
            check(ok && same_bytes(output, data), tprint("%: stb quality % stream differs", label, quality));

            if ok free(output.data);

            free(packed.data);
        }

        // Ours decode there. stb does not check the checksum, so compare the bytes themselves.
        for level: s32.[0, 1, 4, 6, 9] {
            packed := deflate(data, .ZLIB, level);

            length: s32;
            output := stbi_zlib_decode_malloc(packed.data, cast(s32) packed.count, *length);

            check(output != null, tprint("%: stb rejected our level % stream", label, level));

            if output {
                check(length == data.count, tprint("%: stb read % bytes of our level % stream", label, length, level));

                same := length == data.count;

                for 0..data.count - 1 {
                    if !same break;
                    if output[it] != data[it] same = false;
                }

                check(same, tprint("%: stb output differs for our level % stream", label, level));
                c_free(output);
            }

            free(packed.data);
        }
    }

    assert(failures == 0);
    print("ok\n");
}
