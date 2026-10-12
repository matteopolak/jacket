make_layer :: (window: *void) -> *void {
    if !window return null;

    view := msg_ptr(window, sel("contentView"));
    if !view return null;

    layer_class := objc_getClass("CAMetalLayer");
    if !layer_class {
        log_error("no CAMetalLayer");
        return null;
    }

    layer := msg_ptr(layer_class, sel("layer"));

    set_layer(view, layer);

    return layer;
}

checks :: (a: int, b: int) -> bool {
    if a < 0 return false;
    if b < 0 return false;
    if a > b return false;

    total := a + b;

    log("%", total);
    for 0..a total += it;

    print("%\n", total);

    return total > 10;
}

short :: (x: int) -> int {
    if x < 0 return 0;
    y := x * 2;
    return y;
}

pick :: (k: int) -> int {
    if k == {
        case 1; return 2;
        case 2;
            log("two");
            return 3;
        case 3;
            log("three");
            log("more");
            return 4;
    }

    a := k + 1;
    b := a * 2;

    log("%", b);

    return b;
}
