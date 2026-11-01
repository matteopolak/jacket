View :: struct {
    zoom: float = 1;
    x: float;
    y: float;
}

fit_view :: (w: s32, h: s32, width: float, height: float) -> View {
    if w <= 0 || h <= 0 return .{};
    zoom := min(width / w, height / h);
    return .{ zoom, (width - w * zoom) / 2, (height - h * zoom) / 2 };
}

// Keep the image point under the cursor fixed while zooming, including at the clamps.
zoom_at :: (view: *View, factor: float, x: float, y: float) {
    next := clamp(view.zoom * factor, 0.01, 64.0);
    ratio := next / view.zoom;

    view.x = x - (x - view.x) * ratio;
    view.y = y - (y - view.y) * ratio;
    view.zoom = next;
}
