// Iprof end to end: the metaprogram plugin instruments a small program in a child workspace, the
// instrumented `main` runs (via #run) and its exit report is captured from the logger and checked.
#import "Basic";
#import "Compiler";
#import "String";
Iprof :: #import "Iprof";

TARGET :: #string END
#import "Basic";
#import "String";

square_sum :: (n: int) -> int {
    total := 0;
    for 0..n-1  total += it * it;
    return total;
}

outer :: (rounds: int) -> int {
    total := 0;
    for 1..rounds  total += square_sum(1000);
    return total;
}

skipped :: (n: int) -> int { return n + 1; } @NoProfile

main :: () {
    result := outer(5) + skipped(1);
    assert(result > 0);
}

captured: String_Builder;

capture :: (message: string, data: *void, info: Log_Info) {
    append(*captured, message);
} @NoProfile

verify :: () {
    context.logger = capture;
    main();
    text := builder_to_string(*captured);
    lines := split(text, "\n");

    // Title, column header, then one row per zone that ran: root, main, outer, square_sum.
    assert(begins_with(lines[0], "Iprof: self time"), "%", text);
    assert(contains(lines[1], "self ms") && contains(lines[1], "calls"), "%", text);
    calls_of :: (lines: [] string, zone: string) -> string {
        for lines {
            words := split(it, " ");
            names: [..] string;
            for words  if it  array_add(*names, it);
            if names.count == 4 && names[3] == zone  return names[2];
        }
        return "";
    }
    assert(calls_of(lines, "square_sum") == "5", "%", text);
    assert(calls_of(lines, "outer") == "1", "%", text);
    assert(calls_of(lines, "main") == "1", "%", text);
    assert(calls_of(lines, "(global)") == "0", "%", text);
    assert(!contains(text, "skipped"), "%", text);
    print("instrumented report ok\n");
} @NoProfile
END

#run {
    w := compiler_create_workspace("iprof target");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);

    plugin := Iprof.get_plugin();
    plugin.workspace = w;
    arguments := string.["-min_size", "1"];
    cursor := 0;
    while cursor < arguments.count cursor = plugin.handle_one_option(plugin, arguments, cursor);

    compiler_begin_intercept(w);
    plugin.add_source(plugin);
    add_build_string(TARGET, w);

    verified := false;
    while true {
        message := compiler_wait_for_message();
        plugin.message(plugin, message);
        // The bodies arrive together, so once main is instrumented the rest are too.
        if (cast(*Iprof.Iprof_Plugin) plugin).entry_point_done && !verified {
            add_build_string("#run verify();", w);
            verified = true;
        }
        if message.kind == .COMPLETE {
            assert((cast(*Message_Complete) message).error_code == .NONE);
            break;
        }
    }
    compiler_end_intercept(w);
    assert(verified);
    plugin.finish(plugin);
    plugin.shutdown(plugin);
    print("ok\n");
}
