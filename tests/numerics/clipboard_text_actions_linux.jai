// Run only inside the CI-owned private X11 display, never a user's clipboard.
#import "Basic";
G :: #import "GetRect";
L :: #import "GetRect_LeftHanded";

#assert OS == .LINUX;

main :: () {
    private_display := false;
    for argument: get_command_line_arguments() if argument == "--private-x11-display" private_display = true;

    assert(private_display, "run only with --private-x11-display inside the CI-owned X11 server");
    source: G.Text_Input_State;
    destination: L.Text_Input_State;

    defer free(source.pre_modification_text);
    defer free(destination.pre_modification_text);
    G.set_input(*source, "héllo 😀");
    G.select_all(*source);
    G.copy_selection_to_clipboard(*source);
    L.insert_clipboard_at_cursor(*destination);
    assert(destination.text == "héllo 😀");
    assert(source.text == "héllo 😀");

    // A second native read must observe the new owner data, not cached text.
    G.set_input(*source, "next π");
    G.select_all(*source);
    G.copy_selection_to_clipboard(*source, cut = true);
    assert(source.text.count == 0);
    L.clear_text(*destination);
    L.insert_clipboard_at_cursor(*destination);
    assert(destination.text == "next π");

    // Reverse the variants and retain the source storage until the copy returns.
    L.select_all(*destination);
    L.copy_selection_to_clipboard(*destination);
    G.insert_clipboard_at_cursor(*source);
    assert(source.text == "next π");
    print("GetRect canonical Linux clipboard text actions ok\n");
}
