// The extension's generated files and packaging steps, one subcommand each:
//
//   node scripts/build.ts icon [--check]      images/icon.{svg,png} and the two file icons, from one glyph:
//                                             the playground's Jai file icon, a rounded tile with a `J` cut
//                                             out (16x16 units, even-odd fill); PNG rendered with resvg
//   node scripts/build.ts schemas [--check]   schemas/{jailint,jaifmt,jai,jacket}.schema.json from jailint's
//                                             rule table (crates/jailint/src/rules/mod.rs) and the keys the
//                                             config parsers accept, so a new rule or key is completed and
//                                             validated in the TOML files without editing a schema by hand
//   node scripts/build.ts license             copies the repository's LICENSE next to package.json (vsce wants
//                                             one there; the extension is under the repository's license)
//
// `--check` fails when a committed output is stale. The grammar has its own script (build-grammar.ts).
import { copyFileSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const repository = join(root, "..", "..");

// ---- icon ------------------------------------------------------------------------------------

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

const iconFiles = {
  "images/icon.svg": svg(oklchToHex(ICON)),
  "images/jai-file-dark.svg": svg(oklchToHex(ICON)),
  // The light-theme file icon uses the full red: the pale icon colour fades on white.
  "images/jai-file-light.svg": svg(oklchToHex(RED)),
};

async function icon(check: boolean) {
  if (check) {
    const stale = Object.entries(iconFiles).filter(([path, text]) => {
      try {
        return readFileSync(join(root, path), "utf8") !== text;
      } catch {
        return true;
      }
    });
    if (stale.length) {
      console.error(`stale: ${stale.map(([p]) => p).join(", ")}; run \`node scripts/build.ts icon\``);
      process.exitCode = 1;
    }
  } else {
    for (const [path, text] of Object.entries(iconFiles)) writeFileSync(join(root, path), text);
    const { Resvg } = await import("@resvg/resvg-js");
    const png = new Resvg(iconFiles["images/icon.svg"], { fitTo: { mode: "width", value: 256 }, background: "rgba(0,0,0,0)" })
      .render()
      .asPng();
    writeFileSync(join(root, "images/icon.png"), png);
    console.log(`icon colour ${oklchToHex(ICON)}; wrote ${Object.keys(iconFiles).join(", ")}, images/icon.png`);
  }
}

// ---- schemas ---------------------------------------------------------------------------------

const DOCS = "https://github.com/matteopolak/jacket/blob/main/docs/tools";

export function parseRules(source: string) {
  const rules: { name: string; level: string; summary: string }[] = [];
  const entry = /RuleInfo\s*\{\s*name:\s*"([a-z_]+)",\s*category:\s*Category::(\w+),\s*summary:\s*"((?:[^"\\]|\\.)*)"/g;
  for (const match of source.matchAll(entry)) {
    const category = match[2]!.toLowerCase();
    const level = CATEGORIES[category];
    if (level === undefined) throw new Error(`rule ${match[1]} has unknown category ${category}`);
    rules.push({ name: match[1]!, level: level[0], summary: match[3]!.replace(/\\(.)/g, "$1") });
  }
  return rules;
}

// Each category's default level and meaning, as in crates/jailint/src/config.rs.
const CATEGORIES: Record<string, [string, string]> = {
  correctness: ["deny", "code that is wrong: it does not do what it says"],
  suspicious: ["warn", "code that is most likely wrong, or does nothing"],
  complexity: ["warn", "code that says something in a roundabout way"],
  style: ["warn", "code that works but is not written the usual way"],
  perf: ["warn", "code that could be faster"],
  pedantic: ["allow", "stricter checks with more false positives"],
  restriction: ["allow", "checks that forbid things a project may choose to avoid"],
};

const rules = parseRules(readFileSync(join(repository, "crates/jailint/src/rules/mod.rs"), "utf8"));
if (rules.length < 10) throw new Error(`found only ${rules.length} rules in crates/jailint/src/rules/mod.rs`);

