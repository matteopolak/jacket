// Private algorithm unit, not a public DXC/native compiler completion claim.
#load "../../stdlib/internal/bindings/dxc-options/module.jai";

libc_test :: #library,system "c";
fail :: () #foreign libc_test "abort";

Settings :: struct {
    shader_name: string;
    shader_model_version: string;
    entry_point: string;
    defines: [] string;
    extra_args: [] string;
    skip_optimization: bool;
    add_debug_info: bool;
    warnings_as_error: bool;
    turn_off_validation: bool;
    spirv_target_env: string;
    spirv_generate_non_semanantic_debug_info: bool;
}

check :: (value: bool) {
    if !value fail();
}

main :: () {
    settings: Settings;
    settings.shader_name = "shader café.hlsl";
    settings.shader_model_version = "6_7";
    settings.entry_point = "main";
    definitions: [2] string = .["COUNT=3", "LABEL=two words"];
    extra: [2] string = .["-I", "directory with spaces"];

    settings.defines = definitions;
    settings.extra_args = extra;
    settings.skip_optimization = true;
    settings.add_debug_info = true;
    settings.warnings_as_error = true;
    settings.turn_off_validation = true;
    actual := build("ps", settings);

    defer release(actual);
    expected: [18] string = .["shader café.hlsl", "-HV", "2021", "-E", "main", "-T", "ps_6_7",
        "-D", "COUNT=3", "-D", "LABEL=two words", "-Od", "-Zi", "-Qembed_debug", "-WX", "-Vd", "-I", "directory with spaces"];

    check(actual.count == expected.count);
    for i: 0..actual.count - 1 check(actual[i] == expected[i]);

    // The result owns its bytes, including strings taken from input arrays.
    check(actual[8].data != definitions[0].data);
    check(actual[0].data != settings.shader_name.data);
    settings.spirv_target_env = "vulkan1.3";
    settings.spirv_generate_non_semanantic_debug_info = true;
    spirv := build("cs", settings);

    defer release(spirv);
    found_spirv, found_environment, found_debug: bool;

    for text: spirv {
        check(text != "-Qembed_debug");
        if text == "-spirv" found_spirv = true;
        if text == "-fspv-target-env=vulkan1.3" found_environment = true;
        if text == "-fspv-debug=vulkan-with-source" found_debug = true;
    }

    check(found_spirv && found_environment && found_debug);
    settings.spirv_generate_non_semanantic_debug_info = false;
    settings.add_debug_info = false;
    settings.skip_optimization = false;
    settings.warnings_as_error = false;
    settings.turn_off_validation = false;
    minimal_spirv := build("cs", settings);

    defer release(minimal_spirv);

    for text: minimal_spirv {
        check(text != "-fspv-debug=vulkan-with-source" && text != "-Zi" && text != "-Qembed_debug");
        check(text != "-Od" && text != "-WX" && text != "-Vd");
    }

    empty: Settings;
    empty.shader_model_version = "6_7";
    library := build("lib", empty);

    defer release(library);
    check(library.count == 4 && library[0] == "-HV" && library[1] == "2021");
    check(library[2] == "-T" && library[3] == "lib_6_7");
}
