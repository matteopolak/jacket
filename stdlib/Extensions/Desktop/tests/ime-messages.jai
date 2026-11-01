// test: only(.MACOS) serial
// Load into this test's module to exercise private callbacks without exporting native internals.
#load "../module.jai";

main :: () {
    pool := autorelease_pool();

    defer release(pool);
    view := create_text_input_view();

    defer release(view);
    text_set_marked(view, null, native_string("か😀な"), .{ 3, 0 }, .{ NO_TEXT_LOCATION, 0 });
    // Native callbacks outside a pump are retained for the next frame.
    assert(pending_composition.count == 1);
    begin_desktop_events();
    assert(text_has_marked(view, null) != 0);
    selected := text_selected_range(view, null);
    marked := text_marked_range(view, null);

    assert(selected.location == 3 && selected.length == 0);
    assert(marked.location == 0 && marked.length == 4);
    assert(composition_events.count == 1);
    assert(composition_events[0].text == "か😀な" && composition_events[0].cursor == 7);
    assert(!composition_events[0].commit && composition_events[0].position == I.events_this_frame.count);
    text_set_marked(view, null, native_string(""), .{}, .{});
    assert(text_has_marked(view, null) == 0);
    assert(composition_events[1].cancel);
    text_set_marked(view, null, native_string("marked"), .{}, .{});
    text_insert(view, null, native_string("仮名"), .{});
    assert(text_has_marked(view, null) == 0);
    assert(composition_events[3].text == "仮名" && composition_events[3].commit);
    // A callback from the next native event retains its position after ordinary input.
    if !I.events_this_frame.allocator.proc I.events_this_frame.allocator = context.default_allocator;

    array_add(*I.events_this_frame, .{ type = .TEXT_INPUT, utf32 = 65 });
    text_insert(view, null, native_string("a"), .{});
    assert(composition_events[4].position == 1);
    I.input_per_frame_event_and_flag_update();
    pumping_events = false;
    text_set_marked(view, null, native_string(""), .{}, .{});
    assert(pending_composition.count == 1);
    begin_desktop_events();
    assert(composition_events.count == 1 && composition_events[0].cancel && composition_events[0].position == 0);
    pumping_events = false;
    for composition_events free(it.text);

    array_free(composition_events);
}
