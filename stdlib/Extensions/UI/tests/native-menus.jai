#import "Input";
#import "Extensions/Desktop";
#import "Extensions/UI";
#import "Extensions/Testing";

menu_click_queues_exactly_one_primary_shortcut :: (t: *Test) {
    ui := headless();

    defer free_ui(*ui);
    items := Menu_Item.[.{ menu = "File", title = "Save As", action = 2, key = #char "S", shift = true }];

    send_menu_action(*ui, items, 2);
    expect_equal(ui.input.events.count, 1);
    event := ui.input.events[0];

    expect(event.kind == .KEY);
    expect(event.key == cast(Key_Code) #char "S");
    expect(primary(event.mods) && event.mods.shift);
    send_menu_action(*ui, items, 999);
    expect_equal(ui.input.events.count, 1);
} @test

menu_without_shortcut_queues_a_command :: (t: *Test) {
    ui := headless();

    defer free_ui(*ui);
    items := Menu_Item.[.{ menu = "Tools", title = "Run", action = 42 }];

    send_menu_action(*ui, items, 42);
    expect_equal(ui.input.events.count, 1);
    expect(ui.input.events[0].kind == .COMMAND);
    expect_equal(ui.input.events[0].action, 42);
} @test

Command_Receiver :: struct {
    received: u16;
    calls: int;
}

receive_command :: (receiver: *Command_Receiver) {
    for event: current_ui().input.keys if event.kind == .COMMAND {
        receiver.received = event.action;
        receiver.calls += 1;
    }
}

command_reaches_the_build_frame_once :: (t: *Test) {
    ui := headless();

    defer free_ui(*ui);
    receiver: Command_Receiver;

    send_command(*ui, 42);
    frame(*ui, *receiver, receive_command);
    frame(*ui, *receiver, receive_command);
    expect_equal(receiver.received, 42);
    expect_equal(receiver.calls, 1);
} @test
