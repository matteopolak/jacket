// A small VT terminal; see README.md for flags, keys and supported sequences.
#import "Basic";
#import "Extensions/Environment";
#import "String";
#import "Extensions/UI";
#import "Input";
#import,dir "vt";
Pty :: #import "Extensions/Pty";
Args :: #import "Extensions/Args";
#load "terminal.jai";
#load "smoke.jai";

Cli :: struct {
    command: string; @short=c
    smoke: bool;
    frame: string;
}

HELP :: Args.Help(Cli).{
    about = "A small terminal on Extensions/UI, with a real macOS/Linux PTY.",
    command = "Run a command through /bin/sh instead of $SHELL",
    smoke = "Run headless PTY and display-list checks",
    frame = "Write an offscreen PNG during --smoke (requires GPU)",
};

main :: () {
    cli, _ := Args.parse(Cli, HELP);
    if cli.smoke {
        smoke(cli.frame);
        return;
    }

    #if OS != .MACOS && OS != .LINUX {
        print("term: PTYs are supported on macOS and Linux; ConPTY is not implemented\n");
        exit(2);
    }

    shell := environment_variable("SHELL");
    if shell.count == 0 shell = "/bin/sh";
    if cli.command.count > 0 shell = "/bin/sh";

    app: Terminal;
    app.grid = make_grid(80, 24);
    ok: bool;
    app.session, ok = Pty.open_session(shell, cli.command);
    if !ok {
        print("term: cannot open PTY\n");
        exit(1);
    }

    defer close_terminal(*app);
    if !run_app(.{ title = "Jacket terminal", width = 820, height = 540 }, *app, build_terminal) exit(1);
}
