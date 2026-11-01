// Manual-mode Iprof workload. No timing-dependent assertions: the smoke checks structure.
#import "Basic";
Args :: #import "Extensions/Args";
Trace :: #import "Extensions/Iprof_Trace";
I :: #import "Iprof"(IMPORT_MODE = .CLIENT, MANUAL_MODE = true, USE_GRAPHICS = false, DO_HISTORY = false);

Cli :: struct {
    output: string; @positional @required
}

HELP :: Args.Help(Cli).{ about = "Record a small nested Iprof workload", output = "Output .jipt file" };

main :: () {
    cli, _ := Args.parse(Cli, HELP);
    r: Trace.Recorder;

    I.Prof_init_highlevel();

    for name, i: string.["frame", "work", "leaf"] {
        I.zones[i].name = name;
        I.zones[i].initted = true;
    }

    Trace.start_capture(*r);
    checksum: u64;

    for frame: 0..3 {
        I.zone_begin(0);

        for 0..frame {
            I.zone_begin(1);
            I.zone_begin(2);
            for n: 0..10000 checksum += cast(u64) n;

            I.zone_end();
            I.zone_end();
        }

        I.zone_end();
        I.update(true);
    }

    Trace.stop_capture(*r);
    defer Trace.free_trace(*r.trace);
    if !Trace.save_trace(*r.trace, cli.output) exit(1);

    print("recorded frames=% zones=% checksum=%\n", r.trace.frames.count, r.trace.names.count, checksum);
}
