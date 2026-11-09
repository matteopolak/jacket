// A node's transform relative to its parent, as a `Matrix4` (column vectors, `m * v`). A node with a
// `matrix` uses it, converted from glTF's column-major order; otherwise the result is translation
// times rotation times scale, with absent parts as identity. Rotation is the `[x, y, z, w]`
// quaternion. The node must come from a loaded or validated asset, which checks the lengths.
//
// ```jai
// #import "Math";
//
// asset, error := gltf_load("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{\"translation\":[1,2,3],\"scale\":[2,2,2]}]}");
// assert(error.kind == .NONE);
// defer gltf_free(*asset);
// local := gltf_node_local(asset.document.nodes[0]);
// assert(transform_point(local, xyz(1, 1, 1)) == xyz(3, 4, 5));
// ```
gltf_node_local :: (node: Gltf_Node) -> Matrix4 {
    result := Matrix4_Identity;

    if node.matrix.count == 16 {
        for column: 0..3 for row : 0..3 result.floats[row * 4 + column] = node.matrix[column * 4 + row];
        return result;
    }

    if node.rotation.count == 4 result = rotation_matrix(Matrix4, Quaternion.{ node.rotation[0], node.rotation[1], node.rotation[2], node.rotation[3] });
    if node.scale.count == 3 result = scale(result, xyz(node.scale[0], node.scale[1], node.scale[2]));

    if node.translation.count == 3 {
        result._14 = node.translation[0];
        result._24 = node.translation[1];
        result._34 = node.translation[2];
    }

    return result;
}

// A node's transform in the scene: its local matrix after those of its ancestors, root first. The
// parent of every node is indexed once by [gltf_load] or [gltf_validate], so each call only walks
// up the chain. It gives false for an index out of range, or an asset that was never validated.
//
// ```jai
// #import "Math";
//
// asset, error := gltf_load("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{\"translation\":[1,0,0],\"children\":[1]},{\"translation\":[0,2,0]}]}");
// assert(error.kind == .NONE);
// defer gltf_free(*asset);
// world, ok := gltf_node_world(*asset, 1);
// assert(ok);
// assert(transform_point(world, xyz(0, 0, 0)) == xyz(1, 2, 0));
// ```
gltf_node_world :: (asset: *Gltf_Asset, index: s64) -> Matrix4, bool {
    nodes := asset.document.nodes;
    if index < 0 || index >= nodes.count || asset.parents.count != nodes.count return Matrix4_Identity, false;

    result := gltf_node_local(nodes[index]);
    parent := asset.parents[index];

    for 1..nodes.count {
        if parent < 0 break;

        result = gltf_node_local(nodes[parent]) * result;
        parent = asset.parents[parent];
    }

    return result, parent < 0;
}
