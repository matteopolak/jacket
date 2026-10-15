// Reviewed Window_Creation/Simp/Input API with real SDK close notifications.
// Readback must survive closing the current drawable and reopening a window.
#import "Basic";

#if OS == .MACOS {
    W :: #import "Window_Creation";
    I :: #import "Input";
    S :: #import "Simp";
    G :: #import "GL";
    O :: #import "Objective_C";
}

main :: () {
    #if OS == .MACOS {
        for 1..3 {
            window := W.create_window(160, 120, "native context π 🪟");

            assert(window != null);
            I.update_window_events();
            S.set_render_target(window);
            S.clear_render_target(1, 0, 1, 1);
            pixel: [4] u8;

            G.gl.glReadPixels(1, 1, 1, 1, G.GL_RGBA, G.GL_UNSIGNED_BYTE, pixel.data);
            assert(pixel[0] == 255 && pixel[1] == 0 && pixel[2] == 255);
            assert(G.gl.glGetError() == G.GL_NO_ERROR);
            W.swap_view_buffers(window);
            Call :: #type (receiver: *void, selector: O.Selector) #c_call;
            call := cast(Call) O.objc_msgSend;

            call(window, O.selector("close"));
            I.update_window_events();
            closed := false;
            for event: I.events_this_frame if event.type == .QUIT && event.window == window closed = true;

            assert(closed, "synchronous native close lost its window identity before the next pump");
            call(window, O.selector("release"));
        }
    }
}
