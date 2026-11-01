// Streaming VT parser and bounded grid. No window, PTY or UI dependencies.
#import "Basic";
#import "String";
U :: #import "Unicode";
#load "grid.jai";
#load "parser.jai";

Attr :: struct {
    fg: u32 = 0xd8dee9;
    bg: u32 = 0x161a22;
    bold: bool;
    italic: bool;
    underline: bool;
    inverse: bool;
}

Cell :: struct {
    code: u32 = 32;
    width: int = 1; // 0 is the trailing half of a wide glyph
    attr: Attr;
}

Grid :: struct {
    cols: int;
    rows: int;
    cells: [] Cell;
    saved: [] Cell;
    history: [..] Cell;
    history_limit: int = 2000;
    history_head: int;
    x: int;
    y: int;
    save_x: int;
    save_y: int;
    primary_x: int;
    primary_y: int;
    top: int;
    bottom: int;
    attr: Attr;
    alternate: bool;
    bracketed: bool;
    application_cursor: bool;
    replies: [..] u8;
    cursor_visible: bool = true;
    wrap_pending: bool;
    state: int; // ground, escape, CSI, OSC, OSC escape, charset
    params: [32] int;
    param_count: int = 1;
    private_mode: bool;
    sequence_overflow: bool;
    osc: [..] u8;
    title: string;
    utf_code: u32;
    utf_min: u32;
    utf_left: int;
}
