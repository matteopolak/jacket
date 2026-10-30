// The Jacket text editor. `jaic examples/editor/main.jai -interp -- [folder or file]` opens it on
// a folder (the current one by default) or on a file's folder with the file open. See
// docs/extensions/ui-editor.md.

#load "editor.jai";

main :: () {
    args := get_command_line_arguments();
    target := ifx args.count > 1 then args[1] else ".";
    root := target;
    file := "";

    is_dir, ok := is_directory(target);

    if ok && !is_dir {
        file = target;
        root = parent_directory(target);
    } else if !ok {
        print("%: no such file or folder\n", target);
        exit(1);
    }

    root = absolute_path(root);

    ed: Editor;

    open_editor(*ed, root, file);
    defer close_editor(*ed);

    if !run_app(.{ title = "Jacket editor", width = 1100, height = 720 }, *ed, build) exit(1);
}
