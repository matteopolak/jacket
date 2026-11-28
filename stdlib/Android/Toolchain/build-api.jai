// Builds an APK for `config` with the backend chosen by `options.mode`.
//
// Returns `true` and the path of the APK on success, or `false` and an empty string on failure; the caller
// frees the path. The mode `.GRADLE` calls [android_generate_apk_gradle], `.COMMAND_LINE_TOOLS` calls
// [android_generate_apk_command_line_tools] and the last mode calls [android_generate_apk_handmade].
android_generate_apk :: (options: Android_Build_Options, config: Android_Project_Config) -> bool, string {
    if options.mode == .GRADLE return android_generate_apk_gradle(options, config);
    if options.mode == .COMMAND_LINE_TOOLS return android_generate_apk_command_line_tools(options, config);
    if options.mode == .ILL_WRITE_IT_AND_WELL_DO_IT_LIVE_FUCKING_THING_SUCKS return android_generate_apk_handmade(options, config);
    return false, "";
}

// Builds an APK by running Gradle in `config.project_path`.
//
// Copies `options.shared_library_path` and `options.assets_path` into the project first, then runs
// `assembleDebug` or `assembleRelease` (by `options.build_type`). Only
// `app/build/outputs/apk/debug/app-debug.apk` or `app/build/outputs/apk/release/app-release.apk` is looked at.
// When `options.output_path` is set the file is copied there as `options.output_apk_name` (`.apk` added if
// missing, no folder part allowed) and that path is returned; otherwise the path inside the project is
// returned. Returns `false` and an empty string on any failure, including a missing APK.
android_generate_apk_gradle :: (options: Android_Build_Options, config: Android_Project_Config) -> bool, string {
    return android_build_gradle(options, config);
}

// Builds an APK with the SDK's `aapt2`, `zipalign` and `apksigner` instead of Gradle.
//
// Uses `options.sdk_path`, or the SDK found by the environment when it is empty, and the newest installed
// build-tools. Generates the Gradle project when it has no manifest. Links resources, packs the library and
// assets, aligns the file and signs it with `apksigner`, then runs `apksigner verify`. Intermediate files go
// under `options.intermediate_path`. The key comes from `options.signing_key_source`: `.CUSTOM` uses
// `options.custom_signing_key`; `.SDK_DEFAULT_DEBUG` uses `~/.android/debug.keystore` (or `debug.keystore` in
// the SDK) with alias `androiddebugkey` and password `android`; `.AUTO_GENERATE` creates
// `generated-signing-key.jks` with `keytool` on first use, with a fixed alias and password. Passwords are
// passed to `apksigner` on its command line. Returns `false` and an empty string when a tool is missing or
// fails.
android_generate_apk_command_line_tools :: (options: Android_Build_Options, config: Android_Project_Config) -> bool, string {
    return android_build_command_line(options, config);
}

// Is the mode that assembles the APK without external tools. Not implemented.
//
// Packs the APK into `options.intermediate_path` and then always returns `false` and an empty string, because
// the signing step is not wired in. No finished APK is produced.
android_generate_apk_handmade :: (options: Android_Build_Options, config: Android_Project_Config) -> bool, string {
    return android_build_handmade(options, config);
}

#load "../../internal/android-toolchain/android-build.jai";
