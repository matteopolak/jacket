#scope_file
#import "Basic";
#import "String";
#import "Extensions/Gltf";
#import "Extensions/Testing";
#import "Base64";
#import,file "../../test-support/counting-allocator.jai";

SPARSE :: "{\"asset\":{\"version\":\"2.0\"},\"buffers\":[{\"byteLength\":8,\"uri\":\"data:application/octet-stream;base64,AQAAAACAAEA=\"}],\"bufferViews\":[{\"buffer\":0,\"byteLength\":1},{\"buffer\":0,\"byteOffset\":4,\"byteLength\":4}],\"accessors\":[{\"componentType\":5126,\"count\":3,\"type\":\"SCALAR\",\"sparse\":{\"count\":1,\"indices\":{\"bufferView\":0,\"componentType\":5121},\"values\":{\"bufferView\":1}}}]}";

#scope_export
roundtrip_and_sparse :: () {
    asset, error := gltf_load(SPARSE);

    require(error.kind == .NONE);
    defer gltf_free(*asset);
    cursor, ce := gltf_accessor(*asset, 0);

    require(ce.kind == .NONE);

    for i: 0..2 {
        v, ok, e := gltf_next_floats(*cursor);
        expect(ok && e.kind == .NONE);
        expect_equal(v[0], ifx i == 1 then 2.0078125 else 0.0);
    }

    _, ok := gltf_next_floats(*cursor);

    expect(!ok);

    for binary: bool.[false, true] {
        bytes, e := gltf_write(*asset, binary);

        require(e.kind == .NONE);
        defer free(bytes);
        again, e2 := gltf_load(bytes);

        defer gltf_error_free(*e2);
        require(e2.kind == .NONE, tprint("% % %", binary, e2.message, bytes));
        defer gltf_free(*again);
        expect_equal(again.document.accessors.count, 1);
    }
} @test

// A document using one value of each enum-typed field; the arguments replace the values under test.
typed_document :: (mode: s64, wrap: s64, alpha: string, interpolation: string, path: string, camera: string, component: s64, kind: string) -> string {
    data: [32] u8;

    data[6] = 0x80; // the float 1.0 after a zero one: the animation times
    data[7] = 0x3f;
    encoded := base64_encode(string.{ 32, data.data });

    defer free(encoded);

    return tprint("{\"asset\":{\"version\":\"2.0\"},\"buffers\":[{\"byteLength\":32,\"uri\":\"data:application/octet-stream;base64,%\"}],\"bufferViews\":[{\"buffer\":0,\"byteOffset\":8,\"byteLength\":24,\"target\":34962},{\"buffer\":0,\"byteLength\":8}],\"accessors\":[{\"bufferView\":0,\"componentType\":%,\"count\":2,\"type\":\"%\",\"min\":[0,0,0],\"max\":[0,0,0]},{\"bufferView\":1,\"componentType\":5126,\"count\":2,\"type\":\"SCALAR\",\"min\":[0],\"max\":[1]}],\"nodes\":[{\"mesh\":0}],\"meshes\":[{\"primitives\":[{\"attributes\":{\"POSITION\":0},\"mode\":%}]}],\"samplers\":[{\"magFilter\":9729,\"minFilter\":9987,\"wrapS\":%}],\"materials\":[{\"alphaMode\":\"%\"}],\"cameras\":[{\"type\":\"%\",\"orthographic\":{\"xmag\":1,\"ymag\":1,\"zfar\":10,\"znear\":1}}],\"animations\":[{\"samplers\":[{\"input\":1,\"output\":0,\"interpolation\":\"%\"}],\"channels\":[{\"sampler\":0,\"target\":{\"node\":0,\"path\":\"%\"}}]}]}", encoded, component, kind, mode, wrap, alpha, camera, interpolation, path);
}

typed_fields_and_roundtrip :: () {
    text := typed_document(1, 33071, "MASK", "STEP", "translation", "orthographic", 5126, "VEC3");
    asset, error := gltf_load(text);

    require(error.kind == .NONE, tprint("%", error.message));
    defer gltf_free(*asset);
    d := asset.document;

    for check: 0..1 {
        expect_equal(d.accessors[0].component_type, Gltf_Component.FLOAT);
        expect_equal(d.accessors[0].type, Gltf_Accessor_Type.VEC3);
        expect_equal(d.buffer_views[0].target.value, Gltf_Target.ARRAY_BUFFER);
        expect_equal(d.meshes[0].primitives[0].mode, Gltf_Mode.LINES);
        expect_equal(d.samplers[0].mag_filter.value, Gltf_Filter.LINEAR);
        expect_equal(d.samplers[0].min_filter.value, Gltf_Filter.LINEAR_MIPMAP_LINEAR);
        expect_equal(d.samplers[0].wrap_s, Gltf_Wrap.CLAMP_TO_EDGE);
        expect_equal(d.samplers[0].wrap_t, Gltf_Wrap.REPEAT); // the default
        expect_equal(d.materials[0].alpha_mode, Gltf_Alpha_Mode.MASK);
        expect_equal(d.cameras[0].type, Gltf_Camera_Type.ORTHOGRAPHIC);
        expect_equal(d.animations[0].samplers[0].interpolation, Gltf_Interpolation.STEP);
        expect_equal(d.animations[0].channels[0].target.path, Gltf_Path.TRANSLATION);

        if check == 1 break;

        // Writing keeps the spec's numbers and strings, and the result loads to the same values.
        bytes, e := gltf_write(*asset);

        require(e.kind == .NONE);
        defer free(bytes);

        for expected: string.["\"componentType\":5126", "\"type\":\"VEC3\"", "\"target\":34962", "\"mode\":1", "\"magFilter\":9729", "\"minFilter\":9987", "\"wrapS\":33071", "\"wrapT\":10497", "\"alphaMode\":\"MASK\"", "\"type\":\"orthographic\"", "\"interpolation\":\"STEP\"", "\"path\":\"translation\""] {
            expect(contains(bytes, expected), expected);
        }

        again, e2 := gltf_load(bytes);

        defer gltf_error_free(*e2);
        require(e2.kind == .NONE, e2.message);
        gltf_free(*asset);
        asset = again;
        d = asset.document;
    }
} @test

