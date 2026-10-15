// Reviewed Android/Toolchain Android_Config_Changes values; SDK pipe grammar.
#import "Basic";
A :: #import "Android/Toolchain";

main :: () {
    assert(size_of(A.Android_Config_Changes) == 4);
    assert(type_info(A.Android_Config_Changes).name == "Android_Config_Changes");
    names := string.["mcc", "mnc", "locale", "touchscreen", "keyboard", "keyboardHidden", "navigation", "orientation", "screenLayout", "uiMode", "screenSize", "smallestScreenSize", "density", "layoutDirection", "colorMode", "fontScale"];
    for name, index: names {
        expected := ifx index < 15 then cast(s32) (1 << index) else 0x40000000;

        assert(cast(s32) A.android_config_changes_from_string(name) == expected);

        for other, other_index: names {
            builder: String_Builder;

            append(*builder, name);
            append(*builder, "|");
            append(*builder, other);
            text := builder_to_string(*builder);

            free_buffers(*builder);
            other_value := ifx other_index < 15 then cast(s32) (1 << other_index) else 0x40000000;

            assert(cast(s32) A.android_config_changes_from_string(text) == (expected | other_value));
            free(text);
        }
    }

    assert(cast(s32) A.android_config_changes_from_string("") == 0);
    assert(cast(s32) A.android_config_changes_from_string("orientation|screenSize|smallestScreenSize|screenLayout") == 0xd80);
    print("ok\n");
}
