#import "Basic";
AT :: #import "Android/Toolchain";
FU :: #import "File_Utilities";
C :: #import "Compiler";
F :: #import "File";

main :: () {
    arm, arm_sdk := AT.get_android_target_triple(.ARM64);
    intel, intel_sdk := AT.get_android_target_triple(.X64);

    assert(arm == "aarch64-linux-android" && arm_sdk == "aarch64-linux-android34");
    assert(intel == "x86_64-linux-android" && intel_sdk == "x86_64-linux-android34");
    args := get_command_line_arguments();
    if args.count > 1 {
        found, root := AT.android_find_ndk_path();

        defer free(root);
        assert(found && root == args[1]);
        paths := AT.get_ndk_paths();

        defer {
            free(paths.root);
            free(paths.toolchain);
            free(paths.sysroot);
        }

        assert(paths.root == args[1]);
        directory, valid := FU.is_directory(paths.sysroot);

        assert(directory && valid);
        include, platform := AT.get_ndk_libc_paths(.ARM64);

        defer free(include);
        defer free(platform);
        assert(include == sprint("%/usr/include", paths.sysroot));
        assert(platform == sprint("%/usr/include/aarch64-linux-android", paths.sysroot));
        candidates := AT.get_ndk_libclang_paths();

        defer {
            for candidate: candidates free(candidate);
            array_free(candidates);
        }

        assert(candidates.count == 1);
        assert(AT.copy_android_libcpp(.ARM64, "generated-sdk-output"));
        contents, copied := F.read_entire_file("generated-sdk-output/libc++_shared.so");

        defer free(contents);
        assert(copied && contents == "controlled library payload");

        message: C.Message_Phase;
        message.executable_name = "generated-sdk-output/should-not-exist.so";
        options: C.Build_Options;
        options.cpu_target = .ARM64;
        options.output_type = .DYNAMIC_LIBRARY;
        AT.run_android_link_command(*message, options, string.["--controlled-test"]);
        assert(message.executable_write_failed && message.linker_exit_code == 42);
    }

    print("Android SDK/NDK host path contracts passed\n");
}
