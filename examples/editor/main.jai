// The Jacket text editor. `jaic examples/editor/main.jai -interp -- [folder or file]` opens it on
// a folder (the current one by default) or on a file's folder with the file open. See
// docs/extensions/ui-editor.md.

#load "smoke-session.jai";
Args :: #import "Extensions/Args";

Cli :: struct {
    target: string = "."; @positional
    smoke: bool;
}

HELP :: Args.Help(Cli).{
    about = "Jacket editor with jailsp language features. Set JAI_EDITOR_SERVER to a server path or off.",
    target = "Folder or file to open (default: current directory)",
    smoke = "Run the non-interactive headless editor and live LSP checks",
};

main :: () {
    cli, _ := Args.parse(Cli, HELP);
    if cli.smoke {
        run_smoke(environment_variable("EDITOR_SMOKE_GPU") == "1");
        return;
    }

    target := absolute_path(cli.target);
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

    if !run_app(.{ title = "Jacket editor", menus = editor_menus, width = 1100, height = 720 }, *ed, build) exit(1);
}
