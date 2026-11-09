// Decode one named extension object, such as `KHR_materials_ior`, from the `extensions` of a document,
// node, material or texture reference into a typed struct. The result is true when the extension is
// present; an object of the wrong shape gives a `.JSON` error and `found` is still true. Extensions
// that no struct covers stay raw in `extensions`. Release the value with `free_json` (needed only for
// types with pointers or extras, but always allowed) and the error with [gltf_error_free].
//
// ```jai
// asset, error := gltf_load("{\"asset\":{\"version\":\"2.0\"},\"extensionsUsed\":[\"KHR_materials_ior\"],\"materials\":[{\"extensions\":{\"KHR_materials_ior\":{\"ior\":1.33}}},{}]}");
// assert(error.kind == .NONE);
// defer gltf_free(*asset);
// ior, found, ior_error := gltf_extension(*asset.document.materials[0].extensions, "KHR_materials_ior", Gltf_Ior);
// assert(found && ior_error.kind == .NONE && ior.ior == 1.33);
// _, found = gltf_extension(*asset.document.materials[1].extensions, "KHR_materials_ior", Gltf_Ior);
// assert(!found);
// ```
gltf_extension :: (extensions: *Json_Value, name: string, $T: Type) -> T, bool, Gltf_Error {
    empty: T;
    if extensions.kind != .OBJECT return empty, false, .{};

    member := json_get(extensions, name);
    if !member return empty, false, .{};

    value, decode := from_json(T, member);

    if decode.kind != .NONE {
        e := failure(.JSON, decode.message, json_path_text(decode), decode.offset);
        free_json(*decode);
        return empty, true, e;
    }

    return value, true, .{};
}

