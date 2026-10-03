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
