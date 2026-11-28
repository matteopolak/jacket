// The 64-bit golden-ratio multiplier, `0x9e3779b97f4a7c15`, the same constant [combine_hashes] multiplies by.
KNUTH_GOLDEN_RATIO_64 :: 11400714819323198485;

// Mixes `b` into the running hash `a` and returns the result.
//
// The mix is order dependent, so `combine_hashes(a, b)` differs from `combine_hashes(b, a)`. Arithmetic wraps on overflow.
combine_hashes :: (a: u64, b: u64) -> u64 #no_aoc {
    return a * 0x9e3779b97f4a7c15 + b;
}

// Returns a 64-bit id for a call site, used to key widget state between frames.
//
// Hashes the file name, line and column of `loc` together with `identifier`. Pass distinct `identifier` values to tell apart
// widgets created from the same call site, such as the iterations of a loop.
get_hash :: (loc: Source_Code_Location, identifier: s64) -> u64 #no_aoc {
    h: u64 = 0;
    for byte: loc.fully_pathed_filename h = combine_hashes(h, cast(u64) byte);
    return combine_hashes(combine_hashes(combine_hashes(h, cast,no_check(u64) loc.line_number), cast,no_check(u64) loc.character_number), cast,no_check(u64) identifier);
}

// Returns the [Rect] with origin `x`, `y` and size `w` by `h`.
get_rect :: (x: float, y: float, w: float, h: float) -> Rect {
    return .{ x, y, w, h };
}

// Returns `rect` grown by `amount` pixels on all four sides.
//
// A negative `amount` shrinks it. The size is not clamped, so shrinking past the size gives a negative width or height.
expand :: (rect: Rect, amount: float) -> Rect {
    return .{ rect.x - amount, rect.y - amount, rect.w + 2 * amount, rect.h + 2 * amount };
}

// Returns `rect` grown by `amount` pixels on its left and right sides.
expand_x :: (rect: Rect, amount: float) -> Rect {
    return .{ rect.x - amount, rect.y, rect.w + 2 * amount, rect.h };
}

// Returns `rect` grown by `amount` pixels on its low-y and high-y sides.
expand_y :: (rect: Rect, amount: float) -> Rect {
    return .{ rect.x, rect.y - amount, rect.w, rect.h + 2 * amount };
}

// Returns `rect` reduced by `amount` pixels on all four sides, the inverse of [expand].
shrink :: (rect: Rect, amount: float) -> Rect {
    return expand(rect, -amount);
}

// Returns `rect` reduced by `amount` pixels on its left and right sides.
shrink_x :: (rect: Rect, amount: float) -> Rect {
    return expand_x(rect, -amount);
}

// Returns `rect` reduced by `amount` pixels on its low-y and high-y sides.
shrink_y :: (rect: Rect, amount: float) -> Rect {
    return expand_y(rect, -amount);
}

// Splits `rect` into a strip of width about `amount` at its left edge and the remainder to the right of it.
//
// `margin` is the gap left between the two pieces: each piece loses `margin / 2` along the cut. The heights are unchanged.
cut_left :: (rect: Rect, amount: float, margin := 0.0) -> (left: Rect, remainder: Rect) {
    return .{ rect.x, rect.y, amount - margin / 2, rect.h }, .{ rect.x + amount + margin / 2, rect.y, rect.w - amount - margin / 2, rect.h };
}

// Splits `rect` into a strip of width about `amount` at its right edge and the remainder to the left of it.
//
// `margin` is the gap left between the two pieces: each piece loses `margin / 2` along the cut. The heights are unchanged.
cut_right :: (rect: Rect, amount: float, margin := 0.0) -> (right: Rect, remainder: Rect) {
    return .{ rect.x + rect.w - amount + margin / 2, rect.y, amount - margin / 2, rect.h }, .{ rect.x, rect.y, rect.w - amount - margin / 2, rect.h };
}

// Splits `rect` into a strip of height about `amount` at its bottom edge and the remainder above it.
//
// The y axis points up in GetRect, so the bottom strip is the one with the lowest y. With `LEFT_HANDED` the y axis points down and the
// strip is the one with the highest y. `margin` is the gap left between the two pieces: each piece loses `margin / 2` along the cut.
cut_bottom :: (rect: Rect, amount: float, margin := 0.0) -> (bottom: Rect, remainder: Rect) {
    #if LEFT_HANDED return cut_high_y(rect, amount, margin);
    else return cut_low_y(rect, amount, margin);
}

