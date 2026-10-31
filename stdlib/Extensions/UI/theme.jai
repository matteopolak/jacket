// Colours, sizes and spacing widgets take unless a style says otherwise.

// Light or dark defaults for `default_theme`.
//
// ```jai,skip_on(wasm)
// light := default_theme(.LIGHT);
// dark := default_theme(.DARK);
// assert(light.background != dark.background);
// ```
Theme_Mode :: enum u8 {
    DARK;
    LIGHT;
}

// The look of the library's widgets. Change fields on `ui.theme` (or `set_theme`) to restyle everything.
//
// ```jai,skip_on(wasm)
// t := default_theme();
// t.accent = hex(0xFF5500);
// ```
Theme :: struct {
    background: Color; // the window
    surface: Color; // panels and bars
    surface_raised: Color; // controls: buttons, fields
    surface_hover: Color;
    surface_pressed: Color;
    text: Color;
    text_muted: Color;
    accent: Color;
    accent_text: Color; // text on the accent colour
    outline: Color;
    selection: Color;
    caret: Color;
    error: Color;
    warning: Color;
    font_size: float = 14;
    mono_size: float = 13;
    radius: float = 5;
    padding: float = 8; // inside a control
    gap: float = 8; // between controls
}

// The built-in theme for `mode`.
//
// - mode: dark or light colours.
//
// ```jai,skip_on(wasm)
// ui := headless(.{ theme = .LIGHT });
// defer free_ui(*ui);
// assert(ui.theme.background == default_theme(.LIGHT).background);
// ```
default_theme :: (mode: Theme_Mode = .DARK) -> Theme {
    theme: Theme;

    if mode == .DARK {
        theme.background = hex(0x1E1F22);
        theme.surface = hex(0x25262A);
        theme.surface_raised = hex(0x303236);
        theme.surface_hover = hex(0x3A3D42);
        theme.surface_pressed = hex(0x2A2C30);
        theme.text = hex(0xDCDDE0);
        theme.text_muted = hex(0x8B8E94);
        theme.accent = hex(0x4C7DF0);
        theme.accent_text = hex(0xFFFFFF);
        theme.outline = hex(0x3F4247);
        theme.selection = hex(0x4C7DF0, 0.38);
        theme.caret = hex(0xF2F3F5);
        theme.error = hex(0xF0605D);
        theme.warning = hex(0xE5B94A);
    } else {
        theme.background = hex(0xF6F6F7);
        theme.surface = hex(0xEBEBED);
        theme.surface_raised = hex(0xFFFFFF);
        theme.surface_hover = hex(0xE2E3E6);
        theme.surface_pressed = hex(0xD5D7DB);
        theme.text = hex(0x1D1E20);
        theme.text_muted = hex(0x6B6E75);
        theme.accent = hex(0x2F62E0);
        theme.accent_text = hex(0xFFFFFF);
        theme.outline = hex(0xC9CBD0);
        theme.selection = hex(0x2F62E0, 0.28);
        theme.caret = hex(0x1D1E20);
        theme.error = hex(0xD6403D);
        theme.warning = hex(0xB5851A);
    }

    return theme;
}

// Replace the theme of the UI being built.
//
// - theme: the theme to use from now on.
//
// ```jai,skip_on(wasm)
// ui := headless();
// defer free_ui(*ui);
// push_ui(*ui);
// custom := default_theme();
// custom.accent = hex(0xFF5500);
// set_theme(custom);
// assert(theme().accent == custom.accent);
// pop_ui();
// ```
set_theme :: (theme: Theme) {
    context.ui.theme = theme;
}

// The theme of the UI being built.
//
// ```jai,skip_on(wasm)
// ui := headless();
// defer free_ui(*ui);
// push_ui(*ui);
// accent := theme().accent;
// pop_ui();
// ```
theme :: () -> *Theme {
    return *context.ui.theme;
}
