#!/usr/bin/env python3
"""WCAG contrast checks for both explicit themes, system dark, and category badges."""
import math
import re
import tomllib
import unittest

from toolkit.repo import ROOT

CSS = ROOT / "crates/jacket/src/doc/site/assets/site.css"
TEXT = "fg fg2 muted link accent type str kw ok err warn foreign".split()
SURFACES = "bg surface sunken sel okbg errbg warnbg".split()


OKLCH = r"oklch\(([\d.]+)% ([\d.]+) ([\d.]+)deg\)"


def rgb(value):
    """sRGB channels (0-255, unrounded) of `#rrggbb` or `oklch(L% C Hdeg)`, clamped into the gamut."""
    if value.startswith("#"):
        return tuple(int(value[i:i + 2], 16) for i in (1, 3, 5))
    lightness, chroma, hue = map(float, re.fullmatch(OKLCH, value).groups())
    lightness /= 100
    a, b = chroma * math.cos(math.radians(hue)), chroma * math.sin(math.radians(hue))
    lms = [(lightness + x * a + y * b) ** 3 for x, y in
           ((.3963377774, .2158037573), (-.1055613458, -.0638541728), (-.0894841775, -1.2914855480))]
    linear = [sum(m * v for m, v in zip(row, lms)) for row in
              ((4.0767416621, -3.3077115913, .2309699292),
               (-1.2684380046, 2.6097574011, -.3413193965),
               (-.0041960863, -.7034186147, 1.7076147010))]
    return tuple(255 * (v * 12.92 if v <= .0031308 else 1.055 * v ** (1 / 2.4) - .055)
                 for v in (min(max(v, 0), 1) for v in linear))


def luminance(color):
    channels = [v / 255 for v in color]
    linear = [v / 12.92 if v <= .04045 else ((v + .055) / 1.055) ** 2.4 for v in channels]
    return sum(v * w for v, w in zip(linear, (.2126, .7152, .0722)))


def contrast(a, b):
    a, b = sorted((luminance(a), luminance(b)))
    return (b + .05) / (a + .05)


def mix(a, b, fraction):
    return tuple(x * fraction + y * (1 - fraction) for x, y in zip(a, b))


def palettes():
    return [dict(re.findall(r"--([\w-]+):\s*(oklch\([\d.% deg]+\))", block))
            for block in re.findall(r":root[^{}]*\{([^{}]+)\}", CSS.read_text())[:3]]


def ratios(palette, dark):
    p = {k: rgb(v) for k, v in palette.items()}
    for token in TEXT:
        yield token, min(contrast(p[token], p[bg]) for bg in SURFACES), 4.5
    # Badge text must pass on the actual composited tint, not just the page.
    for token in ("muted", "accent", "warn", "foreign"):
        yield token + " badge", min(contrast(p[token], mix(p[token], p[bg], .17 if dark else .11))
                                    for bg in ("bg", "surface")), 4.5
    # The search field has no border; its placeholder and icon identify it on its fill.
    yield "muted search", min(contrast(p["muted"], p[bg]) for bg in ("sunken", "sel")), 4.5


class PaletteTests(unittest.TestCase):
    def test_tokens(self):
        light, dark, system = palettes()
        self.assertEqual(dark, system)
        for theme, palette in enumerate((light, dark)):
            for token, ratio, required in ratios(palette, theme):
                with self.subTest(theme=theme, token=token):
                    self.assertGreaterEqual(ratio, required, f"{ratio:.3f}:1 needs {required}:1")

    def test_translucent_header(self):
        glasses = re.findall(r"--glass: (oklch\([\d.% deg]+?) / (\d+)%\)", CSS.read_text())
        self.assertEqual(glasses[1], glasses[2])
        for dark in (0, 1):
            color, percent = glasses[dark]
            r, g, b = rgb(color.rstrip() + ")")
            alpha = float(percent) / 100
            # Worst underlying content: black in light mode, white in dark mode.
            background = mix((r, g, b), (255 if dark else 0,) * 3, alpha)
            for token in ("fg", "fg2", "muted", "link"):
                self.assertGreaterEqual(contrast(rgb(palettes()[dark][token]), background), 4.5)

    def test_categories(self):
        config = tomllib.loads((ROOT / "jacket.toml").read_text())
        for category in config["doc"]["category"]:
            for dark, key in enumerate(("color", "color_dark")):
                color = category.get(key, category.get("color"))
                if not color:
                    continue
                palette = palettes()[dark]
                backgrounds = [rgb(palette[bg]) for bg in ("bg", "surface")]
                backgrounds.append(mix(rgb(palette["fg"]), rgb(palette["surface"]), .025))
                for background in backgrounds:
                    self.assertGreaterEqual(contrast(rgb(color), background), 4.5)
                    self.assertGreaterEqual(contrast(rgb(color), mix(rgb(color), background, .17 if dark else .11)), 4.5)


if __name__ == "__main__":
    unittest.main()
