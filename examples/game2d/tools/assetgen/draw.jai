// Drawing the sprite sheet and the font. Shapes are signed distance functions evaluated at every
// pixel centre of a cell, with one pixel of soft edge, composited with "over".

Canvas :: struct {
    width: s64;
    height: s64;
    pixels: [] Vector4;
}

canvas_make :: (width: s64, height: s64) -> Canvas {
    canvas: Canvas;

    canvas.width = width;
    canvas.height = height;
    canvas.pixels = NewArray(width * height, Vector4);

    return canvas;
}

// Composites `color` (straight alpha) times `coverage` over the pixel.
blend :: (canvas: *Canvas, x: s64, y: s64, color: Vector4, coverage: float) {
    if x < 0 || y < 0 || x >= canvas.width || y >= canvas.height return;

    source_alpha := color.w * clamp(coverage, 0, 1);

    if source_alpha <= 0 return;

    below := *canvas.pixels[y * canvas.width + x];
    out_alpha := source_alpha + below.w * (1 - source_alpha);

    below.x = (color.x * source_alpha + below.x * below.w * (1 - source_alpha)) / out_alpha;
    below.y = (color.y * source_alpha + below.y * below.w * (1 - source_alpha)) / out_alpha;
    below.z = (color.z * source_alpha + below.z * below.w * (1 - source_alpha)) / out_alpha;
    below.w = out_alpha;
}

Shape :: #type (x: float, y: float) -> float;

// Fills the cell at (`cx`, `cy`) with `color` where `shape(x, y) < 0`; x and y are in pixels from
// the cell's centre.
fill :: (canvas: *Canvas, cx: s64, cy: s64, size: s64, color: Vector4, shape: Shape) {
    half := cast(float) size / 2;

    for py: 0..size - 1 for px : 0..size - 1 {
        x := cast(float) px + 0.5 - half;
        y := cast(float) py + 0.5 - half;

        blend(canvas, cx + px, cy + py, color, 0.5 - shape(x, y));
    }
}

sd_circle :: (x: float, y: float, radius: float) -> float {
    return sqrt(x * x + y * y) - radius;
}

sd_box :: (x: float, y: float, hw: float, hh: float) -> float {
    dx := abs(x) - hw;
    dy := abs(y) - hh;

    return length(Vector2.{ max(dx, 0), max(dy, 0) }) + min(max(dx, dy), 0);
}

// Distance to a convex polygon given counter-clockwise (in y-up terms) as a list of points.
sd_convex :: (x: float, y: float, points: [] Vector2) -> float {
    worst := -1000.0;

    for 0..points.count - 1 {
        a := points[it];
        b := points[(it + 1) % points.count];
        edge := b - a;
        normal := normalize(Vector2.{ edge.y, -edge.x });
        distance := (x - a.x) * normal.x + (y - a.y) * normal.y;

        worst = max(worst, distance);
    }

    return worst;
}

ring :: (d: float, thickness: float) -> float {
    return abs(d) - thickness / 2;
}

cell_origin :: (cell: Cell) -> s64, s64 {
    index := cast(s64) cell;

    return (index % SHEET_COLUMNS) * SHEET_CELL, (index / SHEET_COLUMNS) * SHEET_CELL;
}

encode_sprite_sheet :: () -> [] u8 {
    sheet := canvas_make(SHEET_COLUMNS * SHEET_CELL, SHEET_ROWS * SHEET_CELL);

    for cell: enum_values_as_enum(Cell) {
        cx, cy := cell_origin(cell);

        draw_cell(*sheet, cell, cx, cy);
    }

    return encode_canvas(sheet);
}

encode_canvas :: (canvas: Canvas) -> [] u8 {
    image: Image;

    image.width = cast(s32) canvas.width;
    image.height = cast(s32) canvas.height;
    image.format = .RGBA8;
    image.pixels = NewArray(canvas.width * canvas.height * 4, u8);

    for canvas.pixels {
        image.pixels[it_index * 4 + 0] = cast(u8) (clamp(it.x, 0, 1) * 255 + 0.5);
        image.pixels[it_index * 4 + 1] = cast(u8) (clamp(it.y, 0, 1) * 255 + 0.5);
        image.pixels[it_index * 4 + 2] = cast(u8) (clamp(it.z, 0, 1) * 255 + 0.5);
        image.pixels[it_index * 4 + 3] = cast(u8) (clamp(it.w, 0, 1) * 255 + 0.5);
    }

    return encode_png(image, 9);
}

