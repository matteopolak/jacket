// test: only(.MACOS)
// create_app_bundle copies the executable into Contents/MacOS, creates Contents/Resources, and writes an Info.plist with the name, executable and package type.
#import "Basic";
F :: #import "File";
P :: #import "Process";
M :: #import "MacOS_Bundler";
#import "Extensions/Environment";

command :: (args: ..string) {
    status := P.run_command(..args);
    assert(status.type == .EXITED && status.exit_code == 0);
}

main :: () {
    root := output_path("macos-bundler/layout");

    command("/bin/rm", "-rf", root);
    command("/bin/mkdir", "-p", root);

    exe := sprint("%/tool", root);

    assert(F.write_entire_file(exe, "payload"));
    command("/bin/chmod", "+x", exe);

    app, resources := M.create_app_bundle(sprint("%/Sample", root), exe, "", "", false, true);

    assert(app == sprint("%/Sample.app", root));
    assert(resources == sprint("%/Sample.app/Contents/Resources", root));

    copied, copied_ok := F.read_entire_file(sprint("%/Contents/MacOS/tool", app));

    assert(copied_ok);
    print("executable copy: %\n", copied);

    command("/bin/test", "-x", sprint("%/Contents/MacOS/tool", app));
    print("executable bit kept: true\n");

    print("resources writable: %\n", F.write_entire_file(sprint("%/marker.txt", resources), "x"));

    plist, plist_ok := F.read_entire_file(sprint("%/Contents/Info.plist", app));

    assert(plist_ok);
    print("%", plist);
}
