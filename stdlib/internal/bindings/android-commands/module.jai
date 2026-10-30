// Generic app state separates public record identity from host pthread ABI.
read_command :: (app: *$T) -> s8 {
    cmd: s8;
    if P.read(app.msgread, *cmd, 1) != 1 return -1;
    if cmd == COMMAND_SAVE_STATE release_saved_state(app);
    return cmd;
}

release_saved_state :: (app: *$T) {
    lock(app);
    defer unlock(app);

    if app.savedState {
        Native.release(app.savedState);
        app.savedState = null;
        app.savedStateSize = 0;
    }
} @FreesArgument

pre_state :: (app: *$T, cmd: s8) {
    if cmd == COMMAND_INIT_WINDOW {
        lock(app);
        app.window = app.pendingWindow;
        signal(app);
        unlock(app);
    } else if cmd == COMMAND_TERM_WINDOW {
        signal(app);
    } else if cmd == COMMAND_START || cmd == COMMAND_RESUME || cmd == COMMAND_PAUSE || cmd == COMMAND_STOP {
        lock(app);
        app.activityState = cmd;
        signal(app);
        unlock(app);
    } else if cmd == COMMAND_DESTROY {
        app.destroyRequested = 1;
    }
}

post_state :: (app: *$T, cmd: s8) {
    if cmd == COMMAND_TERM_WINDOW || cmd == COMMAND_SAVE_STATE {
        lock(app);
        if cmd == COMMAND_TERM_WINDOW app.window = null;
        else app.stateSaved = 1;

        signal(app);
        unlock(app);
    } else if cmd == COMMAND_RESUME {
        release_saved_state(app);
    }
}

lock :: (app: *$T) {
    require(P.pthread_mutex_lock(*app.mutex));
}

unlock :: (app: *$T) {
    require(P.pthread_mutex_unlock(*app.mutex));
}

signal :: (app: *$T) {
    require(P.pthread_cond_broadcast(*app.cond));
}

#scope_file

COMMAND_INIT_WINDOW :: 1;
COMMAND_TERM_WINDOW :: 2;
COMMAND_START :: 10;
COMMAND_RESUME :: 11;
COMMAND_SAVE_STATE :: 12;
COMMAND_PAUSE :: 13;
COMMAND_STOP :: 14;
COMMAND_DESTROY :: 15;

P :: #import "internal/bindings/posix-api";
Native :: #import "internal/bindings/native";

libc :: #library,system "c"; @native(external)
abort :: () #foreign libc;

require :: (status: s32) {
    if status != 0 abort();
}
