// Bounded glTF and GLB decoding, validation and accessor iteration, without implicit IO.
// Example: asset, error := gltf_load(text); defer gltf_free(*asset);
// Closed sets in the document are enums: `asset.document.accessors[0].component_type == .FLOAT`, `.type == .VEC3`.
Gltf_Error_Kind :: enum {
    NONE;
    JSON;
    CONTAINER;
    LIMIT;
    RESOLVE;
    INVALID;
}

Gltf_Error :: struct {
    kind: Gltf_Error_Kind;
    message: string;
    offset: s64;
    path: string;
}

Gltf_Options :: struct {
    max_size: s64 = 64 * 1024 * 1024;
    max_buffer_bytes: s64 = 512 * 1024 * 1024;
    max_count: s64 = 1000000;
    max_depth: s64 = 64;
}

// Resolver results are owned allocations, including on failure. Never return more than limit.
Gltf_Resolve :: #type (user: *void, uri: string, limit: s64) -> string, bool;

Gltf_Resolver :: struct {
    resolve: Gltf_Resolve;
    user: *void;
}

Gltf_Asset :: struct {
    document: Gltf_Document;
    buffers: [..] string;
}

Gltf_Cursor :: struct {
    asset: *Gltf_Asset;
    accessor: s64;
    element: s64;
    sparse_element: s64;
    components: s64;
}

// Decode text or GLB bytes and validate before returning. Example: a, e := gltf_load(text);
gltf_load :: (bytes: string, resolver := Gltf_Resolver.{}, options := Gltf_Options.{}) -> Gltf_Asset, Gltf_Error {
    result: Gltf_Asset;
    if bytes.count > options.max_size || options.max_size < 0 || options.max_buffer_bytes < 0 || options.max_count < 0 || options.max_depth < 1 return result, failure(.LIMIT, "invalid limits or input too large", "/");

    text := bytes;
    bin: string;

    if bytes.count >= 4 && slice(bytes, 0, 4) == "glTF" {
        if bytes.count < 20 || word(bytes, 4, 4) != 2 || word(bytes, 8, 4) != bytes.count return result, failure(.CONTAINER, "invalid GLB header", "/", 0);

        at: s64 = 12;
        json_seen, bin_seen: bool;

        while at < bytes.count {
            if bytes.count - at < 8 return result, failure(.CONTAINER, "truncated chunk header", "/", at);

            length := cast(s64) word(bytes, at, 4);
            kind := word(bytes, at + 4, 4);

            if length % 4 != 0 || length > bytes.count - at - 8 return result, failure(.CONTAINER, "invalid chunk length or alignment", "/", at);

            chunk := slice(bytes, at + 8, length);

            if !json_seen && kind != 0x4e4f534a return result, failure(.CONTAINER, "JSON must be first", "/", at);

            if kind == 0x4e4f534a {
                if json_seen return result, failure(.CONTAINER, "duplicate JSON chunk", "/", at);
                text = chunk;
                json_seen = true;
            } else if kind == 0x004e4942 {
                if bin_seen || at != 20 + text.count return result, failure(.CONTAINER, "BIN must be second and unique", "/", at);
                bin = chunk;
                bin_seen = true;
            }

            at += 8 + length;
        }
    }

    opts := Json_Options.{ max_size = options.max_size, max_depth = options.max_depth };
    doc, je := from_json(Gltf_Document, text, opts);

    if je.kind != .NONE {
        e := failure(.JSON, je.message, tprint("/%", je.path), je.offset);
        free_json(*je);
        return result, e;
    }

    result.document = doc;
    shape, shape_error := from_json(Json_Value, text, opts);
    if shape_error.kind != .NONE {
        gltf_free(*result);
        e := failure(.JSON, shape_error.message, tprint("/%", shape_error.path), shape_error.offset);
        free_json(*shape_error);
        return .{}, e;
    }

    used: Table(string, bool);

    defer table_reset(*used);

    for name: doc.extensions_used {
        found, _ := table_find(*used, name);
        if found || !name.count {
            free_json(*shape);
            gltf_free(*result);
            return .{}, failure(.INVALID, "duplicate or empty extension name", "/extensionsUsed");
        }

        table_add(*used, name, true);
    }

    for name: doc.extensions_required {
        found, _ := table_find(*used, name);
        if !found {
            free_json(*shape);
            gltf_free(*result);
            return .{}, failure(.INVALID, "required extension is not used", "/extensionsRequired");
        }
    }

    shape_failure := check_shape(*shape, "", *used, options);

    free_json(*shape);

    if shape_failure.kind != .NONE {
        gltf_free(*result);
        return .{}, shape_failure;
    }

    total: s64;

    for b, i: doc.buffers {
        path := tprint("/buffers/%", i);

        if b.byte_length > options.max_buffer_bytes - total {
            gltf_free(*result);
            return .{}, failure(.LIMIT, "buffer budget exceeded", path);
        }

        data: string;
        ok := true;

        if !b.uri.count {
            if i != 0 || !bin.data || bin.count < b.byte_length || bin.count - b.byte_length > 3 {
                gltf_free(*result);
                return .{}, failure(.CONTAINER, "missing or incorrectly sized BIN", path);
            }

            data = copy_string(slice(bin, 0, b.byte_length));
        } else data, ok = resolve_resource(b.uri, resolver, options.max_buffer_bytes - total, false);

        if !ok || data.count < b.byte_length || data.count > options.max_buffer_bytes - total {
            free(data);
            gltf_free(*result);
            return .{}, failure(.RESOLVE, "buffer unavailable, short or over limit", path);
        }

        total += data.count;
        array_add(*result.buffers, data);
    }

    e := gltf_validate(*result, options);
    if e.kind != .NONE {
        gltf_free(*result);
        return .{}, e;
    }

    e = validate_images(*result, resolver, options, total);
    if e.kind != .NONE {
        gltf_free(*result);
        return .{}, e;
    }

    return result, .{};
}

