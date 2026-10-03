// Generic app state separates public record identity from host pthread ABI.
read_command :: (app: *$T) -> s8 {
    cmd: s8;
    if P.read(app.msgread, *cmd, 1) != 1 return -1;
    if cmd == 12 release_saved_state(app);
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
}

pre_state :: (app: *$T, cmd: s8) {
    if cmd == 1 {
        lock(app);
        app.window = app.pendingWindow;
        signal(app);
        unlock(app);
    } else if cmd == 2 {
        signal(app);
    } else if cmd == 10 || cmd == 11 || cmd == 13 || cmd == 14 {
        lock(app);
        app.activityState = cmd;
        signal(app);
        unlock(app);
    } else if cmd == 15 {
        app.destroyRequested = 1;
    }
}

post_state :: (app: *$T, cmd: s8) {
    if cmd == 2 || cmd == 12 {
        lock(app);
        if cmd == 2 app.window = null;
        else app.stateSaved = 1;

        signal(app);
        unlock(app);
    } else if cmd == 11 {
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
P :: #import "POSIX";
Native :: #import "internal/bindings/native";

libc :: #library,system "c";
abort :: () #foreign libc;

require :: (status: s32) {
    if status != 0 abort();
}
