// Load a pinned sample with external buffers and PNG resources.
// Example: jacket bench --path stdlib/Extensions/Gltf
#import "Basic";
#import "File";
#import "Extensions/Testing";
#import "Extensions/Environment";
#import "Extensions/Gltf";

box_textured :: (b: *Bench) {
    base := environment_variable("JACKET_OUTPUT_DIR");
    if !base.count base = tprint("%../../../../target", #filepath);

    folder := tprint("%/gltf-conformance/samples/Models/BoxTextured/glTF", base);
    bytes, ok := read_entire_file(tprint("%/BoxTextured.gltf", folder));

    assert(ok, "run `PYTHONPATH=tools python3 tools/portable/build_portable.py data` first");
    defer free(bytes);
    set_bytes(b, bytes.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        asset, error := gltf_load(black_box(bytes), .{ gltf_local_resolve, *folder });

        black_box(asset);
        black_box(error);
        assert(error.kind == .NONE);
        gltf_free(*asset);
        gltf_error_free(*error);
    }
} @bench
