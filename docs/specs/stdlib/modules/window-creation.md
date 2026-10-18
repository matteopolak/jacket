# Window_Creation


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `WT :: #import "Window_Type"`
- `Window_Type :: WT.Window_Type`

- [OS == .WINDOWS] `WINDOWED_STYLE : u32 : WS_OVERLAPPEDWINDOW`
- [OS == .WINDOWS] `FULLSCREEN_STYLE : u32 : WS_VISIBLE | WS_POPUP`
- [OS == .WINDOWS] `SECONDARY_WINDOW_STYLE : u32 : WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_THICKFRAME`
- [OS == .WINDOWS] `create_window :: (width: int, height: int, window_name: string, window_x := -1, window_y := -1, parent: Window_Type = null, background_color_rgb := DEFAULT_WINDOW_CREATION_COLOR, wanted_msaa := DEFAULT_MSAA) -> HWND`
- [OS == .WINDOWS] `get_dimensions :: (hwnd: HWND, right_handed: bool) -> (x: s32, y: s32, width: s32, height: s32, success := true)`
- [OS == .WINDOWS] `get_mouse_pointer_position :: (hwnd: HWND, right_handed: bool) -> (x: int, y: int, success: bool)`
- [OS == .WINDOWS] `get_mouse_pointer_position :: inline (right_handed := false) -> x: int, y: int, success: bool`
- [OS == .WINDOWS] `toggle_fullscreen :: (hwnd: HWND, desire_fullscreen: bool, saved: *Saved_Window_Info) -> (succcess: bool, width: s32, height: s32)`
- [OS == .LINUX] `create_window :: (width: int, height: int, window_name: string, window_x := 0, window_y := 0, parent: Window_Type = None, background_color_rgb := DEFAULT_WINDOW_CREATION_COLOR, wanted_msaa: s32 = DEFAULT_MSAA) -> Window`
- [OS == .LINUX] `get_dimensions :: (win: Window, right_handed: bool) -> (x: s32, y: s32, width: s32, height: s32, success := true)`
- [OS == .LINUX] `get_mouse_pointer_position :: (dpy: *Display, win: Window, right_handed: bool) -> x: int, y: int, success: bool`
- [OS == .LINUX] `get_mouse_pointer_position :: (win: Window, right_handed: bool) -> x: int, y: int, success: bool`
- [OS == .LINUX] `get_mouse_pointer_position :: inline (right_handed := false) -> x: int, y: int, success: bool`
- [OS == .LINUX] `toggle_fullscreen :: (win: Window, desire_fullscreen: bool, saved: *Saved_Window_Info) -> success: bool, width: s32, height: s32`
- [OS == .LINUX] `window_minimized := false`
- [OS == .MACOS] `application_initialized := false`
- [OS == .MACOS] `init_mac_app :: ()`
- [OS == .MACOS] `create_window :: (width: int, height: int, window_name: string, window_x := -1, window_y := -1, parent: Window_Type = null, background_color_rgb := DEFAULT_WINDOW_CREATION_COLOR, wanted_msaa := DEFAULT_MSAA) -> *NSWindow`
- [OS == .MACOS] `swap_view_buffers :: (win: *NSWindow)`
- [OS == .MACOS] `get_dimensions :: (window: *NSWindow, right_handed: bool) -> (x: s32, y: s32, width: s32, height: s32, success := true)`
- [OS == .MACOS] `get_mouse_pointer_position :: (window: *NSWindow, right_handed: bool) -> x: int, y: int, success: bool`
- [OS == .MACOS] `get_mouse_pointer_position :: inline (right_handed := false) -> x: int, y: int, success: bool`
- [OS == .MACOS] `toggle_fullscreen :: (win: *NSWindow, desire_fullscreen: bool, saved: *Saved_Window_Info) -> success: bool, width: s32, height: s32`
- [OS == .ANDROID] `create_window :: (width: int, height: int, window_name: string, window_x := 0, window_y := 0, parent: Window_Type = null, background_color_rgb := DEFAULT_WINDOW_CREATION_COLOR, wanted_msaa := DEFAULT_MSAA) -> *Android.ANativeWindow`
- [OS == .ANDROID] `get_dimensions :: (win: *Android.ANativeWindow, right_handed: bool) -> (x: s32, y: s32, width: s32, height: s32, success := true)`
- [OS == .ANDROID] `get_mouse_pointer_position :: (win: *Android.ANativeWindow, right_handed: bool) -> x: int, y: int, success: bool`
- [OS == .ANDROID] `get_mouse_pointer_position :: inline (right_handed := false) -> x: int, y: int, success: bool`
- [OS == .ANDROID] `toggle_fullscreen :: (win: *Android.ANativeWindow, desire_fullscreen: bool, saved: *Saved_Window_Info) -> success: bool, width: s32, height: s32`
- [OS == .ANDROID] `android_set_fullscreen_for_real :: ()`
- [OS == .WINDOWS | .LINUX | .MACOS | .ANDROID] `Saved_Window_Info :: struct`


## Platform-specific public type members

#### `Saved_Window_Info` (struct, target-specific layouts)

##### Windows

```jai
Saved_Window_Info :: struct {
    rect: RECT;
    style: s32;
    ex_style: s32;
}
```
##### macOS

```jai
Saved_Window_Info :: struct {
    mask: NSWindowStyleMask;
    window_frame: NSRect;
}
```
##### Linux and Android

```jai
Saved_Window_Info :: struct {}
```
## Contracts and constraints

This module creates and queries native windows on Windows, Linux, macOS, and Android. The selected target determines the native handle type and `Saved_Window_Info` layout. Dimension and mouse queries return success indicators; `right_handed` selects the vertical coordinate orientation where supported. Windows ignores `wanted_msaa`; macOS ignores `parent`, `background_color_rgb`, and `wanted_msaa`. Platform-specific fullscreen behavior and dependencies follow the selected native window system.

## Module parameters

```jai
#module_parameters (DEFAULT_MSAA: s32 = 4);
```
Platform coverage note: Nintendo, PS5 branches exist, but their declarations are not specified here; exports for those targets are unspecified.

## Window creation defaults

`create_window`'s `background_color_rgb` parameter defaults to the contextual module constant `float.[0.15, 0.15, 0.2]`, inferred as a three-element `float` array. This is a dark blue-gray RGB color; the constant is in module scope and is not a separate exported name. `wanted_msaa` defaults to `DEFAULT_MSAA`, whose module-parameter default is `4` (`s32`). The `wanted_msaa` argument is ignored on Windows, macOS, and Android; Linux uses it as the requested multisample-buffer count.

The default position is target-specific: Windows uses `(-1, -1)` to request the work-area origin; macOS centers only when both coordinates are `-1`; Linux and Android default to `(0, 0)`. A null parent is the default on all listed targets (Linux spells its null value `None`). The effective `background_color_rgb` parameter type is `[3] float`. Windows uses it when initializing the shared window class, and Linux uses it for the X11 window background; macOS ignores it. Android's implementation asserts that `android_app` is initialized, ignores every creation argument, and returns the current `android_app.window`, so dimensions, title, position, parent, color, and MSAA do not create or configure a distinct native window there.
