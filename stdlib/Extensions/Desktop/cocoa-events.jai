#scope_module
// Intercept only while the extension pumps Input. Return ordinary events unchanged; consumed
// shortcuts and text never reach Input's ordinary key/text translator, avoiding duplicate edits.
Next_Event :: #type (receiver: *void, selector: O.Selector, mask: u64, until: *void, mode: *void, dequeue: s8) -> *void #c_call;
previous_next: Next_Event;

pump_cocoa :: () {
    cls := O.objc_getClass("NSApplication");
    method := get_instance_method(cls, O.selector("nextEventMatchingMask:untilDate:inMode:dequeue:"));

    previous_next = cast(Next_Event) replace_implementation(method, cast(O.IMP) desktop_next_event);
    defer replace_implementation(method, cast(O.IMP) previous_next);
    I.update_window_events();
}

desktop_next_event :: (receiver: *void, selector: O.Selector, mask: u64, until: *void, mode: *void, dequeue: s8) -> *void #c_call {
    push_context, defer_pop;

    while true {
        event := previous_next(receiver, selector, mask, until, mode, dequeue);
        if !event || !dequeue return event;
        if result_message(event, "type", u64) != 10 return event;

        window := result_message(event, "window", *void);
        consumed := false;

        for menu: menus if menu.window == window {
            if argument_result_message(menu.handle, "performKeyEquivalent:", event, s8) != 0 consumed = true;
        }

        if consumed continue;

        flags := result_message(event, "modifierFlags", u64);
        if text_input_enabled(window) && (flags & ((1 << 20) | (1 << 18))) == 0 {
            if interpret_text_key(window, event) continue;
        }

        return event;
    }
}

get_instance_method :: (class: O.Class, selector: O.Selector) -> *void #foreign objc "class_getInstanceMethod";
replace_implementation :: (method: *void, implementation: O.IMP) -> O.IMP #foreign objc "method_setImplementation";
