#scope_file
#import "Basic";
#import "String";
#import "File";
#import "Extensions/Environment";
#import "Extensions/Testing";
#import "Extensions/Gltf";
Raster :: #import "Extensions/Image";
#import "Base64";

image_resolver :: (user: *void, uri: string, limit: s64) -> string, bool {
    // Exercise the loader's checks even when the callback ignores its limit.
    return copy_string((cast(*string) user).*), uri == "pixel.png";
}

check_document :: (text: string, valid: bool) {
    tally: Tally;
    accepted: bool;

    {
        push_allocator(tally_proc, *tally);
        a, e := gltf_load(text);

        accepted = e.kind == .NONE;
        gltf_free(*a);
        gltf_error_free(*e);
    }

    expect_equal(accepted, valid, text);
    expect_equal(tally.outstanding, 0);
}

// Eight signature bytes, then the length, type, 13 data bytes and checksum of IHDR.
PNG_HEADER_END :: 8 + 25;

#scope_export
image_resources_and_budgets :: () {
    image := Raster.make_image(2, 2, .RGBA8);

    defer Raster.deinit(*image);
    png := Raster.encode_png(image);

    defer free(png.data);
    bytes := cast(string) png;
    encoded := base64_encode(bytes);

    defer free(encoded);
    uri := tprint("data:image/png;base64,%", encoded);
    text := tprint("{\"asset\":{\"version\":\"2.0\"},\"images\":[{\"uri\":\"%\"}]}", uri);

    check_document(text, true);

    for n: 0..bytes.count - 1 {
        part := base64_encode(slice(bytes, 0, n));

        defer free(part);
        cut := tprint("{\"asset\":{\"version\":\"2.0\"},\"images\":[{\"uri\":\"data:image/png;base64,%\"}]}", part);

        // Images are checked by header: a PNG loads once its signature and IHDR chunk are whole.
        check_document(cut, n >= PNG_HEADER_END);
    }

    corrupt := copy_string(bytes);

    defer free(corrupt);
    corrupt[PNG_HEADER_END - 1] ^= 1; // IHDR checksum
    corrupt_encoded := base64_encode(corrupt);

    defer free(corrupt_encoded);
    check_document(tprint("{\"asset\":{\"version\":\"2.0\"},\"images\":[{\"uri\":\"data:image/png;base64,%\"}]}", corrupt_encoded), false);

    mismatch := tprint("{\"asset\":{\"version\":\"2.0\"},\"images\":[{\"uri\":\"%\",\"mimeType\":\"image/jpeg\"}]}", uri);

    check_document(mismatch, false);
    embedded := tprint("{\"asset\":{\"version\":\"2.0\"},\"buffers\":[{\"uri\":\"data:application/octet-stream;base64,%\",\"byteLength\":%}],\"bufferViews\":[{\"buffer\":0,\"byteLength\":%}],\"images\":[{\"bufferView\":0,\"mimeType\":\"image/png\"}]}", encoded, bytes.count, bytes.count);

    check_document(embedded, true);
    external :: "{\"asset\":{\"version\":\"2.0\"},\"images\":[{\"uri\":\"pixel.png\"}]}";
    for source: string.[text, external, embedded] {
        for budget: s64.[bytes.count - 1, bytes.count, bytes.count + 15, bytes.count + 16] {
            tally: Tally;
            accepted: bool;

            {
                push_allocator(tally_proc, *tally);
                a, e := gltf_load(source, .{ image_resolver, *bytes }, .{ max_buffer_bytes = budget });

                accepted = e.kind == .NONE;
                gltf_free(*a);
                gltf_error_free(*e);
            }

            expect_equal(accepted, budget >= bytes.count + 16);
            expect_equal(tally.outstanding, 0);
        }
    }

    check_document(external, false);
} @test

