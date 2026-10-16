#import "Basic";
W :: #import "File_Watcher";

changed :: (_watcher: *W.File_Watcher(void), _change: *W.File_Change, _data: *void) {}

main :: () {
    watcher: W.File_Watcher;

    #if OS == .WINDOWS {
        assert(watcher.directories.count == 0);
        #assert(!#exists(watcher.event_streams));
        #assert(!#exists(watcher.inotify_instance));
        #assert(!#exists(W.Directory_Info));
    } else #if OS == .LINUX {
        assert(watcher.inotify_instance == -1 && watcher.inotify_data == null);
        assert(watcher.inotify_bytes_read == 0 && watcher.inotify_cursor == 0);
        assert(watcher.path_pool == null && watcher.watch_paths.count == 0);
        #assert(!#exists(watcher.event_streams));
        #assert(!#exists(watcher.directories));
    } else #if OS == .MACOS {
        assert(watcher.event_streams.count == 0);
        #assert(!#exists(watcher.inotify_instance));
        #assert(!#exists(watcher.directories));
    }

    assert(W.init(*watcher, changed));
    W.add_directories(*watcher, ".");
    W.process_changes(*watcher);
    W.deinit(watcher);
    print("ok\n");
}
