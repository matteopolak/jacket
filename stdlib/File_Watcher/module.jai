// File_Watcher: being told when files in a folder are created, changed, moved or removed.
//
// A `File_Watcher` watches folders (and, by default, everything below them) through the operating system's own
// notifications: inotify on Linux, FSEvents on macOS and `ReadDirectoryChangesW` on Windows. Nothing runs in the
// background for you: call `process_changes` from your own loop and it calls your callback for each change that
// has settled. Changes to one path within a short window are merged into one `File_Change` with the events combined.
// Other targets do not have this module.
//
// ```jai,skip_on(wasm)
// on_change :: (watcher: *File_Watcher(void), change: *File_Change, data: *void) {}
//
// watcher: File_Watcher;
// assert(!init(*watcher, on_change, merge_window_seconds = -1)); // refused, nothing to release
// ```

#if OS == .MACOS {
    #load "../internal/watching/mac.jai";
} else #if OS == .LINUX || OS == .WINDOWS {
    #load "../internal/watching/portable-model.jai";
    #load "../internal/watching/portable.jai";
    #if OS == .LINUX #load "../internal/watching/inotify.jai";
    #if OS == .WINDOWS #load "../internal/watching/windows.jai";
} else {
    #assert false, "File_Watcher backend is unavailable for this target";
}
