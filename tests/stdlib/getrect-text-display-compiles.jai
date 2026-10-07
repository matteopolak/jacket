// `text_display` passes its `Text_Display_State` to `get_status_flags`, which takes a `*Active_Widget`:
// the state's widget member must be `#as`.
#import "Basic";

// GetRect (through Simp) needs native C libraries and a window system, so the WASM target (the browser
// playground) cannot import it: there the test is empty.
#if OS != .WASM {
    #import "GetRect";
}

main :: () {
    #if OS != .WASM {
        text: Flowable_Text;
        r: Rect;
        if false text_display(r, *text, 0, 20);
    }
    print("ok\n");
}
