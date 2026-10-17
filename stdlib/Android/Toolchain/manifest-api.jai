deinit :: (using manifest: *Android_Manifest_Binary) {
    for s: string_pool free(s);

    free(string_pool.data);
    free(resource_ids.data);
    for e: elements free(e.attributes.data);

    array_free(elements);
    if free_data_on_deinit free(raw_data);

    memset(manifest, 0, size_of(Android_Manifest_Binary));
} @Releases

read_binary_manifest_from_memory :: (data: string, filename := "<memory>") -> bool, Android_Manifest_Binary {
    result: Android_Manifest_Binary;
    result.raw_data = data;
    if !am_read(*result) {
        deinit(*result);
        if __debug_manifest_reading print("Invalid binary manifest: %\n", filename);
        return false, .{};
    }

    return true, result;
}

read_binary_manifest_from_file :: (filename: string) -> bool, Android_Manifest_Binary {
    data, loaded := Manifest_File.read_entire_file(filename);
    if !loaded return false, .{};

    ok, result := read_binary_manifest_from_memory(data, filename);
    if !ok {
        free(data);
        return false, .{};
    }

    result.free_data_on_deinit = true;

    return true, result;
}

read_binary_manifest_from_apk :: (apk_filename: string) -> bool, Android_Manifest_Binary {
    apk := apk_load(apk_filename);

    defer apk_deinit(*apk);

    for entry: apk {
        if entry.filename != "AndroidManifest.xml" continue;
        if entry.offset > apk.data.count || entry.compressed_size > apk.data.count - entry.offset return false, .{};

        compressed := string.{ entry.compressed_size, apk.data.data + entry.offset };
        data: string;

        if entry.compression_method == .NONE {
            if entry.compressed_size != entry.uncompressed_size return false, .{};
            data = copy_string(compressed);
        } else if entry.compression_method == .DEFLATE {
            decoded: bool;
            decoded, data = am_inflate(compressed, entry.uncompressed_size);
            if !decoded return false, .{};
        } else return false, .{};

        if zip_crc32(data) != apk.zip_dir.entries[it_index].crc {
            free(data);
            return false, .{};
        }

        ok, result := read_binary_manifest_from_memory(data, apk_filename);
        if !ok {
            free(data);
            return false, .{};
        }

        result.free_data_on_deinit = true;

        return true, result;
    }

    return false, .{};
}

write_binary_app_manifest_to_memory :: (config: Android_Project_Config) -> bool, string {
    b: String_Builder;
    defer free_buffers(*b);
    if !write_binary_app_manifest_to_builder(*b, config) return false, "";
    return true, builder_to_string(*b);
}

write_binary_app_manifest_to_file :: (using config: Android_Project_Config, out_filepath: string) -> bool {
    ok, data := write_binary_app_manifest_to_memory(config);
    if !ok return false;
    defer free(data);
    return Manifest_File.write_entire_file(out_filepath, data);
}

write_binary_app_manifest_to_builder :: (b: *String_Builder, config: Android_Project_Config) -> bool {
    return am_write(b, config);
}

print_binary_manifest :: (using manifest: Android_Manifest_Binary, print_strings := false, print_resource_ids := false) {
    if print_strings for s: string_pool print("%: %\n", it_index, s);
    if print_resource_ids for id: resource_ids print("%: %\n", it_index, id);

    depth := 0;

    for e: elements {
        if e.kind == .END depth -= 1;
        for 0..depth - 1 print("  ");

        if e.kind == .END print("</%>\n", e.name);
        else {
            print("<%", e.name);

            for a: e.attributes {
                print(" %", string_pool[a.name.index]);
                if a.rawValue.index != DUMMY_U32 print("=\"%\"", string_pool[a.rawValue.index]);
                else print("=\"%\"", a.typedValue.data);
            }

            print(">\n");
            depth += 1;
        }
    }
}

#load "../../internal/android-toolchain/android-manifest.jai";
#load "../../internal/android-toolchain/android-manifest-writer.jai";
#load "../../internal/android-toolchain/android-manifest-inflate.jai";
#scope_file
Manifest_File :: #import "File";
#import "Basic";
