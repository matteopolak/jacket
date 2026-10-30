// Layout and interaction without a window: the tree is built by ordinary procedures, laid out with
// fixed-width synthetic fonts (every character is 0.6 of the font size wide), and driven by queued
// input. Nothing here needs a GPU or a font file.
#import "Basic";
#import "Extensions/Testing";
#import "Extensions/UI";

Counter :: struct {
    count: int;
}

counter_build :: (app: *Counter) {
    for column(.{ gap = 8, padding = 16 }) {
        label(tprint("Clicked % times", app.count));
        if button("Click me") app.count += 1;
        if app.count > 0 && button("Reset") app.count = 0;
    }
}

a_button_click_reaches_the_app :: () {
    app: Counter;
    ui := headless(.{ width = 320, height = 200 });

    defer free_ui(*ui);

    frame(*ui, *app, counter_build);
    expect_equal(app.count, 0);

    click(*ui, find_label(*ui, "Click me"));
    frame(*ui, *app, counter_build);
    expect_equal(app.count, 1);

    frame(*ui, *app, counter_build);
    expect_equal(app.count, 1);
    expect_equal(find_role(*ui, .TEXT, 0).text, "Clicked 1 times");
    expect(find_label(*ui, "Reset") != null);

    click(*ui, find_label(*ui, "Reset"));
    frame(*ui, *app, counter_build);
    expect_equal(app.count, 0);
    frame(*ui, *app, counter_build);
    expect(find_label(*ui, "Reset") == null);
} @test

columns_stack_children_with_gaps_and_padding :: () {
    ui := headless(.{ width = 200, height = 100 });

    defer free_ui(*ui);

    app: Counter;

    frame(*ui, *app, counter_build);

    label_node := find_role(*ui, .TEXT, 0);
    button_node := find_label(*ui, "Click me");

    // 14 point text: 14 * 1.25 = 17.5 high, "Clicked 0 times" is 15 characters of 8.4 points.
    expect_equal(label_node.rect.x, 16.0);
    expect_equal(label_node.rect.y, 16.0);
    expect_near(label_node.rect.h, 17.5, 0.01);
    expect_near(button_node.rect.y, 16.0 + 17.5 + 8.0, 0.01);
    expect_near(label_node.rect.w, 168.0, 0.01);
} @test

grow_shares_leftover_space :: () {
    ui := headless(.{ width = 300, height = 100 });

    defer free_ui(*ui);

    build :: (app: *int) {
        for row(.{ gap = 10 }) {
            spacer(1);
            label("fixed", .{});
            spacer(3);
        }
    }

    dummy := 0;

    frame(*ui, *dummy, build);

    first := find_role(*ui, .GROUP, 2);
    text := find_role(*ui, .TEXT, 0);
    last := find_role(*ui, .GROUP, 3);

    // 300 - 5 chars * 8.4 - 2 gaps of 10 = 238 shared 1 to 3.
    expect_near(first.rect.w, 59.5, 0.01);
    expect_near(last.rect.w, 178.5, 0.01);
    expect_near(text.rect.x, 69.5, 0.01);
} @test

scroll_containers_clip_and_scroll :: () {
    ui := headless(.{ width = 200, height = 100 });

    defer free_ui(*ui);

    build :: (app: *int) {
        for scroll(.{ grow = 1 }) {
            for i: 0..19 {
                key(i);
                label(tprint("row %", i));
            }
        }
    }

    dummy := 0;

    frame(*ui, *dummy, build);

    viewport := find_role(*ui, .SCROLL, 0);

    expect_near(viewport.rect.h, 100.0, 0.01);
    expect_near(viewport.content.y, 20 * 17.5, 0.01);
    expect_equal(find(*ui, .{ text = "row 0" }).rect.y, 0.0);

    send_mouse_move(*ui, 50, 50);
    send_scroll(*ui, 0, 35);
    frame(*ui, *dummy, build);
    frame(*ui, *dummy, build);

    expect_near(find(*ui, .{ text = "row 0" }).rect.y, -35.0, 0.01);

    // Scrolling stops at the end of the content.
    send_scroll(*ui, 0, 100000);
    frame(*ui, *dummy, build);
    frame(*ui, *dummy, build);
    expect_near(find(*ui, .{ text = "row 19" }).rect.y + 17.5, 100.0, 0.01);
} @test

nodes_keep_their_state_between_frames_by_key :: () {
    ui := headless(.{ width = 200, height = 100 });

    defer free_ui(*ui);

    Item :: struct {
        id: int;
        name: string;
    }

    items: [3] Item = .[.{ 1, "one" }, .{ 2, "two" }, .{ 3, "three" }];

    build :: (items: *[3] Item) {
        for column() {
            for items.* {
                key(it.id);
                node := label(it.name);
                counter := state(node, int);
                counter.* += 1;
                node.value = tprint("%", counter.*);
            }
        }
    }

    frame(*ui, *items, build);
    frame(*ui, *items, build);

    // Reordering keeps each label's state with it.
    items = .[.{ 3, "three" }, .{ 1, "one" }, .{ 2, "two" }];
    frame(*ui, *items, build);

    expect_equal(find(*ui, .{ text = "three" }).value, "3");
    expect_equal(find(*ui, .{ text = "one" }).value, "3");
    expect_equal(find_role(*ui, .TEXT, 0).text, "three");
} @test

layout_dump_lists_the_tree :: (t: *Test) {
    ui := headless(.{ width = 200, height = 60 });

    defer free_ui(*ui);

    app: Counter;

    frame(*ui, *app, counter_build);
    expect_snapshot(t, layout_dump(*ui));
} @test

split_panes_follow_the_divider_position :: () {
    ui := headless(.{ width = 400, height = 100 });

    defer free_ui(*ui);

    Panes :: struct {
        position: float = 100;
    }

    build :: (panes: *Panes) {
        for pane: split(.HORIZONTAL, *panes.position) {
            if pane == 0 label("left");
            else label("right");
        }
    }

    panes: Panes;

    frame(*ui, *panes, build);
    frame(*ui, *panes, build);

    expect_near(find(*ui, .{ text = "left" }).rect.w, 100.0, 0.01);
    expect_near(find(*ui, .{ text = "right" }).rect.x, 105.0, 0.01);
} @test