// Splits `rect` into a strip of height about `amount` at its top edge and the remainder below it.
//
// The y axis points up in GetRect, so the top strip is the one with the highest y. With `LEFT_HANDED` the y axis points down and the
// strip is the one with the lowest y. `margin` is the gap left between the two pieces: each piece loses `margin / 2` along the cut.
cut_top :: (rect: Rect, amount: float, margin := 0.0) -> (top: Rect, remainder: Rect) {
    #if LEFT_HANDED return cut_low_y(rect, amount, margin);
    else return cut_high_y(rect, amount, margin);
}

// Returns whether the point `x`, `y` lies inside `r`, edges included.
is_inside :: (x: float, y: float, r: Rect) -> bool {
    return x >= r.x && x <= r.x + r.w && y >= r.y && y <= r.y + r.h;
}

// Returns the area covered by both `a` and `b`.
//
// When they don't overlap the result has a width or height of 0, positioned at the larger of the two origins.
intersect :: (a: Rect, b: Rect) -> Rect {
    x := max(a.x, b.x);
    y := max(a.y, b.y);
    return .{ x, y, max(0.0, min(a.x + a.w, b.x + b.w) - x), max(0.0, min(a.y + a.h, b.y + b.h) - y) };
}

// Returns the smallest [Rect] that contains both `a` and `b`.
bound :: (a: Rect, b: Rect) -> Rect {
    x := min(a.x, b.x);
    y := min(a.y, b.y);
    return .{ x, y, max(a.x + a.w, b.x + b.w) - x, max(a.y + a.h, b.y + b.h) - y };
}

// Returns the four corners of `r` in drawing order.
//
// With `snap_coordinates` each corner is first rounded to a whole pixel. The corners start at the (x, y) corner and run in the
// order that matches the y axis direction: counter-clockwise in GetRect, clockwise with `LEFT_HANDED`.
get_quad :: (using r: Rect, snap_coordinates := false) -> (p0: Vector2, p1: Vector2, p2: Vector2, p3: Vector2) {
    x0 := x;
    y0 := y;
    x1 := x + w;
    y1 := y + h;

    if snap_coordinates {
        x0 = pixel(x0);
        y0 = pixel(y0);
        x1 = pixel(x1);
        y1 = pixel(y1);
    }

    #if LEFT_HANDED return .{ x0, y0 }, .{ x0, y1 }, .{ x1, y1 }, .{ x1, y0 };
    else return .{ x0, y0 }, .{ x1, y0 }, .{ x1, y1 }, .{ x0, y1 };
}

// Returns whether the point `x`, `y` is inside the current scissor rectangle, or true when no scissor is set.
is_visible_in_scissor :: (x: float, y: float) -> bool {
    if !has_scissor return true;
    return is_inside(x, y, current_scissor);
}

// Returns whether any part of `rect` overlaps the current scissor rectangle, or true when no scissor is set.
//
// A rect that only touches the scissor's edge counts as outside.
is_visible_in_scissor :: (using rect: Rect) -> bool {
    if !has_scissor return true;
    r := intersect(rect, current_scissor);
    return r.w > 0 && r.h > 0;
}

// Makes `r` the scissor rectangle, replacing the current one without saving it.
//
// Flushes any pending immediate drawing first, then calls `draw_procs.set_scissor` if registered with `r` rounded outward to whole
// pixels. The callback receives the two corners (x0, y0, x1, y1), not a size. Use [push_scissor] and [pop_scissor] to nest.
set_scissor :: (r: Rect) {
    current_scissor = r;
    has_scissor = true;
    if draw_procs.immediate_flush != null draw_procs.immediate_flush();
    if draw_procs.set_scissor != null draw_procs.set_scissor(cast(s32) floor(r.x), cast(s32) floor(r.y), cast(s32) ceil(r.x + r.w), cast(s32) ceil(r.y + r.h));
}

// Intersects `_rect` with the current scissor, if one is set, and makes that the scissor, saving the previous one.
//
// Every call needs a matching [pop_scissor].
push_scissor :: (_rect: Rect) {
    array_add(*scissor_stack, .{ current_scissor, has_scissor });
    r := _rect;
    if has_scissor r = intersect(r, current_scissor);
    set_scissor(r);
}

