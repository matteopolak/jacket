create_window :: (width: int, height: int, window_name: string, window_x := 0, window_y := 0, parent: Window_Type = None, background_color_rgb := DEFAULT_WINDOW_CREATION_COLOR, wanted_msaa: s32 = DEFAULT_MSAA) -> Window {
    if width <= 0 || height <= 0 || width > 0x7fffffff || height > 0x7fffffff return None;

    display := Connection.get_display();
    if !display return None;

    screen := XDefaultScreen(display);
    root := XRootWindow(display, screen);
    attributes := s32.[GLX_X_RENDERABLE, 1, GLX_DRAWABLE_TYPE, GLX_WINDOW_BIT, GLX_RENDER_TYPE, GLX_RGBA_BIT, GLX_X_VISUAL_TYPE, GLX_TRUE_COLOR, GLX_RED_SIZE, 8, GLX_GREEN_SIZE, 8, GLX_BLUE_SIZE, 8, GLX_ALPHA_SIZE, 8, GLX_DOUBLEBUFFER, 1, 0];
    count: s32;
    configs := glXChooseFBConfig(display, screen, attributes.data, *count);

    if !configs return None;

    defer XFree(configs);
    selected: *XVisualInfo;
    best_samples: s32 = -1;

    for index: 0..count - 1 {
        samples, buffers: s32;
        if glXGetFBConfigAttrib(display, configs[index], GLX_SAMPLE_BUFFERS, *buffers) != 0 continue;
        if buffers != 0 && glXGetFBConfigAttrib(display, configs[index], GLX_SAMPLES, *samples) != 0 continue;
        if samples > Basic.max(cast(s32) 0, wanted_msaa) || samples <= best_samples continue;

        visual := visual_for_config(display, configs[index]);
        if !visual continue;
        if selected XFree(selected);

        selected = visual;
        best_samples = samples;
    }

    if !selected return None;

    defer XFree(selected);
    colormap := XCreateColormap(display, root, selected.visual, AllocNone);
    if colormap == None return None;

    committed := false;

    defer {
        if !committed XFreeColormap(display, colormap);
    }

    color: XColor;
    color.red = cast(u16) (background_color_rgb[0] * 65535);
    color.green = cast(u16) (background_color_rgb[1] * 65535);
    color.blue = cast(u16) (background_color_rgb[2] * 65535);
    if XAllocColor(display, colormap, *color) == 0 return None;

    native: XSetWindowAttributes;
    native.colormap = colormap;
    native.background_pixel = color.pixel;
    native.event_mask = KeyPressMask | KeyReleaseMask | ButtonPressMask | ButtonReleaseMask | PointerMotionMask | StructureNotifyMask | FocusChangeMask | ExposureMask;
    actual_parent := parent;
    if actual_parent == None actual_parent = root;

    window := XCreateWindow(display, actual_parent, cast(s32) window_x, cast(s32) window_y, cast(u32) width, cast(u32) height, 0, selected.depth, InputOutput, selected.visual, CWColormap | CWBackPixel | CWEventMask, *native);
    if window == None return None;

    title := Basic.to_c_string(window_name);

    defer Basic.free(title);
    XStoreName(display, window, title);
    close_atom := XInternAtom(display, "WM_DELETE_WINDOW", 0);
    if close_atom == None || XSetWMProtocols(display, window, *close_atom, 1) == 0 {
        XDestroyWindow(display, window);
        return None;
    }

    Basic.array_add(*x_global_windows, window);
    Connection.register_window(window, colormap);
    XMapWindow(display, window);
    XFlush(display);
    committed = true;

    return window;
}

#scope_module
CWBackPixel :: cast(u64) 1 << 1;

Basic :: #import "Basic";

glx_native :: #library,system "GLX";
visual_for_config :: (display: *Display, config: GLXFBConfig) -> *XVisualInfo #foreign glx_native "glXGetVisualFromFBConfig";
