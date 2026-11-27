#scope_file
#import "Basic";
#import "Math";
#import "Extensions/Testing";
#import "Extensions/Gltf";

nearly :: (a: float32, b: float32) -> bool {
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

    expect(nearly(p.x, 1) && nearly(p.y, 2) && nearly(p.z, 0));

    // A glTF matrix is column-major: its translation is the last column.
    m := gltf_node_local(nodes[2]);

    expect(m._34 == 5 && m._14 == 0);
    expect(gltf_node_local(nodes[3]) == Matrix4_Identity);

    world, ok := gltf_node_world(*asset, 2);

    require(ok);
    w := transform_point(world, xyz(0, 0, 0));

    expect(nearly(w.x, 2) && nearly(w.y, 2) && nearly(w.z, 13));

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

world_matrices_match_one_at_a_time :: () {
    asset, error := gltf_load(TRANSFORMS);

    require(error.kind == .NONE, error.message);
    defer gltf_free(*asset);

    worlds, ok := gltf_world_matrices(*asset);

    require(ok);
    defer array_free(worlds);
    expect(worlds.count == asset.document.nodes.count);

    for worlds {
        one, one_ok := gltf_node_world(*asset, it_index);

        expect(one_ok && it == one);
    }
} @test

// Children listed before their parents still see the parent's transform.
world_matrices_with_a_child_before_its_parent :: () {
    asset, error := gltf_load("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{\"translation\":[0,0,1]},{\"translation\":[0,1,0],\"children\":[0]},{\"translation\":[1,0,0],\"children\":[1]}]}");

    require(error.kind == .NONE, error.message);
    defer gltf_free(*asset);

    worlds, ok := gltf_world_matrices(*asset);

    require(ok);
    defer array_free(worlds);

    p := transform_point(worlds[0], xyz(0, 0, 0));

    expect(nearly(p.x, 1) && nearly(p.y, 1) && nearly(p.z, 1));
} @test

world_matrices_need_checked_asset :: () {
    asset: Gltf_Asset;

    asset.document.nodes = .[.{}];
    worlds, ok := gltf_world_matrices(*asset);

    expect(!ok && worlds.count == 0);
} @test

decomposing_a_node_matrix_gives_its_parts_back :: () {
    asset, error := gltf_load(TRANSFORMS);

    require(error.kind == .NONE, error.message);
    defer gltf_free(*asset);

    translation, rotation, scale, ok := gltf_decompose(gltf_node_local(asset.document.nodes[1]));

    require(ok);
    expect(nearly(translation.x, 1) && nearly(translation.y, 0) && nearly(translation.z, 0));
    expect(nearly(scale.x, 2) && nearly(scale.y, 2) && nearly(scale.z, 2));
    expect(nearly(rotation.z, cast(float32) SQRTH) && nearly(rotation.w, cast(float32) SQRTH) && nearly(rotation.x, 0) && nearly(rotation.y, 0));
} @test

decomposing_a_mirrored_matrix_puts_the_flip_in_the_scale :: () {
    mirrored := gltf_node_local(.{ scale = .[-1, 2, 3], translation = .[4, 5, 6] });

    translation, rotation, scale, ok := gltf_decompose(mirrored);

    require(ok);
    expect(nearly(translation.x, 4) && nearly(translation.y, 5) && nearly(translation.z, 6));
    expect(nearly(scale.x, -1) && nearly(scale.y, 2) && nearly(scale.z, 3));

    back := gltf_node_local(.{ translation = .[translation.x, translation.y, translation.z], rotation = .[rotation.x, rotation.y, rotation.z, rotation.w], scale = .[scale.x, scale.y, scale.z] });

    for 0..15 expect(nearly(back.floats[it], mirrored.floats[it]));
} @test

decomposing_refuses_a_matrix_with_no_scale_or_a_projection :: () {
    flat := Matrix4_Identity;

    flat._11 = 0;
    _, _, _, ok := gltf_decompose(flat);

    expect(!ok);

    projective := Matrix4_Identity;

    projective._41 = 1;
    _, _, _, ok = gltf_decompose(projective);

    expect(!ok);
} @test
