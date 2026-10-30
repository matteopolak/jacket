// A tour of the UI library's widgets in one window: buttons, a checkbox, a text field, a list,
// tabs, a split with a scrolling pane and a small code editor. Native only (it needs a window and a
// GPU). Run it with `jaic examples/ui-gallery/main.jai -interp` or build and run it.
//
//     jaic examples/ui-gallery/main.jai -interp -- --screenshot gallery.png
//
// writes a PNG of the first frame instead of opening a window; `-- --frames 30` closes the window
// after 30 frames.

#import "Basic";
#import "Math";
#import "Extensions/UI";

Gallery :: struct {
    count: int;
    dark: bool = true;
    tab: int;
    selected: int;
    split: float = 220;
    name: Text_Edit;
    code: Text_Edit;
    started: bool;
}

FRUIT :: string.["Apple", "Banana", "Cherry", "Date", "Elderberry", "Fig", "Grape", "Honeydew", "Kiwi", "Lemon", "Mango", "Nectarine"];
TABS :: string.["Widgets", "Editor"];

build :: (g: *Gallery) {
    if !g.started {
        g.started = true;
        set_contents(*g.code, "// An editor widget:\n// type, select, undo (Cmd+Z) and scroll.\n\nmain :: () {\n    print(\"Hello, UI!\\n\");\n}\n");
    }

    for column() {
        for tab_bar() {
            for name, i: TABS {
                if tab(name, g.tab == i) == .SELECT g.tab = i;
            }
        }

        if g.tab == 0 widgets(g);
        else editor(g);
    }
}

widgets :: (g: *Gallery) {
    for pane: split(.HORIZONTAL, *g.split) {
        if pane == 0 {
            for scroll(.{ grow = 1, padding = 6 }) {
                for fruit, i: FRUIT {
                    key(i);
                    if list_item(fruit, g.selected == i) g.selected = i;
                }
            }
        } else {
            for column(.{ gap = 10, padding = 16 }) {
                label("Widgets", .{ size = 22 });
                label(tprint("You picked %", FRUIT[g.selected]), .{ muted = true });

                for row(.{ gap = 8, align = .CENTER }) {
                    label(tprint("Clicked % times", g.count));
                    if button("Click me", .{ variant = .PRIMARY }) g.count += 1;
                    if g.count > 0 && button("Reset") g.count = 0;
                }

                if checkbox(*g.dark, "Dark theme") set_theme(default_theme(ifx g.dark then Theme_Mode.DARK else Theme_Mode.LIGHT));

                for row(.{ gap = 8, align = .CENTER }) {
                    label("Name");
                    text_field(*g.name, .{ placeholder = "Type here", grow = 1 });
                }

                label(tprint("Hello, %!", ifx g.name.buffer.length then contents(*g.name) else "stranger"));
            }
        }
    }
}

editor :: (g: *Gallery) {
    text_editor(*g.code, .{ line_numbers = true });
}

main :: () {
    args := get_command_line_arguments();
    gallery: Gallery;
    gallery.name.single_line = true;

    if args.count >= 3 && args[1] == "--screenshot" {
        ui := headless(.{ width = 720, height = 440, scale = 2, system_fonts = true });

        defer free_ui(*ui);

        frame(*ui, *gallery, build);

        if !write_screenshot(*ui, args[2]) {
            print("could not write %\n", args[2]);
            exit(1);
        }

        print("wrote %\n", args[2]);

        return;
    }

    frames := 0;
    if args.count >= 3 && args[1] == "--frames" frames = string_to_int(args[2]);

    if !run_app(.{ title = "UI gallery", width = 720, height = 440, frame_limit = frames }, *gallery, build) exit(1);
}
