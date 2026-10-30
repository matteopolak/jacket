// Declarations and procedures, structs and unions, enums and flags.
//
// Each file of this workspace is one section, and `main` below runs them in order.
// Edit anything: the program re-runs as you type. README.md is the guide.

#import "Basic";
#import "Math";

#load "basics.jai";
#load "structs.jai";
#load "enums.jai";

main :: () {
    heading("Basics");
    demo_basics();

    heading("Structs, using and unions");
    demo_structs();

    heading("Enums and flags");
    demo_enums();
}

heading :: (title: string) {
    rule := "────────────────────────────────────────";
    print("\n%\n %\n%\n", rule, title, rule);
    reset_temporary_storage(); // Each section may use the temporary allocator; start it clean.
}
