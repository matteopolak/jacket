#scope_file
#import "Basic";
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
    expect(e.path == "$.ior");
    gltf_error_free(*e);
} @test

// Read, edit, write, read again: the edit to a typed struct survives.
edited_extensions_are_written :: () {
    asset, error := gltf_load(EXTENSIONS);

    require(error.kind == .NONE, error.message);
    defer gltf_free(*asset);
    d := *asset.document;
    ext := *d.materials[0].extensions;

    ior, _, _ := gltf_extension(ext, "KHR_materials_ior", Gltf_Ior);

    ior.ior = 2.4;
    set := gltf_set_extension(ext, "KHR_materials_ior", ior);

    require(set.kind == .NONE, set.message);

    transmission, _, _ := gltf_extension(ext, "KHR_materials_transmission", Gltf_Transmission);

    transmission.transmission_factor = 0.75;
    transmission.transmission_texture.tex_coord = 3;
    set = gltf_set_extension(ext, "KHR_materials_transmission", transmission);
    require(set.kind == .NONE, set.message);
    free_json(*transmission);

    transform, _, _ := gltf_extension(*d.materials[0].emissive_texture.extensions, "KHR_texture_transform", Gltf_Texture_Transform);

    transform.offset = .[0.125, 0.5];
    transform.tex_coord = .{ state = .NULL };
    set = gltf_set_extension(*d.materials[0].emissive_texture.extensions, "KHR_texture_transform", transform);
    require(set.kind == .NONE, set.message);

    lights, _, _ := gltf_extension(*d.extensions, "KHR_lights_punctual", Gltf_Lights);

    lights.lights[1].intensity = 9;
    lights.lights[0].name = copy_string("sun");
    set = gltf_set_extension(*d.extensions, "KHR_lights_punctual", lights);
    require(set.kind == .NONE, set.message);
    free_json(*lights);

    for binary: .[false, true] {
        bytes, write_error := gltf_write(*asset, binary);

        require(write_error.kind == .NONE, write_error.message);
        defer free(bytes);

        again, read_error := gltf_load(bytes);

        require(read_error.kind == .NONE, read_error.message);
        defer gltf_free(*again);
        a := *again.document;
        e := *a.materials[0].extensions;

        read_ior, _, _ := gltf_extension(e, "KHR_materials_ior", Gltf_Ior);
        read_transmission, _, _ := gltf_extension(e, "KHR_materials_transmission", Gltf_Transmission);

        defer free_json(*read_transmission);

        expect(read_ior.ior == 2.4);
        expect(read_transmission.transmission_factor == 0.75);
        require(read_transmission.transmission_texture != null);
        expect_equal(read_transmission.transmission_texture.tex_coord, 3);

        // Extensions that were not touched are as they were.
        strength, _, _ := gltf_extension(e, "KHR_materials_emissive_strength", Gltf_Emissive_Strength);

        expect(strength.emissive_strength == 4.5);
        expect_equal(json_count(json_get(json_get(e, "EXT_unknown"), "anything")), 2);

        read_transform, _, _ := gltf_extension(*a.materials[0].emissive_texture.extensions, "KHR_texture_transform", Gltf_Texture_Transform);

        expect(read_transform.offset[0] == 0.125 && read_transform.offset[1] == 0.5 && read_transform.scale[1] == 3 && read_transform.rotation == 1.5);
        expect(read_transform.tex_coord.state == .NULL);

        read_lights, _, _ := gltf_extension(*a.extensions, "KHR_lights_punctual", Gltf_Lights);

        defer free_json(*read_lights);

        require_equal(read_lights.lights.count, 2);
        expect(read_lights.lights[0].name == "sun" && read_lights.lights[1].intensity == 9 && read_lights.lights[0].spot.outer_cone_angle == 0.5);
    }
} @test

new_extension_needs_declaring :: () {
    asset, error := gltf_load("{\"asset\":{\"version\":\"2.0\"},\"materials\":[{},{}]}");

    require(error.kind == .NONE, error.message);
    defer gltf_free(*asset);

    // The extensions of a material that had none become an object.
    set := gltf_set_extension(*asset.document.materials[0].extensions, "KHR_materials_ior", Gltf_Ior.{ ior = 1.8 });

    require(set.kind == .NONE, set.message);

    // Not declared: the output could not be loaded again, so writing refuses.
    bytes, write_error := gltf_write(*asset);

    require(write_error.kind == .INVALID);
    gltf_error_free(*write_error);

    gltf_use_extension(*asset, "KHR_materials_ior");
    gltf_use_extension(*asset, "KHR_materials_ior");
    expect_equal(asset.document.extensions_used.count, 1);

    bytes, write_error = gltf_write(*asset);

    require(write_error.kind == .NONE, write_error.message);
    defer free(bytes);

    again, read_error := gltf_load(bytes);

    require(read_error.kind == .NONE, read_error.message);
    defer gltf_free(*again);

    ior, found, _ := gltf_extension(*again.document.materials[0].extensions, "KHR_materials_ior", Gltf_Ior);

    expect(found && ior.ior == 1.8);

    _, other, _ := gltf_extension(*again.document.materials[1].extensions, "KHR_materials_ior", Gltf_Ior);

    expect(!other);
} @test

