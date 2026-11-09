// Load a pinned sample with external buffers and PNG resources.
// Example: jacket bench --path stdlib/Extensions/Gltf
#import "Basic";
#import "File";
#import "Extensions/Testing";
#import "Extensions/Environment";
#import "Extensions/Gltf";

box_textured :: (b: *Bench) {
    folder := output_path("gltf-conformance/samples/Models/BoxTextured/glTF");
    bytes, ok := read_entire_file(tprint("%/BoxTextured.gltf", folder));

    assert(ok, "run `PYTHONPATH=tools python3 tools/gltf_conformance/fetch.py --samples` first (a 1.1 GB download)");
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
