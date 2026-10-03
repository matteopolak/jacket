#scope_module

android_project_escape :: (text: string, xml := true) -> string {
    b: String_Builder;

    defer free_buffers(*b);

    for c: text {
        if xml {
            if c == 38 append(*b, "&amp;");
            else if c == 60 append(*b, "&lt;");
            else if c == 62 append(*b, "&gt;");
            else if c == 34 append(*b, "&quot;");
            else if c == 39 append(*b, "&apos;");
            else append(*b, c);
        } else {
            if c == 92 append(*b, "\\\\");
            else if c == 39 append(*b, "\\'");
            else if c == 10 append(*b, "\\n");
            else if c == 13 append(*b, "\\r");
            else append(*b, c);
        }
    }

    return builder_to_string(*b);
}

android_project_orientation :: (orientation: Android_Screen_Orientation) -> string {
    values := string.["unspecified", "landscape", "portrait", "user", "behind", "sensor", "nosensor", "sensorLandscape", "sensorPortrait", "reverseLandscape", "reversePortrait", "fullSensor", "userLandscape", "userPortrait", "fullUser", "locked"];
    index := cast(int) orientation;
    if index < 0 || index >= values.count return "unspecified";
    return values[index];
}

android_project_manifest :: (config: Android_Project_Config) -> string {
    b: String_Builder;

    defer free_buffers(*b);
    app := android_project_escape(config.app_id);
    name := android_project_escape(config.app_name);
    library := android_project_escape(config.lib_name);
    version := android_project_escape(config.version_name);

    defer free(app);
    defer free(name);
    defer free(library);
    defer free(version);
    print_to_builder(*b, "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<manifest xmlns:android=\"http://schemas.android.com/apk/res/android\" package=\"%\" android:versionCode=\"%\" android:versionName=\"%\">\n", app, config.version_code, version);
    print_to_builder(*b, "  <uses-sdk android:minSdkVersion=\"%\" android:targetSdkVersion=\"%\"/>\n", config.min_sdk_level, config.target_sdk_level);

    for permission: config.permissions {
        escaped := android_project_escape(permission);
        print_to_builder(*b, "  <uses-permission android:name=\"%\"/>\n", escaped);
        free(escaped);
    }

    for feature: config.features {
        escaped := android_project_escape(feature.name);
        required := "false";

        if feature.required required = "true";

        print_to_builder(*b, "  <uses-feature android:name=\"%\" android:required=\"%\"/>\n", escaped, required);
        free(escaped);
    }

    print_to_builder(*b, "  <application android:label=\"%\" android:hasCode=\"false\" android:theme=\"%\">\n", name, android_theme_to_string(config.theme));

    for item: config.application_meta_data {
        key := android_project_escape(item.name);
        value := android_project_escape(item.value);

        print_to_builder(*b, "    <meta-data android:name=\"%\" android:value=\"%\"/>\n", key, value);
        free(key);
        free(value);
    }

    if config.vr append(*b, "    <meta-data android:name=\"com.samsung.android.vr.application.mode\" android:value=\"vr_only\"/>\n");
    if config.quest_enable_passthrough_splash append(*b, "    <meta-data android:name=\"com.oculus.ossplash.background\" android:value=\"passthrough\"/>\n");

    print_to_builder(*b, "    <activity android:name=\"android.app.NativeActivity\" android:exported=\"true\" android:screenOrientation=\"%\" android:configChanges=\"orientation|screenSize|keyboardHidden\">\n", android_project_orientation(config.screen_orientation));
    print_to_builder(*b, "      <meta-data android:name=\"android.app.lib_name\" android:value=\"%\"/>\n", library);
    append(*b, "      <intent-filter><action android:name=\"android.intent.action.MAIN\"/><category android:name=\"android.intent.category.LAUNCHER\"/></intent-filter>\n    </activity>\n  </application>\n</manifest>\n");

    return builder_to_string(*b);
}

android_project_write :: (root: string, relative: string, data: string) -> bool {
    path := tool_path_join(root, relative);
    defer free(path);
    return Project_File.write_entire_file(path, data);
}

android_project_write :: (root: string, relative: string, builder: *String_Builder) -> bool {
    path := tool_path_join(root, relative);
    defer free(path);
    return Project_File.write_entire_file(path, builder, false);
}

android_project_generate :: (config: Android_Project_Config, recreate: bool) -> bool {
    if config.project_path.count == 0 return false;

    existing := tool_path_join(config.project_path, "app/build.gradle");

    defer free(existing);
    if !recreate && Project_Files.file_exists(existing) return false;

    directories := string.["app/src/main/jniLibs", "app/src/main/assets", "gradle/wrapper", ".idea"];
    for directory: directories {
        path := tool_path_join(config.project_path, directory);
        ok := Project_File.make_directory_if_it_does_not_exist(path, true);
        free(path);
        if !ok return false;
    }

    app_id := android_project_escape(config.app_id, false);
    app_name := android_project_escape(config.app_name, false);
    version := android_project_escape(config.version_name, false);
    plugin := android_project_escape(config.android_gradle_plugin_version, false);

    defer free(app_id);
    defer free(app_name);
    defer free(version);
    defer free(plugin);
    settings: String_Builder;

    defer free_buffers(*settings);
    append(*settings, "pluginManagement { repositories { google(); mavenCentral(); gradlePluginPortal() } }\ndependencyResolutionManagement { repositories { google(); mavenCentral() } }\n");
    print_to_builder(*settings, "rootProject.name = '%'\ninclude ':app'\n", app_name);
    if !android_project_write(config.project_path, "settings.gradle", *settings) return false;

    build: String_Builder;

    defer free_buffers(*build);
    print_to_builder(*build, "plugins { id 'com.android.application' version '%' apply false }\n", plugin);
    if !android_project_write(config.project_path, "build.gradle", *build) return false;

    reset(*build);
    print_to_builder(*build, "plugins { id 'com.android.application' }\nandroid {\n    namespace '%'\n    compileSdk %\n    defaultConfig {\n        applicationId '%'\n        minSdk %\n        targetSdk %\n        versionCode %\n        versionName '%'\n    }\n    buildTypes { release { minifyEnabled false } }\n}\n", app_id, config.target_sdk_level, app_id, config.min_sdk_level, config.target_sdk_level, config.version_code, version);
    if !android_project_write(config.project_path, "app/build.gradle", *build) return false;

    manifest := config.android_manifest;
    generated := manifest.count == 0;

    if generated manifest = android_project_manifest(config);

    defer if generated free(manifest);
    if !android_project_write(config.project_path, "app/src/main/AndroidManifest.xml", manifest) return false;
    if !android_project_write(config.project_path, "gradle.properties", "android.useAndroidX=true\n") return false;

    reset(*build);
    print_to_builder(*build, "distributionBase=GRADLE_USER_HOME\ndistributionPath=wrapper/dists\ndistributionUrl=https\\://services.gradle.org/distributions/gradle-%-bin.zip\nzipStoreBase=GRADLE_USER_HOME\nzipStorePath=wrapper/dists\n", config.gradle_version);
    if !android_project_write(config.project_path, "gradle/wrapper/gradle-wrapper.properties", *build) return false;

    workspace := tool_path_join(config.project_path, ".idea/workspace.xml");

    defer free(workspace);

    if !Project_Files.file_exists(workspace) {
        if !Project_File.write_entire_file(workspace, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<project version=\"4\"/>\n") return false;
    }

    return true;
}

#scope_file
Project_File :: #import "File";
Project_Files :: #import "File_Utilities";
