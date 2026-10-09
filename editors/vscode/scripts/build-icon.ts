// Writes the extension's icons from one glyph: the playground's Jai file icon, a rounded tile
// with a `J` cut out (16x16 units, even-odd fill).
//
//   images/icon.svg, images/icon.png   Marketplace icon (256x256 PNG, transparent background)
//   images/jai-file-dark.svg           file icon for .jai files on dark themes
//   images/jai-file-light.svg          ... and on light themes
//
//   node scripts/build-icon.ts           # regenerate (PNG rendered with resvg)
//   node scripts/build-icon.ts --check   # fail if the committed SVGs are stale
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const GLYPH =
  "M4 1.5h8A2.5 2.5 0 0 1 14.5 4v8a2.5 2.5 0 0 1-2.5 2.5H4A2.5 2.5 0 0 1 1.5 12V4A2.5 2.5 0 0 1 4 1.5z " +
  "M8.9 4.25h1.85V9.4a2.75 2.75 0 0 1-5.5 0h1.85a.9.9 0 0 0 1.8 0z";

// The playground's red, oklch(63.259% 0.24086 31.631), and its icon colour,
// color-mix(in oklch, red 62%, white): white has no hue, so the mix keeps the red's.
const RED = { l: 0.63259, c: 0.24086, h: 31.631 };
const ICON = { l: 0.62 * RED.l + 0.38 * 1, c: 0.62 * RED.c, h: RED.h };

// Linear-light sRGB channel to an 8-bit gamma-encoded one.
function encode(x: number) {
  const v = Math.min(1, Math.max(0, x));
  return Math.round(255 * (v <= 0.0031308 ? 12.92 * v : 1.055 * v ** (1 / 2.4) - 0.055));
}

export function oklchToHex({ l, c, h }: { l: number; c: number; h: number }) {
  const a = c * Math.cos((h * Math.PI) / 180);
  const b = c * Math.sin((h * Math.PI) / 180);
  const lCube = (l + 0.3963377774 * a + 0.2158037573 * b) ** 3;
  const mCube = (l - 0.1055613458 * a - 0.0638541728 * b) ** 3;
  const sCube = (l - 0.0894841775 * a - 1.291485548 * b) ** 3;
  const linear = [
    4.0767416621 * lCube - 3.3077115913 * mCube + 0.2309699292 * sCube,
    -1.2684380046 * lCube + 2.6097574011 * mCube - 0.3413193965 * sCube,
    -0.0041960863 * lCube - 0.7034186147 * mCube + 1.707614701 * sCube,
  ];
  return "#" + linear.map((x) => encode(x).toString(16).padStart(2, "0")).join("");
}

const svg = (fill: string) =>
  `<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16">` +
  `<path fill="${fill}" fill-rule="evenodd" d="${GLYPH}"/></svg>\n`;

const files = {
  "images/icon.svg": svg(oklchToHex(ICON)),
  "images/jai-file-dark.svg": svg(oklchToHex(ICON)),
  // The light-theme file icon uses the full red: the pale icon colour fades on white.
  "images/jai-file-light.svg": svg(oklchToHex(RED)),
};

if (process.argv.includes("--check")) {
  const stale = Object.entries(files).filter(([path, text]) => {
    try {
      return readFileSync(join(root, path), "utf8") !== text;
    } catch {
      return true;
    }
  });
  if (stale.length) {
    console.error(`stale: ${stale.map(([p]) => p).join(", ")}; run \`node scripts/build-icon.ts\``);
    process.exit(1);
  }
} else {
  for (const [path, text] of Object.entries(files)) writeFileSync(join(root, path), text);
  const { Resvg } = await import("@resvg/resvg-js");
  const png = new Resvg(files["images/icon.svg"], { fitTo: { mode: "width", value: 256 }, background: "rgba(0,0,0,0)" })
    .render()
    .asPng();
  writeFileSync(join(root, "images/icon.png"), png);
  console.log(`icon colour ${oklchToHex(ICON)}; wrote ${Object.keys(files).join(", ")}, images/icon.png`);
}
