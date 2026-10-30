// Drawing with Simp. Everything is a textured quad from one of two sheets (sprites and font), so a
// frame is two batches. The world is VIEW_W x VIEW_H units; `scale` turns units into pixels, and the
// same code draws into a window or into an offscreen texture that the smoke test reads back.

Renderer :: struct {
    window: Window_Type;
    sheet: Texture;
    font: Texture;
    target: Texture;
    offscreen: bool;
    scale: float = 4;
    shake: Vector2;
    clear: Vector4 = .{ 0.03, 0.03, 0.08, 1 };
}

// Opens the window. Returns false when there is no display or no OpenGL.
renderer_open_window :: (r: *Renderer, scale: s64) -> bool {
    r.scale = cast(float) scale;
    r.window = create_window(VIEW_W * scale, VIEW_H * scale, "game2d");

    if !r.window return false;

    for 1..3 {
        update_window_events();
        sleep_milliseconds(5);
    }

    prepare_window(r.window, 0);
    set_render_target(r.window, .LEFT_HANDED);

    return true;
}

// A texture to draw into, for the smoke test. It still needs a window for the OpenGL context.
renderer_open_offscreen :: (r: *Renderer, scale: s64) -> bool {
    r.window = create_window(64, 64, "game2d offscreen");

    if !r.window return false;

    for 1..3 {
        update_window_events();
        sleep_milliseconds(5);
    }

    prepare_window(r.window, 0);
    set_render_target(r.window, .LEFT_HANDED);

    r.offscreen = true;
    r.scale = cast(float) scale;
    r.target = texture_create_render_target(cast(s32) (VIEW_W * scale), cast(s32) (VIEW_H * scale), .RGBA8);

    return true;
}

renderer_begin :: (r: *Renderer) {
    if r.offscreen set_render_target(*r.target, .LEFT_HANDED);
    else set_render_target(r.window, .LEFT_HANDED);

    clear_render_target(r.clear.x, r.clear.y, r.clear.z, r.clear.w);
}

renderer_end :: (r: *Renderer) {
    immediate_flush();

    if !r.offscreen swap_buffers(r.window);
}

// Reads the offscreen target back as RGBA8, top row first.
renderer_read :: (r: *Renderer) -> Bitmap {
    immediate_flush();

    reader := pixel_read_begin(*r.target, .RGBA8, -1, -1, false, true);

    for 1..2000 {
        if pixel_read_is_done(reader) break;

        sleep_milliseconds(1);
    }

    return pixel_read_end(reader);
}

// Half a texel in from each side, so a cell never samples its neighbour.
cell_uv :: (texture: *Texture, column: s64, row: s64, cols: s64, rows: s64) -> Vector2, Vector2 {
    inset_x := 0.5 / cast(float) texture.width;
    inset_y := 0.5 / cast(float) texture.height;
    u0 := cast(float) column / cast(float) cols + inset_x;
    v0 := cast(float) row / cast(float) rows + inset_y;
    u1 := cast(float) (column + 1) / cast(float) cols - inset_x;
    v1 := cast(float) (row + 1) / cast(float) rows - inset_y;

    return .{ u0, v0 }, .{ u1, v1 };
}

// A quad of `size` world units centred on (x, y), turned by `angle` radians.
quad :: (r: *Renderer, x: float, y: float, w: float, h: float, angle: float, uv0: Vector2, uv1: Vector2, color: Vector4) {
    s := r.scale;
    cx := (x + r.shake.x) * s;
    cy := (y + r.shake.y) * s;
    hw := w * s / 2;
    hh := h * s / 2;
    c := cos(angle);
    si := sin(angle);

    corner :: (dx: float, dy: float, cx: float, cy: float, c: float, si: float) -> Vector2 {
        return .{ cx + dx * c - dy * si, cy + dx * si + dy * c };
    }

    immediate_quad(corner(-hw, -hh, cx, cy, c, si), corner(hw, -hh, cx, cy, c, si), corner(hw, hh, cx, cy, c, si), corner(-hw, hh, cx, cy, c, si),
        color, uv0, .{ uv1.x, uv0.y }, uv1, .{ uv0.x, uv1.y });
}

use_sprites :: (r: *Renderer) {
    set_shader_for_images(*r.sheet);
}

use_font :: (r: *Renderer) {
    set_shader_for_images(*r.font);
}

sprite :: (r: *Renderer, cell: Cell, x: float, y: float, size: float, angle := 0.0, color := Vector4.{ 1, 1, 1, 1 }) {
    index := cast(s64) cell;
    uv0, uv1 := cell_uv(*r.sheet, index % SHEET_COLUMNS, index / SHEET_COLUMNS, SHEET_COLUMNS, SHEET_ROWS);

    quad(r, x, y, size, size, angle, uv0, uv1, color);
}

// A filled rectangle with its top-left corner at (x, y), drawn with the sheet's white cell.
rect :: (r: *Renderer, x: float, y: float, w: float, h: float, color: Vector4) {
    uv0, uv1 := cell_uv(*r.sheet, 0, 0, SHEET_COLUMNS, SHEET_ROWS);

    quad(r, x + w / 2, y + h / 2, w, h, 0, uv0 + .{ 0.003, 0.003 }, uv1 - .{ 0.003, 0.003 }, color);
}

