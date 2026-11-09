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

// Store a typed extension in the `extensions` of a document, node, material or texture reference,
// replacing the extension of that name: the write-back of [gltf_extension]. An `extensions` that is
// absent becomes an object. The value is encoded straight away, so it may be freed or changed
// afterwards. An `extensions` that is not an object gives an error and is left
// unchanged. Declare the extension with [gltf_use_extension], or the written
// asset will not load.
//
// ```jai
// asset, error := gltf_load("{\"asset\":{\"version\":\"2.0\"},\"extensionsUsed\":[\"KHR_materials_ior\"],\"materials\":[{\"extensions\":{\"KHR_materials_ior\":{\"ior\":1.33}}}]}");
// assert(error.kind == .NONE);
// defer gltf_free(*asset);
// extensions := *asset.document.materials[0].extensions;
// ior, _, _ := gltf_extension(extensions, "KHR_materials_ior", Gltf_Ior);
// ior.ior = 2.4;
// set_error := gltf_set_extension(extensions, "KHR_materials_ior", ior);
// assert(set_error.kind == .NONE);
// again, _, _ := gltf_extension(extensions, "KHR_materials_ior", Gltf_Ior);
// assert(again.ior == 2.4);
// ```
gltf_set_extension :: (extensions: *Json_Value, name: string, value: $T) -> Gltf_Error {
    if extensions.kind != .NULL && extensions.kind != .OBJECT return failure(.INVALID, "extensions must be an object", "");

    if extensions.kind == .NULL extensions.* = json_object();

    json_set(extensions, name, to_json_value(value));

    return .{};
}

