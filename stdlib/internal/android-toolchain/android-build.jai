#scope_module
Build_File :: #import "File";
Build_Files :: #import "File_Utilities";
Build_String :: #import "String";
Build_Process :: #import "Process";

android_build_process :: (arguments: [] string, directory := "", verbose := false) -> bool {
    result, output, errors, timed_out := Build_Process.run_command(..arguments,
        working_directory = directory, capture_and_return_output = true, print_captured_output = verbose);

    defer free(output);
    defer free(errors);

    return result.type == .EXITED && result.exit_code == 0 && !timed_out;
}

android_build_output :: (options: Android_Build_Options, source: string) -> bool, string {
    if !Build_Files.file_exists(source) return false, "";
    if options.output_path.count == 0 return true, copy_string(source);
    if options.output_apk_name.count == 0 || Build_String.path_filename(options.output_apk_name) != options.output_apk_name return false, "";
    if !Build_File.make_directory_if_it_does_not_exist(options.output_path, true) return false, "";

    name := options.output_apk_name;
    owned_name: string;

    if !Build_String.ends_with(name, ".apk") {
        owned_name = Build_String.join(name, ".apk");
        name = owned_name;
    }

    defer free(owned_name);
    destination := Build_String.path_join(options.output_path, name);
    if source != destination && !Build_Files.copy_file(source, destination) {
        free(destination);
        return false, "";
    }

    return true, destination;
}

android_build_gradle :: (options: Android_Build_Options, config: Android_Project_Config) -> bool, string {
    release := options.build_type == .RELEASE;

    if options.shared_library_path.count > 0 {
        if !android_build_valid_cpu(options.cpu_target) return false, "";

        suffix := tprint("app/src/main/jniLibs/%", get_android_project_jni_architecture_name(options.cpu_target));
        library_dir := Build_String.path_join(config.project_path, suffix);

        defer free(library_dir);
        if !Build_File.make_directory_if_it_does_not_exist(library_dir, true) return false, "";

        library := Build_String.path_join(library_dir, tprint("lib%.so", config.lib_name));

        defer free(library);
        if !Build_Files.copy_file(options.shared_library_path, library) return false, "";
    }

    if options.assets_path.count > 0 {
        assets := Build_String.path_join(config.project_path, "app/src/main/assets");

        defer free(assets);
        target := assets;
        owned_target: string;

        if options.assets_path_type == .SINGLE_DIRECTORY {
            owned_target = Build_String.path_join(assets, Build_String.path_filename(Build_String.trim_right(options.assets_path, "/\\")));
            target = owned_target;
        }

        defer free(owned_target);
        if !Build_Files.copy_directory(options.assets_path, target) return false, "";
    }

    if !run_gradle_task(config.project_path, ifx release then "assembleRelease" else "assembleDebug") return false, "";

    path := Build_String.path_join(config.project_path, ifx release then "app/build/outputs/apk/release/app-release.apk" else "app/build/outputs/apk/debug/app-debug.apk");

    defer free(path);

    // Unsigned release artifacts are never reported as completed APKs.
    return android_build_output(options, path);
}

android_build_valid_cpu :: (cpu: CPU_Tag) -> bool {
    return cpu == .X64 || cpu == .ARM64;
}

android_build_tool_directory :: (sdk: string) -> string {
    root := Build_String.path_join(sdk, "build-tools");

    defer free(root);
    best: string;
    select :: (info: *Build_Files.File_Visit_Info, best: *string) {
        if !info.is_directory || info.had_error return;

        // Numeric dotted versions are compared component by component.
        if best.count == 0 || android_build_newer_version(info.short_name, Build_String.path_filename(best.*)) {
            free(best.*);
            best.* = copy_string(info.full_name);
        }
    }

    if !Build_Files.visit_files(root, false, *best, select, visit_files = false, visit_directories = true, follow_directory_symlinks = false) {
        free(best);
        return "";
    }

    return best;
}

