// test: only(.MACOS)
// With only_copy_subdirectories_and_files the resource folder's contents land directly in Contents/Resources; otherwise the folder itself is copied, and an .icns icon is copied as Icon.icns.
#import "Basic";
F :: #import "File";
P :: #import "Process";
M :: #import "MacOS_Bundler";
#import "Extensions/Environment";

command :: (args: ..string) {
    status := P.run_command(..args);
    assert(status.type == .EXITED && status.exit_code == 0);
}

read_text :: (path: string) -> string, bool {
    return F.read_entire_file(path);
}

main :: () {
    root := output_path("macos-bundler/resources");

    command("/bin/rm", "-rf", root);
    command("/bin/mkdir", "-p", root);

    exe := sprint("%/tool", root);

    assert(F.write_entire_file(exe, "payload"));
    command("/bin/chmod", "+x", exe);

    source := sprint("%/resources", root);

    command("/bin/mkdir", "-p", sprint("%/sub", source));
    assert(F.write_entire_file(sprint("%/top.txt", source), "top"));
    assert(F.write_entire_file(sprint("%/sub/inner.txt", source), "inner"));
    assert(F.write_entire_file(sprint("%/.hidden", source), "hidden"));

    icon := sprint("%/Source.icns", root);

    assert(F.write_entire_file(icon, "icns-bytes"));

    flat, flat_resources := M.create_app_bundle(sprint("%/Flat", root), exe, icon, source, true, false);

    assert(flat_resources == sprint("%/Contents/Resources", flat));

    text, ok := read_text(sprint("%/top.txt", flat_resources));

    print("flat top: % %\n", ok, text);
    text, ok = read_text(sprint("%/sub/inner.txt", flat_resources));
    print("flat nested: % %\n", ok, text);
    text, ok = read_text(sprint("%/.hidden", flat_resources));
    print("flat hidden: % %\n", ok, text);
    text, ok = read_text(sprint("%/resources/top.txt", flat_resources));
    print("flat extra folder present: %\n", ok);
    text, ok = read_text(sprint("%/Icon.icns", flat_resources));
    print("flat icon: % %\n", ok, text);
    _, flat_plist_ok := read_text(sprint("%/Contents/Info.plist", flat));

    print("flat plist written: %\n", flat_plist_ok);

    tree, tree_resources := M.create_app_bundle(sprint("%/Tree", root), exe, icon, source, false, true);

    assert(tree_resources == sprint("%/Contents/Resources", tree));

    text, ok = read_text(sprint("%/resources/top.txt", tree_resources));
    print("tree top: % %\n", ok, text);
    text, ok = read_text(sprint("%/resources/sub/inner.txt", tree_resources));
    print("tree nested: % %\n", ok, text);
    text, ok = read_text(sprint("%/Icon.icns", tree_resources));
    print("tree icon: % %\n", ok, text);
    tree_plist, tree_plist_ok := read_text(sprint("%/Contents/Info.plist", tree));

    print("tree plist: % %\n", tree_plist_ok, tree_plist);
}
