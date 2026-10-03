#if USE_GLUE_NATIVE_PORT {
    android_app_read_cmd :: (android_app: *Android_App) -> s8 {
        return Commands.read_command(android_app);
    }

    android_app_pre_exec_cmd :: (android_app: *Android_App, cmd: s8) {
        if cmd == cast(s8) APP_CMD.INPUT_CHANGED {
            Commands.lock(android_app);
            defer Commands.unlock(android_app);
            if android_app.inputQueue AInputQueue_detachLooper(android_app.inputQueue);

            android_app.inputQueue = android_app.pendingInputQueue;
            if android_app.inputQueue {
                AInputQueue_attachLooper(android_app.inputQueue, android_app.looper, cast(s32) LOOPER_ID.INPUT,
                                        null, *android_app.inputPollSource);
            }

            Commands.signal(android_app);
        } else if cmd == cast(s8) APP_CMD.CONFIG_CHANGED {
            AConfiguration_fromAssetManager(android_app.config, android_app.activity.assetManager);
        } else {
            Commands.pre_state(android_app, cmd);
        }

        if cmd == cast(s8) APP_CMD.INIT_WINDOW && android_app.window binding_received_window = true;
    }

    android_app_post_exec_cmd :: (android_app: *Android_App, cmd: s8) {
        Commands.post_state(android_app, cmd);
    }
}

#if !USE_GLUE_NATIVE_PORT {
    // Only a borrowed external-runtime app may enter this SDK delegate. The
    // compact public record is a view, never storage allocated for the SDK.
    android_app_pre_exec_cmd :: (android_app: *Android_App, cmd: s8) -> void #c_call {
        binding_sdk_pre_exec_cmd(android_app, cmd);
        if cmd == cast(s8) APP_CMD.INIT_WINDOW && android_app.window binding_received_window = true;
    }
}

android_file_exists_in_package :: (name: string) -> bool {
    app := binding_app();
    terminated := Native.terminated_copy(Policy.package_name(name));

    defer Native.release(terminated);
    asset := AAssetManager_open(app.activity.assetManager, terminated, .UNKNOWN);
    if !asset return false;

    AAsset_close(asset);

    return true;
}

android_directory_exists_in_package :: (name: string) -> bool {
    app := binding_app();
    terminated := Native.terminated_copy(name);

    defer Native.release(terminated);
    directory := AAssetManager_openDir(app.activity.assetManager, terminated);
    if !directory return false;

    defer AAssetDir_close(directory);

    return AAssetDir_getNextFileName(directory) != null;
}

android_finish_activity_and_exit :: (status := cast(s32) 0) {
    app := binding_app();
    if binding_received_window || app.window {
        binding_exit(0);
        binding_abort();
    }

    ANativeActivity_finish(app.activity);

    while app.destroyRequested == 0 {
        source: *Android_Poll_Source;
        events: s32;
        result := ALooper_pollAll(-1, null, *events, cast(**void) *source);

        if result == -4 binding_abort();
        if source source.process(app, source);
    }

    binding_exit(status);
    binding_abort();
}

android_logger :: (message: string, #discard data: *void, info: Log_Info) {
    Log.write(message, info, binding_log_sink);
}

#scope_file
Commands :: #import "internal/bindings/android-commands";
Policy :: #import "internal/bindings/android-policy";
Native :: #import "internal/bindings/native";
Basic :: #import "Basic";

#if OS == .ANDROID {
    Log :: #import "internal/bindings/android-log";
}

libc :: #library,system "c";
binding_exit :: (status: s32) #foreign libc "exit";
binding_abort :: () #foreign libc "abort";

#if !USE_GLUE_NATIVE_PORT {
    binding_glue :: #library,system "android_native_app_glue";
    binding_sdk_pre_exec_cmd :: (app: *Android_App, cmd: s8) #foreign binding_glue "android_app_pre_exec_cmd";
}

binding_received_window: bool;

binding_log_sink :: (priority: s32, tag: *u8, message: *u8) -> s32 #c_call {
    return __android_log_write(cast(Log_Priority) priority, tag, message);
}

binding_app :: () -> *Android_App {
    app: *Android_App;

    #if DEFINE_ANDROID_MAIN_AND_CALL_THIS {
        app = android_app;
    }

    Basic.assert(app != null && app.activity != null, "Android app/activity is not initialized");
    if !app || !app.activity binding_abort();

    return app;
}