// KHR_materials_emissive_strength: scales `emissive_factor` beyond 1. Read it from `material.extensions`.
//
// ```jai
// asset, error := gltf_load("{\"asset\":{\"version\":\"2.0\"},\"extensionsUsed\":[\"KHR_materials_emissive_strength\"],\"materials\":[{\"extensions\":{\"KHR_materials_emissive_strength\":{\"emissiveStrength\":4.5}}}]}");
// assert(error.kind == .NONE);
// defer gltf_free(*asset);
// strength, found, _ := gltf_extension(*asset.document.materials[0].extensions, "KHR_materials_emissive_strength", Gltf_Emissive_Strength);
// assert(found && strength.emissive_strength == 4.5);
// ```
Gltf_Emissive_Strength :: struct @json(rename_all=camelCase) {
    emissive_strength: float32 = 1.0; @json(default)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// KHR_materials_ior: the index of refraction of a dielectric surface. Read it from `material.extensions`.
//
// ```jai
// asset, error := gltf_load("{\"asset\":{\"version\":\"2.0\"},\"extensionsUsed\":[\"KHR_materials_ior\"],\"materials\":[{\"extensions\":{\"KHR_materials_ior\":{\"ior\":1.33}}}]}");
// assert(error.kind == .NONE);
// defer gltf_free(*asset);
// ior, found, _ := gltf_extension(*asset.document.materials[0].extensions, "KHR_materials_ior", Gltf_Ior);
// assert(found && ior.ior == 1.33);
// ```
Gltf_Ior :: struct @json(rename_all=camelCase) {
    ior: float32 = 1.5; @json(default)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// KHR_materials_transmission: how much light passes through a surface, with an optional texture. The
// texture is allocated: release the value with `free_json`.
//
// ```jai
// #import "Extensions/Json";
//
// asset, error := gltf_load("{\"asset\":{\"version\":\"2.0\"},\"extensionsUsed\":[\"KHR_materials_transmission\"],\"materials\":[{\"extensions\":{\"KHR_materials_transmission\":{\"transmissionFactor\":0.25}}}]}");
// assert(error.kind == .NONE);
// defer gltf_free(*asset);
// transmission, found, _ := gltf_extension(*asset.document.materials[0].extensions, "KHR_materials_transmission", Gltf_Transmission);
// assert(found && transmission.transmission_factor == 0.25 && transmission.transmission_texture == null);
// free_json(*transmission);
// ```
Gltf_Transmission :: struct @json(rename_all=camelCase) {
    transmission_factor: float32 = 0.0; @json(default)
    transmission_texture: *Gltf_Texture_Info; @json(default,skip_serializing_if=null)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// KHR_materials_clearcoat: a glossy layer over the base material, with its own roughness and normal
// map. The textures are allocated: release the value with `free_json`.
//
// ```jai
// #import "Extensions/Json";
//
// asset, error := gltf_load("{\"asset\":{\"version\":\"2.0\"},\"extensionsUsed\":[\"KHR_materials_clearcoat\"],\"materials\":[{\"extensions\":{\"KHR_materials_clearcoat\":{\"clearcoatFactor\":0.8,\"clearcoatRoughnessFactor\":0.1}}}]}");
// assert(error.kind == .NONE);
// defer gltf_free(*asset);
// coat, found, _ := gltf_extension(*asset.document.materials[0].extensions, "KHR_materials_clearcoat", Gltf_Clearcoat);
// assert(found && coat.clearcoat_factor == 0.8 && coat.clearcoat_roughness_factor == 0.1);
// free_json(*coat);
// ```
Gltf_Clearcoat :: struct @json(rename_all=camelCase) {
    clearcoat_factor: float32 = 0.0; @json(default)
    clearcoat_texture: *Gltf_Texture_Info; @json(default,skip_serializing_if=null)
    clearcoat_roughness_factor: float32 = 0.0; @json(default)
    clearcoat_roughness_texture: *Gltf_Texture_Info; @json(default,skip_serializing_if=null)
    clearcoat_normal_texture: *Gltf_Texture_Info; @json(default,skip_serializing_if=null)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// KHR_texture_transform: moves, rotates and scales the UVs of one texture reference. Read it from the
// reference's `extensions`; a set `tex_coord` replaces the reference's own UV set.
//
// ```jai
// asset, error := gltf_load("{\"asset\":{\"version\":\"2.0\"},\"extensionsUsed\":[\"KHR_texture_transform\"],\"images\":[{\"uri\":\"data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==\"}],\"textures\":[{\"source\":0}],\"materials\":[{\"emissiveTexture\":{\"index\":0,\"extensions\":{\"KHR_texture_transform\":{\"offset\":[0.5,0.25],\"scale\":[2,2]}}}}]}");
// assert(error.kind == .NONE);
// defer gltf_free(*asset);
// reference := asset.document.materials[0].emissive_texture;
// transform, found, _ := gltf_extension(*reference.extensions, "KHR_texture_transform", Gltf_Texture_Transform);
// assert(found && transform.offset[0] == 0.5 && transform.scale[1] == 2 && transform.rotation == 0);
// ```
Gltf_Texture_Transform :: struct @json(rename_all=camelCase) {
    offset: [2] float32 = .[0, 0]; @json(default)
    rotation: float32 = 0.0; @json(default)
    scale: [2] float32 = .[1, 1]; @json(default)
    tex_coord: Json_Maybe(u32);
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// The kind of a KHR_lights_punctual light.
//
// ```jai
// #import "Extensions/Json";
//
// asset, error := gltf_load("{\"asset\":{\"version\":\"2.0\"},\"extensionsUsed\":[\"KHR_lights_punctual\"],\"extensions\":{\"KHR_lights_punctual\":{\"lights\":[{\"type\":\"point\"}]}}}");
// assert(error.kind == .NONE);
// defer gltf_free(*asset);
// lights, found, _ := gltf_extension(*asset.document.extensions, "KHR_lights_punctual", Gltf_Lights);
// assert(found && lights.lights[0].type == .POINT);
// free_json(*lights);
// ```
Gltf_Light_Type :: enum u8 {
    DIRECTIONAL;
    POINT;
    SPOT;
}

// The cone of a spot light, in radians.
//
// ```jai
// spot := Gltf_Light_Spot.{};
// assert(spot.inner_cone_angle == 0 && spot.outer_cone_angle > 0.78 && spot.outer_cone_angle < 0.79);
// ```
Gltf_Light_Spot :: struct @json(rename_all=camelCase) {
    inner_cone_angle: float32 = 0.0; @json(default)
    outer_cone_angle: float32 = cast(float32) 0.7853981633974483; @json(default)
}

// One light of KHR_lights_punctual. A `range` of zero means unlimited, as the glTF default of no
// range; `spot` is set only for spot lights.
//
// ```jai
// #import "Extensions/Json";
//
// asset, error := gltf_load("{\"asset\":{\"version\":\"2.0\"},\"extensionsUsed\":[\"KHR_lights_punctual\"],\"extensions\":{\"KHR_lights_punctual\":{\"lights\":[{\"type\":\"spot\",\"intensity\":3,\"spot\":{\"outerConeAngle\":0.5}}]}}}");
// assert(error.kind == .NONE);
// defer gltf_free(*asset);
// lights, found, _ := gltf_extension(*asset.document.extensions, "KHR_lights_punctual", Gltf_Lights);
// assert(found);
// light := lights.lights[0];
// assert(light.type == .SPOT && light.intensity == 3 && light.color[0] == 1 && light.spot.outer_cone_angle == 0.5);
// free_json(*lights);
// ```
Gltf_Light :: struct @json(rename_all=camelCase) {
    name: string; @json(default,skip_serializing_if=empty)
    color: [3] float32 = .[1, 1, 1]; @json(default)
    intensity: float32 = 1.0; @json(default)
    type: Gltf_Light_Type; @json(rename_all=lowercase)
    range: float32 = 0.0; @json(default)
    spot: *Gltf_Light_Spot; @json(default,skip_serializing_if=null)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// KHR_lights_punctual on the document: the list of lights that nodes refer to by index. Release it
// with `free_json`.
//
// ```jai
// #import "Extensions/Json";
//
// asset, error := gltf_load("{\"asset\":{\"version\":\"2.0\"},\"extensionsUsed\":[\"KHR_lights_punctual\"],\"extensions\":{\"KHR_lights_punctual\":{\"lights\":[{\"type\":\"directional\"},{\"type\":\"point\"}]}}}");
// assert(error.kind == .NONE);
// defer gltf_free(*asset);
// lights, found, _ := gltf_extension(*asset.document.extensions, "KHR_lights_punctual", Gltf_Lights);
// assert(found && lights.lights.count == 2);
// free_json(*lights);
// ```
Gltf_Lights :: struct @json(rename_all=camelCase) {
    lights: [] Gltf_Light; @json(default,skip_serializing_if=empty)
}

// KHR_lights_punctual on a node: the index of the light it carries, into [Gltf_Lights].
//
// ```jai
// asset, error := gltf_load("{\"asset\":{\"version\":\"2.0\"},\"extensionsUsed\":[\"KHR_lights_punctual\"],\"nodes\":[{\"extensions\":{\"KHR_lights_punctual\":{\"light\":1}}}]}");
// assert(error.kind == .NONE);
// defer gltf_free(*asset);
// carried, found, _ := gltf_extension(*asset.document.nodes[0].extensions, "KHR_lights_punctual", Gltf_Node_Light);
// assert(found && carried.light == 1);
// ```
Gltf_Node_Light :: struct @json(rename_all=camelCase) {
    light: u32;
    extras: Json_Value; @json(default,skip_serializing_if=null)
}
