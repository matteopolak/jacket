# GetRect default font

GetRect draws text with Noto Sans when neither the widget's theme nor `ui_init` names a font, so a UI works without loading any font first.

## How it works

`label` (every widget's text goes through it) picks the first font it finds:

1. the theme's `font`;
2. `default_overall_theme.font`;
3. the default font, at a size that follows the rectangle.

The default font's bytes are the `default_font` given to `ui_init`, or `stdlib/internal/fonts/NotoSans-Regular.ttf`, which GetRect reads at compile time and embeds in the program. Its pixel height is `0.6 * rect height`, rounded, so a 60-pixel button gets 36-pixel text. That follows the official GetRect, whose text also scales with the button. Each size is loaded once through `Draw_Procs.get_font_at_size` and kept until the next `ui_init`.

```jai
ui_init();                                   // text in Noto Sans
ui_init(#run read_entire_file("my.ttf"));   // text in my.ttf, sized the same way
theme.font = my_font;                        // this theme always uses my_font
```

With custom `Draw_Procs` and no `get_font_at_size`, there is no default font and `label` passes a null font to `prepare_text`, as before.

## How to change it

- Size: `DEFAULT_FONT_HEIGHT_FRACTION` in `stdlib/internal/numerics/ui_text.jai`.
- Font: replace the file in `stdlib/internal/fonts/` with its licence, and the path in `read_bundled_font`.
- Tests: `stdlib/GetRect/tests/default-font.jai`.

The cache grows by one font per distinct rectangle height, so a UI that animates heights loads many sizes.

## Configuration

None. `ui_init`'s `default_font` replaces the bundled font.

## Dependencies

- `Draw_Procs.get_font_at_size`: Simp's by default.
- Noto Sans, under the SIL Open Font License 1.1 (`stdlib/internal/fonts/OFL.txt`). It adds about 600 KB to a program that imports GetRect.
