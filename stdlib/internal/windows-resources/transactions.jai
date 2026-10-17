// Native SDK mechanics only. Public ID/language policy awaits contracts.
write_icon_resources :: (filename: string, icon: string, group_id: u16, first_image_id: u16, language: u16, delete_existing: bool) -> bool {
    valid, directory := Icons.inspect(icon);
    if !valid return diagnostic(filename, "invalid ICO directory");

    defer Icons.release(*directory);
    group_ok, group := Icons.make_group(directory, first_image_id);
    if !group_ok return diagnostic(filename, "icon resource ID range or allocation failed");

    defer B.free(group);

    // ICO frame offsets need not be DWORD aligned. Keep aligned SDK payload copies
    // alive through EndUpdateResource, independently of the caller's borrowed file.
    payloads := ([] string).{ directory.images.count, cast(*string) B.alloc(directory.images.count * size_of(string)) };
    if !payloads.data return diagnostic(filename, "icon payload allocation failed");

    memset(payloads.data, 0, payloads.count * size_of(string));

    defer {
        for payload: payloads B.free(payload);
        B.array_free(payloads);
    }

    for image, index: directory.images {
        payloads[index] = B.alloc_string(image.bytes.count);
        if !payloads[index].data return diagnostic(filename, "icon payload allocation failed");
        memcpy(payloads[index].data, image.bytes.data, image.bytes.count);
    }

    path := Paths.to_utf16(filename);
    if !path return diagnostic(filename, "invalid UTF-8 executable path or allocation failed");

    defer B.free(path);
    delete_flag := W.BOOL.FALSE;
    if delete_existing delete_flag = .TRUE;

    transaction := BeginUpdateResourceW(path, delete_flag);
    if !transaction return native_error(filename, "BeginUpdateResourceW");

    for payload, index: payloads {
        if UpdateResourceW(transaction, resource_id(3), resource_id(cast(u16) (first_image_id + index)), language, payload.data, cast(u32) payload.count) == .FALSE {
            native_error(filename, "UpdateResourceW RT_ICON");
            discard(transaction, filename);
            return false;
        }
    }

    if UpdateResourceW(transaction, resource_id(14), resource_id(group_id), language, group.data, cast(u32) group.count) == .FALSE {
        native_error(filename, "UpdateResourceW RT_GROUP_ICON");
        discard(transaction, filename);
        return false;
    }

    if EndUpdateResourceW(transaction, .FALSE) == .FALSE return native_error(filename, "EndUpdateResourceW commit");

    return true;
}

write_manifest_resource :: (filename: string, manifest: string, id: u16, language: u16) -> bool {
    if manifest.count <= 0 || manifest.count > 0xffffffff || !manifest.data return diagnostic(filename, "invalid manifest extent");

    path := Paths.to_utf16(filename);
    if !path return diagnostic(filename, "invalid UTF-8 executable path or allocation failed");

    defer B.free(path);
    transaction := BeginUpdateResourceW(path, .FALSE);
    if !transaction return native_error(filename, "BeginUpdateResourceW");

    if UpdateResourceW(transaction, resource_id(24), resource_id(id), language, manifest.data, cast(u32) manifest.count) == .FALSE {
        native_error(filename, "UpdateResourceW RT_MANIFEST");
        discard(transaction, filename);
        return false;
    }

    if EndUpdateResourceW(transaction, .FALSE) == .FALSE return native_error(filename, "EndUpdateResourceW commit");

    return true;
}

#scope_file
kernel :: #library,system "kernel32";
BeginUpdateResourceW :: (filename: W.LPCWSTR, delete_existing: W.BOOL) -> W.HANDLE #foreign kernel;
UpdateResourceW :: (transaction: W.HANDLE, resource_type: W.LPCWSTR, name: W.LPCWSTR, language: W.WORD, data: *void, byte_count: W.DWORD) -> W.BOOL #foreign kernel;
EndUpdateResourceW :: (transaction: W.HANDLE, discard_changes: W.BOOL) -> W.BOOL #foreign kernel;

resource_id :: (id: u16) -> W.LPCWSTR {
    return cast(W.LPCWSTR) cast(u64) id;
}

diagnostic :: (filename: string, message: string) -> bool {
    B.log_error("Resource update '%': %", filename, message);
    return false;
}

native_error :: (filename: string, operation: string) -> bool {
    code := W.GetLastError();
    B.log_error("Resource update '%': % failed (Win32 error %)", filename, operation, code);
    return false;
}

discard :: (transaction: W.HANDLE, filename: string) {
    if EndUpdateResourceW(transaction, .TRUE) == .FALSE native_error(filename, "EndUpdateResourceW discard");
}
