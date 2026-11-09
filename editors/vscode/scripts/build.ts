// The extension's generated files and packaging steps, one subcommand each:
//
//   node scripts/build.ts icon [--check]      images/icon.png, rendered with resvg from the hand-drawn
//                                             images/icon.svg
//   node scripts/build.ts doc-image [--check]  1200×630 documentation social image from docs/images/logo.svg
//   node scripts/build.ts schemas [--check]   schemas/jacket.schema.json from jailint's
//                                             rules (crates/jailint/src/rules) and the keys the
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

// images/icon.svg and the two file icons are drawn by hand; the PNG the marketplace shows is
// rendered from icon.svg.
async function iconPng() {
  const { Resvg } = await import("@resvg/resvg-js");
  return new Resvg(renderable(readFileSync(join(root, "images/icon.svg"), "utf8")), { fitTo: { mode: "width", value: 256 }, background: "rgba(0,0,0,0)" }).render().asPng();
}

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

// resvg does not read `oklch()` colours, so they are written as hex first.
function renderable(svg: string) {
  return svg.replace(/oklch\(([\d.]+)% ([\d.]+) ([\d.]+)\)/g, (_, l, c, h) =>
    oklchToHex({ l: Number(l) / 100, c: Number(c), h: Number(h) }),
  );
}

async function icon(check: boolean) {
  const png = await iconPng();
  const file = join(root, "images/icon.png");
  if (check) {
    if (!readFileSync(file).equals(png)) {
      console.error("stale: images/icon.png; run `node scripts/build.ts icon`");
      process.exitCode = 1;
    }
  } else {
    writeFileSync(file, png);
    console.log("wrote images/icon.png");
  }
}

async function docImage(check: boolean) {
  const { Resvg } = await import("@resvg/resvg-js");
  const logo = renderable(readFileSync(join(repository, "docs/images/logo.svg"), "utf8"));
  const centered = logo.replace(/<svg ([^>]*?)width="[^"]*" height="[^"]*"/, '<svg x="440" y="155" $1width="320" height="320"');
  const page = `<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="630"><rect width="1200" height="630" fill="#1b1b1f"/>${centered}</svg>`;
  const png = new Resvg(page).render().asPng();
  const file = join(root, "images/doc-og.png");
  if (check) {
    if (!readFileSync(file).equals(png)) { console.error("images/doc-og.png is stale"); process.exitCode = 1; }
  } else writeFileSync(file, png);
}

// ---- schemas ---------------------------------------------------------------------------------

const DOCS = "https://github.com/matteopolak/jacket/blob/main/docs/tools";