set_extension_refuses_a_non_object :: () {
    extensions := json_array();

    defer free_json(*extensions);

    set := gltf_set_extension(*extensions, "KHR_materials_ior", Gltf_Ior.{});

    expect(set.kind == .INVALID);
    gltf_error_free(*set);
    expect_equal(json_count(*extensions), 0);
} @test

SURFACE_EXTENSIONS :: "{\"asset\":{\"version\":\"2.0\"},\"extensionsUsed\":[\"KHR_materials_sheen\",\"KHR_materials_specular\",\"KHR_materials_volume\",\"KHR_materials_iridescence\",\"KHR_materials_anisotropy\",\"KHR_materials_unlit\"],\"images\":[{\"uri\":\"data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==\"}],\"textures\":[{\"source\":0}],\"materials\":[{\"extensions\":{\"KHR_materials_sheen\":{\"sheenColorFactor\":[0.9,0.5,0.1],\"sheenColorTexture\":{\"index\":0,\"texCoord\":1},\"sheenRoughnessFactor\":0.3,\"sheenRoughnessTexture\":{\"index\":0}},\"KHR_materials_specular\":{\"specularFactor\":0.5,\"specularTexture\":{\"index\":0},\"specularColorFactor\":[1,0.8,0.6],\"specularColorTexture\":{\"index\":0}},\"KHR_materials_volume\":{\"thicknessFactor\":0.5,\"thicknessTexture\":{\"index\":0},\"attenuationDistance\":2,\"attenuationColor\":[0.2,0.9,0.4]},\"KHR_materials_iridescence\":{\"iridescenceFactor\":1,\"iridescenceTexture\":{\"index\":0},\"iridescenceIor\":1.5,\"iridescenceThicknessMinimum\":200,\"iridescenceThicknessMaximum\":800,\"iridescenceThicknessTexture\":{\"index\":0}},\"KHR_materials_anisotropy\":{\"anisotropyStrength\":0.6,\"anisotropyRotation\":1.5,\"anisotropyTexture\":{\"index\":0}},\"KHR_materials_unlit\":{}}},{\"extensions\":{\"KHR_materials_sheen\":{},\"KHR_materials_specular\":{},\"KHR_materials_volume\":{},\"KHR_materials_iridescence\":{},\"KHR_materials_anisotropy\":{}}}]}";

