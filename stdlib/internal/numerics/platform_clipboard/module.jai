// Private OS clipboard transport. Errors are returned rather than replacing the
// system clipboard with process-local data. Only native C ABI declarations occur
// here; no public Jai compatibility declarations are synthesized.
#module_parameters(PASTEBOARD_NAME := "com.apple.pasteboard.clipboard", CLIENT_ID := 0);

#if OS == .MACOS {
    #load "macos.jai";
} else {
    write_text :: (_text: string, _window: *void = null) -> (success: bool, message: string) {
        return false, "Native clipboard transport is unavailable for this target; no local substitute was used";
    }

    read_text :: (_window: *void = null) -> (success: bool, text: string, message: string) {
        return false, "", "Native clipboard transport is unavailable for this target; no local substitute was used";
    }
}
