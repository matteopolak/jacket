// test: args("$TEMP")
#import "Basic";
#import "String";
#import "File";
#import "Extensions/Jai_Format";

#load "../batch.jai";

// The outcomes of a parallel run equal those of a serial run, entry by entry and in the same order.
main :: () {
    directory := copy_string(trim_right(get_command_line_arguments()[1], "/"));
    config: Format_Config;

    FILES :: 200;

    for 0..FILES - 1 {
        text := "";

        if it % 3 == 0 text = tprint("f%::()  {\nx:=%;\n}\n", it, it); // changes
        else if it % 3 == 1 text = tprint("f% :: () {\n    x := %;\n}\n", it, it); // already formatted
        else if it % 11 == 2 text = tprint("f% :: () {\n    x := (%;\n", it, it); // does not lex as balanced
        else text = tprint("f%::()  {\n    x:=%;\n}\n", it, it);

        path := tprint("%/%.jai", directory, it);

        assert(write_entire_file(path, text));
    }

    build :: (directory: string, config: *Format_Config, write: bool) -> [] Outcome {
        outcomes := NewArray(FILES + 1, Outcome);

        for *outcomes {
            it.path = copy_string(tprint("%/%.jai", directory, ifx it_index == FILES then -1 else it_index));
            it.config = config;
            it.write = write;
        }

        return outcomes;
    }

    serial := build(directory, *config, false);
    parallel := build(directory, *config, false);

    format_batch(serial, 1);
    format_batch(parallel, 8);

    changed, failed, missing: int;

    for serial {
        other := parallel[it_index];

        assert(it.path == other.path, "entry % moved", it_index);
        assert(it.status == other.status, "% differs: % against %", it.path, it.status, other.status);
        assert(it.error == other.error && it.long_lines == other.long_lines && it.diff_line == other.diff_line, "% differs", it.path);

        if it.status == .CHANGED changed += 1;
        if it.status == .FORMAT_FAILED failed += 1;
        if it.status == .READ_FAILED missing += 1;
    }

    assert(changed > 0 && failed > 0 && missing == 1, "the files exercise every outcome: % % %", changed, failed, missing);

    // Writing formats the files, and a second pass finds nothing to change.
    rewrite := build(directory, *config, true);

    format_batch(rewrite, 8);

    again := build(directory, *config, false);

    format_batch(again, 8);

    for again assert(it.status != .CHANGED, "% still changes", it.path);

    for rewrite assert(it.status == serial[it_index].status || it.status == .CHANGED, "% was written wrongly", it.path);

    print("ok\n");
}