android_build_newer_version :: (a: string, b: string) -> bool {
    ai: s64;
    bi: s64;

    while ai < a.count || bi < b.count {
        av: s64;
        bv: s64;

        while ai < a.count && a[ai] >= 48 && a[ai] <= 57 {
            if av > 1000000 return false;
            av = av * 10 + cast(s64) a[ai] - 48;
            ai += 1;
        }

        while bi < b.count && b[bi] >= 48 && b[bi] <= 57 {
            if bv > 1000000 return true;
            bv = bv * 10 + cast(s64) b[bi] - 48;
            bi += 1;
        }

        if av != bv return av > bv;
        if ai < a.count && a[ai] != 46 return false;
        if bi < b.count && b[bi] != 46 return true;
        if ai < a.count ai += 1;
        if bi < b.count bi += 1;
    }

    return false;
}

android_build_tool :: (directory: string, name: string) -> string {
    #if OS == .WINDOWS {
        if name == "apksigner" return Build_String.path_join(directory, "apksigner.bat");
        return Build_String.path_join(directory, tprint("%.exe", name));
    } else return Build_String.path_join(directory, name);
}

android_build_add_tree :: (writer: *Zip_Writer, source: string, prefix: string) -> bool {
    root, valid := Build_File.get_absolute_path(source);
    if !valid return false;

    defer free(root);

    State :: struct {
        writer: *Zip_Writer;
        root: string;
        prefix: string;
        success := true;
    }

    state := State.{ writer, root, prefix };

    add :: (info: *Build_Files.File_Visit_Info, state: *State) {
        if info.had_error {
            state.success = false;
            return;
        }

        if !state.success || info.is_directory return;

        relative := string.{ info.full_name.count - state.root.count - 1, info.full_name.data + state.root.count + 1 };
        name := Build_String.path_join(state.prefix, relative);

        defer free(name);
        for *byte: name if byte.* == 92 byte.* = 47;

        // The writer borrows names through central-directory emission.
        retained_name := copy_temporary_string(name);
        if !zip_add_file(state.writer, info.full_name, retained_name) state.success = false;
    }

    visited := Build_Files.visit_files(root, true, *state, add, follow_directory_symlinks = false);

    return visited && state.success;
}

android_build_package :: (options: Android_Build_Options, config: Android_Project_Config, resource_directory: string, manifest: string, unsigned_path: string) -> bool {
    writer: Zip_Writer;

    zip_init_writer(*writer, max_alignment = 4096);
    defer free_buffers(*writer.builder);

    if resource_directory.count > 0 {
        if !android_build_add_tree(*writer, resource_directory, "") return false;
    } else if !zip_add_file_from_memory(*writer, manifest, "AndroidManifest.xml") return false;

    if options.shared_library_path.count > 0 {
        if !android_build_valid_cpu(options.cpu_target) || !Build_Files.file_exists(options.shared_library_path) return false;
        name := tprint("lib/%/lib%.so", get_android_project_jni_architecture_name(options.cpu_target), config.lib_name);
        if !zip_add_file(*writer, options.shared_library_path, name, alignment = 4096) return false;
    }

    if options.assets_path.count > 0 {
        prefix: string = "assets";
        owned_prefix: string;

        if options.assets_path_type == .SINGLE_DIRECTORY {
            owned_prefix = Build_String.path_join(prefix, Build_String.path_filename(Build_String.trim_right(options.assets_path, "/\\")));
            prefix = owned_prefix;
        }

        defer free(owned_prefix);
        if !android_build_add_tree(*writer, options.assets_path, prefix) return false;
    }

    zip_write_central_directory(*writer);
    bytes := builder_to_string(*writer.builder);

    defer free(bytes);
    if writer.builder.failed return false;

    return Build_File.write_entire_file(unsigned_path, bytes);
}

