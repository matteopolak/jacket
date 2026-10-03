# Clipboard

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `os_clipboard_get_text :: () -> string`
- `os_clipboard_set_text :: (text: string)`
- `os_clipboard_set_bitmap :: (width: s32, height: s32, pixels: *u8, channels: s32, flip := false)`
- `os_clipboard_empty :: ()`

## Public type members

No public aggregate fields or enum members were identified.

## Behavioral contract and constraints

The module exposes clipboard read/write operations through platform-specific implementations. The signatures do not return an operation status. Returned text and accepted formats depend on the active platform backend; the public interface does not define a common failure-reporting policy or a common bitmap encoding.

On Linux, the public API also has `Window`-taking overloads: `os_clipboard_get_text :: (win: Window) -> string`, `os_clipboard_set_text :: (win: Window, text: string)`, and `os_clipboard_set_bitmap :: (win: Window, width: s32, height: s32, pixels: *u8, channels: s32, flip := false)`. The Linux branch also exports no-window overloads of these three procedures. Its returned text is documented as allocated through `alloc` and must be manually freed. It does not declare `os_clipboard_empty`.

The Linux declarations depend on X11. `X11.init_global_display()` must be called before using the clipboard API so that the display connection and clipboard atoms are initialized; X11 initialization also establishes Xlib threading before opening its display. The X11 selection protocol is asynchronous and requires the application to dispatch selection-request events through the X11 input handler. The X11 provider advertises UTF-8 text and can advertise `image/bmp` when its public RGB pixel buffer is populated. These X11 provider facts do not define a clipboard-specific channel layout, the meaning of `flip`, whether the supplied pixel pointer may be retained, the selection-owner relationship to the `Window` argument, timeout behavior for another client, or persistence after application exit. Those details are not specified by the public Clipboard contract.
