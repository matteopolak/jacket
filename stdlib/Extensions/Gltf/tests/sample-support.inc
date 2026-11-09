#import "Basic";
#import "String";
#import "File";
#import "File_Utilities";
#import "Extensions/Gltf";
#import "Extensions/Json";

Sample_Run :: struct {
    count: s64;
    failures: s64;
    skipped: s64;
}

visit_sample :: (info: *File_Visit_Info, run: *Sample_Run) {
    if !ends_with(info.full_name, ".gltf") && !ends_with(info.full_name, ".glb") return;

    // EXT_texture_webp supplies the only image source; image extensions are not interpreted.
    if ends_with(info.full_name, "/SheenWoodLeatherSofa/glTF-Binary/SheenWoodLeatherSofa.glb") || ends_with(info.full_name, "/SheenWoodLeatherSofa/glTF/SheenWoodLeatherSofa.gltf") {
        run.skipped += 1;
        return;
    }

    // KHR_mesh_quantization changes the core attribute component rules.
    if ends_with(info.full_name, "/MeshoptCubeTest/glTF/MeshoptCubeTest.gltf") {
        run.skipped += 1;
        return;
    }

    bytes, ok := read_entire_file(info.full_name);

    assert(ok);
    defer free(bytes);
    slash := find_index_from_right(info.full_name, #char "/");
    folder := slice(info.full_name, 0, slash);
    a, e := gltf_load(bytes, .{ gltf_local_resolve, *folder });

    defer gltf_free(*a);
    defer gltf_error_free(*e);
    run.count += 1;

    if e.kind == .NONE for *material: a.document.materials check_material_extensions(*material.extensions, info.full_name, run);

    if e.kind != .NONE {
        print("FAIL %: % %\n", info.full_name, e.path, e.message);
        run.failures += 1;
    }
}

// Every typed material extension a sample uses decodes without error.
check_material_extensions :: (extensions: *Json_Value, file: string, run: *Sample_Run) {
    check_extension(extensions, "KHR_materials_emissive_strength", Gltf_Emissive_Strength, file, run);
    check_extension(extensions, "KHR_materials_ior", Gltf_Ior, file, run);
    check_extension(extensions, "KHR_materials_transmission", Gltf_Transmission, file, run);
    check_extension(extensions, "KHR_materials_clearcoat", Gltf_Clearcoat, file, run);
    check_extension(extensions, "KHR_materials_sheen", Gltf_Sheen, file, run);
    check_extension(extensions, "KHR_materials_specular", Gltf_Specular, file, run);
    check_extension(extensions, "KHR_materials_volume", Gltf_Volume, file, run);
    check_extension(extensions, "KHR_materials_iridescence", Gltf_Iridescence, file, run);
    check_extension(extensions, "KHR_materials_anisotropy", Gltf_Anisotropy, file, run);
    check_extension(extensions, "KHR_materials_unlit", Gltf_Unlit, file, run);
}

check_extension :: (extensions: *Json_Value, name: string, $T: Type, file: string, run: *Sample_Run) {
    value, found, e := gltf_extension(extensions, name, T);

    defer gltf_error_free(*e);

    if !found return;

    if e.kind != .NONE {
        print("FAIL %: % %\n", file, name, e.message);
        run.failures += 1;

        return;
    }

    free_json(*value);
}
