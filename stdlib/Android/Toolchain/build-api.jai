android_generate_apk :: (options: Android_Build_Options, config: Android_Project_Config) -> bool, string {
    if options.mode == .GRADLE return android_generate_apk_gradle(options, config);
    if options.mode == .COMMAND_LINE_TOOLS return android_generate_apk_command_line_tools(options, config);
    if options.mode == .ILL_WRITE_IT_AND_WELL_DO_IT_LIVE_FUCKING_THING_SUCKS return android_generate_apk_handmade(options, config);
    return false, "";
}

android_generate_apk_gradle :: (options: Android_Build_Options, config: Android_Project_Config) -> bool, string {
    return android_build_gradle(options, config);
}

android_generate_apk_command_line_tools :: (options: Android_Build_Options, config: Android_Project_Config) -> bool, string {
    return android_build_command_line(options, config);
}

android_generate_apk_handmade :: (options: Android_Build_Options, config: Android_Project_Config) -> bool, string {
    return android_build_handmade(options, config);
}

#load "../../internal/android-toolchain/android-build.jai";
