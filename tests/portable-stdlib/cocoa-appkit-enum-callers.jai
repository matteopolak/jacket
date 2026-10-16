// SOURCE ONLY: reviewed public window creation/fullscreen bind to canonical AppKit enums.
// Native GUI execution is serialized by integration, not part of this source gate.
W :: #import "Window_Creation";

main :: () {
    window := W.create_window(1, 1, "AppKit caller contract");
    saved: W.Saved_Window_Info;
    W.toggle_fullscreen(window, true, *saved);
}
