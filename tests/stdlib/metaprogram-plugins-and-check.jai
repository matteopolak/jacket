// Metaprogram_Plugins and Check, driven from a #run: argument splitting, build_with_plugins
// calling every hook in order, option delivery through init and handle_one_option, and the
// Check plugin's format-argument and C-porting checks on a child workspace. Compile-time
// only: build_with_plugins turns off this program's own output, as a first metaprogram's would.
#import "Basic";
#import "Compiler";
#import "Metaprogram_Plugins";
#import "String";
Check :: #import "Check";

GOOD_SOURCE :: #string JAI
#import "Basic";
copy_it :: (a: [4] int) -> int { return a[0]; }
main :: () {
    numbers: [4] int;
    print("% and %\n", 1, copy_it(numbers));
}
JAI

// Check looks at procedures noted @PrintLike; these calls give as many arguments as they use.
PRINT_LIKE :: #string JAI
say :: (format: string, args: ..Any) {} @PrintLike
main :: () {
    say("% and %%", 1);
    say("%1 %1 %2", 1, 2);
}
JAI

Recorder :: struct {
    using base: Metaprogram_Plugin;
    calls: [..] string;
    options_seen: [..] string;
    messages: int;
    completed: bool;
    source: string;
}

record :: (p: *Metaprogram_Plugin, what: string) {
    array_add(*(cast(*Recorder) p).calls, what);
}

make_recorder :: (source: string, with_init: bool) -> *Recorder {
    r := New(Recorder);
    r.source = source;
    r.before_intercept = (p: *Metaprogram_Plugin, flags: *Intercept_Flags) {
        record(p, "before_intercept");
    };
    r.add_source = (p: *Metaprogram_Plugin) {
        record(p, "add_source");
        options := get_build_options(p.workspace);
        options.output_type = .NO_OUTPUT;
        set_build_options(options, p.workspace);
        add_build_string((cast(*Recorder) p).source, p.workspace);
    };
    r.message = (p: *Metaprogram_Plugin, message: *Message) {
        r := cast(*Recorder) p;
        r.messages += 1;
        if message.kind == .COMPLETE r.completed = true;
    };
    r.finish = (p: *Metaprogram_Plugin) {
        record(p, "finish");
    };
    r.shutdown = (p: *Metaprogram_Plugin) {
        record(p, "shutdown");
    };
    if with_init {
        r.init = (p: *Metaprogram_Plugin, options: [] string) -> bool {
            record(p, "init");
            for options array_add(*(cast(*Recorder) p).options_seen, it);
            return true;
        };
    } else {
        // Takes options shaped "-name value", two at a time.
        r.handle_one_option = (p: *Metaprogram_Plugin, options: [] string, cursor: int) -> int {
            if cursor + 1 >= options.count || !begins_with(options[cursor], "-") return cursor;
            array_add(*(cast(*Recorder) p).options_seen, options[cursor + 1]);
            return cursor + 2;
        };
    }
    return r;
}

argument_splitting :: () {
    args := string.["first.jai", "-plug", "Check", "-x", "+plug", "Iprof", "-release"];
    ok, plugins, remaining := parse_plugin_arguments(args);
    assert(ok && plugins.count == 2 && plugins[0].name == "Check" && plugins[1].name == "Iprof", "plugins: %", plugins);
    assert(remaining.count == 3 && remaining[0] == "first.jai" && remaining[1] == "-x" && remaining[2] == "-release", "remaining: %", remaining);
    ok, plugins, remaining = parse_plugin_arguments(.[]);
    assert(ok && plugins.count == 0 && remaining.count == 0);
    results: [..] *Metaprogram_Plugin;
    assert(init_plugins(Plugin_To_Create.[], *results, get_current_workspace()), "no plugins to create is fine");
}

hooks_in_order :: () {
    // Rules_Single.jai only declares a constant; the plugin adds main through add_source.
    file := tprint("%modules/Rules_Single.jai", path_strip_filename(#file));
    with_init := make_recorder(GOOD_SOURCE, true);
    stepwise := make_recorder("", false);
    plugins := (*Metaprogram_Plugin).[*with_init.base, *stepwise.base];
    options := string.["-level", "3", "-mode", "fast"];
    assert(build_with_plugins(file, plugins, options), "the plugin build succeeds");
    expected := string.["before_intercept", "add_source", "finish", "shutdown"];
    assert(stepwise.calls.count == expected.count, "stepwise calls: %", stepwise.calls);
    for expected assert(stepwise.calls[it_index] == it, "hook % was %", it_index, stepwise.calls[it_index]);
    assert(with_init.calls.count == 5 && with_init.calls[0] == "init", "init comes first: %", with_init.calls);
    assert(with_init.options_seen.count == 4 && with_init.options_seen[3] == "fast", "init sees every option: %", with_init.options_seen);
    assert(stepwise.options_seen.count == 2 && stepwise.options_seen[0] == "3" && stepwise.options_seen[1] == "fast", "handle_one_option claims its pairs: %", stepwise.options_seen);
    assert(with_init.messages > 3 && with_init.completed && stepwise.completed && with_init.messages == stepwise.messages, "every plugin sees every message");
    assert(with_init.workspace == stepwise.workspace && with_init.workspace != get_current_workspace());
}

check_plugin :: () {
    file := tprint("%modules/Rules_Single.jai", path_strip_filename(#file));
    // A clean program passes the format check; the C-porting check only warns.
    checker := make_recorder(GOOD_SOURCE, true);
    checker.message = (p: *Metaprogram_Plugin, message: *Message) {
        Check.do_error_checking(p, message);
        settings: Check.C_Porting_Check_Settings;
        Check.check_c_porting(message, settings);
    };
    assert(build_with_plugins(file, (*Metaprogram_Plugin).[*checker.base], .[]), "a correct program passes Check");
    Check.finish_error_checking();

    // Check walks the @PrintLike calls. A count that differs would be a continuable error,
    // which fails the build, so only matching calls can be checked from a test that passes.
    strict := make_recorder(PRINT_LIKE, true);
    strict.message = Check.do_error_checking;
    assert(build_with_plugins(file, (*Metaprogram_Plugin).[*strict.base], .[]), "matching format counts pass Check");
}

#run {
    argument_splitting();
    hooks_in_order();
    check_plugin();
    print("ok\n");
}
