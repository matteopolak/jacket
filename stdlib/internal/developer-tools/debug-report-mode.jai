// Turns the assert dialog of [interactive_assert] on or off. Off, failures are only printed.
set_report_mode :: (interactive: bool) {
    State.interactive_reporting = interactive;
}

#scope_file
State :: #import "internal/debug-state";
