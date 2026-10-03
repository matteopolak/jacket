#module_parameters ()(
    DEFINE_ANDROID_MAIN_AND_CALL_THIS: () -> () = null,
    WAIT_TILL_APP_IS_IN_FOREGROUND_TO_CALL_MAIN := true,
    USE_GLUE_NATIVE_PORT := true,
    WARN_ABOUT_SHADOWED_ASSETS := true,
    INCLUDE_MEDIA_BINDINGS := false,
    INCLUDE_CAMERA_BINDINGS := false
);

#assert OS == .ANDROID, "Android requires its documented native platform";

#load "declarations.jai";
#load "wrappers.jai";
