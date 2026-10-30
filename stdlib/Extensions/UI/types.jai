// Geometry, colour and style types shared by the whole library. Everything is in logical points;
// the renderer multiplies by the window's content scale.

// An axis-aligned rectangle: `x`, `y` is the top left corner.
Rect :: struct {
    x: float;
    y: float;
    w: float;
    h: float;
}

// A colour with straight (not premultiplied) alpha, each channel 0 to 1.
Color :: Vector4;

// A colour from 0 to 255 channels.
//
// - r: red, 0 to 255.
// - g: green, 0 to 255.
// - b: blue, 0 to 255.
// - a: alpha, 0 (clear) to 255 (opaque).
rgb :: (r: int, g: int, b: int, a: int = 255) -> Color {
    return .{ r / 255.0, g / 255.0, b / 255.0, a / 255.0 };
}

// A colour from a `0xRRGGBB` number.
//
// - value: the colour as `0xRRGGBB`.
// - alpha: alpha, 0 to 1.
hex :: (value: u32, alpha: float = 1) -> Color {
    return .{ ((value >> 16) & 0xFF) / 255.0, ((value >> 8) & 0xFF) / 255.0, (value & 0xFF) / 255.0, alpha };
}

// `color` with its alpha replaced.
//
// - color: the colour to start from.
// - alpha: the new alpha, 0 to 1.
with_alpha :: (color: Color, alpha: float) -> Color {
    return .{ color.x, color.y, color.z, alpha };
}

// Whether the point lies inside `rect`.
//
// - rect: the rectangle. Its left and top edges count as inside, its right and bottom edges do not.
// - x: the point's horizontal position.
// - y: the point's vertical position.
contains :: (rect: Rect, x: float, y: float) -> bool {
    return x >= rect.x && y >= rect.y && x < rect.x + rect.w && y < rect.y + rect.h;
}

// The overlap of two rectangles (zero-sized when they do not touch).
//
// - a: the first rectangle.
// - b: the second rectangle.
intersect :: (a: Rect, b: Rect) -> Rect {
    x0 := max(a.x, b.x);
    y0 := max(a.y, b.y);
    x1 := min(a.x + a.w, b.x + b.w);
    y1 := min(a.y + a.h, b.y + b.h);

    return .{ x0, y0, max(x1 - x0, 0), max(y1 - y0, 0) };
}

// `rect` shrunk by `amount` on every side.
//
// - rect: the rectangle to shrink.
// - amount: how far to move each edge in. The size stops at zero instead of going negative.
inset :: (rect: Rect, amount: float) -> Rect {
    return .{ rect.x + amount, rect.y + amount, max(rect.w - 2 * amount, 0), max(rect.h - 2 * amount, 0) };
}

// Which way a container lays out its children.
Axis :: enum u8 {
    VERTICAL; // a column
    HORIZONTAL; // a row
    STACK; // children overlap, each placed in the container's rectangle
}

// How a container places children across its axis.
Align :: enum u8 {
    STRETCH; // fill the container
    START;
    CENTER;
    END;
}

// How a container places children along its axis when none of them grow.
Justify :: enum u8 {
    START;
    CENTER;
    END;
    SPACE_BETWEEN;
}

// What a widget is, kept on its node for tests, for tools and for a later accessibility export.
Role :: enum u8 {
    GROUP;
    TEXT;
    BUTTON;
    CHECKBOX;
    TEXT_FIELD;
    TEXT_EDITOR;
    SCROLL;
    SPLIT;
    TAB_LIST;
    TAB;
    LIST_ITEM;
    TREE_ITEM;
    SEPARATOR;
    CUSTOM;
}

// Layout and paint options for a container or widget. Zero values mean "automatic" or "none", so a
// style is written with only what differs: `.{gap = 8, padding = 12}`.
Style :: struct {
    gap: float; // space between children
    padding: float; // space inside the edge, all sides
    padding_x: float; // added to `padding` on the left and right
    padding_y: float; // added to `padding` on the top and bottom
    grow: float; // share of the space left over in the parent's axis
    width: float; // fixed width (0: from content)
    height: float; // fixed height (0: from content)
    min_width: float;
    min_height: float;
    align: Align; // placement of children across the axis
    justify: Justify; // placement of children along the axis
    background: Color; // alpha 0: none
    border_color: Color;
    border: float; // border width
    radius: float; // corner radius
    clip: bool; // clip children to the rectangle
}

// `value` rounded to the nearest whole number (halves go up).
//
// - value: the number to round.
round :: (value: float) -> float {
    return floor(value + 0.5);
}
