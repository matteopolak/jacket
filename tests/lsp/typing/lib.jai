Shape :: enum u8 {
    CIRCLE;
    SQUARE;
}

Body :: struct {
    shape: Shape;
    size: float;
    name: string;
}

scaled :: (body: Body, factor: float) -> Body {
    result := body;
    result.size = body.size * factor;
    return result;
}

area :: (body: Body) -> float {
    if body.shape == .CIRCLE return 3.14159 * body.size * body.size; // jailint: allow(approx_constant)
    return body.size * body.size;
}

describe :: (body: Body) -> string {
    return tprint("% (%)", body.name, body.shape);
}
