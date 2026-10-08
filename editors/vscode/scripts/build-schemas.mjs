// Writes schemas/jailint.schema.json from jailint's rule table (crates/jailint/src/rules/mod.rs),
// so a new rule is completed and validated in jailint.toml without editing the schema by hand,
// schemas/jaifmt.schema.json from the keys Jai_Format's config parser accepts, and
// schemas/jai.schema.json for jai.toml (the settings jailsp's project.rs parses).
//
//   node scripts/build-schemas.mjs           # regenerate
//   node scripts/build-schemas.mjs --check   # fail if a committed schema is stale
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const repository = join(root, "..", "..");
const DOCS = "https://github.com/matteopolak/jai/blob/main/docs/tools";

export function parseRules(source) {
  const rules = [];
  const entry = /RuleInfo\s*\{\s*name:\s*"([a-z_]+)",\s*default:\s*Level::(\w+),\s*summary:\s*"((?:[^"\\]|\\.)*)"/g;
  for (const match of source.matchAll(entry)) {
    rules.push({ name: match[1], level: match[2].toLowerCase(), summary: match[3].replace(/\\(.)/g, "$1") });
  }
  return rules;
}

const rules = parseRules(readFileSync(join(repository, "crates/jailint/src/rules/mod.rs"), "utf8"));
if (rules.length < 10) throw new Error(`found only ${rules.length} rules in crates/jailint/src/rules/mod.rs`);

const level = (rule) => ({
  type: "string",
  enum: ["allow", "warn", "deny"],
  default: rule.level,
  markdownDescription: `${rule.summary} (default \`${rule.level}\`). [Docs](${DOCS}/jailint.md#${rule.name})`,
});

const jailint = {
  $schema: "http://json-schema.org/draft-07/schema#",
  $id: "https://github.com/matteopolak/jai/editors/vscode/schemas/jailint.schema.json",
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
      description: "Rule levels.",
      additionalProperties: false,
      properties: Object.fromEntries(rules.map((rule) => [rule.name, level(rule)])),
    },
  },
};

const count = (minimum, maximum, fallback, text) => ({
  type: "integer",
  minimum,
  maximum,
  ...(fallback === undefined ? {} : { default: fallback }),
  markdownDescription: text,
});
const jaifmt = {
  $schema: "http://json-schema.org/draft-07/schema#",
  $id: "https://github.com/matteopolak/jai/editors/vscode/schemas/jaifmt.schema.json",
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

const paths = (text) => ({ type: "array", items: { type: "string" }, markdownDescription: text });
const project = {
  $schema: "http://json-schema.org/draft-07/schema#",
  $id: "https://github.com/matteopolak/jai/editors/vscode/schemas/jai.schema.json",
  title: "jai.toml",
  description: `Project settings for jailsp: the files it builds and where its modules are. https://github.com/matteopolak/jai/blob/main/docs/compiler/language-server.md#project-settings-jaitoml`,
  type: "object",
  additionalProperties: false,
  properties: {
    build_files: paths(
      "The files you give `jaic build` (or `add_build_file`), relative to this file, such as `[\"first.jai\"]` or `[\"src/main.jai\"]`. Default: those of `build.jai`, `first.jai`, `main.jai` and `src/main.jai` that exist.",
    ),
    import_path: paths(
      "More folders to import modules from, relative to this file, as `-import_dir` adds (`Build_Options.import_path`). The `modules` folder next to the build file is always searched.",
    ),
  },
};

let stale = false;
for (const [name, schema] of [["jailint", jailint], ["jaifmt", jaifmt], ["jai", project]]) {
  const path = join(root, "schemas", `${name}.schema.json`);
  const text = JSON.stringify(schema, null, 2) + "\n";
  if (process.argv.includes("--check")) {
    let current = "";
    try {
      current = readFileSync(path, "utf8");
    } catch {}
    if (current !== text) {
      console.error(`schemas/${name}.schema.json is stale: run \`node scripts/build-schemas.mjs\``);
      stale = true;
    }
  } else {
    writeFileSync(path, text);
  }
}
if (stale) process.exit(1);
