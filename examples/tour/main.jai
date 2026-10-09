// Welcome to the Jai tour!
//
// Every file in this workspace shows off one corner of the language. `main`
// below reads its command line with Extensions/Args and runs one stop
// (`run --stop enums`), all of them (`run --all`) or, with no arguments, shows
// a menu. Try `run --help`. tour.md has a map of the files.

#import "Basic";
#import "Math";
#import "String";
#import "File";
Args :: #import "Extensions/Args";

#load "basics/basics.jai";
#load "types/structs.jai";
#load "types/enums.jai";
#load "data/arrays.jai";
#load "data/strings.jai";
#load "memory/memory.jai";
#load "generics/polymorphism.jai";
#load "meta/compile_time.jai";
#load "meta/macros.jai";
#load "meta/reflection.jai";
#load "meta/metaprogram.jai";
#load "safety/safety.jai";
#load "threads/threads.jai";
#load "files/files.jai";
#load "machine/machine.jai";
#load "finale/raymarch.jai";
#load "gpu/raymarch_gpu.jai";

// The stops. Args reads `--stop <name>` from the lowercased member names
// (`COMPILE_TIME` is `compile-time`), and a member's value is its place in `stops`.
Stop :: enum {
    BASICS;
    STRUCTS;
    ENUMS;
    ARRAYS;
    STRINGS;
    MEMORY;
    POLYMORPHISM;
    COMPILE_TIME;
    MACROS;
    REFLECTION;
    METAPROGRAM;
    SAFETY;
    THREADS;
    FILES;
    MACHINE;
    FINALE;
    GPU;
}

Cli :: struct {
    stop: Stop; @short=s @conflicts=all
    all: bool; @short=a
}

HELP :: Args.Help(Cli).{
    about = "A whirlwind tour of Jai. With no options, shows a menu of the stops.",
    stop = "Run only this stop",
    all = "Run every stop, in order",
};

// A table of procedures: each entry is a title and a procedure value.
Stop_Entry :: struct {
    title: string;
    run: () -> ();
}

stops :: Stop_Entry.[
    .{ "Basics", tour_basics },
    .{ "Structs, using and unions", tour_structs },
    .{ "Enums and flags", tour_enums },
    .{ "Arrays", tour_arrays },
    .{ "Strings and formatting", tour_strings },
    .{ "Memory and the context", tour_memory },
    .{ "Polymorphism", tour_polymorphism },
    .{ "Compile-time execution", tour_compile_time },
    .{ "Macros and custom loops", tour_macros },
    .{ "Reflection", tour_reflection },
    .{ "A metaprogram", tour_metaprogram },
    .{ "Runtime safety", tour_safety },
    .{ "Threads", tour_threads },
    .{ "Files", tour_files },
    .{ "#asm and Long_Double", tour_machine },
    .{ "Finale: a ray-marched scene", tour_finale },
    .{ "Encore: ray marching on the GPU", tour_gpu },
];

#assert stops.count == type_info(Stop).names.count;

main :: () {
    cli, set := Args.parse(Cli, HELP);
    print("A whirlwind tour of Jai, running in your browser.\n");

    if set.stop {
        run_stop(cast(int) cli.stop);
    } else if cli.all {
        run_all();
    } else {
        menu();
    }
}

run_stop :: (index: int) {
    heading(index + 1, stops[index].title);
    stops[index].run();
    // Each stop may use the temporary allocator; clear it afterwards.
    reset_temporary_storage();
}

run_all :: () {
    for 0..stops.count - 1 run_stop(it);
    print("\nThat's the tour. Now break something!\n");
}

// Lists the stops and reads choices from standard input until `q` or its end.
menu :: () {
    first := true;
    while true {
        print("\nStops:\n");
        for stops {
            if it_index < 9 print(" ");
            print("%. %\n", it_index + 1, it.title);
        }
        print("\nPick a stop by number or name, Enter for all, q to quit: ");

        line, ok := read_stdin_line();
        defer free(line);
        choice := trim(line);

        // At the end of input, run everything if nothing has run yet (a pipe or a
        // script gets the whole tour), then stop.
        if !ok {
            print("\n");
            if first run_all();
            return;
        }
        first = false;
        if choice == "q" || choice == "quit" return;
        if !choice {
            run_all();
            continue;
        }

        index := find_stop(choice);
        if index < 0 print("No stop called \"%\".\n", choice);
        else run_stop(index);
    }
}

// A stop's number, or its name as `--stop` spells it; -1 if there is no such stop.
find_stop :: (choice: string) -> int {
    rest := choice;
    number, is_number := parse_int(*rest);
    if is_number && !rest return ifx number >= 1 && number <= stops.count then number - 1 else -1;

    for type_info(Stop).names {
        name := copy_temporary_string(it);
        for 0..name.count - 1 if name[it] == #char "_" name[it] = #char "-";
        if equal_nocase(name, choice) return it_index;
    }
    return -1;
}

heading :: (number: int, title: string) {
    rule := "────────────────────────────────────────";
    print("\n%\n %. %\n%\n", rule, number, title, rule);
}
