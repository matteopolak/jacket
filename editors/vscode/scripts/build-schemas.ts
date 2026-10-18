// Writes schemas/jailint.schema.json from jailint's rule table (crates/jailint/src/rules/mod.rs),
// so a new rule is completed and validated in jailint.toml without editing the schema by hand,
// schemas/jaifmt.schema.json from the keys Jai_Format's config parser accepts, and
// schemas/jai.schema.json for the legacy jai.toml (the settings jailsp's project.rs parses), and
// schemas/jacket.schema.json for jacket.toml, which holds all of them in sections.
//
//   node scripts/build-schemas.ts           # regenerate
//   node scripts/build-schemas.ts --check   # fail if a committed schema is stale
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const repository = join(root, "..", "..");
const DOCS = "https://github.com/matteopolak/jacket/blob/main/docs/tools";

export function parseRules(source: string) {
  const rules: { name: string; level: string; summary: string }[] = [];
  const entry = /RuleInfo\s*\{\s*name:\s*"([a-z_]+)",\s*default:\s*Level::(\w+),\s*summary:\s*"((?:[^"\\]|\\.)*)"/g;
  for (const match of source.matchAll(entry)) {
    rules.push({ name: match[1]!, level: match[2]!.toLowerCase(), summary: match[3]!.replace(/\\(.)/g, "$1") });
  }
  return rules;
}

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
      description: "Rule levels.",
      additionalProperties: false,
      properties: Object.fromEntries(rules.map((rule) => [rule.name, level(rule)])),
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

let stale = false;
for (const [name, schema] of [["jailint", jailint], ["jaifmt", jaifmt], ["jai", project], ["jacket", jacket]]) {
  const path = join(root, "schemas", `${name}.schema.json`);
  const text = JSON.stringify(schema, null, 2) + "\n";
  if (process.argv.includes("--check")) {
    let current = "";
    try {
      current = readFileSync(path, "utf8");
    } catch {}
    if (current !== text) {
      console.error(`schemas/${name}.schema.json is stale: run \`node scripts/build-schemas.ts\``);
      stale = true;
    }
  } else {
    writeFileSync(path, text);
  }
}
if (stale) process.exit(1);
