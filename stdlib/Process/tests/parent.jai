#import "Basic";
C :: #import "Process";

KEEP :: #exists(PROCESS_KEEP_ALIVE);

main :: () {
    command := "sleep 0.2; printf retained > process-killed-marker.tmp";

    #if KEEP command = "sleep 0.2; printf retained > process-retained-marker.tmp";
    process: C.Process;

    assert(C.create_process(*process, "/bin/sh", "-c", command, kill_process_if_parent_exits = !KEEP));
    other: C.Process;

    assert(C.create_process(*other, "/bin/sleep", "0.4", kill_process_if_parent_exits = false));
    // Intentionally let the parent terminate with a live process. The runner
    // checks the OS-visible result after this source-built program exits.
    print("ok\n");
}
