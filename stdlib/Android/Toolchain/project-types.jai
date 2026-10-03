Android_Project_Config :: struct {
    app_name := "MyJaiApp";
    project_path := "AndroidProject";
    app_id := "jai.example";
    version_code := 1;
    version_name := "0.1";
    min_sdk_level := 29;
    target_sdk_level := SDK_LEVEL;
    lib_name := "main";
    theme := Android_Default_Theme.Theme_Black_NoTitleBar;
    screen_orientation := Android_Screen_Orientation.UNSPECIFIED;
    vr := false;
    quest_enable_passthrough_splash := false;
    android_manifest: string;
    Meta_Data_Item :: struct {
        name: string;
        value: string;
    }
    application_meta_data: [] Meta_Data_Item;
    permissions: [] string;
    Feature :: struct {
        name: string;
        required: bool;
    }
    features: [] Feature;
    android_gradle_plugin_version := "8.2.0";
    gradle_version := "8.2";
}

Android_Screen_Orientation :: enum {
    UNSPECIFIED;
    LANDSCAPE;
    PORTRAIT;
    USER;
    BEHIND;
    SENSOR;
    NOSENSOR;
    SENSOR_LANDSCAPE;
    SENSOR_PORTRAIT;
    REVERSE_LANDSCAPE;
    REVERSE_PORTRAIT;
    FULL_SENSOR;
    USER_LANDSCAPE;
    USER_PORTRAIT;
    FULL_USER;
    LOCKED;
}

Android_Default_Theme :: enum u16 {
    Theme :: 0x0005;
    Theme_NoTitleBar :: 0x0006;
    Theme_NoTitleBar_Fullscreen :: 0x0007;
    Theme_Black :: 0x0008;
    Theme_Black_NoTitleBar :: 0x0009;
    Theme_Black_NoTitleBar_Fullscreen :: 0x000a;
    Theme_Dialog :: 0x000b;
    Theme_Light :: 0x000c;
    Theme_Light_NoTitleBar :: 0x000d;
    Theme_Light_NoTitleBar_Fullscreen :: 0x000e;
    Theme_Translucent :: 0x000f;
    Theme_Translucent_NoTitleBar :: 0x0010;
    Theme_Translucent_NoTitleBar_Fullscreen :: 0x0011;
}
