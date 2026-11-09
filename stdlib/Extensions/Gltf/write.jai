// Encode an asset as self-contained buffer JSON or GLB; image URIs remain unchanged.
//.
//
// ```jai
// asset, error := gltf_load("{\"asset\":{\"version\":\"2.0\"},\"scene\":0,\"scenes\":[{\"nodes\":[0]}],\"nodes\":[{\"mesh\":0}],\"meshes\":[{\"primitives\":[{\"attributes\":{\"POSITION\":0},\"indices\":1}]}],\"buffers\":[{\"byteLength\":42,\"uri\":\"data:application/octet-stream;base64,AAAAAAAAAAAAAAAAAACAPwAAAAAAAAAAAAAAAAAAgD8AAAAAAAABAAIA\"}],\"bufferViews\":[{\"buffer\":0,\"byteLength\":36,\"target\":34962},{\"buffer\":0,\"byteOffset\":36,\"byteLength\":6,\"target\":34963}],\"accessors\":[{\"bufferView\":0,\"componentType\":5126,\"count\":3,\"type\":\"VEC3\",\"min\":[0,0,0],\"max\":[1,1,0]},{\"bufferView\":1,\"componentType\":5123,\"count\":3,\"type\":\"SCALAR\"}]}");
// assert(error.kind == .NONE);
// defer gltf_free(*asset);
// text, write_error := gltf_write(*asset);
// assert(write_error.kind == .NONE);
// defer free(text);
//
// again, read_error := gltf_load(text);
// assert(read_error.kind == .NONE && again.buffers[0].count == 42);
// gltf_free(*again);
//
// glb, _ := gltf_write(*asset, binary = true); // the same asset as a GLB
// defer free(glb);
// assert(glb.count > 12 && glb[0] == #char "g" && glb[1] == #char "l");
// ```
gltf_write :: (asset: *Gltf_Asset, binary := false, options := Gltf_Options.{}) -> string, Gltf_Error {
    e := gltf_validate(asset, options);
    if e.kind != .NONE return "", e;

    // A typed copy keeps ownership and optional-field presence independent of the caller.
    tree := to_json_value(asset.document);

    defer free_json(*tree);

    // The output must load again: extension objects need their names in `extensionsUsed`.
    used: Table(string, bool);

    defer table_reset(*used);
    for name: asset.document.extensions_used table_add(*used, name, true);

    e = check_shape(*tree, "", *used, options);
    if e.kind != .NONE return "", e;

    doc, je := from_json(Gltf_Document, *tree, Json_Options.{ max_depth = options.max_depth });
    if je.kind != .NONE {
        e = failure(.JSON, je.message, tprint("/%", je.path), je.offset);
        free_json(*je);
        return "", e;
    }

    defer free_json(*doc);
    bin: string;

    for *b, i: doc.buffers {
        free(b.uri);
        b.uri = "";
        data := slice(asset.buffers[i], 0, b.byte_length);
        if binary && i == 0 bin = data;
        else {
            encoded := base64_encode(data);
            defer free(encoded);
            b.uri = join("data:application/octet-stream;base64,", encoded);
        }
    }

    text := to_json(doc);

    if !binary {
        if text.count > options.max_size {
            free(text);
            return "", failure(.LIMIT, "JSON exceeds output limit", "/");
        }

        return text, .{};
    }

    defer free(text);
    json_size := (text.count + 3) & ~3;
    bin_size := (bin.count + 3) & ~3;
    length := 20 + json_size;

    if bin.count length += 8 + bin_size;
    if length > options.max_size || length > 0xffffffff return "", failure(.LIMIT, "GLB exceeds output limit", "/");

    builder: String_Builder;

    append(*builder, "glTF");
    put_word(*builder, 2);
    put_word(*builder, cast(u32) length);
    put_word(*builder, cast(u32) json_size);
    put_word(*builder, 0x4e4f534a);
    append(*builder, text);
    for text.count..json_size - 1 append(*builder, " ");

    if bin.count {
        put_word(*builder, cast(u32) bin_size);
        put_word(*builder, 0x004e4942);
        append(*builder, bin);
        for bin.count..bin_size - 1 append(*builder, "\0");
    }

    return builder_to_string(*builder), .{};
}

#scope_module
put_word :: (b: *String_Builder, value: u32) {
    bytes: [4] u8;
    for i: 0..3 bytes[i] = cast(u8) ((value >> cast(u32) (8 * i)) & 255);
    append(b, string.{ 4, bytes.data });
}
