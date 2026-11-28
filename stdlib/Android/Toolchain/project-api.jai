// Returns the platform style resource name for `theme`, such as `@android:style/Theme.Black`.
//
// Returns an empty string for a value that is not one of the [Android_Default_Theme] members.
android_theme_to_string :: (theme: Android_Default_Theme) -> string {
    if theme == .Theme return "@android:style/Theme";
    if theme == .Theme_NoTitleBar return "@android:style/Theme.NoTitleBar";
    if theme == .Theme_NoTitleBar_Fullscreen return "@android:style/Theme.NoTitleBar.Fullscreen";
    if theme == .Theme_Black return "@android:style/Theme.Black";
    if theme == .Theme_Black_NoTitleBar return "@android:style/Theme.Black.NoTitleBar";
    if theme == .Theme_Black_NoTitleBar_Fullscreen return "@android:style/Theme.Black.NoTitleBar.Fullscreen";
    if theme == .Theme_Dialog return "@android:style/Theme.Dialog";
    if theme == .Theme_Light return "@android:style/Theme.Light";
    if theme == .Theme_Light_NoTitleBar return "@android:style/Theme.Light.NoTitleBar";
    if theme == .Theme_Light_NoTitleBar_Fullscreen return "@android:style/Theme.Light.NoTitleBar.Fullscreen";
    if theme == .Theme_Translucent return "@android:style/Theme.Translucent";
    if theme == .Theme_Translucent_NoTitleBar return "@android:style/Theme.Translucent.NoTitleBar";
    if theme == .Theme_Translucent_NoTitleBar_Fullscreen return "@android:style/Theme.Translucent.NoTitleBar.Fullscreen";

    return "";
}

// Writes a Gradle project for `config` into `config.project_path`.
//
// Creates the Gradle build files, `gradle.properties`, the wrapper properties and
// `app/src/main/AndroidManifest.xml`, which is `config.android_manifest` when set and a generated
// native-activity manifest otherwise. Does not write the Gradle wrapper scripts. Values from `config` are
// escaped for their file. With `recreate_existing_files` false the call returns `false` when
// `app/build.gradle` already exists. Also returns `false` when `project_path` is empty or a file cannot be
// written.
generate_android_project :: (using config: Android_Project_Config, recreate_existing_files := true) -> bool {
    return android_project_generate(config, recreate_existing_files);
}

#load "../../internal/android-toolchain/android-project.jai";