android_build_remove_tree :: (path: string) -> bool {
    success := true;
    remove :: (info: *Build_Files.File_Visit_Info, success: *bool) {
        if info.had_error {
            success.* = false;
            return;
        }

        if info.is_directory && !info.is_symlink {
            if !android_build_remove_tree(info.full_name) success.* = false;
        } else if !Build_File.file_delete(info.full_name) success.* = false;
    }

    visited := Build_Files.visit_files(path, false, *success, remove, visit_directories = true, follow_directory_symlinks = false);

    if !visited || !success return false;

    return Build_File.delete_directory(path);
}

android_build_handmade :: (options: Android_Build_Options, config: Android_Project_Config) -> bool, string {
    if !Build_File.make_directory_if_it_does_not_exist(options.intermediate_path, true) return false, "";

    ok, manifest := write_binary_app_manifest_to_memory(config);
    if !ok return false, "";

    defer free(manifest);
    unsigned := Build_String.path_join(options.intermediate_path, "handmade-unsigned.apk");

    defer free(unsigned);
    if !android_build_package(options, config, "", manifest, unsigned) return false, "";

    context.logger("Handmade APK packaging completed; canonical apk_sign/apk_sign2 signing dependency is not implemented\n", context.logger_data, Log_Info.{ common_flags = .ERROR });

    return false, "";
}

android_build_command_line :: (options: Android_Build_Options, config: Android_Project_Config) -> bool, string {
    sdk := copy_string(options.sdk_path);
    if sdk.count == 0 {
        free(sdk);
        found: bool;
        found, sdk = android_find_sdk_path();
        if !found return false, "";
    }

    defer free(sdk);
    tools := android_build_tool_directory(sdk);

    defer free(tools);
    if tools.count == 0 return false, "";

    aapt := android_build_tool(tools, "aapt2");
    align := android_build_tool(tools, "zipalign");
    signer := android_build_tool(tools, "apksigner");

    defer free(aapt);
    defer free(align);
    defer free(signer);
    if !Build_Files.file_exists(aapt) || !Build_Files.file_exists(align) || !Build_Files.file_exists(signer) return false, "";

    platform := Build_String.path_join(sdk, tprint("platforms/android-%/android.jar", options.sdk_level));

    defer free(platform);
    if !Build_Files.file_exists(platform) || !Build_File.make_directory_if_it_does_not_exist(options.intermediate_path, true) return false, "";

    manifest_path := Build_String.path_join(config.project_path, "app/src/main/AndroidManifest.xml");

    defer free(manifest_path);

    if !Build_Files.file_exists(manifest_path) {
        if !generate_android_project(config) return false, "";
    }

    resources := Build_String.path_join(options.intermediate_path, "linked-resources");

    defer free(resources);
    // Remove previous resources before linking so a successful build cannot retain stale entries.
    if Build_Files.file_exists(resources) && !android_build_remove_tree(resources) return false, "";
    if !Build_File.make_directory_if_it_does_not_exist(resources, true) return false, "";
    if !android_build_process(string.[aapt, "link", "--manifest", manifest_path, "-I", platform, "--min-sdk-version", tprint("%", config.min_sdk_level), "--target-sdk-version", tprint("%", config.target_sdk_level), "--output-to-dir", "-o", resources], verbose = __no_gradle_build_be_verbose) return false, "";

    binary_manifest := Build_String.path_join(resources, "AndroidManifest.xml");

    defer free(binary_manifest);
    if !Build_Files.file_exists(binary_manifest) return false, "";

    unsigned := Build_String.path_join(options.intermediate_path, "command-line-unsigned.apk");
    aligned := Build_String.path_join(options.intermediate_path, "command-line-aligned.apk");
    signed := Build_String.path_join(options.intermediate_path, "command-line-signed.apk");

    defer free(unsigned);
    defer free(aligned);
    defer free(signed);
    if !android_build_package(options, config, resources, "", unsigned) return false, "";
    if Build_Files.file_exists(aligned) && !Build_File.file_delete(aligned) return false, "";
    if !android_build_process(string.[align, "-f", "-p", "4", unsigned, aligned], verbose = __no_gradle_build_be_verbose) || !Build_Files.file_exists(aligned) return false, "";
    if !android_build_sign_external(options, sdk, signer, aligned, signed) return false, "";

    return android_build_output(options, signed);
}