// Release a loaded asset. Example: gltf_free(*asset);
gltf_free :: (asset: *Gltf_Asset) {
    for asset.buffers free(it);
    array_free(asset.buffers);
    free_json(*asset.document);
    asset.* = .{};
}

// Release an error's owned strings. Example: gltf_error_free(*error);
gltf_error_free :: (error: *Gltf_Error) {
    free(error.message);
    free(error.path);
    error.* = .{};
}

#load "types.jai";
#load "schema.jai";
#load "validate.jai";
#load "accessor.jai";
#load "write.jai";
#load "local.jai";

#scope_module
failure :: (kind: Gltf_Error_Kind, message: string, path: string, offset: s64 = 0) -> Gltf_Error {
    return .{ kind, copy_string(message), offset, copy_string(path) };
}

word :: (bytes: string, offset: s64, size: s64) -> u32 {
    result: u32;
    for i: 0..size - 1 result |= (cast(u32) bytes[offset + i]) << (cast(u32) (8 * i));
    return result;
}

reference :: (value: Json_Maybe(u32)) -> s64 {
    if value.state == .SET return value.value;
    return -1;
}

#import "Basic";
#import "Math";
#import "String";
#import "Hash_Table";
#import "Extensions/Json";
#import "Base64";

// One bounded URI path for both buffer and image resources.
resolve_resource :: (uri: string, resolver: Gltf_Resolver, limit: s64, image: bool) -> string, bool {
    if begins_with(uri, "data:") {
        comma := find_index_from_left(uri, #char ",");
        if comma < 0 return "", false;

        header := slice(uri, 0, comma);
        valid := ifx image then (header == "data:image/png;base64" || header == "data:image/jpeg;base64") else (header == "data:application/octet-stream;base64" || header == "data:application/gltf-buffer;base64");
        encoded := slice(uri, comma + 1, uri.count - comma - 1);

        if !valid || encoded.count / 4 * 3 - 2 > limit return "", false;

        return base64_decode(encoded);
    }

    if resolver.resolve return resolver.resolve(resolver.user, uri, limit);

    return "", false;
}

validate_images :: (asset: *Gltf_Asset, resolver: Gltf_Resolver, options: Gltf_Options, total: s64) -> Gltf_Error {
    for image, i: asset.document.images {
        p := tprint("/images/%", i);
        data: string;
        owned := image.uri.count != 0;

        defer if owned free(data);

        if owned {
            ok: bool;
            data, ok = resolve_resource(image.uri, resolver, options.max_buffer_bytes - total, true);
            if !ok || data.count > options.max_buffer_bytes - total return failure(.RESOLVE, "image unavailable or over limit", p);
            total += data.count;
        } else if image.buffer_view.state == .SET {
            view := asset.document.buffer_views[image.buffer_view.value];
            data = slice(asset.buffers[view.buffer], view.byte_offset, view.byte_length);
        } else continue; // Extension-only image sources remain opaque.

        format := Raster.detect_image_format(cast([] u8) data);
        mime := ifx format == .PNG then "image/png" else ifx format == .JPEG then "image/jpeg" else "";

        if !mime.count || (image.mime_type.count && image.mime_type != mime) || (begins_with(image.uri, "data:") && !begins_with(image.uri, tprint("data:%;", mime))) return failure(.INVALID, "image format and MIME type must agree", p);

        // Only the header is read, as the Khronos validator does; callers decode the pixels they use.
        _, ok, message := Raster.read_image_info(cast([] u8) data, min(Raster.DEFAULT_MAX_PIXELS, (options.max_buffer_bytes - total) / 4));

        if !ok return failure(.INVALID, message, p);
    }

    return .{};
}

Raster :: #import "Extensions/Image";
