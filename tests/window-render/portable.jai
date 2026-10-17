#import "Basic";
I :: #import "Input";
S :: #import "Simp";

main :: () {
    I.input_button_states[8] = .DOWN | .START;
    I.input_button_states[9] = .END;
    I.touch_pointers[0].state = .DOWN | .START;
    I.mouse_delta_x = 12;
    I.add_resize_record(0, 320, 200);
    assert(I.get_window_resizes().count == 1);
    I.input_per_frame_event_and_flag_update();
    assert(I.input_button_states[8] == .DOWN);
    assert(I.input_button_states[9] == .NONE);
    assert(I.touch_pointers[0].state == .DOWN);
    assert(I.mouse_delta_x == 0);
    assert(I.get_window_resizes().count == 0);
    size, stride := S.get_image_size(.RGBA8, 7, 3);

    assert(size == 84 && stride == 28);
    size, stride = S.get_image_size(.BC1, 7, 5);
    assert(size == 32 && stride == 16);
    assert(S.get_image_bytes_per_texel(.BC1) == 0);
    assert(S.get_image_bytes_per_block(.BC1) == 8);
    assert(S.get_image_bytes_per_block(.BC3) == 16);
    bitmap: S.Bitmap;

    S.bitmap_alloc(*bitmap, 7, 3, .RGBA8);
    assert(bitmap.data.count == 84 && bitmap.stride == 28);
    assert(bitmap.width == 7 && bitmap.height == 3);
    for *bitmap.data it.* = cast(u8) it_index;

    S.deinit(*bitmap);
    assert(bitmap.data.count == 0 && bitmap.data.data == null);
    print("ok\n");
}
