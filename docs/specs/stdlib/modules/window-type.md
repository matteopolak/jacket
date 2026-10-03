# Window_Type


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- [OS == .WINDOWS] `Window_Type :: Windows.HWND`
- [OS == .WINDOWS] `INVALID_WINDOW : Window_Type : null`
- [OS == .LINUX] `Window_Type : : X11.Window`
- [OS == .LINUX] `INVALID_WINDOW : Window_Type : 0`
- [OS == .MACOS] `Window_Type :: *ObjC.NSWindow`
- [OS == .MACOS] `INVALID_WINDOW : Window_Type : null`
- [OS == .PS5] `Window_Type :: *void`
- [OS == .PS5] `INVALID_WINDOW : Window_Type : null`
- [OS == .NN_SWITCH2] `Window_Type :: *void`
- [OS == .NN_SWITCH2] `INVALID_WINDOW : Window_Type : null`
- [OS == .ANDROID] `Window_Type :: *Android.ANativeWindow`
- [OS == .ANDROID] `INVALID_WINDOW : Window_Type : null`


## Contracts and constraints

This module aliases the window-handle type selected for each target. The alias does not own, create, or destroy the native window; lifetime and validity are controlled by the native windowing API. `INVALID_WINDOW` is a sentinel only, and Linux's zero value is not guaranteed to be invalid.

The underlying public types are: Windows `HWND` (`*void`); Linux `X11.Window` (`XID`, `u64`); macOS `*ObjC.NSWindow`; Android `*Android.ANativeWindow` (pointer to an empty opaque binding struct); and `*void` on PS5 and Switch 2. The Linux alias is an integer X11 resource ID rather than a pointer. The macOS and Android aliases are borrowed native object pointers; this module provides no retain/release operation.
