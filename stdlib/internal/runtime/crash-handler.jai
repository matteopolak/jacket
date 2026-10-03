init :: () {
    error := native_init();
    assert(error == 0, "native crash-handler initialization failed: %", error);
}

handle_signal :: (sig: s32, info: *siginfo_t, secret: *void) #c_call {
    native_handle(sig, info, secret);
}

#scope_file
#import "Basic";

native_runtime :: #library "crash-native/portable_crash";
native_init :: () -> s32 #foreign native_runtime "portable_crash_init";
native_handle :: (sig: s32, info: *void, secret: *void) #foreign native_runtime "portable_crash_handle";
