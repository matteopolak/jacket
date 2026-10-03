Android_Config_Changes :: enum_flags s32 {
    // Display geometry, appearance and language.
    orientation :: 0x0080;
    screenLayout :: 0x0100;
    screenSize :: 0x0400;
    smallestScreenSize :: 0x0800;
    density :: 0x1000;
    colorMode :: 0x4000;
    fontScale :: 0x40000000;
    uiMode :: 0x0200;
    locale :: 0x0004;
    layoutDirection :: 0x2000;
    // Input peripherals and network configuration.
    touchscreen :: 0x0008;
    keyboard :: 0x0010;
    keyboardHidden :: 0x0020;
    navigation :: 0x0040;
    mcc :: 0x0001;
    mnc :: 0x0002;
}

android_config_changes_from_string :: (s: string) -> Android_Config_Changes {
    result: Android_Config_Changes;
    start: s64;

    if s.count == 0 return result;

    for end: 0..s.count {
        if end != s.count && s[end] != #char "|" continue;

        word := string.{ end - start, s.data + start };
        flag: Android_Config_Changes;

        if word == "orientation" flag = .orientation;
        else if word == "screenLayout" flag = .screenLayout;
        else if word == "screenSize" flag = .screenSize;
        else if word == "smallestScreenSize" flag = .smallestScreenSize;
        else if word == "density" flag = .density;
        else if word == "colorMode" flag = .colorMode;
        else if word == "fontScale" flag = .fontScale;
        else if word == "uiMode" flag = .uiMode;
        else if word == "locale" flag = .locale;
        else if word == "layoutDirection" flag = .layoutDirection;
        else if word == "touchscreen" flag = .touchscreen;
        else if word == "keyboard" flag = .keyboard;
        else if word == "keyboardHidden" flag = .keyboardHidden;
        else if word == "navigation" flag = .navigation;
        else if word == "mcc" flag = .mcc;
        else if word == "mnc" flag = .mnc;
        else {
            context.logger("invalid Android configuration change name\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
            assert(false, "invalid Android configuration change name");
            return cast(Android_Config_Changes) 0;
        }

        result |= flag;
        start = end + 1;
    }

    return result;
}

#scope_module
#import "Basic";
