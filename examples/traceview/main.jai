// Native Iprof timeline viewer; --smoke renders the same UI without a window or GPU.
#import "Basic";
#import "String";
#import "File";
Images :: #import "Extensions/Image";
#import "Extensions/UI";
Args :: #import "Extensions/Args";
Trace :: #import "Extensions/Iprof_Trace";
Model :: #import,dir "model";

Cli :: struct {
    path: string; @positional @required
    smoke: bool;
    output: string = "traceview.png";
    selected: s64; @range(0,1000000)
    compare: s64; @range(0,1000000)
    search: string;
}

HELP :: Args.Help(Cli).{
    about = "Browse Iprof frames, nested calls and zone statistics",
    path = "Trace recorded with Extensions/Iprof_Trace",
    smoke = "Render offscreen and exercise controls, then exit",
    output = "Offscreen PNG path",
    selected = "Initial frame index (zero based)",
    compare = "Baseline frame index",
    search = "Initial case-sensitive zone name filter",
};

App :: struct {
    trace: Trace.Trace;
    selected: s64;
    baseline: s64;
    column: Model.Sort_Column = .TOTAL;
    descending: bool = true;
    search: Text_Edit;
}

#load "view.jai";
#load "smoke.jai";

main :: () {
    cli, _ := Args.parse(Cli, HELP);
    trace, ok := Trace.load_trace(cli.path);

    if !ok {
        print("traceview: invalid or unreadable trace\n", to_standard_error = true);
        exit(2);
    }

    defer Trace.free_trace(*trace);

    if trace.frames.count == 0 || cli.selected >= trace.frames.count || cli.compare >= trace.frames.count {
        print("traceview: frame index outside trace\n", to_standard_error = true);
        exit(2);
    }

    app := App.{ trace = trace, selected = cli.selected, baseline = cli.compare };
    app.search.single_line = true;
    set_contents(*app.search, cli.search);
    defer free_text_edit(*app.search);

    if cli.smoke {
        if !smoke(*app, cli.output) exit(1);
    } else if !run_app(.{ title = "Traceview — Iprof", width = 1100, height = 800 }, *app, build) exit(1);
}
