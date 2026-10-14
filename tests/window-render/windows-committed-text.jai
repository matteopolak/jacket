#import "Basic";
I :: #import "Input";
W :: #import "Windows";

main :: () {
    I.input_per_frame_event_and_flag_update();
    I.MyWindowProc(null, W.WM_CHAR, 31, 0);
    assert(I.events_this_frame.count == 0);
    I.MyWindowProc(null, W.WM_CHAR, 0xd83d, 0);
    I.MyWindowProc(null, W.WM_CHAR, 0xde00, 0);
    assert(I.events_this_frame.count == 2);
    assert(I.events_this_frame[0].utf32 == 0xd83d && I.events_this_frame[1].utf32 == 0xde00);
    assert(I.events_this_frame[0].text_input_count == 0);
    print("ok\n");
}
