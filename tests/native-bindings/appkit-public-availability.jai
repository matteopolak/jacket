A :: #import "Objective_C/AppKit";
Again :: #import "Objective_C/AppKit";

View :: A.NSView;
Screen :: A.NSScreen;
#assert View == Again.NSView;
#assert Screen == Again.NSScreen;
#assert type_info(View).name == "NSView";
#assert type_info(Screen).name == "NSScreen";

main :: () {}
