#import "Basic";
#import "String";
A :: #import "Android/Toolchain";
F :: #import "File";

main :: () {
    config: A.Android_Project_Config;

    assert(config.target_sdk_level == 34);
    assert(A.android_theme_to_string(config.theme) == "@android:style/Theme.Black.NoTitleBar");
    config.project_path = "stdlib/Android/Toolchain/tests/generated";
    config.app_name = "App & <Name>";
    config.app_id = "test.project";
    config.permissions = string.["test.permission.quoted\""];
    config.application_meta_data = A.Android_Project_Config.Meta_Data_Item.[.{ "a&b", "<value>" }];
    config.features = A.Android_Project_Config.Feature.[.{ "test.feature", true }];
    assert(A.generate_android_project(config));
    manifest, ok := F.read_entire_file("stdlib/Android/Toolchain/tests/generated/app/src/main/AndroidManifest.xml");

    assert(ok);
    assert(contains(manifest, "App &amp; &lt;Name&gt;"));
    assert(contains(manifest, "test.permission.quoted&quot;"));
    assert(contains(manifest, "android:required=\"true\""));
    assert(contains(manifest, "android.app.lib_name"));
    assert(!A.generate_android_project(config, false));
    assert(F.write_entire_file("stdlib/Android/Toolchain/tests/generated/.idea/workspace.xml", "keep"));
    config.android_manifest = "<manifest/>";
    assert(A.generate_android_project(config));
    override, override_ok := F.read_entire_file("stdlib/Android/Toolchain/tests/generated/app/src/main/AndroidManifest.xml");

    assert(override_ok && override == "<manifest/>");
    workspace, workspace_ok := F.read_entire_file("stdlib/Android/Toolchain/tests/generated/.idea/workspace.xml");

    assert(workspace_ok && workspace == "keep");
    print("Android project contract passed\n");
}

contains :: (text: string, needle: string) -> bool {
    if needle.count > text.count return false;

    for i: 0..text.count - needle.count {
        if slice(text, i, needle.count) == needle return true;
    }

    return false;
}
