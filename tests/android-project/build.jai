#import "Basic";
A :: #import "Android/Toolchain";
F :: #import "File";
U :: #import "File_Utilities";
P :: #import "Process";

main :: () {
    #if OS_IS_UNIX {
        project := "target/jai android build fixture";

        assert(F.make_directory_if_it_does_not_exist(project, true));
        wrapper := "target/jai android build fixture/gradlew";
        script := "#!/bin/sh\ncase \"$1\" in\n assembleDebug) mkdir -p app/build/outputs/apk/debug; printf apk > app/build/outputs/apk/debug/app-debug.apk;;\n assembleRelease) mkdir -p app/build/outputs/apk/release; printf release > app/build/outputs/apk/release/app-release.apk;;\n installDebug|installRelease) exit 31;;\n help) exit 0;;\n *) exit 23;;\nesac\n";

        assert(F.write_entire_file(wrapper, script));
        assert(A.run_gradle_task(project, "help", false));
        assert(!A.run_gradle_task(project, "fail", false));
        assert(!A.run_gradle_task("target/jai absent android build project", "help", false));
        assert(!A.gradle_install_apk(project));
        assert(!A.gradle_install_apk(project, true));
        config: A.Android_Project_Config;
        config.project_path = project;
        options: A.Android_Build_Options;
        success, output := A.android_generate_apk_gradle(options, config);

        assert(success);
        assert(U.file_exists(output));
        free(output);
        options.build_type = .RELEASE;
        options.output_path = "target/jai android build copied";
        options.output_apk_name = "my app";
        success, output = A.android_generate_apk(options, config);
        assert(success);
        assert(output == "target/jai android build copied/my app.apk");
        data, read_ok := F.read_entire_file(output);

        assert(read_ok && data == "release");
        free(data);
        free(output);
        assert(F.write_entire_file(wrapper, "#!/bin/sh\nexit 9\n"));
        success, output = A.android_generate_apk_gradle(options, config);
        assert(!success && output.count == 0);
        assert(F.write_entire_file(wrapper, "#!/bin/sh\nexit 0\n"));
        assert(F.file_delete("target/jai android build fixture/app/build/outputs/apk/debug/app-debug.apk"));
        options.build_type = .DEBUG;
        success, output = A.android_generate_apk_gradle(options, config);
        assert(!success && output.count == 0);
        options.sdk_path = "target/jai absent android sdk";
        success, output = A.android_generate_apk_command_line_tools(options, config);
        assert(!success && output.count == 0);
        command_line_fixture(config);
        A.__no_gradle_build_be_verbose = false;
    }
}

command_line_fixture :: (config: A.Android_Project_Config) {
    sdk := "target/jai android fixture sdk";
    tools := "target/jai android fixture sdk/build-tools/34.0.0";

    assert(F.make_directory_if_it_does_not_exist(tools, true));
    assert(F.make_directory_if_it_does_not_exist("target/jai android fixture sdk/build-tools/9.0.0", true));
    assert(F.make_directory_if_it_does_not_exist("target/jai android fixture sdk/platforms/android-34", true));
    assert(F.write_entire_file("target/jai android fixture sdk/platforms/android-34/android.jar", "fixture jar"));
    assert(F.make_directory_if_it_does_not_exist("target/jai android build fixture/app/src/main", true));
    assert(F.write_entire_file("target/jai android build fixture/app/src/main/AndroidManifest.xml", "fixture manifest"));
    aapt := "target/jai android fixture sdk/build-tools/34.0.0/aapt2";
    align := "target/jai android fixture sdk/build-tools/34.0.0/zipalign";
    signer := "target/jai android fixture sdk/build-tools/34.0.0/apksigner";

    assert(F.write_entire_file(aapt, "#!/bin/sh\nwhile [ $# -gt 0 ]; do\n if [ \"$1\" = -o ]; then shift; out=$1; fi\n shift\ndone\nmkdir -p \"$out\"; printf binary-fixture > \"$out/AndroidManifest.xml\"\n"));
    assert(F.write_entire_file(align, "#!/bin/sh\nshift 3\ncp \"$1\" \"$2\"\n"));
    assert(F.write_entire_file(signer, "#!/bin/sh\nprintf %s \"$1\" > 'target/jai android signer invocation'\nexit 7\n"));
    result := P.run_command("/bin/chmod", "+x", aapt, align, signer);

    assert(result.type == .EXITED && result.exit_code == 0);
    assert(F.write_entire_file("target/jai android fixture key.jks", "fixture key"));
    options: A.Android_Build_Options;
    options.mode = .COMMAND_LINE_TOOLS;
    options.sdk_path = sdk;
    options.intermediate_path = "target/jai android intermediates";
    options.signing_key_source = .CUSTOM;
    options.custom_signing_key.kind = .JKS;
    options.custom_signing_key.keystore_path = "target/jai android fixture key.jks";
    options.custom_signing_key.alias = "fixture";
    options.custom_signing_key.password = "fixture";
    options.output_path = "target/jai android command output";
    options.output_apk_name = "command";
    assert(F.make_directory_if_it_does_not_exist("target/jai android fixture assets/nested", true));
    assert(F.write_entire_file("target/jai android fixture assets/nested/example", "asset"));
    options.assets_path = "target/jai android fixture assets";
    options.assets_path_type = .CONTENTS_OF_PATH;
    assert(F.write_entire_file("target/jai android fixture native.so", "library"));
    options.shared_library_path = "target/jai android fixture native.so";
    success, output := A.android_generate_apk(options, config);

    assert(!success && output.count == 0);
    invocation, read_ok := F.read_entire_file("target/jai android signer invocation");

    assert(read_ok && invocation == "sign");
    free(invocation);
    // Inspect unsigned ZIP staging only; the process fixture always refuses signing.
    apk := A.apk_load("target/jai android intermediates/command-line-unsigned.apk");

    defer A.apk_deinit(*apk);
    assert(apk.zip_dir.entries.count == 3);
    found_asset := false;
    found_library := false;

    for apk {
        if it.filename == "assets/nested/example" found_asset = true;

        if it.filename == "lib/arm64-v8a/libmain.so" {
            found_library = true;
            assert(it.offset % 4096 == 0);
        }
    }

    assert(found_asset && found_library);
    assert(F.write_entire_file(aapt, "#!/bin/sh\nexit 0\n"));
    success, output = A.android_generate_apk_command_line_tools(options, config);
    assert(!success && output.count == 0);
}
