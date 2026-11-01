#scope_module
#import "internal/window/cocoa";

// AppKit text services live with the content view. Input drains owned messages after dispatch.
Text_Message :: struct {
    window: *void;
    text: string;
    kind: int; // 0 marked, 1 commit, 2 cancel
    cursor: int = -1; // UTF-8 offset within marked text
}

text_input_enabled :: (window: *void) -> bool {
    view := result_message(window, "contentView", *void);
    return get_association(view, *text_enabled_key) != null;
}

set_text_input :: (window: *void, enabled: bool, x: float, y: float, height: float) {
    pool := autorelease_pool();

    defer release(pool);
    install_text_view(window);
    view := result_message(window, "contentView", *void);

    if !enabled && get_association(view, *text_marked_key) != null {
        set_association(view, *text_marked_key, null, 1);
        queue_text_message(view, null, 2);
        input := result_message(view, "inputContext", *void);
        void_message(input, "discardMarkedText");
    }

    set_association(view, *text_enabled_key, ifx enabled then view else null, 0);
    rect: O.NSRect;
    rect.origin.x = x;
    rect.origin.y = content_size(window).height - y - height;
    rect.size = .{ 1, cast(O.CGFloat) height };
    value := argument_result_message(O.objc_getClass("NSValue"), "valueWithRect:", rect, *void);

    set_association(view, *text_rect_key, value, 1);
}

interpret_text_key :: (window: *void, event: *void) -> bool {
    view := result_message(window, "contentView", *void);
    marked := get_association(view, *text_marked_key) != null;
    count := composition_events.count;
    events := argument_result_message(O.objc_getClass("NSArray"), "arrayWithObject:", event, *void);

    argument_message(view, "interpretKeyEvents:", events);

    return marked || composition_events.count != count;
}

create_text_input_view :: (base: O.Class = null) -> *void {
    if !text_view_class {
        if !base base = O.objc_getClass("NSView");

        cls := O.objc_allocateClassPair(base, "JacketDesktopTextInputView", 0);

        Basic.assert(cls != null);
        add_text_method(cls, "acceptsFirstResponder", cast(O.IMP) text_accepts, "c@:");
        add_text_method(cls, "hasMarkedText", cast(O.IMP) text_has_marked, "c@:");
        add_text_method(cls, "markedRange", cast(O.IMP) text_marked_range, "{_NSRange=QQ}@:");
        add_text_method(cls, "selectedRange", cast(O.IMP) text_selected_range, "{_NSRange=QQ}@:");
        add_text_method(cls, "setMarkedText:selectedRange:replacementRange:", cast(O.IMP) text_set_marked, "v@:@{_NSRange=QQ}{_NSRange=QQ}");
        add_text_method(cls, "unmarkText", cast(O.IMP) text_unmark, "v@:");
        add_text_method(cls, "insertText:replacementRange:", cast(O.IMP) text_insert, "v@:@{_NSRange=QQ}");
        add_text_method(cls, "doCommandBySelector:", cast(O.IMP) text_command, "v@::");
        add_text_method(cls, "validAttributesForMarkedText", cast(O.IMP) text_attributes, "@@:");
        add_text_method(cls, "attributedSubstringForProposedRange:actualRange:", cast(O.IMP) text_substring, "@@:{_NSRange=QQ}^{_NSRange=QQ}");
        add_text_method(cls, "characterIndexForPoint:", cast(O.IMP) text_index, "Q@:{CGPoint=dd}");
        add_text_method(cls, "firstRectForCharacterRange:actualRange:", cast(O.IMP) text_first_rect, "{CGRect={CGPoint=dd}{CGSize=dd}}@:{_NSRange=QQ}^{_NSRange=QQ}");
        protocol := get_text_protocol("NSTextInputClient");

        Basic.assert(protocol != null && add_text_protocol(cls, protocol) != 0);
        O.objc_registerClassPair(cls);
        text_view_class = cls;
    }

    return result_message(text_view_class, "new", *void);
}

#scope_module
text_view_class: O.Class;
text_enabled_key, text_marked_key, text_selected_key, text_rect_key: u64;

Text_Range :: struct {
    location: u64;
    length: u64;
}

NO_TEXT_LOCATION :: 0x7fffffffffffffff;

install_text_view :: (window: *void) {
    view := result_message(window, "contentView", *void);
    if get_association(view, *original_class_key) return;

    probe := create_text_input_view(object_get_class(view));

    release(probe);
    original := object_set_class(view, text_view_class);

    set_association(view, *original_class_key, original, 0);
    argument_message(window, "makeFirstResponder:", view);
}

add_text_method :: (cls: O.Class, name: string, method: O.IMP, encoding: string) {
    Basic.assert(O.class_addMethod(cls, O.selector(name), method, Basic.temp_c_string(encoding)) != 0);
}

