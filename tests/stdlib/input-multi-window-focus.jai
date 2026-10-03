// Source regression witness. These tokens identify abstract queue origins;
// they are not native handles, and this witness never invokes a native pump.
Input :: #import "Input"(ENABLE_NATIVE_INPUT_ADAPTERS=false);
#import "Basic";

press :: (key:Input.Key_Code) -> Input.Event {
    event:Input.Event; event.type=.KEYBOARD; event.key_code=key; event.key_pressed=true;
    return event;
}
release :: (key:Input.Key_Code) -> Input.Event {
    event:Input.Event; event.type=.KEYBOARD; event.key_code=key;
    return event;
}
is_down :: (key:Input.Key_Code) -> bool {
    return (Input.input_button_states[cast(u32)key] & .DOWN)!=0;
}
main :: () {
    Input.input_event_source=null;
    first:Input.Window_Type=1001;
    second:Input.Window_Type=1002;
    key:Input.Key_Code=.ARROW_LEFT;
    Input.set_input_focus(false);
    Input.update_window_events();

    // FocusIn(B) before FocusOut(A) must retain application focus.
    Input.set_input_focus(first,true);
    Input.set_input_focus(second,true);
    Input.set_input_focus(first,false);
    assert(Input.input_application_has_focus);
    assert(!Input.window_has_input_focus(first));
    assert(Input.window_has_input_focus(second));
    Input.set_input_focus(second,false);
    assert(!Input.input_application_has_focus);

    // The reverse ordering must preserve focus in the remaining window too.
    Input.set_input_focus(first,true);
    Input.set_input_focus(second,true);
    Input.set_input_focus(second,false);
    assert(Input.input_application_has_focus);
    assert(Input.window_has_input_focus(first));
    assert(!Input.window_has_input_focus(second));
    Input.set_input_focus(first,false);
    assert(!Input.input_application_has_focus);

    // The same key can remain down in B after A loses focus.
    Input.set_input_focus(first,true);
    Input.set_input_focus(second,true);
    Input.queue_input_event(first,press(key));
    Input.queue_input_event(second,press(key));
    Input.update_window_events();
    assert(is_down(key));
    assert(Input.window_key_is_down(first,key));
    assert(Input.window_key_is_down(second,key));
    Input.update_window_events();
    Input.set_input_focus(first,false);
    assert(Input.input_application_has_focus);
    Input.update_window_events();
    assert(!Input.window_key_is_down(first,key));
    assert(Input.window_key_is_down(second,key));
    assert((Input.get_window_key_state(first,key)&.END)!=0);
    assert((Input.get_window_key_state(second,key)&.END)==0);
    assert(is_down(key));
    assert((Input.input_button_states[cast(u32)key]&.END)==0);
    assert(Input.routed_events_this_frame.count==1);
    assert(Input.routed_events_this_frame[0].window==first);
    assert(!Input.routed_events_this_frame[0].event.key_pressed);
    Input.set_input_focus(second,false);
    Input.update_window_events();
    assert(!Input.input_application_has_focus);
    assert(!is_down(key));
    assert((Input.input_button_states[cast(u32)key]&.END)!=0);
    assert(Input.routed_events_this_frame.count==1);
    assert(Input.routed_events_this_frame[0].window==second);

    // Focus loss in the same pump batch must see that origin's queued press.
    Input.set_input_focus(first,true);
    Input.set_input_focus(second,true);
    Input.queue_input_event(first,press(key));
    Input.queue_input_event(second,press(key));
    Input.set_input_focus(first,false);
    Input.update_window_events();
    assert(!Input.window_key_is_down(first,key));
    assert(Input.window_key_is_down(second,key));
    assert(is_down(key));
    assert(Input.routed_events_this_frame.count==3);
    assert(Input.routed_events_this_frame[0].window==first);
    assert(Input.routed_events_this_frame[0].event.key_pressed);
    assert(Input.routed_events_this_frame[1].window==second);
    assert(Input.routed_events_this_frame[1].event.key_pressed);
    assert(Input.routed_events_this_frame[2].window==first);
    assert(!Input.routed_events_this_frame[2].event.key_pressed);
    Input.set_input_focus(second,false);
    Input.update_window_events();

    // An ordinary release from A must not clear B's independent hold.
    Input.queue_input_event(first,press(key));
    Input.queue_input_event(second,press(key));
    Input.update_window_events();
    Input.queue_input_event(first,release(key));
    Input.update_window_events();
    assert(!Input.window_key_is_down(first,key));
    assert(Input.window_key_is_down(second,key));
    assert(is_down(key));
    Input.queue_input_event(second,release(key));
    Input.update_window_events();
    assert(!is_down(key));

    // Global focus loss releases scoped and unscoped owners once each.
    Input.set_input_focus(first,true);
    Input.set_input_focus(second,true);
    Input.queue_input_event(first,press(key));
    Input.queue_input_event(second,press(key));
    Input.queue_input_event(press(key));
    Input.update_window_events();
    Input.set_input_focus(false);
    assert(!Input.input_application_has_focus);
    assert(!Input.window_has_input_focus(first));
    assert(!Input.window_has_input_focus(second));
    Input.update_window_events();
    assert(!is_down(key));
    assert(Input.routed_events_this_frame.count==2);
    assert(Input.unrouted_events_this_frame.count==1);
    assert(Input.events_this_frame.count==3);
    assert(Input.routed_events_this_frame[0].window==first);
    assert(Input.routed_events_this_frame[1].window==second);
    for event:Input.events_this_frame assert(!event.key_pressed);

    // Unscoped input stays down when a scoped owner releases the same key.
    Input.queue_input_event(first,press(key));
    Input.queue_input_event(press(key));
    Input.update_window_events();
    Input.set_input_focus(first,false);
    Input.update_window_events();
    assert(!Input.window_key_is_down(first,key));
    assert(is_down(key));
    Input.queue_input_event(release(key));
    Input.update_window_events();
    assert(!is_down(key));

    // Explicit legacy focus remains authoritative until globally cleared.
    Input.set_input_focus(true);
    Input.set_input_focus(first,false);
    assert(Input.input_application_has_focus);
    Input.set_input_focus(false);
    assert(!Input.input_application_has_focus);

    // Existing callers may write the public global states directly.
    Input.input_button_states[cast(u32)key]=.DOWN;
    Input.set_input_focus(false);
    Input.update_window_events();
    assert(!is_down(key));
    assert(Input.routed_events_this_frame.count==0);
    assert(Input.unrouted_events_this_frame.count==1);
}
