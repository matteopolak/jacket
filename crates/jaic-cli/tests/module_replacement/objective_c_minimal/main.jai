// The library's native macOS code with a project-owned Objective_C that has almost nothing in it.
#import "Basic";
#import "Clipboard";
#import "File_Watcher";
#import "Thread";
#import "Process";
Native :: #import "Objective_C";

main :: () {
    class := Native.Project_Class.{ name = "project" };
    watcher: File_Watcher;
    thread: Thread;

    print("% % %\n", Native.describe(class), os_clipboard_get_text(), run_command("true"));
    print("% %\n", watcher.verbose, thread.index);
}