android_build_sign_external :: (options: Android_Build_Options, sdk: string, signer: string, aligned: string, signed: string) -> bool {
    key := options.custom_signing_key;
    owned_key_path: string;

    defer free(owned_key_path);

    if options.signing_key_source == .SDK_DEFAULT_DEBUG {
        Build_System :: #import "System";
        home, found_home := Build_System.get_home_directory();

        if found_home owned_key_path = Build_String.path_join(home, ".android/debug.keystore");

        if owned_key_path.count == 0 || !Build_Files.file_exists(owned_key_path) {
            free(owned_key_path);
            owned_key_path = Build_String.path_join(sdk, "debug.keystore");
        }

        key.kind = .JKS;
        key.keystore_path = owned_key_path;
        key.alias = "androiddebugkey";
        key.password = "android";
    } else if options.signing_key_source == .AUTO_GENERATE {
        owned_key_path = Build_String.path_join(options.intermediate_path, "generated-signing-key.jks");
        key.kind = .JKS;
        key.keystore_path = owned_key_path;
        key.alias = "jaiiskewl";
        key.password = "password";

        if !Build_Files.file_exists(owned_key_path) {
            tool: string = "keytool";
            owned_tool: string;

            if options.java_bin_path.count > 0 {
                #if OS == .WINDOWS owned_tool = Build_String.path_join(options.java_bin_path, "keytool.exe");
                else owned_tool = Build_String.path_join(options.java_bin_path, "keytool");
                tool = owned_tool;
            }

            defer free(owned_tool);
            if !android_build_process(string.[tool, "-genkeypair", "-noprompt", "-storetype", "JKS", "-keystore", key.keystore_path,
                "-alias", key.alias, "-storepass", key.password, "-keypass", key.password,
                "-keyalg", "EC", "-groupname", "secp256r1", "-dname", "CN=Jai Android,O=Jai,C=US", "-validity", "10000"],
                verbose = __no_gradle_build_be_verbose) || !Build_Files.file_exists(owned_key_path) return false;
        }
    }

    if Build_Files.file_exists(signed) && !Build_File.file_delete(signed) return false;

    arguments: [..] string;

    defer array_free(arguments);

    #if OS == .WINDOWS {
        array_add(*arguments, "cmd.exe");
        array_add(*arguments, "/c");
    }

    array_add(*arguments, signer);
    array_add(*arguments, "sign");

    if key.kind == .JKS {
        if !Build_Files.file_exists(key.keystore_path) return false;

        array_add(*arguments, "--ks");
        array_add(*arguments, key.keystore_path);
        array_add(*arguments, "--ks-key-alias");
        array_add(*arguments, key.alias);
        array_add(*arguments, "--ks-pass");
        array_add(*arguments, tprint("pass:%", key.password));
        array_add(*arguments, "--key-pass");
        array_add(*arguments, tprint("pass:%", key.password));
    } else {
        if !Build_Files.file_exists(key.direct.private_key_path) || !Build_Files.file_exists(key.direct.certificate_path) return false;

        array_add(*arguments, "--key");
        array_add(*arguments, key.direct.private_key_path);
        array_add(*arguments, "--cert");
        array_add(*arguments, key.direct.certificate_path);
    }

    array_add(*arguments, "--out");
    array_add(*arguments, signed);
    array_add(*arguments, aligned);
    if !android_build_process(arguments, verbose = __no_gradle_build_be_verbose) || !Build_Files.file_exists(signed) return false;

    #if OS == .WINDOWS return android_build_process(string.["cmd.exe", "/c", signer, "verify", signed], verbose = __no_gradle_build_be_verbose);
    else return android_build_process(string.[signer, "verify", signed], verbose = __no_gradle_build_be_verbose);
}