const level = (rule: { name: string; level: string; summary: string }) => ({
  type: "string",
  enum: ["allow", "warn", "deny"],
  default: rule.level,
  markdownDescription: `${rule.summary} (default \`${rule.level}\`). [Docs](${DOCS}/jailint.md#${rule.name})`,
});

const jailint = {
  $schema: "http://json-schema.org/draft-07/schema#",
  $id: "https://github.com/matteopolak/jacket/editors/vscode/schemas/jailint.schema.json",
  title: "jailint.toml",
  description: "Settings for jailint and the lints jailsp reports.",
  type: "object",
  additionalProperties: false,
  properties: {
    exclude: {
      type: "array",
      items: { type: "string" },
      markdownDescription: "Globs (relative to this file) of files and directories not to lint: `*`, `**`, `?`.",
    },
    rules: {
      type: "object",
      description: "Levels for rules and categories.",
      additionalProperties: false,
      properties: Object.fromEntries([
        ...Object.entries(CATEGORIES).map(([name, [categoryLevel, summary]]) => [
          name,
          {
            type: "string",
            enum: ["allow", "warn", "deny"],
            default: categoryLevel,
            markdownDescription: `Every ${name} rule: ${summary} (default \`${categoryLevel}\`). A rule's own entry wins. [Docs](${DOCS}/jailint.md#rules)`,
          },
        ]),
        ...rules.map((rule) => [rule.name, level(rule)]),
      ]),
    },
  },
};

const count = (minimum: number, maximum: number, fallback: number | undefined, text: string) => ({
  type: "integer",
  minimum,
  maximum,
  ...(fallback === undefined ? {} : { default: fallback }),
  markdownDescription: text,
});
const jaifmt = {
  $schema: "http://json-schema.org/draft-07/schema#",
  $id: "https://github.com/matteopolak/jacket/editors/vscode/schemas/jaifmt.schema.json",
  title: "jaifmt.toml",
  description: `Settings for the jaifmt formatter. ${DOCS}/jaifmt.md#configuration`,
  type: "object",
  additionalProperties: false,
  properties: {
    indent_width: count(1, 16, 4, "Spaces per indentation level."),
    case_indent: count(0, 16, undefined, "Indentation of `case` lines in `if x == {`. Default: `indent_width`."),
    case_body_indent: count(0, 16, undefined, "Indentation of the statements under a `case`. Default: `indent_width`."),
    max_blank_lines: count(0, 100, 2, "Consecutive blank lines kept."),
    max_width: count(10, 10000, 100, "Line width reported by `jaifmt --verbose`; lines are never wrapped."),
    brace_style: {
      type: "string",
      enum: ["same_line", "preserve"],
      default: "same_line",
      markdownDescription: "`same_line` joins a `{` on its own line to its header; `preserve` leaves it.",
    },
    ignore: {
      type: "array",
      items: { type: "string" },
      markdownDescription: "Globs (relative to this file) of files and directories not to format.",
    },
  },
};

const paths = (text: string) => ({ type: "array", items: { type: "string" }, markdownDescription: text });
const project = {
  $schema: "http://json-schema.org/draft-07/schema#",
  $id: "https://github.com/matteopolak/jacket/editors/vscode/schemas/jai.schema.json",
  title: "jai.toml",
  description: `Project settings for jailsp: the files it builds and where its modules are. https://github.com/matteopolak/jacket/blob/main/docs/compiler/language-server.md#project-settings-jaitoml`,
  type: "object",
  additionalProperties: false,
  properties: {
    build_files: paths(
      "The files you give `jaic` (or `add_build_file`), relative to this file, such as `[\"first.jai\"]` or `[\"src/main.jai\"]`. Default: those of `build.jai`, `first.jai`, `main.jai` and `src/main.jai` that exist.",
    ),
    import_path: paths(
      "More folders to import modules from, relative to this file, as `-import_dir` adds (`Build_Options.import_path`). The `modules` folder next to the build file is always searched.",
    ),
  },
};

