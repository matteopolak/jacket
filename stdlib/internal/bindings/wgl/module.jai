// Native WGL setup owns a temporary window/DC/context, never the caller's window.
create :: (window: W.HWND, share: W.HGLRC, major: s32, minor: s32,
           compatibility: bool, debug: bool, wanted_samples: s32) -> W.HGLRC, W.HDC, s32 {
    if !window || major < 0 || minor < 0 || wanted_samples < 0 return null, null, 0;
    if major == 0 && minor != 0 return null, null, 0;

    // Keep the bootstrap on the target display for driver-specific entry points.
    bounds: W.RECT;
    if W.GetWindowRect(window, *bounds) == 0 return null, null, 0;

    class_name: [7] u16 = .[83, 84, 65, 84, 73, 67, 0];
    bootstrap := W.CreateWindowExW(0, class_name.data, null, 0, bounds.left, bounds.top,
                                  1, 1, null, null, null, null);

    if !bootstrap return null, null, 0;

    defer if W.DestroyWindow(bootstrap) == 0 abort();

    temporary_dc := W.GetDC(bootstrap);
    if !temporary_dc return null, null, 0;

    defer if W.ReleaseDC(bootstrap, temporary_dc) == 0 abort();

    descriptor := pixel_descriptor();
    basic_format := W.ChoosePixelFormat(temporary_dc, *descriptor);

    if basic_format == 0 || W.SetPixelFormat(temporary_dc, basic_format, *descriptor) == 0 return null, null, 0;

    temporary := W.wglCreateContext(temporary_dc);
    if !temporary return null, null, 0;

    defer if W.wglDeleteContext(temporary) == 0 abort();

    previous_context := W.wglGetCurrentContext();
    previous_dc := W.wglGetCurrentDC();

    if W.wglMakeCurrent(temporary_dc, temporary) == 0 return null, null, 0;

    defer if W.wglMakeCurrent(previous_dc, previous_context) == 0 abort();

    extensions_arb := cast(Extensions_ARB) resolve("wglGetExtensionsStringARB");
    extensions_ext := cast(Extensions_EXT) resolve("wglGetExtensionsStringEXT");
    extensions: *u8;

    if extensions_arb extensions = extensions_arb(temporary_dc);
    if !extensions && extensions_ext extensions = extensions_ext();

    choose: Choose;
    query: Query;
    construct: Construct;

    if has_extension(extensions, "WGL_ARB_pixel_format") {
        choose = cast(Choose) resolve("wglChoosePixelFormatARB");
        query = cast(Query) resolve("wglGetPixelFormatAttribivARB");
        if !choose || !query return null, null, 0;
    }

    if has_extension(extensions, "WGL_ARB_create_context") {
        construct = cast(Construct) resolve("wglCreateContextAttribsARB");
        if !construct return null, null, 0;
    }

    profile := major > 3 || (major == 3 && minor >= 2);
    if profile && !has_extension(extensions, "WGL_ARB_create_context_profile") return null, null, 0;
    if !construct && (major != 0 || debug || compatibility) return null, null, 0;

    dc := W.GetDC(window);
    if !dc return null, null, 0;

    retained := false;

    defer if !retained && W.ReleaseDC(window, dc) == 0 abort();

    multisample := has_extension(extensions, "WGL_ARB_multisample");
    format := W.GetPixelFormat(dc);

    if format == 0 {
        if choose {
            samples := ifx multisample then wanted_samples else 0;

            while true {
                // RGB24 + alpha8 + depth24, double buffered RGBA window surface.
                attributes: [19] s32 = .[0x2001, 1, 0x2010, 1, 0x2011, 1,
                                         0x2013, 0x202B, 0x2014, 24, 0x201B, 8, 0x2022, 24, 0, 0, 0, 0, 0];

                if multisample {
                    attributes[14] = 0x2041;
                    attributes[15] = ifx samples > 0 then 1 else 0;
                    attributes[16] = 0x2042;
                    attributes[17] = samples;
                }

                count: u32;
                if choose(dc, attributes.data, null, 1, *format, *count) == 0 return null, null, 0;
                if count != 0 && format != 0 break;
                if samples == 0 return null, null, 0;

                samples /= 2;
            }
        } else {
            format = W.ChoosePixelFormat(dc, *descriptor);
            if format == 0 return null, null, 0;
        }

        if W.DescribePixelFormat(dc, format, cast(u32) size_of(W.PIXELFORMATDESCRIPTOR), *descriptor) == 0 return null, null, 0;
        if !valid_descriptor(descriptor) return null, null, 0;
        if W.SetPixelFormat(dc, format, *descriptor) == 0 return null, null, 0;
    } else {
        if W.DescribePixelFormat(dc, format, cast(u32) size_of(W.PIXELFORMATDESCRIPTOR), *descriptor) == 0 return null, null, 0;
        if !valid_descriptor(descriptor) return null, null, 0;
    }

    actual_samples: s32;

    if multisample && query {
        names: [2] s32 = .[0x2041, 0x2042];
        values: [2] s32;
        if query(dc, format, 0, 2, names.data, values.data) == 0 return null, null, 0;
        if values[0] != 0 actual_samples = values[1];
    }

    result: W.HGLRC;

    if construct {
        attributes: [9] s32 = .[0x2091, 1, 0x2092, minor, 0x2094, 0, 0, 0, 0];
        if major != 0 attributes[1] = major;
        if debug attributes[5] = 1;

        if profile {
            attributes[6] = 0x9126;
            attributes[7] = ifx compatibility then 2 else 1;
        }

        result = construct(dc, share, attributes.data);
    } else {
        result = W.wglCreateContext(dc);
        if result && share && W.wglShareLists(share, result) == 0 {
            if W.wglDeleteContext(result) == 0 abort();
            return null, null, 0;
        }
    }

    if !result return null, null, 0;

    retained = true;

    return result, dc, actual_samples;
}