remaining_validator_regressions :: () {
    base := environment_variable("JACKET_OUTPUT_DIR");
    if !base.count base = tprint("%../../../../target", #filepath);

    root := tprint("%/gltf-conformance/validator/test/base/data", base);

    for name: string.[
        "mesh_data/joints_weights_complex",
        "mesh/unequal_joints",
        "skin/no_joints_in_scene",
        "skin_data/invalid_ibm",
        "animation/channel_duplicate_targets",
        "node/matrix_indecomposable",
        "image/invalid_image_data",
        "image/invalid_data_uri",
        "image/not_found",
        "image/invalid_uri_scheme",
        "image/png_eos",
        "image/invalid_data_uri_mime_type"
    ] {
        path := tprint("%/%.gltf", root, name);
        bytes, ok := read_entire_file(path);

        require(ok, "run `PYTHONPATH=tools python3 tools/portable/build_portable.py data` first");
        defer free(bytes);
        slash := find_index_from_right(path, #char "/");
        folder := slice(path, 0, slash);
        tally: Tally;
        rejected: bool;

        {
            push_allocator(tally_proc, *tally);
            a, e := gltf_load(bytes, .{ gltf_local_resolve, *folder });

            rejected = e.kind != .NONE;
            gltf_free(*a);
            gltf_error_free(*e);
        }

        expect(rejected, name);
        expect_equal(tally.outstanding, 0, name);
    }
} @test

matrix_decomposition :: () {
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{\"matrix\":[1,0,0,0,0,1,0,0,0,0,1,0,3,4,5,1]}]}", true);
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{\"matrix\":[-1,0,0,0,0,2,0,0,0,0,3,0,0,0,0,1]}]}", true);
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{\"matrix\":[0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,1]}]}", true);
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{\"matrix\":[1,0,0,0,1,1,0,0,0,0,1,0,0,0,0,1]}]}", false);
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{\"matrix\":[1,0,0,1,0,1,0,0,0,0,1,0,0,0,0,1]}]}", false);
} @test

skin_vertex_constraints :: () {
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{\"mesh\":0,\"skin\":0},{\"children\":[2]},{}],\"skins\":[{\"joints\":[1,2]}],\"meshes\":[{\"primitives\":[{\"mode\":0,\"attributes\":{\"POSITION\":0,\"JOINTS_0\":1,\"WEIGHTS_0\":2,\"JOINTS_1\":3,\"WEIGHTS_1\":4}}]}],\"accessors\":[{\"componentType\":5126,\"type\":\"VEC3\",\"count\":1,\"min\":[0,0,0],\"max\":[0,0,0]},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":0},{\"componentType\":5126,\"type\":\"VEC4\",\"count\":1,\"bufferView\":1},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":2},{\"componentType\":5126,\"type\":\"VEC4\",\"count\":1,\"bufferView\":3}],\"bufferViews\":[{\"buffer\":0,\"byteOffset\":0,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":4,\"byteLength\":16},{\"buffer\":0,\"byteOffset\":20,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":24,\"byteLength\":16}],\"buffers\":[{\"byteLength\":40,\"uri\":\"data:application/octet-stream;base64,AAEAAAAAAD8AAAA/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==\"}]}", true);
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{\"mesh\":0,\"skin\":0},{\"children\":[2]},{}],\"skins\":[{\"joints\":[1,2]}],\"meshes\":[{\"primitives\":[{\"mode\":0,\"attributes\":{\"POSITION\":0,\"JOINTS_0\":1,\"WEIGHTS_0\":2,\"JOINTS_1\":3,\"WEIGHTS_1\":4}}]}],\"accessors\":[{\"componentType\":5126,\"type\":\"VEC3\",\"count\":1,\"min\":[0,0,0],\"max\":[0,0,0]},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":0},{\"componentType\":5126,\"type\":\"VEC4\",\"count\":1,\"bufferView\":1},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":2},{\"componentType\":5126,\"type\":\"VEC4\",\"count\":1,\"bufferView\":3}],\"bufferViews\":[{\"buffer\":0,\"byteOffset\":0,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":4,\"byteLength\":16},{\"buffer\":0,\"byteOffset\":20,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":24,\"byteLength\":16}],\"buffers\":[{\"byteLength\":40,\"uri\":\"data:application/octet-stream;base64,AAEAAAAAAL8AAMA/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==\"}]}", false);
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{\"mesh\":0,\"skin\":0},{\"children\":[2]},{}],\"skins\":[{\"joints\":[1,2]}],\"meshes\":[{\"primitives\":[{\"mode\":0,\"attributes\":{\"POSITION\":0,\"JOINTS_0\":1,\"WEIGHTS_0\":2,\"JOINTS_1\":3,\"WEIGHTS_1\":4}}]}],\"accessors\":[{\"componentType\":5126,\"type\":\"VEC3\",\"count\":1,\"min\":[0,0,0],\"max\":[0,0,0]},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":0},{\"componentType\":5126,\"type\":\"VEC4\",\"count\":1,\"bufferView\":1},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":2},{\"componentType\":5126,\"type\":\"VEC4\",\"count\":1,\"bufferView\":3}],\"bufferViews\":[{\"buffer\":0,\"byteOffset\":0,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":4,\"byteLength\":16},{\"buffer\":0,\"byteOffset\":20,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":24,\"byteLength\":16}],\"buffers\":[{\"byteLength\":40,\"uri\":\"data:application/octet-stream;base64,AAEAAM3MTD7NzEw+AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==\"}]}", false);
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{\"mesh\":0,\"skin\":0},{\"children\":[2]},{}],\"skins\":[{\"joints\":[1,2]}],\"meshes\":[{\"primitives\":[{\"mode\":0,\"attributes\":{\"POSITION\":0,\"JOINTS_0\":1,\"WEIGHTS_0\":2,\"JOINTS_1\":3,\"WEIGHTS_1\":4}}]}],\"accessors\":[{\"componentType\":5126,\"type\":\"VEC3\",\"count\":1,\"min\":[0,0,0],\"max\":[0,0,0]},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":0},{\"componentType\":5126,\"type\":\"VEC4\",\"count\":1,\"bufferView\":1},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":2},{\"componentType\":5126,\"type\":\"VEC4\",\"count\":1,\"bufferView\":3}],\"bufferViews\":[{\"buffer\":0,\"byteOffset\":0,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":4,\"byteLength\":16},{\"buffer\":0,\"byteOffset\":20,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":24,\"byteLength\":16}],\"buffers\":[{\"byteLength\":40,\"uri\":\"data:application/octet-stream;base64,AAEAAAAAAD8AAAAAAAAAAAAAAAAAAAAAAAAAPwAAAAAAAAAAAAAAAA==\"}]}", false);
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{\"mesh\":0,\"skin\":0},{\"children\":[2]},{}],\"skins\":[{\"joints\":[1,2]}],\"meshes\":[{\"primitives\":[{\"mode\":0,\"attributes\":{\"POSITION\":0,\"JOINTS_0\":1,\"WEIGHTS_0\":2,\"JOINTS_1\":3,\"WEIGHTS_1\":4}}]}],\"accessors\":[{\"componentType\":5126,\"type\":\"VEC3\",\"count\":1,\"min\":[0,0,0],\"max\":[0,0,0]},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":0},{\"componentType\":5126,\"type\":\"VEC4\",\"count\":1,\"bufferView\":1},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":2},{\"componentType\":5126,\"type\":\"VEC4\",\"count\":1,\"bufferView\":3}],\"bufferViews\":[{\"buffer\":0,\"byteOffset\":0,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":4,\"byteLength\":16},{\"buffer\":0,\"byteOffset\":20,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":24,\"byteLength\":16}],\"buffers\":[{\"byteLength\":40,\"uri\":\"data:application/octet-stream;base64,AgEAAAAAAD8AAAA/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==\"}]}", false);
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{\"mesh\":0,\"skin\":0},{\"children\":[2]},{}],\"skins\":[{\"joints\":[1,2]}],\"meshes\":[{\"primitives\":[{\"mode\":0,\"attributes\":{\"POSITION\":0,\"JOINTS_0\":1,\"WEIGHTS_0\":2,\"JOINTS_1\":3}}]}],\"accessors\":[{\"componentType\":5126,\"type\":\"VEC3\",\"count\":1,\"min\":[0,0,0],\"max\":[0,0,0]},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":0},{\"componentType\":5126,\"type\":\"VEC4\",\"count\":1,\"bufferView\":1},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":2},{\"componentType\":5126,\"type\":\"VEC4\",\"count\":1,\"bufferView\":3}],\"bufferViews\":[{\"buffer\":0,\"byteOffset\":0,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":4,\"byteLength\":16},{\"buffer\":0,\"byteOffset\":20,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":24,\"byteLength\":16}],\"buffers\":[{\"byteLength\":40,\"uri\":\"data:application/octet-stream;base64,AAEAAAAAAD8AAAA/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==\"}]}", false);
} @test

inverse_bind_rows :: () {
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{}],\"skins\":[{\"joints\":[0],\"inverseBindMatrices\":0}],\"accessors\":[{\"componentType\":5126,\"count\":1,\"type\":\"MAT4\",\"bufferView\":0}],\"bufferViews\":[{\"buffer\":0,\"byteLength\":64}],\"buffers\":[{\"byteLength\":64,\"uri\":\"data:application/octet-stream;base64,AACAPwAAAAAAAAAAAACAPwAAAAAAAIA/AAAAAAAAAAAAAAAAAAAAAAAAgD8AAAAAAAAAAAAAAAAAAAAAAACAPw==\"}]}", false);
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{}],\"skins\":[{\"joints\":[0],\"inverseBindMatrices\":0}],\"accessors\":[{\"componentType\":5126,\"count\":1,\"type\":\"MAT4\",\"bufferView\":0}],\"bufferViews\":[{\"buffer\":0,\"byteLength\":64}],\"buffers\":[{\"byteLength\":64,\"uri\":\"data:application/octet-stream;base64,AACAPwAAAAAAAAAAAAAAAAAAAAAAAIA/AAAAAAAAgD8AAAAAAAAAAAAAgD8AAAAAAAAAAAAAAAAAAAAAAACAPw==\"}]}", false);
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{}],\"skins\":[{\"joints\":[0],\"inverseBindMatrices\":0}],\"accessors\":[{\"componentType\":5126,\"count\":1,\"type\":\"MAT4\",\"bufferView\":0}],\"bufferViews\":[{\"buffer\":0,\"byteLength\":64}],\"buffers\":[{\"byteLength\":64,\"uri\":\"data:application/octet-stream;base64,AACAPwAAAAAAAAAAAAAAAAAAAAAAAIA/AAAAAAAAAAAAAAAAAAAAAAAAgD8AAIA/AAAAAAAAAAAAAAAAAACAPw==\"}]}", false);
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{}],\"skins\":[{\"joints\":[0],\"inverseBindMatrices\":0}],\"accessors\":[{\"componentType\":5126,\"count\":1,\"type\":\"MAT4\",\"bufferView\":0}],\"bufferViews\":[{\"buffer\":0,\"byteLength\":64}],\"buffers\":[{\"byteLength\":64,\"uri\":\"data:application/octet-stream;base64,AACAPwAAAAAAAAAAAAAAAAAAAAAAAIA/AAAAAAAAAAAAAAAAAAAAAAAAgD8AAAAAAAAAAAAAAAAAAAAAAAAAQA==\"}]}", false);
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{}],\"skins\":[{\"joints\":[0],\"inverseBindMatrices\":0}],\"accessors\":[{\"componentType\":5126,\"count\":1,\"type\":\"MAT4\",\"bufferView\":0}],\"bufferViews\":[{\"buffer\":0,\"byteLength\":64}],\"buffers\":[{\"byteLength\":64,\"uri\":\"data:application/octet-stream;base64,AACAPwAAAAAAAAAAAAAAAAAAAAAAAIA/AAAAAAAAAAAAAAAAAAAAAAAAgD8AAAAAAAAAAAAAAAAAAAAAAACAPw==\"}]}", true);
} @test

