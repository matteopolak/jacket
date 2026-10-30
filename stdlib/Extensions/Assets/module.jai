// Load a file a program ships with (a texture, a sound, a level) by a relative path, from
// disk natively and over fetch() in the browser playground. A jaic extension, not official Jai.
// See docs/extensions/assets.md.
//
//     #import "Extensions/Assets";
//
//     bytes, ok := load_asset("assets/texture.png");
//     defer array_free(bytes);
//
// Natively the path is tried as given (from the working directory), then next to the executable.
// In the browser the page answers `jai_asset_fetch` by fetching the path relative to the asset
// base URL it was set up with; without that host function (a page with no asset base, a WASI
// command) the path is read from the program's file system instead. The bytes are allocated with
// context.allocator.

load_asset :: (path: string) -> bytes: [] u8, ok: bool {
    if !asset_path_is_relative(path) return .[], false;

    #if OS == .WASM {
        name :: "jai_asset_fetch";
        if jai_host_provides(name.data, cast(u64) name.count) {
            handle := jai_asset_fetch(path.data, cast(u64) path.count);
            if handle <= 0 return .[], false;

            size := jai_asset_size(handle);
            bytes := NewArray(size, u8, initialized = false);

            if jai_asset_take(handle, bytes.data, cast(u64) bytes.count) == 0 {
                array_free(bytes);
                return .[], false;
            }

            return bytes, true;
        }
    }

    text, ok := read_entire_file(path, log_errors = false);

    #if OS != .WASM {
        if !ok {
            directory := path_strip_filename(get_path_of_running_executable());
            if directory text, ok = read_entire_file(tprint("%1%2", directory, path), log_errors = false);
        }
    }

    if !ok return .[], false;

    return cast([] u8) text, true;
}

// A relative path that stays inside the asset folder: no leading '/', no drive or URL scheme, no
// '..' component and no backslashes. load_asset refuses anything else on every platform, so a
// program behaves the same natively and in the browser.
asset_path_is_relative :: (path: string) -> bool {
    if !path || path[0] == #char "/" return false;

    start := 0;

    for i: 0..path.count {
        if i < path.count {
            c := path[i];
            if c == #char "\\" || c == #char ":" || c == 0 return false;
            if c != #char "/" continue;
        }

        part := string.{ i - start, path.data + start };
        if !part || part == ".." return false;

        start = i + 1;
    }

    return true;
}

#scope_file
#import "Basic";
#import "File";
#import "String";

#if OS != .WASM {
    #import "System";
}

#if OS == .WASM {
    // Host procedures (no library: the page answers them by name).
    jai_host_provides :: (name: *u8, count: u64) -> u32 #foreign;
    // Fetch `path` relative to the page's asset base. A positive handle on success; 0 or less
    // when it is refused, missing or too large. Suspends the program while the page waits.
    jai_asset_fetch :: (path: *u8, count: u64) -> s64 #foreign;
    // The byte count of a fetched asset.
    jai_asset_size :: (handle: s64) -> s64 #foreign;
    // Copy a fetched asset into `buffer` (exactly its size) and forget it. 1 on success.
    jai_asset_take :: (handle: s64, buffer: *u8, count: u64) -> u32 #foreign;
}
