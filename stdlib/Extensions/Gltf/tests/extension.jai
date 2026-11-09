#scope_file
#import "Math";
#import "Extensions/Json";
#import "Extensions/Testing";
#import "Extensions/Gltf";

#scope_export
EXTENSIONS :: "{\"asset\":{\"version\":\"2.0\"},\"extensionsUsed\":[\"KHR_materials_emissive_strength\",\"KHR_materials_ior\",\"KHR_materials_transmission\",\"KHR_materials_clearcoat\",\"KHR_texture_transform\",\"KHR_lights_punctual\",\"EXT_unknown\"],\"extensions\":{\"KHR_lights_punctual\":{\"lights\":[{\"type\":\"spot\",\"color\":[1,0.5,0],\"intensity\":3,\"range\":10,\"spot\":{\"outerConeAngle\":0.5}},{\"type\":\"directional\"}]}},\"scene\":0,\"scenes\":[{\"nodes\":[0]}],\"nodes\":[{\"extensions\":{\"KHR_lights_punctual\":{\"light\":1}}}],\"images\":[{\"uri\":\"data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==\"}],\"textures\":[{\"source\":0}],\"materials\":[{\"extensions\":{\"KHR_materials_emissive_strength\":{\"emissiveStrength\":4.5},\"KHR_materials_ior\":{\"ior\":1.33},\"KHR_materials_transmission\":{\"transmissionFactor\":0.25,\"transmissionTexture\":{\"index\":0,\"texCoord\":1}},\"KHR_materials_clearcoat\":{\"clearcoatFactor\":0.8,\"clearcoatRoughnessFactor\":0.1},\"EXT_unknown\":{\"anything\":[1,2]}},\"emissiveTexture\":{\"index\":0,\"extensions\":{\"KHR_texture_transform\":{\"offset\":[0.5,0.25],\"rotation\":1.5,\"scale\":[2,3],\"texCoord\":2}}}},{}]}";

typed_extensions :: () {
    asset, error := gltf_load(EXTENSIONS);

    require(error.kind == .NONE, error.message);
    defer gltf_free(*asset);
    d := *asset.document;
    ext := *d.materials[0].extensions;

    strength, found, e := gltf_extension(ext, "KHR_materials_emissive_strength", Gltf_Emissive_Strength);

    require(found && e.kind == .NONE);
    expect(strength.emissive_strength == 4.5);

    ior, ior_found, _ := gltf_extension(ext, "KHR_materials_ior", Gltf_Ior);

    expect(ior_found && ior.ior == 1.33);

    transmission, t_found, te := gltf_extension(ext, "KHR_materials_transmission", Gltf_Transmission);

    require(t_found && te.kind == .NONE);
    defer free_json(*transmission);
    expect(transmission.transmission_factor == 0.25);
    require(transmission.transmission_texture != null);
    expect(transmission.transmission_texture.tex_coord == 1);

    coat, c_found, _ := gltf_extension(ext, "KHR_materials_clearcoat", Gltf_Clearcoat);

    expect(c_found && coat.clearcoat_factor == 0.8 && coat.clearcoat_roughness_factor == 0.1);
    expect(coat.clearcoat_texture == null);

    // An absent extension is not found and leaves the defaults.
    empty_ior, none, _ := gltf_extension(*d.materials[1].extensions, "KHR_materials_ior", Gltf_Ior);

    expect(!none && empty_ior.ior == 1.5);

    // Other extensions stay raw.
    unknown := json_get(ext, "EXT_unknown");

    require(unknown != null);
    expect_equal(json_count(json_get(unknown, "anything")), 2);

    transform, x_found, xe := gltf_extension(*d.materials[0].emissive_texture.extensions, "KHR_texture_transform", Gltf_Texture_Transform);

    require(x_found && xe.kind == .NONE);
    expect(transform.offset[0] == 0.5 && transform.offset[1] == 0.25 && transform.rotation == 1.5 && transform.scale[1] == 3);
    expect(transform.tex_coord.state == .SET && transform.tex_coord.value == 2);

    lights, l_found, le := gltf_extension(*d.extensions, "KHR_lights_punctual", Gltf_Lights);

    require(l_found && le.kind == .NONE);
    defer free_json(*lights);
    require_equal(lights.lights.count, 2);
    spot := lights.lights[0];

    expect(spot.type == .SPOT && spot.color[1] == 0.5 && spot.intensity == 3 && spot.range == 10);
    require(spot.spot != null);
    expect(spot.spot.inner_cone_angle == 0 && spot.spot.outer_cone_angle == 0.5);
    expect(lights.lights[1].type == .DIRECTIONAL && lights.lights[1].intensity == 1 && lights.lights[1].range == 0);

    on_node, n_found, _ := gltf_extension(*d.nodes[0].extensions, "KHR_lights_punctual", Gltf_Node_Light);

    expect(n_found && on_node.light == 1);
} @test

mistyped_extension_is_an_error :: () {
    asset, error := gltf_load("{\"asset\":{\"version\":\"2.0\"},\"extensionsUsed\":[\"KHR_materials_ior\"],\"materials\":[{\"extensions\":{\"KHR_materials_ior\":{\"ior\":\"high\"}}}]}");

    require(error.kind == .NONE, error.message);
    defer gltf_free(*asset);

    _, found, e := gltf_extension(*asset.document.materials[0].extensions, "KHR_materials_ior", Gltf_Ior);

    expect(found && e.kind == .JSON);
    gltf_error_free(*e);
} @test
