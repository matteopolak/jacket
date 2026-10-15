// Reviewed android.md public API. Device embedding supplies a live app and
// named packaged assets. Desktop execution cannot establish these assertions.
A :: #import "Android"(DEFINE_ANDROID_MAIN_AND_CALL_THIS = entry);
Compact :: #import "Android"(USE_GLUE_NATIVE_PORT = false);
#import "Basic";

entry :: () {}

exercise :: (app: *A.Android_App, present: string, absent: string, directory: string) {
    assert(app != null && app.activity != null);
    A.android_app = app;
    assert(A.android_file_exists_in_package(present));
    assert(A.android_file_exists_in_package(sprint("./%", present)));
    assert(!A.android_file_exists_in_package(absent));
    assert(A.android_directory_exists_in_package(directory));
    assert(!A.android_directory_exists_in_package(absent));
    info: Log_Info;

    for flags: Log_Flags.[.NONE, .ERROR, .WARNING, .VERBOSE_ONLY, .VERY_VERBOSE_ONLY] {
        info.common_flags = flags;
        A.android_logger("native log payload", cast(*void) 1, info);
    }
}

// Run in a disposable Android activity/process; the procedure must not return.
finish :: (app: *A.Android_App, status: s32) {
    A.android_app = app;
    A.android_finish_activity_and_exit(status);
    assert(false, "Android finish returned");
}

native_commands :: (app: *A.Android_App) {
    cmd := A.android_app_read_cmd(app);
    A.android_app_pre_exec_cmd(app, cmd);
    A.android_app_post_exec_cmd(app, cmd);
}

// The compact pointer must originate in the external runtime's live native
// record; never pass a standalone compact stack record to its SDK functions.
compact_commands :: (app: *Compact.Android_App) {
    cmd := Compact.android_app_read_cmd(app);
    Compact.android_app_pre_exec_cmd(app, cmd);
    Compact.android_app_post_exec_cmd(app, cmd);
}

main :: () {}