text_accepts :: (view: *void, selector: *void) -> s8 #c_call {
    return 1;
}

text_has_marked :: (view: *void, selector: *void) -> s8 #c_call {
    push_context, defer_pop;
    return cast(s8) (get_association(view, *text_marked_key) != null);
}

text_marked_range :: (view: *void, selector: *void) -> Text_Range #c_call {
    push_context, defer_pop;
    marked := get_association(view, *text_marked_key);
    if !marked return .{ NO_TEXT_LOCATION, 0 };
    return .{ 0, result_message(marked, "length", u64) };
}

text_selected_range :: (view: *void, selector: *void) -> Text_Range #c_call {
    push_context, defer_pop;
    value := get_association(view, *text_selected_key);
    if value && get_association(view, *text_marked_key) != null return result_message(value, "rangeValue", Text_Range);
    return .{ 0, 0 };
}

plain_text :: (value: *void) -> *void {
    if argument_result_message(value, "respondsToSelector:", O.selector("string"), s8) != 0 return result_message(value, "string", *void);
    return value;
}

queue_text_message :: (view: *void, value: *void, kind: int, cursor: int = -1) {
    Basic.push_allocator(context.default_allocator);

    text := "";
    if value text = Basic.copy_string(O.to_string(cast(*O.NSString) value));

    queue_composition(cast(WT.Window_Type) result_message(view, "window", *void), text, kind == 1, kind == 2, cursor);
}

text_set_marked :: (view: *void, selector: *void, value: *void, selected: Text_Range, replacement: Text_Range) #c_call {
    push_context, defer_pop;
    value = plain_text(value);
    if result_message(value, "length", u64) == 0 {
        set_association(view, *text_marked_key, null, 1);
        queue_text_message(view, null, 2);
        return;
    }

    selection := argument_result_message(O.objc_getClass("NSValue"), "valueWithRange:", selected, *void);

    set_association(view, *text_selected_key, selection, 1);
    set_association(view, *text_marked_key, value, 1);
    prefix := argument_result_message(value, "substringToIndex:", Basic.min(selected.location, result_message(value, "length", u64)), *void);
    cursor := O.to_string(cast(*O.NSString) prefix).count;

    queue_text_message(view, value, 0, cursor);
}

text_unmark :: (view: *void, selector: *void) #c_call {
    push_context, defer_pop;
    marked := get_association(view, *text_marked_key);
    if marked queue_text_message(view, marked, 1);
    set_association(view, *text_marked_key, null, 1);
}

text_insert :: (view: *void, selector: *void, value: *void, replacement: Text_Range) #c_call {
    push_context, defer_pop;
    set_association(view, *text_marked_key, null, 1);
    queue_text_message(view, plain_text(value), 1);
}

text_command :: (view: *void, selector: *void, command: *void) #c_call {
    push_context, defer_pop;

    if command == O.selector("cancelOperation:") {
        set_association(view, *text_marked_key, null, 1);
        queue_text_message(view, null, 2);
    }
}

text_attributes :: (view: *void, selector: *void) -> *void #c_call {
    push_context, defer_pop;
    return result_message(O.objc_getClass("NSArray"), "array", *void);
}

text_substring :: (view: *void, selector: *void, range: Text_Range, actual: *Text_Range) -> *void #c_call {
    push_context, defer_pop;
    if actual actual.* = .{ NO_TEXT_LOCATION, 0 };
    return null;
}

text_index :: (view: *void, selector: *void, point: O.NSPoint) -> u64 #c_call {
    return 0;
}

text_first_rect :: (view: *void, selector: *void, range: Text_Range, actual: *Text_Range) -> O.NSRect #c_call {
    push_context, defer_pop;
    if actual actual.* = range;

    value := get_association(view, *text_rect_key);
    rect := result_message(value, "rectValue", O.NSRect);
    // The content view is unflipped, at the origin of the window's content coordinates.
    window := result_message(view, "window", *void);

    return argument_result_message(window, "convertRectToScreen:", rect, O.NSRect);
}

get_text_protocol :: (name: *u8) -> *void #foreign objc "objc_getProtocol";
add_text_protocol :: (class: O.Class, protocol: *void) -> s8 #foreign objc "class_addProtocol";

set_association :: (object: *void, key: *void, value: *void, policy: u64) #foreign objc "objc_setAssociatedObject";
get_association :: (object: *void, key: *void) -> *void #foreign objc "objc_getAssociatedObject";

original_class_key: u64;
object_set_class :: (object: *void, class: O.Class) -> O.Class #foreign objc "object_setClass";

object_get_class :: (object: *void) -> O.Class #foreign objc "object_getClass";
