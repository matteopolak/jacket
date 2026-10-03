#scope_module
android_device_serial_storage: string;
android_device_serial_allocator: Allocator;

android_device_retain_serial :: (serial: string) {
    if android_device_serial_storage.data {
        push_allocator(android_device_serial_allocator);
        free(android_device_serial_storage);
    }

    android_device_serial_allocator = context.default_allocator;

    {
        push_allocator(android_device_serial_allocator);
        android_device_serial_storage = copy_string(serial);
    }

    adb_device_serial = android_device_serial_storage;
}

android_device_package_valid :: (package: string) -> bool {
    if package.count == 0 return false;

    component_start := true;

    for byte: package {
        if byte == #char "." {
            if component_start return false;
            component_start = true;
            continue;
        }

        letter := (byte >= #char "a" && byte <= #char "z") || (byte >= #char "A" && byte <= #char "Z") || byte == #char "_";
        digit := byte >= #char "0" && byte <= #char "9";

        if !letter && (!digit || component_start) return false;

        component_start = false;
    }

    return !component_start;
}

android_device_relative_valid :: (path: string) -> bool {
    if path.count == 0 return true;
    if path[0] == #char "/" || path[0] == #char "\\" return false;

    cursor := 0;

    for end: 0..path.count {
        if end < path.count && path[end] != #char "/" && path[end] != #char "\\" {
            if path[end] == 0 || path[end] == #char ":" return false;
            continue;
        }

        component := string.{ end - cursor, path.data + cursor };
        if component == ".." return false;

        cursor = end + 1;
    }

    return true;
}

android_device_manifest_package :: (manifest: Android_Manifest_Binary) -> string {
    for element: manifest.elements {
        if element.kind != .START || element.name != "manifest" continue;

        for attribute: element.attributes {
            if attribute.name.index >= cast(u32) manifest.string_pool.count continue;
            if manifest.string_pool[attribute.name.index] != "package" continue;

            index := attribute.rawValue.index;
            if index >= cast(u32) manifest.string_pool.count index = attribute.typedValue.data;

            if index < cast(u32) manifest.string_pool.count {
                package := manifest.string_pool[index];
                if android_device_package_valid(package) return package;
            }
        }
    }

    return "";
}

#scope_file
#import "Basic";