// Restores the scissor that was current before the last [push_scissor], or clears it if there was none.
//
// Asserts when called without a matching [push_scissor].
pop_scissor :: () {
    assert(scissor_stack.count > 0, "unbalanced scissor stack");
    item := scissor_stack[scissor_stack.count - 1];

    scissor_stack.count -= 1;

    if item.enabled set_scissor(item.rect);
    else {
        has_scissor = false;
        if draw_procs.immediate_flush != null draw_procs.immediate_flush();
        if draw_procs.clear_scissor != null draw_procs.clear_scissor();
    }
}

// Returns `color` with its x, y and z components scaled toward 0 by `t`.
//
// `t` of 0 leaves the color unchanged and 1 gives black. Other components, such as alpha, are unchanged.
darken :: (color: $T/interface VBase3, t: float) -> T {
    r := color;
    r.x *= 1 - t;
    r.y *= 1 - t;
    r.z *= 1 - t;

    return r;
}

// Returns `color` with its x, y and z components blended toward 1 by `t`.
//
// `t` of 0 leaves the color unchanged and 1 gives white. Other components, such as alpha, are unchanged.
whiten :: (color: $T/interface VBase3, t: float) -> T {
    r := color;
    r.x = lerp(r.x, 1.0, t);
    r.y = lerp(r.y, 1.0, t);
    r.z = lerp(r.z, 1.0, t);

    return r;
}

// Returns `color` with `value` added to its x, y and z components.
//
// The result is not clamped to the 0 to 1 range. Other components, such as alpha, are unchanged.
brighten :: (color: $T/interface VBase3, value: float) -> T {
    r := color;
    r.x += value;
    r.y += value;
    r.z += value;

    return r;
}

// Returns the width of `r` divided by its height.
get_aspect :: (r: Rect) -> float {
    return r.w / r.h;
}

// Returns `r` scaled by the ratio of the current render target size to `old_w` by `old_h`.
//
// Use it to carry a layout saved at one window size over to the current one. The current size is the one last given to
// [ui_per_frame_update] or [switch_window].
rescale :: (r: Rect, old_w: s32, old_h: s32) -> Rect {
    sx := cast(float) render_width / cast(float) old_w;
    sy := cast(float) render_height / cast(float) old_h;
    return .{ r.x * sx, r.y * sy, r.w * sx, r.h * sy };
}

// Returns the point `v` scaled by the ratio of the current render target size to `old_w` by `old_h`.
rescale :: (v: Vector2, old_w: s32, old_h: s32) -> Vector2 {
    return .{ v.x * cast(float) render_width / cast(float) old_w, v.y * cast(float) render_height / cast(float) old_h };
}

// Returns a length in pixels derived from `roundedness`, limited to the size of `r`.
//
// `type` says how to read `roundedness`: a fraction of the height or of the width of `r`, a pixel count as given, or `global_value`
// (ignoring `roundedness`). The result is at least 0 and at most the smaller side of `r` times `limit`. It is used for corner radii and
// frame and divider thicknesses.
get_roundedness_for_rect :: (r: Rect, type: Size_Computation, roundedness: float, global_value: float, limit := 0.5) -> float {
    n := roundedness;
    if type == .RELATIVE_TO_HEIGHT n *= r.h;
    else if type == .RELATIVE_TO_WIDTH n *= r.w;
    else if type == .ABSOLUTE_FROM_GLOBAL n = global_value;

    return max(0.0, min(n, min(r.w, r.h) * limit));
}

// Draws `r` as a filled rectangle with optional rounded corners and a frame, and returns the frame thickness and corner radius used, in pixels.
//
// The frame is `frame_color` and the inside is `color`. The corner radius comes from `shape`, and so does the frame thickness unless
// `frame_thickness_override` is given. The frame is limited to half the smaller side of `r`. Nothing is drawn when `r` has no area.
// Only the corners named in `shape.rounding_flags` are rounded. The compass names refer to the y axis: north is the high-y side, or the
// low-y side with `LEFT_HANDED`.
//
// Needs the `immediate_quad` or `immediate_triangle` callback in [Draw_Procs]; asserts when neither is registered.
rounded_rectangle :: (r: Rect, shape: Rectangle_Shape, color: Vector4, frame_color: Vector4, frame_thickness_override := FLOAT32_INFINITY) -> (frame_pixels: float, actual_roundedness: float) {
    radius := get_roundedness_for_rect(r, shape.roundedness_type, shape.roundedness, rounding_pixels);
    frame := frame_thickness_override;

    if isinf(frame) frame = get_roundedness_for_rect(r, shape.frame_thickness_type, shape.frame_thickness, frame_pixels_global);

    frame = clamp(frame, 0.0, max(0.0, min(r.w, r.h) / 2));
    if r.w <= 0 || r.h <= 0 return frame, radius;
    if draw_procs.set_shader_for_color != null draw_procs.set_shader_for_color(true);

    if frame > 0 {
        rounded_fill(r, radius, shape, frame_color);
        rounded_fill(shrink(r, frame), max(0.0, radius - frame), shape, color);
    } else rounded_fill(r, radius, shape, color);

    return frame, radius;
}