const section = (schema: { description: string; properties: object }, text: string) => ({
  type: "object",
  additionalProperties: false,
  markdownDescription: text,
  properties: schema.properties,
});
const jacket = {
  $schema: "http://json-schema.org/draft-07/schema#",
  $id: "https://github.com/matteopolak/jacket/editors/vscode/schemas/jacket.schema.json",
  title: "jacket.toml",
  description: `Project settings for jacket, jaic, jailsp, jailint and jaifmt. ${DOCS}/jacket.md#jackettoml`,
  type: "object",
  additionalProperties: false,
  properties: {
    project: {
      type: "object",
      additionalProperties: false,
      markdownDescription: "What to build and where its modules are.",
      properties: {
        entry: paths("The files you give `jaic`, relative to this file, such as `[\"main.jai\"]`. Default: the first of `build.jai`, `first.jai`, `main.jai`, `src/main.jai` and `module.jai` that exists."),
        import_path: paths("More folders to import modules from, relative to this file (`-import_dir`)."),
      },
    },
    toolchain: {
      type: "object",
      additionalProperties: false,
      markdownDescription: "The toolchain the project is written for.",
      properties: {
        version: { type: "string", markdownDescription: "The `x.y.z` jaic release the project is written for; tools warn when theirs differs." },
      },
    },
    lint: {
      type: "object",
      additionalProperties: false,
      markdownDescription: "Settings for jailint and the lints jailsp reports.",
      properties: {
        ...jailint.properties,
        max_nesting: { type: "integer", minimum: 1, markdownDescription: "How many blocks deep code may nest before `excessive_nesting` fires." },
        max_complexity: { type: "integer", minimum: 0, default: 25, markdownDescription: "The cognitive complexity above which `cognitive_complexity` reports a procedure." },
        magic_number_allow: { type: "array", items: { type: "number" }, markdownDescription: "Numbers `magic_number` never reports, besides `10`, `100`, `1000`, powers of two and all-ones masks, such as `[60, 24, 3.14159]`." },
        magic_number_min: { type: "number", minimum: 0, default: 3, markdownDescription: "Whole numbers smaller than this in magnitude are not reported by `magic_number`." },
        magic_number_in_tests: { type: "boolean", default: false, markdownDescription: "`magic_number` also checks files in `tests` and `test` folders." },
      },
    },
    format: section(jaifmt, "Settings for the jaifmt formatter."),
    experimental: {
      type: "object",
      additionalProperties: false,
      markdownDescription: "Performance knobs that may change between releases; each becomes a `jaic` flag.",
      properties: {
        parse_threads: { type: "integer", minimum: 1, markdownDescription: "Threads that parse files ahead of the compiler (`-parse_threads`)." },
        serial_parse: { type: "boolean", markdownDescription: "Parse files one at a time (`-serial_parse`)." },
        codegen_units: { type: "integer", minimum: 1, markdownDescription: "Split code generation into this many units (`-codegen_units`)." },
      },
    },
  },
};

function schemas(check: boolean) {
  let stale = false;
  for (const [name, schema] of [["jailint", jailint], ["jaifmt", jaifmt], ["jai", project], ["jacket", jacket]]) {
    const path = join(root, "schemas", `${name}.schema.json`);
    const text = JSON.stringify(schema, null, 2) + "\n";
    if (check) {
      let current = "";
      try {
        current = readFileSync(path, "utf8");
      } catch {}
      if (current !== text) {
        console.error(`schemas/${name}.schema.json is stale: run \`node scripts/build.ts schemas\``);
        stale = true;
      }
    } else {
      writeFileSync(path, text);
    }
  }
  if (stale) process.exitCode = 1;
}

// ---- license ---------------------------------------------------------------------------------

function license() {
  copyFileSync(join(repository, "LICENSE"), join(root, "LICENSE"));
}

const [command, ...flags] = process.argv.slice(2);
const check = flags.includes("--check");
if (command === "icon") await icon(check);
else if (command === "schemas") schemas(check);
else if (command === "license") license();
else {
  console.error("usage: node scripts/build.ts icon|schemas [--check] | license");
  process.exit(2);
}
