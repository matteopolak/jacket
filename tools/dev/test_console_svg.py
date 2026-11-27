"""Tests for the terminal-output SVG renderer."""

import unittest

from console_svg import PALETTE, render, spans, visible


class ConsoleSvgTests(unittest.TestCase):
    def test_spans_follow_sgr_escapes(self):
        line = "\x1b[1;32m    PASS\x1b[0m [0.1s] \x1b[2mname\x1b[22m done"
        self.assertEqual(
            spans(line),
            [
                ("    PASS", {"bold": True, "fg": PALETTE[32]}),
                (" [0.1s] ", {}),
                ("name", {"dim": True}),
                (" done", {}),
            ],
        )

    def test_visible_drops_escapes(self):
        self.assertEqual(visible("\x1b[31merror\x1b[0m: x"), "error: x")

    def test_render_keeps_spaces_and_escapes_markup(self):
        svg = render("title", "jaic -check a.jai", "  a < b && c", columns=20)
        self.assertIn("  a &lt; b &amp;&amp; c", svg)
        self.assertIn("jaic -check a.jai", svg)
        self.assertTrue(svg.startswith("<svg") and svg.rstrip().endswith("</svg>"))


if __name__ == "__main__":
    unittest.main()