animation_target_scope :: () {
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{}],\"accessors\":[{\"type\":\"SCALAR\",\"componentType\":5126,\"count\":1,\"min\":[0],\"max\":[0]},{\"type\":\"VEC3\",\"componentType\":5126,\"count\":1}],\"animations\":[{\"samplers\":[{\"input\":0,\"output\":1}],\"channels\":[{\"sampler\":0,\"target\":{\"node\":0,\"path\":\"scale\"}},{\"sampler\":0,\"target\":{\"node\":0,\"path\":\"translation\"}}]}]}", true);
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{}],\"accessors\":[{\"type\":\"SCALAR\",\"componentType\":5126,\"count\":1,\"min\":[0],\"max\":[0]},{\"type\":\"VEC3\",\"componentType\":5126,\"count\":1}],\"animations\":[{\"samplers\":[{\"input\":0,\"output\":1}],\"channels\":[{\"sampler\":0,\"target\":{\"node\":0,\"path\":\"scale\"}},{\"sampler\":0,\"target\":{\"node\":0,\"path\":\"scale\"}}]}]}", false);
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{}],\"accessors\":[{\"type\":\"SCALAR\",\"componentType\":5126,\"count\":1,\"min\":[0],\"max\":[0]},{\"type\":\"VEC3\",\"componentType\":5126,\"count\":1}],\"animations\":[{\"samplers\":[{\"input\":0,\"output\":1}],\"channels\":[{\"sampler\":0,\"target\":{\"path\":\"scale\"}},{\"sampler\":0,\"target\":{\"path\":\"scale\"}}]}]}", true);
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{}],\"accessors\":[{\"type\":\"SCALAR\",\"componentType\":5126,\"count\":1,\"min\":[0],\"max\":[0]},{\"type\":\"VEC3\",\"componentType\":5126,\"count\":1}],\"animations\":[{\"samplers\":[{\"input\":0,\"output\":1}],\"channels\":[{\"sampler\":0,\"target\":{\"node\":0,\"path\":\"scale\"}},{\"sampler\":0,\"target\":{\"node\":0,\"path\":\"translation\"}}]},{\"samplers\":[{\"input\":0,\"output\":1}],\"channels\":[{\"sampler\":0,\"target\":{\"node\":0,\"path\":\"scale\"}},{\"sampler\":0,\"target\":{\"node\":0,\"path\":\"translation\"}}]}]}", true);
} @test

