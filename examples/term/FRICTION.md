# Friction log

## Fixed in the tools

- POSIX has no PTY allocation bindings or macOS terminal ioctls. Fixed by "feat(pty): add nonblocking pseudoterminal sessions": private libc bindings live in `Extensions/Pty`, keeping the mirrored POSIX API unchanged. Tests cover native output, input, terminal size, foreground SIGWINCH, child reaping and invalid dimensions. On macOS the slave must be opened before setting the initial size; ioctl is C variadic on arm64.

## Open

- Windows ConPTY is not implemented. The program and PTY extension type-check for Windows; the interactive entry point reports unsupported and exits 2.
- This environment exposes no Metal adapter. Headless real-PTY, FreeType/display-list, selection and clipboard-hook checks run; a native window and GPU PNG/pixel readback cannot be verified here. The smoke prints a skip unless `REQUIRE_GPU=1`.
- The UI renderer only has axis-aligned glyph quads. Synthetic italic uses horizontal strips; bold redraws a cached glyph with an offset. True font-weight/style selection would be better for large grids.
- Unicode has scalar conversion but no terminal column-width/grapheme API. The app classifies common CJK/emoji ranges itself and ignores combining accents/variation selectors. ZWJ sequences, full Unicode width tables and complex shaping remain open.
- Resize clears history and clips overlapping cells instead of reflowing. Selection addresses the current viewport; it is not anchored while new output scrolls or the viewport changes.
- This is a VT subset: no mouse reporting, origin mode, programmable tab stops, DEC graphics charset, sixel, hyperlinks, full modifier/function-key encoding, or full xterm compatibility. BEL has no sound. OSC changes the in-app title strip, not the native window title.
