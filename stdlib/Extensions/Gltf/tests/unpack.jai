#scope_file
#import "Basic";
#import "Extensions/Testing";
#import "Extensions/Gltf";

// A float accessor whose middle element is a sparse replacement, and a normalized byte pair beside a padded MAT2.
SPARSE :: "{\"asset\":{\"version\":\"2.0\"},\"buffers\":[{\"byteLength\":8,\"uri\":\"data:application/octet-stream;base64,AQAAAACAAEA=\"}],\"bufferViews\":[{\"buffer\":0,\"byteLength\":1},{\"buffer\":0,\"byteOffset\":4,\"byteLength\":4}],\"accessors\":[{\"componentType\":5126,\"count\":3,\"type\":\"SCALAR\",\"sparse\":{\"count\":1,\"indices\":{\"bufferView\":0,\"componentType\":5121},\"values\":{\"bufferView\":1}}}]}";
NUMERIC :: "{\"asset\":{\"version\":\"2.0\"},\"buffers\":[{\"byteLength\":12,\"uri\":\"data:application/octet-stream;base64,gH8A/wECAAMEBQAA\"}],\"bufferViews\":[{\"buffer\":0,\"byteLength\":2},{\"buffer\":0,\"byteOffset\":4,\"byteLength\":6}],\"accessors\":[{\"bufferView\":0,\"componentType\":5120,\"normalized\":true,\"count\":2,\"type\":\"SCALAR\"},{\"bufferView\":1,\"componentType\":5121,\"count\":1,\"type\":\"MAT2\"}]}";

#scope_export
unpack_whole_accessors :: () {
    asset, error := gltf_load(NUMERIC);

    require(error.kind == .NONE, error.message);
    defer gltf_free(*asset);

    bytes, e0 := gltf_unpack_floats(*asset, 0);

    require(e0.kind == .NONE);
    defer free(bytes.data);
    require_equal(bytes.count, 2);
    expect(bytes[0] == -1 && bytes[1] == 1);

    matrix, e1 := gltf_unpack_floats(*asset, 1);

    require(e1.kind == .NONE);
    defer free(matrix.data);
    require_equal(matrix.count, 4);
    expect(matrix[0] == 1 && matrix[1] == 2 && matrix[2] == 4 && matrix[3] == 5);

    _, bad := gltf_unpack_floats(*asset, 9);

    expect_equal(bad.kind, Gltf_Error_Kind.INVALID);
    gltf_error_free(*bad);
} @test

unpack_applies_sparse_replacement :: () {
    asset, error := gltf_load(SPARSE);

    require(error.kind == .NONE, error.message);
    defer gltf_free(*asset);

    values, e := gltf_unpack_floats(*asset, 0);

    require(e.kind == .NONE);
    defer free(values.data);
    require_equal(values.count, 3);
    expect(values[0] == 0 && values[1] == 2.0078125 && values[2] == 0);
} @test