skin_joint_hierarchy :: () {
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{\"children\":[1,2]},{},{}],\"skins\":[{\"joints\":[1,2],\"skeleton\":0}]}", true);
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{\"children\":[1,2]},{},{}],\"skins\":[{\"joints\":[1,2],\"skeleton\":1}]}", false);
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{\"children\":[1]},{\"children\":[2]},{}],\"skins\":[{\"joints\":[1,2],\"skeleton\":1}]}", true);
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{\"children\":[1]},{\"children\":[2]},{}],\"skins\":[{\"joints\":[1,2],\"skeleton\":2}]}", false);
} @test

skin_joints_are_unique :: () {
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{\"children\":[1]},{}],\"skins\":[{\"joints\":[0,1]}]}", true);
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{\"children\":[1]},{}],\"skins\":[{\"joints\":[1,1]}]}", false);
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{\"children\":[1]},{}],\"skins\":[{\"joints\":[0,1,0]}]}", false);
} @test

skin_scene_reachability :: () {
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{\"mesh\":0,\"skin\":0},{\"children\":[2]},{}],\"skins\":[{\"joints\":[1,2]}],\"meshes\":[{\"primitives\":[{\"mode\":0,\"attributes\":{\"POSITION\":0,\"JOINTS_0\":1,\"WEIGHTS_0\":2,\"JOINTS_1\":3,\"WEIGHTS_1\":4}}]}],\"accessors\":[{\"componentType\":5126,\"type\":\"VEC3\",\"count\":1,\"min\":[0,0,0],\"max\":[0,0,0]},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":0},{\"componentType\":5126,\"type\":\"VEC4\",\"count\":1,\"bufferView\":1},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":2},{\"componentType\":5126,\"type\":\"VEC4\",\"count\":1,\"bufferView\":3}],\"bufferViews\":[{\"buffer\":0,\"byteOffset\":0,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":4,\"byteLength\":16},{\"buffer\":0,\"byteOffset\":20,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":24,\"byteLength\":16}],\"buffers\":[{\"byteLength\":40,\"uri\":\"data:application/octet-stream;base64,AAEAAAAAAD8AAAA/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==\"}],\"scenes\":[{\"nodes\":[0,1]}]}", true);
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{\"mesh\":0,\"skin\":0},{\"children\":[2]},{}],\"skins\":[{\"joints\":[1,2]}],\"meshes\":[{\"primitives\":[{\"mode\":0,\"attributes\":{\"POSITION\":0,\"JOINTS_0\":1,\"WEIGHTS_0\":2,\"JOINTS_1\":3,\"WEIGHTS_1\":4}}]}],\"accessors\":[{\"componentType\":5126,\"type\":\"VEC3\",\"count\":1,\"min\":[0,0,0],\"max\":[0,0,0]},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":0},{\"componentType\":5126,\"type\":\"VEC4\",\"count\":1,\"bufferView\":1},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":2},{\"componentType\":5126,\"type\":\"VEC4\",\"count\":1,\"bufferView\":3}],\"bufferViews\":[{\"buffer\":0,\"byteOffset\":0,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":4,\"byteLength\":16},{\"buffer\":0,\"byteOffset\":20,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":24,\"byteLength\":16}],\"buffers\":[{\"byteLength\":40,\"uri\":\"data:application/octet-stream;base64,AAEAAAAAAD8AAAA/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==\"}],\"scenes\":[{\"nodes\":[0]}]}", false);
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{\"mesh\":0,\"skin\":0},{\"children\":[2]},{}],\"skins\":[{\"joints\":[1,2]}],\"meshes\":[{\"primitives\":[{\"mode\":0,\"attributes\":{\"POSITION\":0,\"JOINTS_0\":1,\"WEIGHTS_0\":2,\"JOINTS_1\":3,\"WEIGHTS_1\":4}}]}],\"accessors\":[{\"componentType\":5126,\"type\":\"VEC3\",\"count\":1,\"min\":[0,0,0],\"max\":[0,0,0]},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":0},{\"componentType\":5126,\"type\":\"VEC4\",\"count\":1,\"bufferView\":1},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":2},{\"componentType\":5126,\"type\":\"VEC4\",\"count\":1,\"bufferView\":3}],\"bufferViews\":[{\"buffer\":0,\"byteOffset\":0,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":4,\"byteLength\":16},{\"buffer\":0,\"byteOffset\":20,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":24,\"byteLength\":16}],\"buffers\":[{\"byteLength\":40,\"uri\":\"data:application/octet-stream;base64,AAEAAAAAAD8AAAA/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==\"}],\"scenes\":[{\"nodes\":[1]}]}", true);
} @test

