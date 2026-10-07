// rules: import.1 import.2 import.3 import.4 import.5 import.6 import.7 import.8 import.9 import.11 import.12 import.13 import.14 import.16
// The forms of #import and #load, through modules in tests/stdlib/modules.
#import "Basic";

Plain :: #import "Rules_Plain";
Plain_Again :: #import "Rules_Plain";
using Used :: #import "Rules_Single";
#import "Rules_Both";
Thing :: #import,file "modules/rules_import_files/thing.jai";
Dir :: #import,dir "modules/Rules_Plain";
Str :: #import,string "str_val :: 11; str_twice :: (x: int) -> int { return x * 2; }";
#load "modules/rules_import_files/loaded.jai";
Filtered :: #import,string "filtered_answer :: 5; filtered_other :: 6;";

using,only(filtered_answer) Filtered;

LOADER_BASE :: 9;

main :: () {
    // import.3: a named import's members are reached through the name.
    assert(Plain.answer == 42 && Plain.other == 7, "named import");

    // import.4: `using` a named import also makes its members unqualified.
    assert(Used.single_value == 11 && single_value == 11, "using named import");

    // import.2, import.9: `#import "Name"` finds Name.jai before Name/module.jai, in the
    // importing file's modules directory.
    assert(which_form == "file", "Name.jai is found before Name/module.jai");

    // import.5, import.11: `#import,file` resolves relative to the file that wrote it, at every level.
    assert(Thing.thing_value() == 42, "file import");

    // import.6: `#import,dir` imports the directory's module.jai.
    assert(Dir.answer == 42, "directory import");

    // import.7: `#import,string` compiles the text as a module.
    assert(Str.str_val == 11 && Str.str_twice(4) == 8, "string import");

    // import.8: `#load` shares the loader's namespace both ways.
    assert(loaded_value() == 10, "loaded file");

    // import.11: a `#load` inside a loaded file resolves next to that file.
    assert(Plain.from_deeper() == 142, "nested load");

    // import.12: loading the same file twice is a no-op (no duplicate declarations).
    assert(Plain.from_part == 100, "twice-loaded file");

    // import.13: two plain imports of one module share one instance.
    assert(Plain.bump() == 1 && Plain_Again.bump() == 2 && Dir.bump() == 3, "one instance");

    // import.14: an imported module's #add_context is in place before main runs.
    assert(context.rules_plain_marker == 5, "module's #add_context");
    // import.16: `using,only(name) M;` brings the listed name in unqualified.
    assert(filtered_answer == 5 && Filtered.filtered_other == 6, "using,only");
    print("ok\n");
}
