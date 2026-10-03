set_report_mode :: (interactive: bool) {
    State.interactive_reporting = interactive;
}

#scope_file
State :: #import "internal/debug-state";
