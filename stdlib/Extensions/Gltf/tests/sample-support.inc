#import "Basic";
#import "String";
#import "File";
#import "File_Utilities";
#import "Extensions/Gltf";

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

    if e.kind != .NONE {
        print("FAIL %: % %\n", info.full_name, e.path, e.message);
        run.failures += 1;
    }
}
