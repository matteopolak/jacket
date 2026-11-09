// Decode typed extensions of a document with many materials.
// Example: jacket bench --path stdlib/Extensions/Gltf
#import "Basic";
#import "Extensions/Testing";
#import "Extensions/Json";
#import "Extensions/Gltf";

MATERIALS :: 2000;

many_materials :: () -> string {
    builder: String_Builder;

    append(*builder, "{\"asset\":{\"version\":\"2.0\"},\"extensionsUsed\":[\"KHR_materials_ior\",\"KHR_materials_emissive_strength\",\"KHR_materials_clearcoat\"],\"materials\":[");

    for 0..MATERIALS - 1 {
        if it append(*builder, ",");
        print(*builder, "{\"extensions\":{\"KHR_materials_ior\":{\"ior\":1.%},\"KHR_materials_emissive_strength\":{\"emissiveStrength\":%.5},\"KHR_materials_clearcoat\":{\"clearcoatFactor\":0.8,\"clearcoatRoughnessFactor\":0.1}}}", it % 10, it % 7);
    }

    append(*builder, "]}");

    return builder_to_string(*builder);
}

typed_materials :: (b: *Bench) {
    text := many_materials();

    defer free(text);

    asset, error := gltf_load(text);

    assert(error.kind == .NONE);
    defer gltf_free(*asset);
    set_items(b, MATERIALS * 3);
    reset_timer(b);

    for 0..b.iterations - 1 {
        for *material: asset.document.materials {
            ior, _, e1 := gltf_extension(*material.extensions, "KHR_materials_ior", Gltf_Ior);
            strength, _, e2 := gltf_extension(*material.extensions, "KHR_materials_emissive_strength", Gltf_Emissive_Strength);
            coat, _, e3 := gltf_extension(*material.extensions, "KHR_materials_clearcoat", Gltf_Clearcoat);

            black_box(ior);
            black_box(strength);
            black_box(coat);
            assert(e1.kind == .NONE && e2.kind == .NONE && e3.kind == .NONE);
            free_json(*coat);
        }
    }
} @bench