quantized_skin_weights :: () {
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{\"mesh\":0,\"skin\":0},{\"children\":[2]},{}],\"skins\":[{\"joints\":[1,2]}],\"meshes\":[{\"primitives\":[{\"mode\":0,\"attributes\":{\"POSITION\":0,\"JOINTS_0\":1,\"WEIGHTS_0\":2,\"JOINTS_1\":3,\"WEIGHTS_1\":4}}]}],\"accessors\":[{\"componentType\":5126,\"type\":\"VEC3\",\"count\":1,\"min\":[0,0,0],\"max\":[0,0,0]},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":0},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":1,\"normalized\":true},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":2},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":3,\"normalized\":true}],\"bufferViews\":[{\"buffer\":0,\"byteOffset\":0,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":4,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":8,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":12,\"byteLength\":16}],\"buffers\":[{\"byteLength\":28,\"uri\":\"data:application/octet-stream;base64,AAEAAIB/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==\"}]}", true);
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{\"mesh\":0,\"skin\":0},{\"children\":[2]},{}],\"skins\":[{\"joints\":[1,2]}],\"meshes\":[{\"primitives\":[{\"mode\":0,\"attributes\":{\"POSITION\":0,\"JOINTS_0\":1,\"WEIGHTS_0\":2,\"JOINTS_1\":3,\"WEIGHTS_1\":4}}]}],\"accessors\":[{\"componentType\":5126,\"type\":\"VEC3\",\"count\":1,\"min\":[0,0,0],\"max\":[0,0,0]},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":0},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":1,\"normalized\":true},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":2},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":3,\"normalized\":true}],\"bufferViews\":[{\"buffer\":0,\"byteOffset\":0,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":4,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":8,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":12,\"byteLength\":16}],\"buffers\":[{\"byteLength\":28,\"uri\":\"data:application/octet-stream;base64,AAEAAH9/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==\"}]}", false);
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{\"mesh\":0,\"skin\":0},{\"children\":[2]},{}],\"skins\":[{\"joints\":[1,2]}],\"meshes\":[{\"primitives\":[{\"mode\":0,\"attributes\":{\"POSITION\":0,\"JOINTS_0\":1,\"WEIGHTS_0\":2,\"JOINTS_1\":3,\"WEIGHTS_1\":4}}]}],\"accessors\":[{\"componentType\":5126,\"type\":\"VEC3\",\"count\":1,\"min\":[0,0,0],\"max\":[0,0,0]},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":0},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":1,\"normalized\":true},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":2},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":3,\"normalized\":true}],\"bufferViews\":[{\"buffer\":0,\"byteOffset\":0,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":4,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":8,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":12,\"byteLength\":16}],\"buffers\":[{\"byteLength\":28,\"uri\":\"data:application/octet-stream;base64,AAEAAICAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==\"}]}", false);
} @test

