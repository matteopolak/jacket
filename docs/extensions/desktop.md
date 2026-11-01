# Desktop

Native menus, file dialogs and IME text input for windows made by `Window_Creation`. `@experimental`. macOS and Windows; on Linux `create_native_menu` returns null, dialogs return `UNSUPPORTED` and composition is off, so apps fall back to shortcuts and in-app prompts.

```jai
#import "Extensions/Desktop";

ITEMS :: Menu_Item.[
    .{menu = "File", title = "Open…", action = 1, key = #char "O"},
    .{menu = "File", title = "Save As…", action = 2, key = #char "S", shift = true},
];

menu := create_native_menu(window, ITEMS);   // null on Linux or for an invalid model
defer destroy_native_menu(menu);

while running {
    update_events();                          // instead of Input.update_window_events
    while action := poll_native_menu(menu) { ... }

    result := native_file_dialog(window, .OPEN);
    defer free(result.path);
    if result.status == .ACCEPTED open_file(result.path);
}
```

Apps on [UI](ui.md) don't call this directly: set `App_Options.menus` and the app loop creates the menu, pumps events and positions the IME.

## How it works

- **Menus.** `Menu_Item` is a flat entry: `menu` groups items into top-level menus, `action` is a unique nonzero id, `key` an uppercase letter (Cmd on macOS, Ctrl on Windows), `shift` adds Shift. `valid_menu` checks the model without a window server. A click or shortcut queues its `action`; `poll_native_menu` takes them one at a time. macOS uses NSMenu (one per application), Windows a menu bar plus an accelerator table (one per window), translated before `TranslateMessage` so a consumed shortcut is not also a key press.
- **Dialogs.** `native_file_dialog(window, kind, directory, name)` runs a modal `OPEN`, `SAVE` or `FOLDER` picker (NSOpenPanel/NSSavePanel; Unicode common dialogs and the shell folder browser on Windows). The `Dialog_Result.path` is owned: free it even when empty. `CANCELLED` means the user said no, so don't open a fallback prompt; `UNSUPPORTED` and `FAILED` mean you should.
- **Events.** `update_events` wraps `Input.update_window_events`. On macOS it filters AppKit's event retrieval for the length of the pump; on Windows it subclasses the window procedure. Programs that never import this module keep the plain `Input` behaviour.
- **Composition.** `set_text_input_rect(window, enabled, x, y, height)` turns IME input on and places the candidate window at a caret in top-left window points; turning it off cancels marked text. Each `update_events` fills `composition_events`, each tagged with the `Input.events_this_frame` index it comes before, so callers merge the two streams in order. Text is owned until the next pump; `cursor` is a UTF-8 byte offset (`-1` is the end). macOS swaps the content view's class for one that implements NSTextInputClient; Windows reads IMM32. Callbacks that fire between pumps wait in a queue for the next one.
- **Teardown.** `destroy_native_menu`, then `detach_text_input(window)`, on the window thread, before destroying the window. Detach restores the original view class or window procedure and frees queued text for that window.

## How to change it

- Platform code lives in `cocoa*.jai` and `windows*.jai`; `module.jai` holds the shared types and the Linux stubs, `events.jai` the pump and composition queue. A new platform needs `create_native_menu`, `destroy_native_menu`, `native_file_dialog` and the text hooks behind the same signatures.
- Keep it self-contained: this module may not add public API to `Input`, `Window_Creation` or other non-Extensions modules. It hooks in from outside instead.
- `Native_Menu`'s fields are platform handles; callers only pass the pointer around.
- Menu strings are borrowed while `create_native_menu` runs; nothing keeps them afterwards.

## Tests

```sh
(cd stdlib/Extensions/Desktop && jacket test -j 3)
```

`tests/` covers menu validation and installation, the composition queue and Windows IMM32 decoding. The menu and IME tests need a desktop session; real input methods and how native menus look can't be checked headlessly. `stdlib/Extensions/UI/tests/native-menus.jai` and `composition.jai` cover the UI routing with synthetic events.

## Dependencies

`Window_Type` and `Input`. macOS: the internal Cocoa window bindings and the Objective-C runtime (AppKit). Windows: `Windows`, `Windows_Utf8`, and user32, comdlg32, shell32, ole32 and imm32.
