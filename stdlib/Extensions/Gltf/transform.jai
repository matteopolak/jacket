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

// The world matrices of every node, indexed like `asset.document.nodes`. Each node's chain is
// folded once and shared with its descendants, so the whole scene costs one multiply per node where
// calling [gltf_node_world] per node walks every ancestor chain again. It gives false and no
// matrices for an asset that was never validated. The caller frees the array with `array_free`.
//
// ```jai
// #import "Math";
//
// asset, error := gltf_load("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{\"translation\":[1,0,0],\"children\":[1]},{\"translation\":[0,2,0]}]}");
// assert(error.kind == .NONE);
// defer gltf_free(*asset);
// worlds, ok := gltf_world_matrices(*asset);
// assert(ok);
// defer array_free(worlds);
// assert(transform_point(worlds[1], xyz(0, 0, 0)) == xyz(1, 2, 0));
// ```
gltf_world_matrices :: (asset: *Gltf_Asset) -> [] Matrix4, bool {
    nodes := asset.document.nodes;
    parents := asset.parents;

    if parents.count != nodes.count return .[], false;

    worlds: [] Matrix4;

    array_resize(*worlds, nodes.count);

    done := NewArray(nodes.count, bool, initialized = true,, temp);
    chain: [..] s64;

    chain.allocator = temp;

    for 0..nodes.count - 1 {
        if done[it] continue;

        chain.count = 0;
        cursor: s64 = it;

        while cursor >= 0 && !done[cursor] {
            array_add(*chain, cursor);
            cursor = parents[cursor];
        }

        base := Matrix4_Identity;
        if cursor >= 0 base = worlds[cursor];

        for #v2 < chain {
            base = base * gltf_node_local(nodes[it]);
            worlds[it] = base;
            done[it] = true;
        }
    }

    return worlds, true;
}

// Splits an affine matrix, such as a node's world matrix, into translation, rotation and scale, the
// parts a node's `translation`, `rotation` and `scale` hold. A mirror is carried by a negative `x`
// scale. It gives false for a matrix that has no such parts: a projection, a zero scale, or shear,
// where the columns are not at right angles.
//
// ```jai
// #import "Math";
//
// asset, error := gltf_load("{\"asset\":{\"version\":\"2.0\"},\"nodes\":[{\"translation\":[1,2,3],\"scale\":[2,2,2]}]}");
// assert(error.kind == .NONE);
// defer gltf_free(*asset);
// translation, rotation, scale, ok := gltf_decompose(gltf_node_local(asset.document.nodes[0]));
// assert(ok);
// assert(translation == xyz(1, 2, 3) && scale == xyz(2, 2, 2) && rotation.w == 1);
// ```
gltf_decompose :: (m: Matrix4) -> translation: Vector3, rotation: Quaternion, scale: Vector3, ok: bool {
    none: Quaternion;

    none.w = 1;

    if m._41 != 0 || m._42 != 0 || m._43 != 0 || abs(m._44 - 1) > 0.000001 return .{}, none, .{}, false;

    x := xyz(m._11, m._21, m._31);
    y := xyz(m._12, m._22, m._32);
    z := xyz(m._13, m._23, m._33);
    size := xyz(length(x), length(y), length(z));

    if size.x < 0.0000001 || size.y < 0.0000001 || size.z < 0.0000001 return .{}, none, .{}, false;

    x = x / size.x;
    y = y / size.y;
    z = z / size.z;

    if abs(dot_product(x, y)) > 0.0001 || abs(dot_product(x, z)) > 0.0001 || abs(dot_product(y, z)) > 0.0001 return .{}, none, .{}, false;

    // A left-handed frame is a mirror: move it to x so what remains is a rotation.
    if dot_product(cross_product(x, y), z) < 0 {
        size.x = -size.x;
        x = -x;
    }

    rotation := get_rotation(make_matrix_from_columns(x, y, z, Matrix3));

    normalize_or_identity(*rotation);

    return xyz(m._14, m._24, m._34), rotation, size, true;
}