mixed_quantized_skin_weights :: () {
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{\"mesh\":0,\"skin\":0},{\"children\":[2]},{}],\"skins\":[{\"joints\":[1,2]}],\"meshes\":[{\"primitives\":[{\"mode\":0,\"attributes\":{\"POSITION\":0,\"JOINTS_0\":1,\"WEIGHTS_0\":2,\"JOINTS_1\":3,\"WEIGHTS_1\":4}}]}],\"accessors\":[{\"componentType\":5126,\"type\":\"VEC3\",\"count\":1,\"min\":[0,0,0],\"max\":[0,0,0]},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":0},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":1,\"normalized\":true},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":2},{\"componentType\":5123,\"type\":\"VEC4\",\"count\":1,\"bufferView\":3,\"normalized\":true}],\"bufferViews\":[{\"buffer\":0,\"byteOffset\":0,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":4,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":8,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":12,\"byteLength\":8}],\"buffers\":[{\"byteLength\":20,\"uri\":\"data:application/octet-stream;base64,AAAAAIAAAAABAAAAf38AAAAAAAA=\"}]}", true);
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{\"mesh\":0,\"skin\":0},{\"children\":[2]},{}],\"skins\":[{\"joints\":[1,2]}],\"meshes\":[{\"primitives\":[{\"mode\":0,\"attributes\":{\"POSITION\":0,\"JOINTS_0\":1,\"WEIGHTS_0\":2,\"JOINTS_1\":3,\"WEIGHTS_1\":4}}]}],\"accessors\":[{\"componentType\":5126,\"type\":\"VEC3\",\"count\":1,\"min\":[0,0,0],\"max\":[0,0,0]},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":0},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":1,\"normalized\":true},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":2},{\"componentType\":5123,\"type\":\"VEC4\",\"count\":1,\"bufferView\":3,\"normalized\":true}],\"bufferViews\":[{\"buffer\":0,\"byteOffset\":0,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":4,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":8,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":12,\"byteLength\":8}],\"buffers\":[{\"byteLength\":20,\"uri\":\"data:application/octet-stream;base64,AAAAAIAAAAABAAAAfn8AAAAAAAA=\"}]}", false);
    check_document("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{\"mesh\":0,\"skin\":0},{\"children\":[2]},{}],\"skins\":[{\"joints\":[1,2]}],\"meshes\":[{\"primitives\":[{\"mode\":0,\"attributes\":{\"POSITION\":0,\"JOINTS_0\":1,\"WEIGHTS_0\":2,\"JOINTS_1\":3,\"WEIGHTS_1\":4}}]}],\"accessors\":[{\"componentType\":5126,\"type\":\"VEC3\",\"count\":1,\"min\":[0,0,0],\"max\":[0,0,0]},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":0},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":1,\"normalized\":true},{\"componentType\":5121,\"type\":\"VEC4\",\"count\":1,\"bufferView\":2},{\"componentType\":5123,\"type\":\"VEC4\",\"count\":1,\"bufferView\":3,\"normalized\":true}],\"bufferViews\":[{\"buffer\":0,\"byteOffset\":0,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":4,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":8,\"byteLength\":4},{\"buffer\":0,\"byteOffset\":12,\"byteLength\":8}],\"buffers\":[{\"byteLength\":20,\"uri\":\"data:application/octet-stream;base64,AAAAAIAAAAABAAAAgH8AAAAAAAA=\"}]}", false);
} @test
