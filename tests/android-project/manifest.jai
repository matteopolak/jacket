#import "Basic";
#import "Android/Toolchain";
#load "fixtures/embedded.jai";
F :: #import "File";

main :: () {
    assert(F.make_directory_if_it_does_not_exist("tests/android-project/fixtures", recursive = true));
    assert(F.write_entire_file("tests/android-project/fixtures/independent-utf16.axml", FIXTURE_UTF16));
    assert(F.write_entire_file("tests/android-project/fixtures/independent-stored.apk", FIXTURE_STORED));
    assert(F.write_entire_file("tests/android-project/fixtures/independent-deflated.apk", FIXTURE_DEFLATED));
    config: Android_Project_Config;
    config.app_name = "Café 😀";
    config.permissions = .["android.permission.INTERNET"];
    config.features = .[.{ "android.hardware.camera", false }];
    config.application_meta_data = .[.{ "test.key", "test.value" }];
    ok, data := write_binary_app_manifest_to_memory(config);

    assert(ok && data.count > 8);
    defer free(data);
    assert(write_binary_app_manifest_to_file(config, "tests/android-project/independent.axml"));
    ok2, manifest := read_binary_manifest_from_memory(data);

    assert(ok2);
    defer deinit(*manifest);
    assert(manifest.android_namespace.uri == ANDROID_XML_SCHEMA);
    assert(manifest.elements[0].name == "manifest");
    found := false;
    for s: manifest.string_pool if s == config.app_name found = true;

    assert(found);

    for n: 0..data.count - 1 {
        accepted, truncated := read_binary_manifest_from_memory(string.{ n, data.data });
        assert(!accepted);
        deinit(*truncated);
    }

    damaged := copy_string(data);

    defer free(damaged);

    for n: 8..data.count - 1 {
        damaged[4] = cast(u8) (n & 255);
        damaged[5] = cast(u8) ((n >> 8) & 255);
        damaged[6] = cast(u8) ((n >> 16) & 255);
        damaged[7] = cast(u8) ((n >> 24) & 255);
        accepted, truncated := read_binary_manifest_from_memory(string.{ n, damaged.data });

        assert(!accepted);
        deinit(*truncated);
    }

    assert(write_binary_app_manifest_to_file(config, "tests/android-project/independent.axml"));
    file_ok, file_manifest := read_binary_manifest_from_file("tests/android-project/independent.axml");

    assert(file_ok && file_manifest.free_data_on_deinit);
    deinit(*file_manifest);
    fixture_ok, fixture := read_binary_manifest_from_file("tests/android-project/fixtures/independent-utf16.axml");

    assert(fixture_ok && fixture.string_pool[3] == "Unicode 😀");
    assert(fixture.elements.count == 2 && fixture.elements[0].attributes[0].typedValue.data == 2);
    deinit(*fixture);

    for apk_path: string.["tests/android-project/fixtures/independent-stored.apk", "tests/android-project/fixtures/independent-deflated.apk"] {
        apk_ok, apk_manifest := read_binary_manifest_from_apk(apk_path);
        assert(apk_ok && apk_manifest.string_pool[2] == "jai.independent");
        deinit(*apk_manifest);
    }

    print("ok\n");
}