// The rules, in the order of the `rules!` list in rules/mod.rs; each rule's module exports its own
// `INFO` (name, category, summary).
export function parseRules(registry: string, moduleSource: (name: string) => string) {
  const list = /rules!\s*\{([^}]*)\}/.exec(registry);
  if (!list) throw new Error("no rules! list in crates/jailint/src/rules/mod.rs");
  const names = list[1]!
    .split(",")
    .map((entry) => entry.replace(/^\s*pub\([a-z]+\)\s*/, "").trim())
    .filter(Boolean);
  const rules: { name: string; level: string; summary: string }[] = [];
  const info = /INFO:\s*super::RuleInfo\s*=\s*super::RuleInfo\s*\{\s*name:\s*"([a-z_]+)",\s*category:\s*crate::config::Category::(\w+),\s*summary:\s*"((?:[^"\\]|\\.)*)"/;
  for (const name of names) {
    const match = info.exec(moduleSource(name));
    if (!match) throw new Error(`rule module ${name} has no INFO`);
    if (match[1] !== name) throw new Error(`rule module ${name} is named ${match[1]}`);
    const category = match[2]!.toLowerCase();
    const level = CATEGORIES[category];
    if (level === undefined) throw new Error(`rule ${name} has unknown category ${category}`);
    rules.push({ name, level: level[0], summary: match[3]!.replace(/\\(.)/g, "$1") });
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

const rulesDir = join(repository, "crates/jailint/src/rules");
const rules = parseRules(readFileSync(join(rulesDir, "mod.rs"), "utf8"), (name) =>
  readFileSync(join(rulesDir, `${name}.rs`), "utf8"),
);
if (rules.length < 10) throw new Error(`found only ${rules.length} rules in crates/jailint/src/rules/mod.rs`);

const level = (rule: { name: string; level: string; summary: string }) => ({
  type: "string",
  enum: ["allow", "warn", "deny"],
  default: rule.level,
  markdownDescription: `${rule.summary} (default \`${rule.level}\`). [Docs](${DOCS}/jailint.md#${rule.name})`,
});

// The `[lint]` keys that come from the rule table.
const lintRules = {
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
const flag = (text: string) => ({ type: "boolean", default: false, markdownDescription: text });
// The settings that shape code; `[doc.format]` can override each of them for doc comment examples.
const layoutKeys = {
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
  blank_lines_between_items: { type: "boolean", default: true, markdownDescription: "A blank line around multi-line file-level items and import groups." },
  blank_line_after_declarations: flag("In code, a blank line after a run of declarations."),
  blank_lines_around_blocks: flag("In code, a blank line before and after a statement with a block body."),
  blank_line_between_cases: flag("In a switch, a blank line before a `case` that follows a multi-line arm."),
  blank_line_before_comments: flag("In code, a blank line above a comment that follows code."),
  blank_line_after_guards: flag("In code, a blank line after a run of one-line `if`, `for` or `while`."),
  blank_line_before_return: flag("In code, a blank line above a `return` in a longer body."),
};
const formatKeys = {
  properties: {
    ...layoutKeys,
    format_doc_examples: {
      type: "boolean",
      default: true,
      markdownDescription: "Format the `jai` code blocks inside `//` comments, such as doc comment examples. They follow these settings, except those `[doc.format]` sets.",
    },
    ignore: {
      type: "array",
      items: { type: "string" },
      markdownDescription: "Globs (relative to this file) of files and directories not to format.",
    },
  },
};

const testRunKeys = {
  properties: {
    jobs: { type: "integer", minimum: 1, markdownDescription: "Tests run at once (`-j`). Default: the machine's threads." },
    timeout: { type: "integer", minimum: 0, default: 120, markdownDescription: "Seconds before a test without `@timeout` is killed (`--timeout`); 0 for no limit." },
    slow: { type: "integer", minimum: 0, default: 30, markdownDescription: "Seconds before a running test is reported as slow; 0 for never." },
    retries: { type: "integer", minimum: 0, default: 0, markdownDescription: "Runs of a failed test before it counts as failed (`--retries`); one that then passes is FLAKY." },
    fail_fast: { type: "boolean", default: false, markdownDescription: "Start no more tests after the first failure (`--fail-fast`)." },
    flaky_fails: { type: "boolean", default: false, markdownDescription: "A FLAKY test fails the run." },
    runner: { type: "string", markdownDescription: "Command that runs test programs built for another target, such as `node tools/wasi_run.mjs` (`--runner`)." },
    needs: { type: "array", items: { type: "string" }, markdownDescription: "Capabilities this machine has, for tests with `@needs(...)` (`--needs`), such as `[\"network\"]`." },
  },
};
const paths = (text: string) => ({ type: "array", items: { type: "string" }, markdownDescription: text });
const section = (schema: { properties: object }, text: string) => ({
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
        ...lintRules.properties,
        max_nesting: { type: "integer", minimum: 1, markdownDescription: "How many blocks deep code may nest before `excessive_nesting` fires." },
        max_complexity: { type: "integer", minimum: 0, default: 25, markdownDescription: "The cognitive complexity above which `cognitive_complexity` reports a procedure." },
        max_arguments: { type: "integer", minimum: 0, default: 7, markdownDescription: "The parameter count above which `too_many_arguments` reports a procedure." },
        max_bools: { type: "integer", minimum: 0, default: 3, markdownDescription: "The number of `bool` parameters above which `excessive_bools` reports a procedure." },
        max_stack_array_bytes: { type: "integer", minimum: 0, default: 65536, markdownDescription: "The size in bytes above which `large_stack_array` reports a local fixed array." },
        magic_number_allow: { type: "array", items: { type: "number" }, markdownDescription: "Numbers `magic_number` never reports, besides `10`, `100`, `1000`, powers of two and all-ones masks, such as `[60, 24, 3.14159]`." },
        magic_number_min: { type: "number", minimum: 0, default: 3, markdownDescription: "Whole numbers smaller than this in magnitude are not reported by `magic_number`." },
        magic_number_in_tests: { type: "boolean", default: false, markdownDescription: "`magic_number` also checks files in `tests` and `test` folders." },
        overrides: {
          type: "array",
          markdownDescription: "Rule levels for some paths. Each entry's `rules` win over `[lint.rules]` for the files its `paths` match; a later entry wins over an earlier one, and command-line flags win over both.",
          items: {
            type: "object",
            additionalProperties: false,
            required: ["paths"],
            properties: {
              paths: { type: "array", items: { type: "string" }, minItems: 1, markdownDescription: "Globs (relative to this file) of the files these levels apply to: `*`, `**`, `?`." },
              rules: { $ref: "#/properties/lint/properties/rules" },
            },
          },
        },
        portability: {
          type: "object",
          additionalProperties: false,
          markdownDescription: "The code `platform_leak` holds to every operating system: it is compiled once per target and must not import a file one of them does not build.",
          properties: {
            targets: { type: "array", items: { enum: ["linux", "macos", "windows"] }, uniqueItems: true, markdownDescription: "The operating systems portable code is built for. Default: all three." },
            portable: { type: "array", items: { type: "string" }, markdownDescription: "Globs (relative to this file) of the code that must build on every target, such as `stdlib/**`." },
            platforms: {
              type: "object",
              additionalProperties: { type: "array", items: { enum: ["linux", "macos", "windows"] }, uniqueItems: true },
              markdownDescription: "Globs of files that build only on the operating systems listed (`[]`: on none of them). The most specific matching glob decides. A `// jailint: platforms(linux, macos)` comment at the top of a file does the same for that file.",
            },
          },
        },
      },
    },
    test: {
      type: "object",
      additionalProperties: false,
      markdownDescription: "Run-wide settings for `jacket test`; a test's own options are notes on it.",
      properties: {
        paths: paths("Test roots, relative to this file: folders with a `tests/` directory. Inside one of them, `jacket test` runs only that one. Default: this folder."),
        ...testRunKeys.properties,
        profile: {
          type: "object",
          markdownDescription: "Named sets of settings over `[test]`, chosen with `jacket test --profile NAME`.",
          additionalProperties: { type: "object", additionalProperties: false, properties: testRunKeys.properties },
        },
      },
    },
    format: section(formatKeys, "Settings for the jaifmt formatter."),
    doc: {
      type: "object",
      // Only `format` is described here; the other keys configure the `jacket doc` site.
      markdownDescription: "The site `jacket doc` writes.",
      properties: {
        format: section(
          { properties: Object.fromEntries(Object.entries(layoutKeys).map(([name, key]) => [name, { ...key, default: undefined }])) },
          "`[format]` settings that differ for the examples in doc comments. A key left out keeps the `[format]` value.",
        ),
      },
    },
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
  for (const [name, schema] of [["jacket", jacket]] as const) {
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
else if (command === "doc-image") await docImage(check);
else if (command === "schemas") schemas(check);
else if (command === "license") license();
else {
  console.error("usage: node scripts/build.ts icon|doc-image|schemas [--check] | license");
  process.exit(2);
}
