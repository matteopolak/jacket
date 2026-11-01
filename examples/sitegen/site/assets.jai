// Assets are every file in the source folder that is not a page. Each is published under a name
// that contains a digest of its content, so a browser can cache it forever: `style.css` becomes
// `style.3f2a9c.css`, and changes name when (and only when) its bytes change. Small images can be
// inlined into the page as `data:` URIs instead.

// How many hexadecimal digits of the MD5 digest go into the name.
HASH_LENGTH :: 6;

Asset :: struct {
    // Where it came from, relative to the source folder: `img/logo.png`.
    source: string;
    // Where it goes, relative to the output folder: `img/logo.0a1b2c.png`.
    output: string;
    mime: string;
    data: string;
}

Assets :: struct {
    items: [..] Asset;
    // Position in `items` by source path.
    index: Table(string, s64);
}

// The content-hashed form of `path`: ("css/site.css", data) is "css/site.<digest>.css". A name
// without an extension just gets the digest appended. In temporary storage.
hashed_name :: (path: string, data: string) -> string {
    digest := md5(data);

    defer free(digest);

    short := slice(digest, 0, HASH_LENGTH);
    extension, _ := path_extension(path);

    if extension.count == 0 return tprint("%.%", path, short);

    return tprint("%.%.%", path_strip_extension(path), short, extension);
}

// The MIME type for a file name, from its extension.
mime_type :: (path: string) -> string {
    extension, _ := path_extension(path);
    extension = to_lower_copy(extension,, temp);

    if extension == {
        case "css"; return "text/css";
        case "js"; return "text/javascript";
        case "json"; return "application/json";
        case "txt"; return "text/plain";
        case "svg"; return "image/svg+xml";
        case "png"; return "image/png";
        case "jpg"; return "image/jpeg";
        case "jpeg"; return "image/jpeg";
        case "gif"; return "image/gif";
        case "webp"; return "image/webp";
        case "ico"; return "image/x-icon";
        case "woff2"; return "font/woff2";
        case "pdf"; return "application/pdf";
        case; return "application/octet-stream";
    }
}

// Adds a file to the set. The asset owns copies of both strings.
add_asset :: (assets: *Assets, source: string, data: string) {
    asset := Asset.{
        source = copy_string(source),
        output = copy_string(hashed_name(source, data)),
        mime = mime_type(source),
        data = copy_string(data),
    };

    table_set(*assets.index, asset.source, assets.items.count);
    array_add(*assets.items, asset);
}

// Frees the assets and what they own.
free_assets :: (assets: *Assets) {
    for assets.items {
        free(it.source);
        free(it.output);
        free(it.data);
    }

    array_free(assets.items);
    deinit(*assets.index);
    assets.* = .{};
}

// The asset that came from `source`, or null. The pointer is good until the next `add_asset`.
find_asset :: (assets: *Assets, source: string) -> *Asset {
    found, position := table_find(*assets.index, source);

    if !found return null;

    return *assets.items[position];
}

// Whether the asset is an image small enough to be written into a page: at most `limit` bytes.
// Vector images count too, so an SVG icon can be inlined.
is_inlinable :: (asset: *Asset, limit: s64) -> bool {
    return begins_with(asset.mime, "image/") && asset.data.count <= limit;
}

// "data:image/png;base64,iVBOR...", in temporary storage.
data_uri :: (asset: *Asset) -> string {
    encoded := base64_encode(asset.data);

    defer free(encoded);

    return tprint("data:%;base64,%", asset.mime, encoded);
}
