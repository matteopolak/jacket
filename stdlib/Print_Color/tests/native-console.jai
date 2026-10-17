#import "Basic";
Keep :: #import "Print_Color"(SAVE_RESTORE_ON_WINDOWS = false);
Save :: #import "Print_Color"(SAVE_RESTORE_ON_WINDOWS = true);

// A fake console: one attribute word per stream, and a count of every change.
fake_attributes: [2] u16;
fake_is_console: [2] bool;
fake_applies: s64;

fake_query :: (to_standard_error: bool) -> (u16, bool) {
    index := ifx to_standard_error then 1 else 0;
    return fake_attributes[index], fake_is_console[index];
}

fake_apply :: (to_standard_error: bool, attributes: u16) {
    fake_attributes[ifx to_standard_error then 1 else 0] = attributes;
    fake_applies += 1;
}

reset_fake :: () {
    fake_attributes = .[0x07, 0x07];
    fake_is_console = .[true, true];
    fake_applies = 0;
    Keep.set_console_backend(.{ fake_query, fake_apply });
    Save.set_console_backend(.{ fake_query, fake_apply });
}

early_return :: () {
    Keep.push_console_color(.RED);
    defer Keep.pop_console_color();
    assert(fake_attributes[0] == 0x04, "inside: %", fake_attributes[0]);
    return;
}

main :: () {
    reset_fake();

    // Attribute mapping: ANSI red is bit 1, Windows red is bit 4; background and bright variants.
    assert(Keep.console_attributes_for(0x07, .RED) == 0x04);
    assert(Keep.console_attributes_for(0x07, .BLUE) == 0x01);
    assert(Keep.console_attributes_for(0x07, .YELLOW) == 0x06);
    assert(Keep.console_attributes_for(0x07, .HI_GREEN) == 0x0a);
    assert(Keep.console_attributes_for(0x07, .BG_BLUE) == 0x17);
    assert(Keep.console_attributes_for(0x07, .BG_HI_RED) == 0xc7);
    assert(Keep.console_attributes_for(0x17, .GREEN) == 0x12, "keeps background");
    assert(Keep.console_attributes_for(0x07, .RED, .BOLD) == 0x0c);
    assert(Keep.console_attributes_for(0x07, .RED, .UNDERLINE) == 0x8004);
    assert(Keep.console_attributes_for(0x8004, .GREEN) == 0x0002, "normal drops underline");

    // Nesting restores the enclosing color, not the default.
    Keep.push_console_color(.RED);
    assert(fake_attributes[0] == 0x04);
    Keep.push_console_color(.BG_BLUE);
    assert(fake_attributes[0] == 0x14);
    assert(Keep.console_color_depth() == 2);
    Keep.pop_console_color();
    assert(fake_attributes[0] == 0x04, "inner pop returns to the red block: %", fake_attributes[0]);
    Keep.pop_console_color();
    assert(fake_attributes[0] == 0x07, "outermost pop returns to default");
    assert(Keep.console_color_depth() == 0);

    // The two outermost-pop policies differ once the console changed between blocks.
    reset_fake();
    Keep.push_console_color(.RED);
    Keep.pop_console_color();
    fake_attributes[0] = 0x1e; // the program changed colors itself
    Keep.push_console_color(.GREEN);
    Keep.pop_console_color();
    assert(fake_attributes[0] == 0x07, "default policy returns to the first baseline: %", fake_attributes[0]);

    reset_fake();
    Save.push_console_color(.RED);
    Save.pop_console_color();
    fake_attributes[0] = 0x1e;
    Save.push_console_color(.GREEN);
    Save.pop_console_color();
    assert(fake_attributes[0] == 0x1e, "save/restore returns to what was there: %", fake_attributes[0]);

    // Streams are independent.
    reset_fake();
    Keep.push_console_color(.RED, to_standard_error = true);
    assert(fake_attributes[0] == 0x07 && fake_attributes[1] == 0x04);
    Keep.pop_console_color(to_standard_error = true);
    assert(fake_attributes[1] == 0x07);

    // Cleanup on early return.
    reset_fake();
    early_return();
    assert(fake_attributes[0] == 0x07 && Keep.console_color_depth() == 0);

    // A redirected stream is never touched, but pushes stay balanced.
    reset_fake();
    fake_is_console = .[false, false];
    Keep.push_console_color(.RED);
    assert(Keep.console_color_depth() == 1);
    Keep.pop_console_color();
    assert(Keep.console_color_depth() == 0);
    assert(fake_applies == 0, "no attribute calls on a redirected stream");

    // A pop without a push does no harm to a stream that never changed.
    reset_fake();
    Keep.pop_console_color();
    assert(fake_applies == 0 && Keep.console_color_depth() == 0);

    // Deeper than the table: still balanced, and the console ends where it began.
    reset_fake();
    for 1..40 Keep.push_console_color(.RED);
    for 1..40 Keep.pop_console_color();

    assert(Keep.console_color_depth() == 0 && fake_attributes[0] == 0x07, "deep: %", fake_attributes[0]);

    print("ok\n");
}
