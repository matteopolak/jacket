#scope_file
#import "Basic";
#import "Math";
#import "Extensions/Testing";
#import "Extensions/Gltf";

near :: (a: float32, b: float32) -> bool {
    return abs(a - b) < 0.0001;
}

TRANSFORMS :: "{\"asset\":{\"version\":\"2.0\"},\"scene\":0,\"scenes\":[{\"nodes\":[0,3]}],\"nodes\":[{\"translation\":[1,2,3],\"children\":[1]},{\"translation\":[1,0,0],\"rotation\":[0,0,0.70710678,0.70710678],\"scale\":[2,2,2],\"children\":[2]},{\"matrix\":[1,0,0,0,0,1,0,0,0,0,1,0,0,0,5,1]},{}]}";

#scope_export
node_local_and_world_matrices :: () {
    asset, error := gltf_load(TRANSFORMS);

    require(error.kind == .NONE, error.message);
    defer gltf_free(*asset);
    nodes := asset.document.nodes;

    // Scale applies first, then rotation, then translation.
    p := transform_point(gltf_node_local(nodes[1]), xyz(1, 0, 0));

    expect(near(p.x, 1) && near(p.y, 2) && near(p.z, 0));

    // A glTF matrix is column-major: its translation is the last column.
    m := gltf_node_local(nodes[2]);

    expect(m._34 == 5 && m._14 == 0);
    expect(gltf_node_local(nodes[3]) == Matrix4_Identity);

    world, ok := gltf_node_world(*asset, 2);

    require(ok);
    w := transform_point(world, xyz(0, 0, 0));

    expect(near(w.x, 2) && near(w.y, 2) && near(w.z, 13));

    root, root_ok := gltf_node_world(*asset, 3);

    expect(root_ok && root == Matrix4_Identity);

    _, out_of_range := gltf_node_world(*asset, 4);

    expect(!out_of_range);
} @test

// A hand-built asset has no parent index until validation builds it.
world_needs_checked_asset :: () {
    asset: Gltf_Asset;
    _, ok := gltf_node_world(*asset, 0);

    expect(!ok);
} @test
