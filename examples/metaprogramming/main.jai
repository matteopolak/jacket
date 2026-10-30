// Compile-time execution, macros and custom loops, reflection, and a
// metaprogram that compiles another program. README.md is the guide.

#import "Basic";
#import "Math";
#import "String";

#load "compile_time.jai";
#load "macros.jai";
#load "reflection.jai";
#load "metaprogram.jai";

main :: () {
    heading("Compile-time execution");
    demo_compile_time();

    heading("Macros and custom loops");
    demo_macros();

    heading("Reflection");
    demo_reflection();

    heading("A metaprogram");
    demo_metaprogram();
}

heading :: (title: string) {
    rule := "────────────────────────────────────────";
    print("\n%\n %\n%\n", rule, title, rule);
    reset_temporary_storage(); // Each section may use the temporary allocator; start it clean.
}
