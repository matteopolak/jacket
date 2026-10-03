// Synchronous caller context borrowing; callers must restore on every exit.
enter :: (context: *void) -> *void #foreign bridge "binding_callback_context_enter";
leave :: (previous: *void) #foreign bridge "binding_callback_context_leave";
current :: () -> *void #foreign bridge "binding_callback_context_current";

#scope_file
bridge :: #library,system "binding_callback_context";
