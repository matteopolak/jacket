get_display :: () -> *X11.Display {
    if !X11.x_global_display {
        // Xlib requires this before the first display, including when Clipboard
        // later starts its independent selection-service connection/thread.
        initialized := X11.XInitThreads();
        if initialized == 0 return null;

        X11.x_global_display = X11.XOpenDisplay(null);
    }

    return X11.x_global_display;
}

register_window :: (window: X11.Window, colormap: X11.Colormap) {
    Basic.array_add(*resources, Resource.{ window, colormap });
}

release_window :: (window: X11.Window) {
    for resource, index: resources if resource.window == window {
        if resource.colormap != 0 X11.XFreeColormap(X11.x_global_display, resource.colormap);
        Basic.array_unordered_remove_by_index(*resources, index);
        break;
    }

    for item, index: X11.x_global_windows if item == window {
        Basic.array_unordered_remove_by_index(*X11.x_global_windows, index);
        break;
    }
} @Releases

pointer_motion :: (window: X11.Window, x: s32, y: s32) -> int, int {
    for *resource: resources if resource.window == window {
        dx: int;
        dy: int;

        if resource.have_pointer {
            dx = cast(int) x - resource.x;
            dy = cast(int) y - resource.y;
        }

        resource.have_pointer = true;
        resource.x = x;
        resource.y = y;

        return dx, dy;
    }

    return 0, 0;
}

#scope_module
X11 :: #import "X11";
Basic :: #import "Basic";

Resource :: struct {
    window: X11.Window;
    colormap: X11.Colormap;
    x, y: s32;
    have_pointer: bool;
}

resources: [..] Resource;
