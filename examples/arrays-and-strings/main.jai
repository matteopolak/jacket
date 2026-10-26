// Arrays and strings: fixed arrays, views and dynamic arrays, then strings, `print` formatting
// and String_Builder. README.md is the guide.

#import "Basic";
#import "Math";
#import "String";

#load "arrays.jai";
#load "strings.jai";

main :: () {
    heading("Arrays");
    demo_arrays();

    heading("Strings and formatting");
    demo_strings();
}

heading :: (title: string) {
    rule := "────────────────────────────────────────";
    print("\n%\n %\n%\n", rule, title, rule);
    reset_temporary_storage(); // Each section may use the temporary allocator; start it clean.
}
