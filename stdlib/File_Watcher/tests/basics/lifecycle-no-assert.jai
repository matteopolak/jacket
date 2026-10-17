// Reviewed lifecycle API; essential lock release must survive optional assertions.
#import "Basic";
F :: #import "File_Watcher";

receive :: (watcher: *F.File_Watcher, change: *F.File_Change, user_data: *void) {}

main :: () {
    for iteration: 0..7 {
        watcher: F.File_Watcher;
        if !F.init(*watcher, receive) exit(2);

        changed, waiting, delay := F.process_changes(*watcher);
        if changed || waiting || delay != 0 exit(3);

        F.deinit(watcher);
    }

    print("ok\n");
}
