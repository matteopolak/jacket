// Start allocation-free iteration over an asset from gltf_load or gltf_validate. Do not mutate it.
//.
//
// ```jai
// asset, error := gltf_load("{\"asset\":{\"version\":\"2.0\"},\"scene\":0,\"scenes\":[{\"nodes\":[0]}],\"nodes\":[{\"mesh\":0}],\"meshes\":[{\"primitives\":[{\"attributes\":{\"POSITION\":0},\"indices\":1}]}],\"buffers\":[{\"byteLength\":42,\"uri\":\"data:application/octet-stream;base64,AAAAAAAAAAAAAAAAAACAPwAAAAAAAAAAAAAAAAAAgD8AAAAAAAABAAIA\"}],\"bufferViews\":[{\"buffer\":0,\"byteLength\":36,\"target\":34962},{\"buffer\":0,\"byteOffset\":36,\"byteLength\":6,\"target\":34963}],\"accessors\":[{\"bufferView\":0,\"componentType\":5126,\"count\":3,\"type\":\"VEC3\",\"min\":[0,0,0],\"max\":[1,1,0]},{\"bufferView\":1,\"componentType\":5123,\"count\":3,\"type\":\"SCALAR\"}]}");
// assert(error.kind == .NONE);
// defer gltf_free(*asset);
// cursor, cursor_error := gltf_accessor(*asset, 0);
// assert(cursor_error.kind == .NONE);
//
// count: int;
// while true {
//     vertex, more, step := gltf_next_floats(*cursor);
//     if !more break;
//     assert(step.kind == .NONE);
//     if count == 1 assert(vertex[0] == 1 && vertex[1] == 0);
//     count += 1;
// }
// assert(count == 3);
// ```
gltf_accessor :: (asset: *Gltf_Asset, index: s64) -> Gltf_Cursor, Gltf_Error {
    if index < 0 || index >= asset.document.accessors.count return .{}, failure(.INVALID, "accessor index out of range", "/accessors");

    _, rows, cols := layout(asset.document.accessors[index]);

    return .{ asset = asset, accessor = index, components = rows * cols }, .{};
}

// Read the next vector or matrix (column-major); use cursor.components lanes. End gives false.
//.
//
// ```jai
// asset, error := gltf_load("{\"asset\":{\"version\":\"2.0\"},\"scene\":0,\"scenes\":[{\"nodes\":[0]}],\"nodes\":[{\"mesh\":0}],\"meshes\":[{\"primitives\":[{\"attributes\":{\"POSITION\":0},\"indices\":1}]}],\"buffers\":[{\"byteLength\":42,\"uri\":\"data:application/octet-stream;base64,AAAAAAAAAAAAAAAAAACAPwAAAAAAAAAAAAAAAAAAgD8AAAAAAAABAAIA\"}],\"bufferViews\":[{\"buffer\":0,\"byteLength\":36,\"target\":34962},{\"buffer\":0,\"byteOffset\":36,\"byteLength\":6,\"target\":34963}],\"accessors\":[{\"bufferView\":0,\"componentType\":5126,\"count\":3,\"type\":\"VEC3\",\"min\":[0,0,0],\"max\":[1,1,0]},{\"bufferView\":1,\"componentType\":5123,\"count\":3,\"type\":\"SCALAR\"}]}");
// assert(error.kind == .NONE);
// defer gltf_free(*asset);
// cursor, _ := gltf_accessor(*asset, 0);
// first, more, step := gltf_next_floats(*cursor);
// assert(more && step.kind == .NONE);
// assert(first[0] == 0 && first[1] == 0 && first[2] == 0); // `cursor.components` lanes are used
// ```
gltf_next_floats :: (cursor: *Gltf_Cursor) -> [16] float32, bool, Gltf_Error {
    result: [16] float32;
    if !cursor.asset return result, false, failure(.INVALID, "uninitialized cursor", "/accessors");

    a := cursor.asset.document.accessors[cursor.accessor];
    if cursor.element >= a.count return result, false, .{};

    bytes, start := next_position(cursor);
    if !bytes.data return result, true, .{};

    size, rows, cols := layout(a);
    column := rows * size;

    if cols > 1 column = (column + 3) & ~3;

    for c: 0..cols - 1 for r : 0..rows - 1 {
        raw := word(bytes, start + c * column + r * size, size);
        value, valid := scalar(raw, a.component_type, a.normalized);
        if !valid return result, false, failure(.INVALID, "non-finite accessor value", tprint("/accessors/%", cursor.accessor), start + c * column + r * size);
        result[c * rows + r] = cast(float32) value;
    }

    return result, true, .{};
}