surface_extensions :: () {
    asset, error := gltf_load(SURFACE_EXTENSIONS);

    require(error.kind == .NONE, error.message);
    defer gltf_free(*asset);
    ext := *asset.document.materials[0].extensions;

    sheen, found, e := gltf_extension(ext, "KHR_materials_sheen", Gltf_Sheen);

    require(found && e.kind == .NONE);
    defer free_json(*sheen);
    expect(sheen.sheen_color_factor[0] == 0.9 && sheen.sheen_color_factor[2] == 0.1 && sheen.sheen_roughness_factor == 0.3);
    require(sheen.sheen_color_texture != null && sheen.sheen_roughness_texture != null);
    expect_equal(sheen.sheen_color_texture.tex_coord, 1);

    specular, s_found, se := gltf_extension(ext, "KHR_materials_specular", Gltf_Specular);

    require(s_found && se.kind == .NONE);
    defer free_json(*specular);
    expect(specular.specular_factor == 0.5 && specular.specular_color_factor[1] == 0.8);
    expect(specular.specular_texture != null && specular.specular_color_texture != null);

    volume, v_found, ve := gltf_extension(ext, "KHR_materials_volume", Gltf_Volume);

    require(v_found && ve.kind == .NONE);
    defer free_json(*volume);
    expect(volume.thickness_factor == 0.5 && volume.attenuation_color[1] == 0.9);
    expect(volume.attenuation_distance.state == .SET && volume.attenuation_distance.value == 2);
    expect(volume.thickness_texture != null);

    film, i_found, ie := gltf_extension(ext, "KHR_materials_iridescence", Gltf_Iridescence);

    require(i_found && ie.kind == .NONE);
    defer free_json(*film);
    expect(film.iridescence_factor == 1 && film.iridescence_ior == 1.5);
    expect(film.iridescence_thickness_minimum == 200 && film.iridescence_thickness_maximum == 800);
    expect(film.iridescence_texture != null && film.iridescence_thickness_texture != null);

    stretch, a_found, ae := gltf_extension(ext, "KHR_materials_anisotropy", Gltf_Anisotropy);

    require(a_found && ae.kind == .NONE);
    defer free_json(*stretch);
    expect(stretch.anisotropy_strength == 0.6 && stretch.anisotropy_rotation == 1.5 && stretch.anisotropy_texture != null);

    _, unlit, ue := gltf_extension(ext, "KHR_materials_unlit", Gltf_Unlit);

    expect(unlit && ue.kind == .NONE);

    // Empty objects give the specification's defaults.
    bare := *asset.document.materials[1].extensions;
    plain_sheen, _, _ := gltf_extension(bare, "KHR_materials_sheen", Gltf_Sheen);
    plain_specular, _, _ := gltf_extension(bare, "KHR_materials_specular", Gltf_Specular);
    plain_volume, _, _ := gltf_extension(bare, "KHR_materials_volume", Gltf_Volume);
    plain_film, _, _ := gltf_extension(bare, "KHR_materials_iridescence", Gltf_Iridescence);
    plain_stretch, _, _ := gltf_extension(bare, "KHR_materials_anisotropy", Gltf_Anisotropy);

    expect(plain_sheen.sheen_color_factor[0] == 0 && plain_sheen.sheen_roughness_factor == 0);
    expect(plain_specular.specular_factor == 1 && plain_specular.specular_color_factor[0] == 1);
    expect(plain_volume.thickness_factor == 0 && plain_volume.attenuation_distance.state == .MISSING && plain_volume.attenuation_color[2] == 1);
    expect(plain_film.iridescence_ior == 1.3 && plain_film.iridescence_thickness_minimum == 100 && plain_film.iridescence_thickness_maximum == 400);
    expect(plain_stretch.anisotropy_strength == 0 && plain_stretch.anisotropy_rotation == 0);

    // Written back they load again with the same values.
    stretch.anisotropy_strength = 0.9;
    volume.attenuation_distance = .{ state = .MISSING };
    require(gltf_set_extension(ext, "KHR_materials_anisotropy", stretch).kind == .NONE);
    require(gltf_set_extension(ext, "KHR_materials_volume", volume).kind == .NONE);
    require(gltf_set_extension(ext, "KHR_materials_unlit", Gltf_Unlit.{}).kind == .NONE);

    bytes, write_error := gltf_write(*asset);

    require(write_error.kind == .NONE, write_error.message);
    defer free(bytes);

    again, read_error := gltf_load(bytes);

    require(read_error.kind == .NONE, read_error.message);
    defer gltf_free(*again);
    e2 := *again.document.materials[0].extensions;

    read_stretch, _, _ := gltf_extension(e2, "KHR_materials_anisotropy", Gltf_Anisotropy);

    defer free_json(*read_stretch);
    read_volume, _, _ := gltf_extension(e2, "KHR_materials_volume", Gltf_Volume);

    defer free_json(*read_volume);
    read_sheen, _, _ := gltf_extension(e2, "KHR_materials_sheen", Gltf_Sheen);

    defer free_json(*read_sheen);
    _, still_unlit, _ := gltf_extension(e2, "KHR_materials_unlit", Gltf_Unlit);

    expect(read_stretch.anisotropy_strength == 0.9 && read_stretch.anisotropy_rotation == 1.5);
    expect(read_volume.attenuation_distance.state == .MISSING && read_volume.thickness_factor == 0.5);
    expect(read_sheen.sheen_color_factor[1] == 0.5 && read_sheen.sheen_color_texture != null);
    expect(still_unlit);
} @test

// An asset assembled by hand, not loaded: its strings are copies, and `extensions_used` starts empty.
hand_built_asset_uses_an_extension :: () {
    asset: Gltf_Asset;

    asset.document.asset.version = copy_string("2.0");
    defer gltf_free(*asset);

    gltf_use_extension(*asset, "KHR_materials_ior");
    gltf_use_extension(*asset, "KHR_materials_unlit");
    gltf_use_extension(*asset, "KHR_materials_ior");

    require(asset.document.extensions_used.count == 2);
    expect_equal(asset.document.extensions_used[0], "KHR_materials_ior");
    expect_equal(asset.document.extensions_used[1], "KHR_materials_unlit");
} @test

// Lists a caller wrote as literals are borrowed: freeing the asset leaves them alone, and the first
// change copies them.
hand_built_asset_borrows_literal_lists :: () {
    untouched: Gltf_Asset;

    untouched.document.asset.version = copy_string("2.0");
    untouched.document.extensions_used = .["KHR_materials_ior"];
    untouched.document.extensions_required = .["KHR_materials_ior"];
    gltf_free(*untouched);

    asset: Gltf_Asset;

    asset.document.asset.version = copy_string("2.0");
    asset.document.extensions_used = .["KHR_materials_ior"];
    defer gltf_free(*asset);

    gltf_use_extension(*asset, "KHR_materials_unlit");

    require(asset.document.extensions_used.count == 2);
    expect_equal(asset.document.extensions_used[0], "KHR_materials_ior");
    expect_equal(asset.document.extensions_used[1], "KHR_materials_unlit");
} @test
