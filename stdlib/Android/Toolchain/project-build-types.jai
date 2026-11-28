// Holds the settings of an APK build, passed to [android_generate_apk].
//
// Which fields are read depends on `mode`; the signing fields apply to the command-line mode, and Gradle signs
// with its own configuration.
Android_Build_Options :: struct {
    // Selects the backend used by [android_generate_apk].
    mode: enum {
        GRADLE;
        COMMAND_LINE_TOOLS;
        // Assembles the APK without external tools; not implemented, see [android_generate_apk_handmade].
        ILL_WRITE_IT_AND_WELL_DO_IT_LIVE_FUCKING_THING_SUCKS;
    } = .GRADLE;
    sdk_path: string; // Android SDK folder; empty means look it up.
    java_bin_path: string; // Folder with `keytool` for generating a signing key; empty uses the `PATH`.
    sdk_level := SDK_LEVEL; // The `android.jar` platform and the target API level of native builds.
    build_type: enum {
        DEBUG;
        RELEASE;
    } = .DEBUG;
    DEFAULT_INTERMEDIATE_PATH :: ".build_android";
    intermediate_path := DEFAULT_INTERMEDIATE_PATH; // Folder for files made while building.
    cpu_target := CPU_Tag.ARM64; // Only X64 and ARM64 can be packaged.
    output_path: string; // Folder to copy the finished APK to; empty leaves it where it was built.
    output_apk_name := "what_is_in_a_name"; // File name in `output_path`; `.apk` is added if missing.
    assets_path_type: enum {
        CONTENTS_OF_PATH; // The contents of `assets_path` go straight into the assets folder.
        SINGLE_DIRECTORY; // `assets_path` itself goes into the assets folder, as one subfolder.
    } = .SINGLE_DIRECTORY;
    assets_path: string; // Folder to include as app assets; empty means none.
    shared_library_path: string; // Native library to include as `lib<lib_name>.so`; empty means none.
    use_jks_keystore_in_handmade_mode := false; // Not read by the current code.
    signing_key_source: enum {
        AUTO_GENERATE;
        CUSTOM;
        SDK_DEFAULT_DEBUG;
    } = .AUTO_GENERATE;
    custom_signing_key: Android_Signing_Key; // Used when `signing_key_source` is `.CUSTOM`.
}

#scope_file
#import "Compiler";
