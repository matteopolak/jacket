// Dear ImGui against the pinned library (tools/build_native_libs.py): a context with no window or
// renderer. Builds the font atlas, runs frames that lay out a window with a button, a checkbox and a
// slider, clicks the button and the checkbox with injected mouse events, and checks the draw data
// the frame produces. The bindings name C++ symbols (Itanium or MSVC decorated), so a library built
// from the wrong ImGui revision or with different settings fails here before anything draws.
#import "Basic";
#import "ImGui";

BUTTON_WINDOW_POS :: ImVec2.{ 20, 30 };

// One frame: the window with its widgets. Returns whether the button was clicked this frame.
frame :: (checked: *bool, value: *float, button_min: *ImVec2, button_max: *ImVec2, check_min: *ImVec2) -> bool {
    NewFrame();
    SetNextWindowPos(BUTTON_WINDOW_POS, .Always);
    SetNextWindowSize(ImVec2.{ 320, 180 }, .Always);
    clicked := false;
    if Begin("Test window") {
        clicked = Button("Press");
        <<button_min = GetItemRectMin();
        <<button_max = GetItemRectMax();
        Checkbox("Check", checked);
        <<check_min = GetItemRectMin();
        SliderFloat("Slider", value, 0, 1);
        Text("plain text");
    }
    End();
    Render();
    return clicked;
}

center :: (a: ImVec2, b: ImVec2) -> x: float, y: float {
    return (a.x + b.x) / 2, (a.y + b.y) / 2;
}

main :: () {
    version := to_string(GetVersion());
    assert(version == IMGUI_VERSION, "library version '%', bindings '%'", version, IMGUI_VERSION);

    ctx := CreateContext();
    assert(ctx != null, "CreateContext");
    assert(GetCurrentContext() == ctx, "the new context is current");

    io := GetIO();
    io.DisplaySize = ImVec2.{ 800, 600 };
    io.DeltaTime = 1.0 / 60.0;
    io.IniFilename = null; // no imgui.ini next to the test

    // The built-in font becomes a texture the renderer would upload.
    pixels: *u8;
    width, height: s32;
    io.Fonts.GetTexDataAsRGBA32(io.Fonts, *pixels, *width, *height);
    assert(pixels != null && width > 0 && height > 0, "atlas %x%", width, height);
    lit := 0;
    for i: 0..width * height - 1 if pixels[i * 4 + 3] != 0 lit += 1;
    assert(lit > 100, "the font atlas has % opaque pixels", lit);

    checked := false;
    value := 0.25;
    button_min, button_max, check_min: ImVec2;

    // Frame 1: lay out, no input.
    frame(*checked, *value, *button_min, *button_max, *check_min);
    data := GetDrawData();
    assert(data != null && data.Valid, "draw data");
    assert(data.CmdListsCount > 0 && data.TotalVtxCount > 0 && data.TotalIdxCount > 0,
        "draw lists %, vertices %, indices %", data.CmdListsCount, data.TotalVtxCount, data.TotalIdxCount);
    assert(data.TotalIdxCount % 3 == 0, "triangles: % indices", data.TotalIdxCount);
    assert(button_max.x > button_min.x && button_max.y > button_min.y, "the button has no area: % to %", button_min, button_max);
    assert(button_min.x >= BUTTON_WINDOW_POS.x && button_min.y > BUTTON_WINDOW_POS.y, "the button is inside the window");
    vertices_before := data.TotalVtxCount;

    // Frames 2 and 3: move the pointer onto the button and press it. Frame 4 releases it, which
    // is the click.
    x, y := center(button_min, button_max);
    io.AddMousePosEvent(io, x, y);
    frame(*checked, *value, *button_min, *button_max, *check_min);
    io.AddMouseButtonEvent(io, 0, true);
    pressed := frame(*checked, *value, *button_min, *button_max, *check_min);
    assert(!pressed, "the button is not clicked while it is held");
    io.AddMouseButtonEvent(io, 0, false);
    clicked := frame(*checked, *value, *button_min, *button_max, *check_min);
    assert(clicked, "the button reports the click");

    // The checkbox toggles the same way.
    assert(!checked, "starts unchecked");
    io.AddMousePosEvent(io, check_min.x + 5, check_min.y + 5);
    frame(*checked, *value, *button_min, *button_max, *check_min);
    io.AddMouseButtonEvent(io, 0, true);
    frame(*checked, *value, *button_min, *button_max, *check_min);
    io.AddMouseButtonEvent(io, 0, false);
    frame(*checked, *value, *button_min, *button_max, *check_min);
    assert(checked, "the checkbox toggled");

    // Hovering drew something different from frame 1 (the highlight is more vertices).
    assert(GetDrawData().TotalVtxCount > 0 && vertices_before > 0);

    // The style API: a theme changes colors in the style the context owns.
    style := GetStyle();
    StyleColorsLight();
    light := style.Colors[0];
    StyleColorsDark();
    dark := style.Colors[0];
    assert(light.x != dark.x || light.y != dark.y || light.z != dark.z, "light and dark themes differ");

    DestroyContext(ctx);
    print("imgui ok\n");
}
