// `text_display` passes its `Text_Display_State` to `get_status_flags`, which takes a `*Active_Widget`:
// the state's widget member must be `#as`.
#import "Basic";

// GetRect (through Simp) needs native C libraries and a window system, which the WASM targets lack:
// they are listed in tests/stdlib-runtime-skips.txt.
#import "GetRect";

main :: () {
    text: Flowable_Text;
    r: Rect;
    if false text_display(r, *text, 0, 20);
    print("ok\n");
}
