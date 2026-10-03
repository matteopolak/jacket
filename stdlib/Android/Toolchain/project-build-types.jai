Android_Build_Options :: struct {
    mode: enum {
        GRADLE;
        COMMAND_LINE_TOOLS;
        ILL_WRITE_IT_AND_WELL_DO_IT_LIVE_FUCKING_THING_SUCKS;
    } = .GRADLE;
    sdk_path: string;
    java_bin_path: string;
    sdk_level := SDK_LEVEL;
    build_type: enum {
        DEBUG;
        RELEASE;
    } = .DEBUG;
    DEFAULT_INTERMEDIATE_PATH :: ".build_android";
    intermediate_path := DEFAULT_INTERMEDIATE_PATH;
    cpu_target := CPU_Tag.ARM64;
    output_path: string;
    output_apk_name := "what_is_in_a_name";
    assets_path_type: enum {
        CONTENTS_OF_PATH;
        SINGLE_DIRECTORY;
    } = .SINGLE_DIRECTORY;
    assets_path: string;
    shared_library_path: string;
    use_jks_keystore_in_handmade_mode := false;
    signing_key_source: enum {
        AUTO_GENERATE;
        CUSTOM;
        SDK_DEFAULT_DEBUG;
    } = .AUTO_GENERATE;
    custom_signing_key: Android_Signing_Key;
}

#scope_file
#import "Compiler";
