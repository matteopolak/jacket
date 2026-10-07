// Welcome to the Jai tour!
//
// Every file in this workspace shows off one corner of the language, and
// `main` below runs them all in order. Edit anything: the program re-runs as
// you type. tour.md has a map of the files.

#import "Basic";
#import "Math";
#import "String";

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

main :: () {
    print("A whirlwind tour of Jai, running in your browser.\n");

    // A table of procedures: each entry is a title and a procedure value.
    Stop :: struct {
        title: string;
        run: () -> ();
    }
    stops :: Stop.[
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
    ];

    for stops {
        heading(it_index + 1, it.title);
        it.run();
        // Each stop may use the temporary allocator; clear it between stops.
        reset_temporary_storage();
    }

    print("\nThat's the tour. Now break something!\n");
}

heading :: (number: int, title: string) {
    rule := "────────────────────────────────────────";
    print("\n%\n %. %\n%\n", rule, number, title, rule);
}