// Add `name` to the `extensionsUsed` of a loaded asset unless it is there. A written asset must
// list every extension its objects use; do this before [gltf_set_extension] adds a new one.
//
// ```jai
// asset, error := gltf_load("{\"asset\":{\"version\":\"2.0\"},\"materials\":[{}]}");
// assert(error.kind == .NONE);
// defer gltf_free(*asset);
// gltf_use_extension(*asset, "KHR_materials_ior");
// gltf_use_extension(*asset, "KHR_materials_ior");
// assert(asset.document.extensions_used.count == 1);
// ```
gltf_use_extension :: (asset: *Gltf_Asset, name: string) {
    used := *asset.document.extensions_used;

    for used.* if it == name return;

    list: [..] string;

    array_reserve(*list, used.count + 1);

    for used.* array_add(*list, it);

    array_add(*list, copy_string(name));

    free(used.data);
    used.data = list.data;
    used.count = list.count;
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

// KHR_materials_sheen: a fabric-like layer of fuzz on the surface. Read it from `material.extensions`.
// The textures are allocated: release the value with `free_json`.
//
// ```jai
// #import "Extensions/Json";
//
// asset, error := gltf_load("{\"asset\":{\"version\":\"2.0\"},\"extensionsUsed\":[\"KHR_materials_sheen\"],\"materials\":[{\"extensions\":{\"KHR_materials_sheen\":{\"sheenColorFactor\":[0.9,0.5,0.1],\"sheenRoughnessFactor\":0.3}}}]}");
// assert(error.kind == .NONE);
// defer gltf_free(*asset);
// sheen, found, _ := gltf_extension(*asset.document.materials[0].extensions, "KHR_materials_sheen", Gltf_Sheen);
// assert(found && sheen.sheen_color_factor[1] == 0.5 && sheen.sheen_roughness_factor == 0.3);
// free_json(*sheen);
// ```
Gltf_Sheen :: struct @json(rename_all=camelCase) {
    sheen_color_factor: [3] float32 = .[0, 0, 0]; @json(default)
    sheen_color_texture: *Gltf_Texture_Info; @json(default,skip_serializing_if=null)
    sheen_roughness_factor: float32 = 0.0; @json(default)
    sheen_roughness_texture: *Gltf_Texture_Info; @json(default,skip_serializing_if=null)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// KHR_materials_specular: the strength and tint of the specular reflection of a dielectric. Read it
// from `material.extensions`. The textures are allocated: release the value with `free_json`.
//
// ```jai
// #import "Extensions/Json";
//
// asset, error := gltf_load("{\"asset\":{\"version\":\"2.0\"},\"extensionsUsed\":[\"KHR_materials_specular\"],\"materials\":[{\"extensions\":{\"KHR_materials_specular\":{\"specularColorFactor\":[1,0.8,0.6]}}}]}");
// assert(error.kind == .NONE);
// defer gltf_free(*asset);
// specular, found, _ := gltf_extension(*asset.document.materials[0].extensions, "KHR_materials_specular", Gltf_Specular);
// assert(found && specular.specular_factor == 1 && specular.specular_color_factor[2] == 0.6);
// free_json(*specular);
// ```
Gltf_Specular :: struct @json(rename_all=camelCase) {
    specular_factor: float32 = 1.0; @json(default)
    specular_texture: *Gltf_Texture_Info; @json(default,skip_serializing_if=null)
    specular_color_factor: [3] float32 = .[1, 1, 1]; @json(default)
    specular_color_texture: *Gltf_Texture_Info; @json(default,skip_serializing_if=null)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// KHR_materials_volume: the thickness of a transmissive surface and how light fades inside it. Read it
// from `material.extensions`. `attenuation_distance` is unset when light does not fade (the glTF
// default is infinity, which JSON cannot write). The texture is allocated: release the value with
// `free_json`.
//
// ```jai
// #import "Extensions/Json";
//
// asset, error := gltf_load("{\"asset\":{\"version\":\"2.0\"},\"extensionsUsed\":[\"KHR_materials_volume\"],\"materials\":[{\"extensions\":{\"KHR_materials_volume\":{\"thicknessFactor\":0.5,\"attenuationDistance\":2,\"attenuationColor\":[0.2,0.9,0.4]}}}]}");
// assert(error.kind == .NONE);
// defer gltf_free(*asset);
// volume, found, _ := gltf_extension(*asset.document.materials[0].extensions, "KHR_materials_volume", Gltf_Volume);
// assert(found && volume.thickness_factor == 0.5 && volume.attenuation_distance.state == .SET && volume.attenuation_distance.value == 2);
// free_json(*volume);
// ```
Gltf_Volume :: struct @json(rename_all=camelCase) {
    thickness_factor: float32 = 0.0; @json(default)
    thickness_texture: *Gltf_Texture_Info; @json(default,skip_serializing_if=null)
    attenuation_distance: Json_Maybe(float32);
    attenuation_color: [3] float32 = .[1, 1, 1]; @json(default)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// KHR_materials_iridescence: a thin film that shifts colour with the viewing angle. Read it from
// `material.extensions`. The thickness is in nanometres. The textures are allocated: release the value
// with `free_json`.
//
// ```jai
// #import "Extensions/Json";
//
// asset, error := gltf_load("{\"asset\":{\"version\":\"2.0\"},\"extensionsUsed\":[\"KHR_materials_iridescence\"],\"materials\":[{\"extensions\":{\"KHR_materials_iridescence\":{\"iridescenceFactor\":1,\"iridescenceThicknessMaximum\":800}}}]}");
// assert(error.kind == .NONE);
// defer gltf_free(*asset);
// film, found, _ := gltf_extension(*asset.document.materials[0].extensions, "KHR_materials_iridescence", Gltf_Iridescence);
// assert(found && film.iridescence_factor == 1 && film.iridescence_ior == 1.3 && film.iridescence_thickness_minimum == 100 && film.iridescence_thickness_maximum == 800);
// free_json(*film);
// ```
Gltf_Iridescence :: struct @json(rename_all=camelCase) {
    iridescence_factor: float32 = 0.0; @json(default)
    iridescence_texture: *Gltf_Texture_Info; @json(default,skip_serializing_if=null)
    iridescence_ior: float32 = 1.3; @json(default)
    iridescence_thickness_minimum: float32 = 100.0; @json(default)
    iridescence_thickness_maximum: float32 = 400.0; @json(default)
    iridescence_thickness_texture: *Gltf_Texture_Info; @json(default,skip_serializing_if=null)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// KHR_materials_anisotropy: highlights stretched along a direction, as on brushed metal. Read it from
// `material.extensions`. `anisotropy_rotation` is in radians. The texture is allocated: release the
// value with `free_json`.
//
// ```jai
// #import "Extensions/Json";
//
// asset, error := gltf_load("{\"asset\":{\"version\":\"2.0\"},\"extensionsUsed\":[\"KHR_materials_anisotropy\"],\"materials\":[{\"extensions\":{\"KHR_materials_anisotropy\":{\"anisotropyStrength\":0.6,\"anisotropyRotation\":1.5}}}]}");
// assert(error.kind == .NONE);
// defer gltf_free(*asset);
// stretch, found, _ := gltf_extension(*asset.document.materials[0].extensions, "KHR_materials_anisotropy", Gltf_Anisotropy);
// assert(found && stretch.anisotropy_strength == 0.6 && stretch.anisotropy_rotation == 1.5);
// free_json(*stretch);
// ```
Gltf_Anisotropy :: struct @json(rename_all=camelCase) {
    anisotropy_strength: float32 = 0.0; @json(default)
    anisotropy_rotation: float32 = 0.0; @json(default)
    anisotropy_texture: *Gltf_Texture_Info; @json(default,skip_serializing_if=null)
    extras: Json_Value; @json(default,skip_serializing_if=null)
}

// KHR_materials_unlit: the material ignores lighting and shows its base colour. The extension has no
// properties, so finding it is the information.
//
// ```jai
// asset, error := gltf_load("{\"asset\":{\"version\":\"2.0\"},\"extensionsUsed\":[\"KHR_materials_unlit\"],\"materials\":[{\"extensions\":{\"KHR_materials_unlit\":{}}},{}]}");
// assert(error.kind == .NONE);
// defer gltf_free(*asset);
// _, flat := gltf_extension(*asset.document.materials[0].extensions, "KHR_materials_unlit", Gltf_Unlit);
// _, lit := gltf_extension(*asset.document.materials[1].extensions, "KHR_materials_unlit", Gltf_Unlit);
// assert(flat && !lit);
// ```
Gltf_Unlit :: struct @json(rename_all=camelCase) {
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