out_of_set_enum_values :: () {
    valid := typed_document(1, 33071, "MASK", "STEP", "translation", "orthographic", 5126, "VEC3");

    expect(valid.count > 0);

    for bad: string.[
        typed_document(7, 33071, "MASK", "STEP", "translation", "orthographic", 5126, "VEC3"),
        typed_document(1, 1234, "MASK", "STEP", "translation", "orthographic", 5126, "VEC3"),
        typed_document(1, 33071, "HALF", "STEP", "translation", "orthographic", 5126, "VEC3"),
        typed_document(1, 33071, "MASK", "SMOOTH", "translation", "orthographic", 5126, "VEC3"),
        typed_document(1, 33071, "MASK", "STEP", "position", "orthographic", 5126, "VEC3"),
        typed_document(1, 33071, "MASK", "STEP", "translation", "fisheye", 5126, "VEC3"),
        typed_document(1, 33071, "MASK", "STEP", "translation", "orthographic", 5124, "VEC3"),
        typed_document(1, 33071, "MASK", "STEP", "translation", "orthographic", 5126, "VEC5"),
        typed_document(1, 33071, "MASK", "STEP", "translation", "orthographic", 5126, "vec3")
    ] {
        _, e := gltf_load(bad);

        defer gltf_error_free(*e);
        expect(e.kind != .NONE, bad);
        expect(e.path.count > 0, "the error names a field");
    }
} @test

invalid_references_and_layout :: () {
    cases := string.[
        "{\"nodes\":[{\"children\":[0]}]}",
        "{\"nodes\":[{\"mesh\":0}]}",
        "{\"scenes\":[{\"nodes\":[1]}]}",
        "{\"accessors\":[{\"componentType\":5126,\"count\":4294967295,\"type\":\"VEC3\",\"bufferView\":0}]}",
        "{\"textures\":[{\"source\":0}]}",
        "{\"animations\":[{\"channels\":[{\"sampler\":0,\"target\":{\"node\":0,\"path\":\"translation\"}}],\"samplers\":[]}]}",
        "{\"materials\":[{\"normalTexture\":{\"index\":0}}]}",
        "{\"skins\":[{\"joints\":[0]}]}",
        "{\"meshes\":[{\"primitives\":[{\"attributes\":{\"POSITION\":0}}]}]}"
    ];
    for cases {
        text := tprint("{\"asset\":{\"version\":\"2.0\"},%", slice(it, 1, it.count - 1));
        a, e := gltf_load(text);

        defer gltf_free(*a);
        defer gltf_error_free(*e);
        expect(e.kind != .NONE && e.path.count > 0);
    }
} @test

truncation_and_ownership :: () {
    tally: Tally;
    balanced := true;

    {
        push_allocator(tally_proc, *tally);

        for n: 0..SPARSE.count {
            a, e := gltf_load(slice(SPARSE, 0, n));
            gltf_free(*a);
            gltf_error_free(*e);
            if tally.outstanding != 0 balanced = false;
        }

        a, e := gltf_load(SPARSE);
        bytes, we := gltf_write(*a, true);

        for n: 0..bytes.count - 1 {
            b, be := gltf_load(slice(bytes, 0, n));
            gltf_free(*b);
            gltf_error_free(*be);
        }

        free(bytes);
        gltf_free(*a);
        gltf_error_free(*e);
        gltf_error_free(*we);
    }

    expect(balanced && tally.outstanding == 0);
} @test

numeric_accessors_and_matrix_padding :: () {
    text :: "{\"asset\":{\"version\":\"2.0\"},\"buffers\":[{\"byteLength\":12,\"uri\":\"data:application/octet-stream;base64,gH8A/wECAAMEBQAA\"}],\"bufferViews\":[{\"buffer\":0,\"byteLength\":2},{\"buffer\":0,\"byteOffset\":4,\"byteLength\":6}],\"accessors\":[{\"bufferView\":0,\"componentType\":5120,\"normalized\":true,\"count\":2,\"type\":\"SCALAR\"},{\"bufferView\":1,\"componentType\":5121,\"count\":1,\"type\":\"MAT2\"}]}";
    a, e := gltf_load(text);

    defer gltf_error_free(*e);
    require(e.kind == .NONE, e.message);
    defer gltf_free(*a);
    c, ce := gltf_accessor(*a, 0);

    require(ce.kind == .NONE);
    first, _, fe := gltf_next_floats(*c);

    require(fe.kind == .NONE);
    expect_equal(first[0], -1.0);
    second, _, se := gltf_next_floats(*c);

    require(se.kind == .NONE);
    expect_equal(second[0], 1.0);
    m, me := gltf_accessor(*a, 1);

    require(me.kind == .NONE);
    v, _, ve := gltf_next_floats(*m);

    require(ve.kind == .NONE);
    expect(v[0] == 1 && v[1] == 2 && v[2] == 4 && v[3] == 5);
} @test
