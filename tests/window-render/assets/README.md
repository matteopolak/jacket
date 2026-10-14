# Original native feature fonts

`feature-font.ttf` and `filtered-feature-font.ttf` are generated deterministically by
`python3 tools/window_render_feature_font.py`; `--check` verifies regeneration.
All outlines, advances, names and substitutions were independently authored for
these tests. No reference/library/font asset is copied. The generator is the sole
design source and the binary is committed as native test data, not executable code.

The font contains simple original bar outlines and real TrueType metric/cmap tables.
GSUB defines smallcaps and lining-figure substitutions through single formats1/2,
extension and alternate lookups, with coverage formats1/2. Digit substitutions
deliberately distinguish the reviewed rule that lining figures take precedence over
smallcaps for digits. Tests use production FreeType loading, metrics and rasterization
and real GL atlas/text drawing; no font metrics, glyph objects or GPU resources are
substituted by fixtures. This is test artwork, not a general-purpose UI font.

The second font adds authored native GDEF classes and the IgnoreBaseGlyphs,
IgnoreLigatures and IgnoreMarks lookup flags. It verifies that those flags do not
skip the current glyph of single/alternate substitution: they filter other inputs
when matching a sequence. Production does not claim general sequence shaping.

Binary format sources are Microsoft's public native OpenType documentation:
[head](https://learn.microsoft.com/en-us/typography/opentype/spec/head),
[glyf](https://learn.microsoft.com/en-us/typography/opentype/spec/glyf),
[hhea](https://learn.microsoft.com/en-us/typography/opentype/spec/hhea), and
[GSUB](https://learn.microsoft.com/en-us/typography/opentype/spec/gsub),
[GDEF](https://learn.microsoft.com/en-us/typography/opentype/spec/gdef), and
[lookup processing](https://learn.microsoft.com/en-us/typography/opentype/spec/chapter2).