draw_cell :: (sheet: *Canvas, cell: Cell, cx: s64, cy: s64) {
    size := SHEET_CELL;

    if #complete cell == {
        case .WHITE;
            fill(sheet, cx, cy, size, .{ 1, 1, 1, 1 }, (x, y) => -1);
        case .PLAYER;
            fill(sheet, cx, cy, size, .{ 0.2, 0.55, 0.75, 1 }, (x, y) => sd_convex(x, y, Vector2.[.{ 0, -7 }, .{ 7, 6 }, .{ -7, 6 }]));
            fill(sheet, cx, cy, size, .{ 0.55, 0.95, 1, 1 }, (x, y) => sd_convex(x, y, Vector2.[.{ 0, -6 }, .{ 3, 2 }, .{ -3, 2 }]));
            fill(sheet, cx, cy, size, .{ 1, 0.8, 0.3, 1 }, (x, y) => sd_circle(x, y - 5.5, 1.5));
        case .DRONE;
            fill(sheet, cx, cy, size, .{ 0.9, 0.25, 0.2, 1 }, (x, y) => sd_circle(x, y, 6));
            fill(sheet, cx, cy, size, .{ 1, 0.7, 0.4, 1 }, (x, y) => ring(sd_circle(x, y, 3.5), 1));
            fill(sheet, cx, cy, size, .{ 0.1, 0.05, 0.05, 1 }, (x, y) => sd_circle(x, y, 1.5));
        case .CHASER;
            fill(sheet, cx, cy, size, .{ 0.85, 0.3, 0.9, 1 }, (x, y) => sd_convex(x, y, Vector2.[.{ 0, -7 }, .{ 7, 0 }, .{ 0, 7 }, .{ -7, 0 }]));
            fill(sheet, cx, cy, size, .{ 1, 0.85, 1, 1 }, (x, y) => sd_circle(x, y, 2.5));
        case .TANK;
            fill(sheet, cx, cy, size, .{ 0.35, 0.6, 0.25, 1 }, (x, y) => sd_box(x, y, 7, 7));
            fill(sheet, cx, cy, size, .{ 0.75, 0.9, 0.35, 1 }, (x, y) => ring(sd_box(x, y, 5, 5), 1.2));
            fill(sheet, cx, cy, size, .{ 0.15, 0.25, 0.1, 1 }, (x, y) => sd_box(x, y, 2, 2));
        case .SHOT;
            fill(sheet, cx, cy, size, .{ 0.6, 1, 1, 1 }, (x, y) => sd_box(x, y, 1.2, 4));
        case .ENEMY_SHOT;
            fill(sheet, cx, cy, size, .{ 1, 0.5, 0.3, 1 }, (x, y) => sd_circle(x, y, 2.5));
        case .DOT;
            fill(sheet, cx, cy, size, .{ 1, 1, 1, 0.9 }, (x, y) => sd_circle(x, y, 2.2));
            fill(sheet, cx, cy, size, .{ 1, 1, 1, 0.35 }, (x, y) => sd_circle(x, y, 5));
        case .EXPLOSION_0; draw_explosion(sheet, cx, cy, 0);
        case .EXPLOSION_1; draw_explosion(sheet, cx, cy, 1);
        case .EXPLOSION_2; draw_explosion(sheet, cx, cy, 2);
        case .EXPLOSION_3; draw_explosion(sheet, cx, cy, 3);
        case .SHIELD;
            fill(sheet, cx, cy, size, .{ 0.3, 0.8, 1, 1 }, (x, y) => ring(sd_circle(x, y, 6), 2));
            fill(sheet, cx, cy, size, .{ 0.8, 1, 1, 1 }, (x, y) => sd_circle(x, y, 2));
        case .STAR_SMALL;
            fill(sheet, cx, cy, size, .{ 0.8, 0.85, 1, 0.8 }, (x, y) => sd_circle(x, y, 0.8));
        case .STAR_BIG;
            fill(sheet, cx, cy, size, .{ 1, 0.95, 0.8, 0.9 }, (x, y) => sd_circle(x, y, 1.4));
            fill(sheet, cx, cy, size, .{ 1, 0.95, 0.8, 0.5 }, (x, y) => sd_box(x, y, 4, 0.5));
            fill(sheet, cx, cy, size, .{ 1, 0.95, 0.8, 0.5 }, (x, y) => sd_box(x, y, 0.5, 4));
        case .LIFE;
            fill(sheet, cx, cy, size, .{ 0.2, 0.55, 0.75, 1 }, (x, y) => sd_convex(x, y, Vector2.[.{ 0, -5 }, .{ 5, 5 }, .{ -5, 5 }]));
    }
}

// Four frames of a burst: a flash that grows into a ring of sparks and fades.
draw_explosion :: (sheet: *Canvas, cx: s64, cy: s64, frame: s64) {
    t := cast(float) frame / 3;
    radius := 2 + t * 5.5;
    fade := 1 - t * 0.7;

    for spark: 0..7 {
        angle := cast(float) spark * PI / 4 + t;
        sx := cos(angle) * radius;
        sy := sin(angle) * radius;
        size := 1.6 - t;

        for py: 0..SHEET_CELL - 1 for px : 0..SHEET_CELL - 1 {
            x := cast(float) px + 0.5 - SHEET_CELL / 2.0;
            y := cast(float) py + 0.5 - SHEET_CELL / 2.0;

            blend(sheet, cx + px, cy + py, .{ 1, 0.75 - t * 0.35, 0.25, fade }, 0.5 - sd_circle(x - sx, y - sy, size));
        }
    }

    for py: 0..SHEET_CELL - 1 for px : 0..SHEET_CELL - 1 {
        x := cast(float) px + 0.5 - SHEET_CELL / 2.0;
        y := cast(float) py + 0.5 - SHEET_CELL / 2.0;

        blend(sheet, cx + px, cy + py, .{ 1, 1, 0.8, fade }, 0.5 - sd_circle(x, y, 3 - t * 2.5));
    }
}

