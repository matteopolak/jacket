#scope_module
Active_Widget :: struct {
    widget_type: Type;
    deactivate_proc: (w: *Active_Widget);
    serial_events_proc: (w: *Active_Widget, events: [] Event);
    check_for_activation_proc: (w: *Active_Widget, event: Event) -> bool;
    last_getrect_frame: u64;
    is_on_state_stack := false;
    last_update_time: float64;
    cosmetic_dt: float32;
    window: Window_Type;
    exists_on_occluder: *Active_Widget;
}

Augmented_Keyframe :: struct {
    using base: Color_Keyframe;
    active: bool;
    active_t: float;
    dragging_t: float;
    color_picking_t: float;
    pos_x: float;
}

Output_Type :: enum {
    RESAMPLED_ARRAY :: 0;
    SOURCE_KEYFRAMES :: 1;
}

NUM_POINTERS :: 8;

#if Type_Indicator == void {
    Simp :: #import "Simp";
    Windows_Context :: #import "Window_Creation";

    Font :: Simp.Dynamic_Font;
    Texture :: Simp.Texture;
    Font_Effects :: Simp.Font_Effects;
    Window_Type :: Windows_Context.Window_Type;
} else {
    Font :: Type_Indicator.Font;
    Texture :: Type_Indicator.Texture;
    Font_Effects :: Type_Indicator.Font_Effects;
    Window_Type :: Type_Indicator.Window_Type;
}

// Only a borrowed Objective-C pointer identity is required; no object layout,
// fields or allocation are inferred for this otherwise unspecified type.
NSCursor :: Native_Cursors.NSCursor;

Occlusion_Record :: struct {
    serial: s32;
    rect: Rect;
    widget: *Active_Widget;
}
