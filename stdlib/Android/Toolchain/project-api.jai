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

generate_android_project :: (using config: Android_Project_Config, recreate_existing_files := true) -> bool {
    return android_project_generate(config, recreate_existing_files);
}

#load "../../internal/android-toolchain/android-project.jai";