// Glyphs are drawn as seven rows of five pixels, '#' for a lit pixel, rows separated by '/'.
// There are only capitals: the game shows text in upper case.
GLYPHS :: string.[
    " |...../...../...../...../...../...../.....",
    "!|..#../..#../..#../..#../..#../...../..#..",
    "'|..#../..#../...../...../...../...../.....",
    "(|...#./..#../.#.../.#.../.#.../..#../...#.",
    ")|.#.../..#../...#./...#./...#./..#../.#...",
    "*|...../#.#.#/.###./#####/.###./#.#.#/.....",
    "+|...../..#../..#../#####/..#../..#../.....",
    ",|...../...../...../...../..##./..#../.#...",
    "-|...../...../...../#####/...../...../.....",
    ".|...../...../...../...../...../.##../.##..",
    "/|....#/....#/...#./..#../.#.../#..../#....",
    "0|.###./#...#/#..##/#.#.#/##..#/#...#/.###.",
    "1|..#../.##../..#../..#../..#../..#../.###.",
    "2|.###./#...#/....#/...#./..#../.#.../#####",
    "3|.###./#...#/....#/..##./....#/#...#/.###.",
    "4|...#./..##./.#.#./#..#./#####/...#./...#.",
    "5|#####/#..../####./....#/....#/#...#/.###.",
    "6|..##./.#.../#..../####./#...#/#...#/.###.",
    "7|#####/....#/...#./..#../.#.../.#.../.#...",
    "8|.###./#...#/#...#/.###./#...#/#...#/.###.",
    "9|.###./#...#/#...#/.####/....#/...#./.##..",
    ":|...../.##../.##../...../.##../.##../.....",
    "<|...#./..#../.#.../#..../.#.../..#../...#.",
    "=|...../...../#####/...../#####/...../.....",
    ">|.#.../..#../...#./....#/...#./..#../.#...",
    "?|.###./#...#/....#/...#./..#../...../..#..",
    "A|.###./#...#/#...#/#####/#...#/#...#/#...#",
    "B|####./#...#/#...#/####./#...#/#...#/####.",
    "C|.###./#...#/#..../#..../#..../#...#/.###.",
    "D|####./#...#/#...#/#...#/#...#/#...#/####.",
    "E|#####/#..../#..../####./#..../#..../#####",
    "F|#####/#..../#..../####./#..../#..../#....",
    "G|.###./#...#/#..../#.###/#...#/#...#/.###.",
    "H|#...#/#...#/#...#/#####/#...#/#...#/#...#",
    "I|.###./..#../..#../..#../..#../..#../.###.",
    "J|..###/...#./...#./...#./...#./#..#./.##..",
    "K|#...#/#..#./#.#../##.../#.#../#..#./#...#",
    "L|#..../#..../#..../#..../#..../#..../#####",
    "M|#...#/##.##/#.#.#/#.#.#/#...#/#...#/#...#",
    "N|#...#/##..#/#.#.#/#..##/#...#/#...#/#...#",
    "O|.###./#...#/#...#/#...#/#...#/#...#/.###.",
    "P|####./#...#/#...#/####./#..../#..../#....",
    "Q|.###./#...#/#...#/#...#/#.#.#/#..#./.##.#",
    "R|####./#...#/#...#/####./#.#../#..#./#...#",
    "S|.####/#..../#..../.###./....#/....#/####.",
    "T|#####/..#../..#../..#../..#../..#../..#..",
    "U|#...#/#...#/#...#/#...#/#...#/#...#/.###.",
    "V|#...#/#...#/#...#/#...#/#...#/.#.#./..#..",
    "W|#...#/#...#/#...#/#.#.#/#.#.#/##.##/#...#",
    "X|#...#/#...#/.#.#./..#../.#.#./#...#/#...#",
    "Y|#...#/#...#/.#.#./..#../..#../..#../..#..",
    "Z|#####/....#/...#./..#../.#.../#..../#####",
    "_|...../...../...../...../...../...../#####",
    "%|##..#/##..#/...#./..#../.#.../#..##/#..##",
];

encode_font :: () -> [] u8 {
    font := canvas_make(GLYPH_COLUMNS * GLYPH_W, GLYPH_ROWS * GLYPH_H);

    for glyph: GLYPHS {
        code := glyph[0];
        slot := cast(s64) code - GLYPH_FIRST;
        gx := (slot % GLYPH_COLUMNS) * GLYPH_W;
        gy := (slot / GLYPH_COLUMNS) * GLYPH_H;
        rows := split(slice(glyph, 2, glyph.count - 2), "/");

        for row: 0..rows.count - 1 for column : 0..rows[row].count - 1 {
            if rows[row][column] == #char "#" blend(*font, gx + column + 1, gy + row, .{ 1, 1, 1, 1 }, 1);
        }
    }

    return encode_canvas(font);
}