#scope_module
pixel :: (x: float) -> float {
    return floor(x + 0.5);
}

cut_low_y :: (r: Rect, a: float, m: float) -> (Rect, Rect) {
    return .{ r.x, r.y, r.w, a - m / 2 }, .{ r.x, r.y + a + m / 2, r.w, r.h - a - m / 2 };
}

cut_high_y :: (r: Rect, a: float, m: float) -> (Rect, Rect) {
    return .{ r.x, r.y + r.h - a + m / 2, r.w, a - m / 2 }, .{ r.x, r.y, r.w, r.h - a - m / 2 };
}

Scissor_Save :: struct {
    rect: Rect;
    enabled: bool;
}

scissor_stack: [..] Scissor_Save;
current_scissor: Rect;
has_scissor: bool;

rounded_fill :: (r: Rect, radius: float, shape: Rectangle_Shape, color: Vector4) {
    if r.w <= 0 || r.h <= 0 return;

    if radius == 0 {
        a, b, c, d := get_quad(r, true);
        if draw_procs.immediate_quad != null draw_procs.immediate_quad(a, b, c, d, color);
        else {
            emit_triangle(xyz(a, 0), xyz(b, 0), xyz(c, 0), color);
            emit_triangle(xyz(a, 0), xyz(c, 0), xyz(d, 0), color);
        }

        return;
    }

    center := Vector3.{ pixel(r.x + r.w / 2), pixel(r.y + r.h / 2), 0 };
    segments := cast(int) clamp(ceil(PI * radius / 2 / max(1.0, shape.pixels_per_edge_segment)), 1.0, 256.0);
    first: Vector3;
    previous: Vector3;
    count := 0;

    for corner: 0..3 {
        cx := r.x + radius;
        cy := r.y + radius;
        flag := Rectangle_Shape.Rounding_Flags.SOUTHWEST;

        if corner == 1 {
            cx = r.x + r.w - radius;
            flag = .SOUTHEAST;
        }

        if corner == 2 {
            cx = r.x + r.w - radius;
            cy = r.y + r.h - radius;
            flag = .NORTHEAST;
        }

        if corner == 3 {
            cy = r.y + r.h - radius;
            flag = .NORTHWEST;
        }

        #if LEFT_HANDED {
            if corner == 0 flag = .NORTHWEST;
            if corner == 1 flag = .NORTHEAST;
            if corner == 2 flag = .SOUTHEAST;
            if corner == 3 flag = .SOUTHWEST;
        }

        for step: 0..segments {
            a := -PI + cast(float) corner * PI / 2 + cast(float) step / cast(float) segments * PI / 2;
            point := Vector3.{ pixel(cx + radius * cos(a)), pixel(cy + radius * sin(a)), 0 };

            if (shape.rounding_flags & flag) == 0 {
                if corner == 0 point = .{ pixel(r.x), pixel(r.y), 0 };
                if corner == 1 point = .{ pixel(r.x + r.w), pixel(r.y), 0 };
                if corner == 2 point = .{ pixel(r.x + r.w), pixel(r.y + r.h), 0 };
                if corner == 3 point = .{ pixel(r.x), pixel(r.y + r.h), 0 };
            }

            if count == 0 first = point;
            else emit_triangle(center, previous, point, color);

            previous = point;
            count += 1;
        }
    }

    emit_triangle(center, previous, first, color);
}

emit_triangle :: (a: Vector3, b: Vector3, c: Vector3, color: Vector4) {
    if draw_procs.immediate_triangle != null {
        #if LEFT_HANDED draw_procs.immediate_triangle(a, c, b, color, color, color);
        else draw_procs.immediate_triangle(a, b, c, color, color, color);
    } else {
        assert(draw_procs.immediate_quad != null, "register a triangle or quad drawing callback");
        #if LEFT_HANDED draw_procs.immediate_quad(a.xy, c.xy, b.xy, b.xy, color);
        else draw_procs.immediate_quad(a.xy, b.xy, c.xy, c.xy, color);
    }
}