// Text in capitals; `size` is the height of a glyph in world units. Returns the width drawn.
text :: (r: *Renderer, s: string, x: float, y: float, size := 8.0, color := Vector4.{ 1, 1, 1, 1 }) -> float {
    advance := size * 6.0 / 8.0 * 1.0 + size / 8.0;
    cursor := x;

    for byte: s {
        c := byte;

        if c >= #char "a" && c <= #char "z" c -= 32;

        slot := cast(s64) c - GLYPH_FIRST;

        if slot >= 0 && slot < GLYPH_COLUMNS * GLYPH_ROWS && c != #char " " {
            uv0, uv1 := cell_uv(*r.font, slot % GLYPH_COLUMNS, slot / GLYPH_COLUMNS, GLYPH_COLUMNS, GLYPH_ROWS);

            quad(r, cursor + size / 2, y + size / 2, size, size, 0, uv0, uv1, color);
        }

        cursor += advance;
    }

    return cursor - x;
}

text_width :: (s: string, size := 8.0) -> float {
    return cast(float) s.count * (size * 6.0 / 8.0 + size / 8.0);
}

text_centered :: (r: *Renderer, s: string, y: float, size := 8.0, color := Vector4.{ 1, 1, 1, 1 }) {
    text(r, s, (VIEW_W - text_width(s, size)) / 2, y, size, color);
}

// The scene: stars, enemies, bullets, the ship, particles and the HUD.
Starfield :: struct {
    stars: [48] Vector3; // x, y, depth
}

starfield_init :: (field: *Starfield, seed: u32) {
    state: u64;

    random_seed(*state, seed);

    for *field.stars it.* = .{ random_get_within_range(*state, 0, VIEW_W), random_get_within_range(*state, 0, VIEW_H), random_get_within_range(*state, 0.2, 1) };
}

draw_stars :: (r: *Renderer, field: *Starfield, time: float) {
    use_sprites(r);

    for star: field.stars {
        y := star.y + time * 20 * star.z;

        y -= floor(y / VIEW_H) * VIEW_H;
        sprite(r, ifx star.z > 0.7 then Cell.STAR_BIG else Cell.STAR_SMALL, star.x, y, 8, 0, .{ 1, 1, 1, star.z });
    }
}

draw_world :: (r: *Renderer, world: *World, field: *Starfield, time: float) {
    r.shake = .{ 0, 0 };

    if world.shake > 0 r.shake = .{ sin(time * 90) * world.shake * 0.3, cos(time * 77) * world.shake * 0.3 };

    draw_stars(r, field, time);

    for pickup: world.pickups sprite(r, .SHIELD, pickup.pos.x, pickup.pos.y, 14, time * 2);

    for enemy: world.enemies {
        cell := Cell.DRONE;

        if enemy.kind == .CHASER cell = .CHASER;
        if enemy.kind == .TANK cell = .TANK;

        sprite(r, cell, enemy.pos.x, enemy.pos.y, 16, ifx enemy.kind == .CHASER then enemy.age * 2 else 0.0);
    }

    for bullet: world.bullets {
        if bullet.from_player sprite(r, .SHOT, bullet.pos.x, bullet.pos.y, 10, atan2(bullet.vel.x, -bullet.vel.y));
        else sprite(r, .ENEMY_SHOT, bullet.pos.x, bullet.pos.y, 8);
    }

    if world.state == .PLAYING {
        blink := world.player.invulnerable > 0 && (cast(s64) (world.player.invulnerable * 12)) % 2 == 0;

        if !blink sprite(r, .PLAYER, world.player.pos.x, world.player.pos.y, 16, world.player.angle);

        if world.player.invulnerable > 0 sprite(r, .SHIELD, world.player.pos.x, world.player.pos.y, 22, time * 3, .{ 1, 1, 1, 0.6 });
    }

    for particle: world.particles {
        t := 1 - particle.life / particle.max_life;
        color := particle.color;
        cell := particle.frame_cell;

        color.w = 1 - t;

        if cell == .EXPLOSION_0 {
            cell = cast(Cell) (cast(s64) Cell.EXPLOSION_0 + min(3, cast(s64) (t * 4)));
        }

        sprite(r, cell, particle.pos.x, particle.pos.y, particle.size, 0, color);
    }

    r.shake = .{ 0, 0 };
}

draw_hud :: (r: *Renderer, world: *World, status: string) {
    use_sprites(r);

    for 0..cast(s64) world.lives - 1 sprite(r, .LIFE, 12 + cast(float) it * 12, VIEW_H - 10, 10);

    use_font(r);

    text(r, tprint("SCORE %", world.score), 6, 5, 8);
    text(r, tprint("WAVE %", world.wave), VIEW_W - 6 - text_width("WAVE 00"), 5, 8, .{ 0.7, 0.9, 1, 1 });

    if status text(r, status, VIEW_W - 6 - text_width(status, 6), VIEW_H - 12, 6, .{ 0.6, 0.6, 0.7, 1 });
}
