// Independent complete-read bridge over Android NDK asset ownership APIs.
read_entire_file :: (asset_manager: *AAssetManager, name: string, zero_terminated := false, log_errors := true) -> string, bool {
    if asset_manager == null return "", false;
    filename := Basic.to_c_string(name);
    if filename == null return "", false;
    defer Basic.free(filename);
    asset := AAssetManager_open(asset_manager, filename, .STREAMING);
    if asset == null {
        if log_errors Basic.log_error("Unable to open Android asset: %", name);
        return "", false;
    }
    defer AAsset_close(asset);
    length := AAsset_getLength64(asset);
    if length < 0 || length == 0x7fff_ffff_ffff_ffff return "", false;
    if length == 0 && !zero_terminated return "", true;
    capacity := length;
    if zero_terminated capacity += 1;
    bytes := cast(*u8) Basic.alloc(capacity);
    if bytes == null return "", false;
    offset: s64 = 0;
    while offset < length {
        count := length - offset;
        if count > 0x7fff_ffff count = 0x7fff_ffff;
        received := AAsset_read(asset, bytes + offset, cast(u64) count);
        if received <= 0 || received > count {
            Basic.free(bytes);
            if log_errors Basic.log_error("Incomplete read of Android asset: %", name);
            return "", false;
        }
        offset += received;
    }
    if zero_terminated bytes[length] = 0;
    return string.{length, bytes}, true;
}

#scope_file
Basic :: #import "Basic";