// Read an unsigned SCALAR without float rounding.
//
// ```jai
// asset, error := gltf_load("{\"asset\":{\"version\":\"2.0\"},\"scene\":0,\"scenes\":[{\"nodes\":[0]}],\"nodes\":[{\"mesh\":0}],\"meshes\":[{\"primitives\":[{\"attributes\":{\"POSITION\":0},\"indices\":1}]}],\"buffers\":[{\"byteLength\":42,\"uri\":\"data:application/octet-stream;base64,AAAAAAAAAAAAAAAAAACAPwAAAAAAAAAAAAAAAAAAgD8AAAAAAAABAAIA\"}],\"bufferViews\":[{\"buffer\":0,\"byteLength\":36,\"target\":34962},{\"buffer\":0,\"byteOffset\":36,\"byteLength\":6,\"target\":34963}],\"accessors\":[{\"bufferView\":0,\"componentType\":5126,\"count\":3,\"type\":\"VEC3\",\"min\":[0,0,0],\"max\":[1,1,0]},{\"bufferView\":1,\"componentType\":5123,\"count\":3,\"type\":\"SCALAR\"}]}");
// assert(error.kind == .NONE);
// defer gltf_free(*asset);
// cursor, _ := gltf_accessor(*asset, 1);
// total: u32;
// while true {
//     index, more, step := gltf_next_index(*cursor);
//     if !more break;
//     assert(step.kind == .NONE);
//     total += index;
// }
// assert(total == 3); // 0 + 1 + 2
// ```
gltf_next_index :: (cursor: *Gltf_Cursor) -> u32, bool, Gltf_Error {
    if !cursor.asset return 0, false, failure(.INVALID, "uninitialized cursor", "/accessors");

    a := cursor.asset.document.accessors[cursor.accessor];
    if a.type != .SCALAR || !unsigned_component(a.component_type) || a.normalized return 0, false, failure(.INVALID, "not an index accessor", tprint("/accessors/%", cursor.accessor));
    if cursor.element >= a.count return 0, false, .{};

    bytes, start := next_position(cursor);
    if !bytes.data return 0, true, .{};

    return word(bytes, start, component_size(a.component_type)), true, .{};
}

#scope_module
next_position :: (cursor: *Gltf_Cursor) -> string, s64 {
    a := cursor.asset.document.accessors[cursor.accessor];
    element := cursor.element;

    cursor.element += 1;
    if a.sparse && cursor.sparse_element < a.sparse.count {
        s := a.sparse;
        bytes, start := view_data(cursor.asset, s.indices.buffer_view, s.indices.byte_offset);
        size := component_size(s.indices.component_type);
        index := word(bytes, start + cursor.sparse_element * size, size);

        if index == element {
            bytes, start = view_data(cursor.asset, s.values.buffer_view, s.values.byte_offset);
            start += cursor.sparse_element * element_size(a);
            cursor.sparse_element += 1;
            return bytes, start;
        }
    }

    if a.buffer_view.state != .SET return "", 0;

    v := cursor.asset.document.buffer_views[a.buffer_view.value];
    stride := element_size(a);

    if v.byte_stride.state == .SET stride = v.byte_stride.value;

    bytes, start := view_data(cursor.asset, a.buffer_view.value, a.byte_offset);

    return bytes, start + element * stride;
}

scalar :: (raw: u32, kind: Gltf_Component, normalized: bool) -> float64, bool {
    value: float64;

    if kind == .FLOAT {
        if (raw & 0x7f800000) == 0x7f800000 return 0, false;
        return cast(float64) cast,force(float32) raw, true;
    }

    if kind == .BYTE {
        value = cast(float64) cast,trunc(s8) raw;
        if normalized value = max(cast(float64) -1.0, value / 127.0);
    } else if kind == .SHORT {
        value = cast(float64) cast,trunc(s16) raw;
        if normalized value = max(cast(float64) -1.0, value / 32767.0);
    } else {
        value = cast(float64) raw;
        if normalized value /= ifx kind == .UNSIGNED_BYTE then 255.0 else 65535.0;
    }

    return value, true;
}