#scope_file
W :: #import "Windows";

libc :: #library,system "c";
abort :: () #foreign libc;
strlen :: (text: *u8) -> u64 #foreign libc;
Choose :: #type (dc: W.HDC, integers: *s32, floats: *float, limit: u32, formats: *s32, count: *u32) -> s32 #c_call;
Query :: #type (dc: W.HDC, format: s32, layer: s32, count: u32, attributes: *s32, values: *s32) -> s32 #c_call;
Construct :: #type (dc: W.HDC, share: W.HGLRC, attributes: *s32) -> W.HGLRC #c_call;
Extensions_ARB :: #type (dc: W.HDC) -> *u8 #c_call;
Extensions_EXT :: #type () -> *u8 #c_call;

resolve :: (name: *u8) -> *void {
    address := W.wglGetProcAddress(name);
    value := cast(u64) address;
    if value <= 3 || value == 0xffff_ffff_ffff_ffff return null;
    return address;
}

pixel_descriptor :: () -> W.PIXELFORMATDESCRIPTOR {
    result: W.PIXELFORMATDESCRIPTOR;
    result.nSize = cast(s16) size_of(W.PIXELFORMATDESCRIPTOR);
    result.nVersion = 1;
    result.dwFlags = W.PFD_DRAW_TO_WINDOW | W.PFD_SUPPORT_OPENGL | W.PFD_DOUBLEBUFFER;
    result.iPixelType = W.PFD_TYPE_RGBA;
    result.cColorBits = 24;
    result.cAlphaBits = 8;
    result.cDepthBits = 24;

    return result;
}

valid_descriptor :: (descriptor: W.PIXELFORMATDESCRIPTOR) -> bool {
    needed := W.PFD_DRAW_TO_WINDOW | W.PFD_SUPPORT_OPENGL | W.PFD_DOUBLEBUFFER;
    return (descriptor.dwFlags & needed) == needed && descriptor.iPixelType == W.PFD_TYPE_RGBA;
}

has_extension :: (pointer: *u8, wanted: string) -> bool {
    if !pointer return false;

    text: string = .{ data = pointer, count = cast(s64) strlen(pointer) };
    begin: s64;

    for i: 0..text.count {
        if i == text.count || text[i] == #char " " {
            token: string = .{ data = text.data + begin, count = i - begin };
            if token == wanted return true;
            begin = i + 1;
        }
    }

    return false;
}
