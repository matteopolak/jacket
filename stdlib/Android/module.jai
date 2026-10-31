#module_parameters()(
    // A procedure to run as the app's main. When set, the module defines the Android entry points and calls it.
    DEFINE_ANDROID_MAIN_AND_CALL_THIS: () -> () = null,
    // Waits until the app is in the foreground before calling the main procedure.
    WAIT_TILL_APP_IS_IN_FOREGROUND_TO_CALL_MAIN := true,
    // Uses the module's own native-activity glue.
    USE_GLUE_NATIVE_PORT := true,
    // Warns when an asset hides another with the same name.
    WARN_ABOUT_SHADOWED_ASSETS := true,
    // Includes the bindings for the NDK's media APIs.
    INCLUDE_MEDIA_BINDINGS := false,
    // Includes the bindings for the NDK's camera API.
    INCLUDE_CAMERA_BINDINGS := false
);

#assert OS == .ANDROID, "Android requires its documented native platform";

#load "declarations.jai";
#load "wrappers.jai";
