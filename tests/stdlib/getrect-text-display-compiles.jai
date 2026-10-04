// `text_display` passes its `Text_Display_State` to `get_status_flags`, which takes a `*Active_Widget`:
// the state's widget member must be `#as`.
#import "Basic";
#import "GetRect";

main :: () {
    text: Flowable_Text;
    r: Rect;
    if false text_display(r, *text, 0, 20);
    print("ok\n");
}
