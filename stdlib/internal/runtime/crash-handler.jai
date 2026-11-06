init :: () {
    error := native_init();
    assert(error == 0, "native crash-handler initialization failed: %", error);
}

// Gives the calling thread an alternate signal stack. Does nothing until `init` has run.
attach_thread :: () #c_call {
    native_attach();
}

handle_signal :: (sig: s32, info: *siginfo_t, secret: *void) #c_call {
    native_handle(sig, info, secret);
}

#scope_file
#import "Basic";

native_runtime :: #library "crash-native/portable_crash"; @native(embedded)
// Initializes only native handler/TLS state and storage it allocates itself.
// Fresh allocation instances unreachable by this call keep their bytes and
// lifetime; no aliases to those instances are published, including by callbacks.
// This does not promise purity or preservation of already published storage.
native_init :: () -> s32 #foreign native_runtime "portable_crash_init"; @PreservesUnpassedStorage
native_attach :: () -> s32 #foreign native_runtime "portable_crash_attach_thread";
native_handle :: (sig: s32, info: *void, secret: *void) #foreign native_runtime "portable_crash_handle";
