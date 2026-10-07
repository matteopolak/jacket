// Writes syntaxes/jai.tmLanguage.json. The grammar is written here, in JavaScript, so the
// identifier and keyword patterns are spelled once and shared by the rules that need them.
//
//   node scripts/build-grammar.mjs           # regenerate
//   node scripts/build-grammar.mjs --check   # fail if the committed JSON is stale
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const output = join(root, "syntaxes", "jai.tmLanguage.json");

// Identifiers: ASCII letters, digits and `_`, plus any non-ASCII character (as jaic's lexer).
const ID_START = "A-Za-z_\\x{80}-\\x{10FFFF}";
const ID_CHAR = "A-Za-z0-9_\\x{80}-\\x{10FFFF}";
const ID = `[${ID_START}][${ID_CHAR}]*`;
// Not inside a longer identifier: `\b` does not know about non-ASCII letters or `$`.
const B = `(?<![${ID_CHAR}$])`;
const E = `(?![${ID_CHAR}])`;
const words = (list) => `${B}(?:${list.join("|")})${E}`;
// A list of names before a declaration operator: `a, b :=`.
const namesThen = (op) => `(?=(?:\\s*,\\s*${ID})*\\s*${op})`;

const CONTROL = ["if", "ifx", "then", "else", "case", "for", "while", "break", "continue", "return", "remove", "defer", "push_context"];
const BUILTIN_TYPES = [
  "bool", "int", "float", "float32", "float64", "string", "void",
  "s8", "s16", "s32", "s64", "s128", "u8", "u16", "u32", "u64", "u128",
  "Type", "Any", "Code",
];
const BUILTIN_FUNCTIONS = ["size_of", "type_of", "type_info", "initializer_of", "is_constant", "align_of", "offset_of"];
const CAST_FLAGS = ["no_check", "trunc", "truncate", "force"];

// Directives by role; anything else spelled `#name` is still a directive.
const IMPORT_DIRECTIVES = ["import", "load", "foreign_library", "foreign_system_library", "system_library", "library"];
const CONDITIONAL_DIRECTIVES = ["if", "ifx", "else", "complete", "through", "ifdef", "endif"];
const MODIFIER_DIRECTIVES = [
  "c_call", "foreign", "expand", "compiler", "intrinsic", "no_context", "must", "deprecated",
  "symmetric", "no_debug", "cpp_method", "cpp_return_type_is_non_pod", "elsewhere", "runtime_support",
  "no_call", "no_padding", "no_reset", "align", "place", "overlay", "specified", "type_info_none",
  "type_info_procedures_are_void_pointers", "type_info_no_size_complaint", "program_export",
  "scope_file", "scope_module", "scope_export", "add_context", "module_parameters", "no_abc",
  "no_aoc", "poke_name", "discard", "distinct", "isa", "modify", "as",
];
const VALUE_DIRECTIVES = [
  "file", "line", "filepath", "this", "caller_location", "caller_code", "procedure_name",
  "location", "compile_time", "exists", "bytes", "procedure_of_call", "placeholder",
];

// Here-strings whose terminator names a language: `#string WGSL` ... `WGSL` highlights the body
// as WGSL. The terminator is compared with `tags` ignoring case; any other terminator (END,
// DONE, ...) stays a plain string. `scopes` are the grammars included for the body, the first
// that exists winning where they overlap: VS Code's built-in ones, or for languages it does not
// ship, the usual scope of a grammar extension plus, where we have one, our own bundled grammar.
// `language` is the VS Code language ID the body gets (comments, brackets, snippets), written to
// package.json's `embeddedLanguages`; an ID no installed extension registers is ignored.
//
// Keep the tags in step with LANGUAGE_TAGS in crates/jai-language-server/src/here_string.rs
// (jailsp leaves these here-strings to the grammar; this script fails when the lists differ)
// and, where the languages overlap, with the playground's table (EMBEDDED_LANGUAGES in the
// portfolio's src/lib/jai/embedded-languages.ts).
const EMBEDDED_LANGUAGES = [
  { tags: ["WGSL"], name: "wgsl", scopes: ["source.wgsl", "source.wgsl.jai-embedded"], language: "wgsl" },
  { tags: ["GLSL", "VERT", "FRAG", "COMP", "GEOM", "TESC", "TESE"], name: "glsl", scopes: ["source.glsl"], language: "glsl" },
  { tags: ["HLSL"], name: "hlsl", scopes: ["source.hlsl"], language: "hlsl" },
  { tags: ["MSL", "METAL"], name: "metal", scopes: ["source.metal"], language: "metal" },
  { tags: ["SQL"], name: "sql", scopes: ["source.sql"], language: "sql" },
  { tags: ["JSON"], name: "json", scopes: ["source.json"], language: "json" },
  { tags: ["HTML"], name: "html", scopes: ["text.html.basic"], language: "html" },
  { tags: ["CSS"], name: "css", scopes: ["source.css"], language: "css" },
  { tags: ["JS", "JAVASCRIPT"], name: "javascript", scopes: ["source.js"], language: "javascript" },
  { tags: ["TS", "TYPESCRIPT"], name: "typescript", scopes: ["source.ts"], language: "typescript" },
  { tags: ["PY", "PYTHON"], name: "python", scopes: ["source.python"], language: "python" },
  { tags: ["SH", "BASH", "SHELL"], name: "shellscript", scopes: ["source.shell"], language: "shellscript" },
  { tags: ["C"], name: "c", scopes: ["source.c"], language: "c" },
  { tags: ["CPP"], name: "cpp", scopes: ["source.cpp"], language: "cpp" },
  { tags: ["OBJC"], name: "objc", scopes: ["source.objc"], language: "objective-c" },
  { tags: ["RS", "RUST"], name: "rust", scopes: ["source.rust"], language: "rust" },
  { tags: ["XML"], name: "xml", scopes: ["text.xml"], language: "xml" },
  { tags: ["YAML", "YML"], name: "yaml", scopes: ["source.yaml"], language: "yaml" },
  { tags: ["TOML"], name: "toml", scopes: ["source.toml", "source.toml.jai-settings"], language: "toml" },
  { tags: ["MD", "MARKDOWN"], name: "markdown", scopes: ["text.html.markdown"], language: "markdown" },
  { tags: ["LUA"], name: "lua", scopes: ["source.lua"], language: "lua" },
  { tags: ["JAI"], name: "jai", scopes: ["source.jai"], language: "jai" },
];

// jailsp's copy of the tags must match, or it would paint a tagged body as one string again.
{
  const rust = readFileSync(join(root, "..", "..", "crates", "jai-language-server", "src", "here_string.rs"), "utf8");
  const list = /LANGUAGE_TAGS: &\[&str\] = &\[([^\]]*)\]/.exec(rust);
  const theirs = [...(list?.[1] ?? "").matchAll(/"([^"]+)"/g)].map((m) => m[1]);
  const ours = EMBEDDED_LANGUAGES.flatMap((l) => l.tags);
  if (theirs.join() !== ours.join()) {
    console.error(`LANGUAGE_TAGS in here_string.rs differs from EMBEDDED_LANGUAGES:\n  rust: ${theirs.join(" ")}\n  here: ${ours.join(" ")}`);
    process.exit(1);
  }
}

// `#string,cr TAG` with TAG naming a language. The outer rule looks ahead to capture the tag for
// its end pattern; the inner rule takes the header and then *continues while* a line does not
// start with the tag, so an embedded construct left open (an unclosed `/*`) cannot run past the
// terminator. The rest of the header line is not part of the body, as in the lexer.
function embeddedHereString({ tags, name, scopes }) {
  const flags = `(?:\\s*,\\s*(?:\\\\.|${ID}))*`;
  return {
    name: `meta.here-string.${name}.jai`,
    begin: `(?=#string${flags}\\s*((?i:${tags.join("|")}))(?![${ID_CHAR}]))`,
    end: `^[ \\t]*(\\1)(?![${ID_CHAR}])`,
    endCaptures: { 1: { name: "entity.name.tag.here-string.jai" } },
    patterns: [
      {
        begin: `\\G(#string)(${flags})\\s*(${ID})(.*)`,
        beginCaptures: {
          1: { name: "keyword.other.directive.here-string.jai" },
          2: { name: "storage.modifier.here-string.jai" },
          3: { name: "entity.name.tag.here-string.jai" },
          4: { name: "string.unquoted.here-string.body.jai" },
        },
        while: `^(?![ \\t]*\\3(?![${ID_CHAR}]))`,
        contentName: `meta.embedded.block.${name}`,
        patterns: scopes.map((scope) => ({ include: scope })),
      },
    ],
  };
}

// `<lead> *[..] Type`: the type named after a declaration's colon or a procedure's arrow.
function typeAfter(lead, leadScope) {
  return {
    match: `${lead}\\s*((?:\\*\\s*|\\[[^\\]\\n]*\\]\\s*)*)(?:(\\$)(${ID})|(${ID}))?`,
    captures: {
      1: { name: leadScope },
      2: { patterns: [{ include: "#type-prefix" }] },
      3: { name: "punctuation.definition.polymorph.jai" },
      4: { name: "entity.name.type.parameter.jai" },
      5: {
        patterns: [
          { include: "#keyword" },
          { name: "entity.name.type.jai", match: ".+" },
        ],
      },
    },
  };
}

const grammar = {
  $schema: "https://raw.githubusercontent.com/martinring/tmlanguage/master/tmlanguage.json",
  name: "Jai",
  scopeName: "source.jai",
  fileTypes: ["jai"],
  patterns: [{ include: "#code" }],
  repository: {
    code: {
      patterns: [
        { include: "#comments" },
        { include: "#here-string" },
        { include: "#char" },
        { include: "#string" },
        { include: "#note" },
        { include: "#directive" },
        { include: "#number" },
        { include: "#for-header" },
        { include: "#procedure-declaration" },
        { include: "#type-declaration" },
        { include: "#declaration" },
        { include: "#type-annotation" },
        { include: "#return-type" },
        { include: "#keyword" },
        { include: "#polymorph" },
        { include: "#call" },
        { include: "#member" },
        { include: "#operator" },
        { include: "#punctuation" },
      ],
    },

    comments: {
      patterns: [
        { include: "#block-comment" },
        {
          name: "comment.line.double-slash.jai",
          begin: "//",
          beginCaptures: { 0: { name: "punctuation.definition.comment.jai" } },
          end: "$",
        },
      ],
    },
    // Block comments nest: `/* a /* b */ still a comment */`.
    "block-comment": {
      name: "comment.block.jai",
      begin: "/\\*",
      beginCaptures: { 0: { name: "punctuation.definition.comment.begin.jai" } },
      end: "\\*/",
      endCaptures: { 0: { name: "punctuation.definition.comment.end.jai" } },
      patterns: [{ include: "#block-comment" }],
    },

    "here-string": {
      patterns: [...EMBEDDED_LANGUAGES.map(embeddedHereString), { include: "#plain-here-string" }],
    },
    // `#string TAG` (flags such as `,cr` or `,\%` allowed) up to a line that starts with TAG.
    // The rest of the opening line is not code: the body starts on the next line.
    "plain-here-string": {
      name: "string.unquoted.here-string.jai",
      begin: `(#string)((?:\\s*,\\s*(?:\\\\.|${ID}))*)\\s*(${ID})`,
      beginCaptures: {
        1: { name: "keyword.other.directive.here-string.jai" },
        2: { name: "storage.modifier.here-string.jai" },
        3: { name: "entity.name.tag.here-string.jai" },
      },
      end: `^\\s*(\\3)(?![${ID_CHAR}])`,
      endCaptures: { 1: { name: "entity.name.tag.here-string.jai" } },
      contentName: "string.unquoted.here-string.body.jai",
    },

    char: {
      match: `(#char)\\s*("(?:[^"\\\\]|\\\\.)*")`,
      captures: {
        1: { name: "keyword.other.directive.char.jai" },
        2: { name: "string.quoted.double.char.jai", patterns: [{ include: "#escape" }] },
      },
    },

    // Strings may span lines.
    string: {
      name: "string.quoted.double.jai",
      begin: '"',
      beginCaptures: { 0: { name: "punctuation.definition.string.begin.jai" } },
      end: '"',
      endCaptures: { 0: { name: "punctuation.definition.string.end.jai" } },
      patterns: [{ include: "#escape" }],
    },
    escape: {
      patterns: [
        {
          name: "constant.character.escape.jai",
          match: "\\\\(?:[nrt0eabfv%\\\\\"'\\n]|x[0-9A-Fa-f]{2}|d[0-9]{3}|u[0-9A-Fa-f]{4}|U[0-9A-Fa-f]{8})",
        },
        { name: "invalid.illegal.unknown-escape.jai", match: "\\\\." },
      ],
    },

    // `@note` and `@"note"` attach to declarations.
    note: {
      name: "storage.modifier.note.jai",
      match: `@(?:${ID}|"(?:[^"\\\\]|\\\\.)*")`,
    },

    directive: {
      patterns: [
        {
          name: "keyword.control.import.directive.jai",
          match: `#(?:${IMPORT_DIRECTIVES.join("|")})${E}`,
        },
        {
          name: "keyword.control.conditional.directive.jai",
          match: `#(?:${CONDITIONAL_DIRECTIVES.join("|")})${E}`,
        },
        {
          name: "storage.modifier.directive.jai",
          match: `#(?:${MODIFIER_DIRECTIVES.join("|")})${E}`,
        },
        {
          name: "support.constant.directive.jai",
          match: `#(?:${VALUE_DIRECTIVES.join("|")})${E}`,
        },
        { name: "keyword.other.directive.jai", match: `#${ID}` },
      ],
    },

    number: {
      patterns: [
        { name: "constant.numeric.integer.hexadecimal.jai", match: `${B}0[xX][0-9A-Fa-f_]+${E}` },
        { name: "constant.numeric.integer.binary.jai", match: `${B}0[bB][01_]+${E}` },
        // `0h` spells a float by its bit pattern.
        { name: "constant.numeric.float.hexadecimal.jai", match: `${B}0[hH][0-9A-Fa-f_]+${E}` },
        // `1.5`, `1.`, `1e9`, `1_000.25e-3`, `.5`; not the `1` of `1..2` or `1.foo`.
        {
          name: "constant.numeric.float.decimal.jai",
          match: `${B}(?:[0-9][0-9_]*(?:\\.(?!\\.)(?![${ID_START}])[0-9_]*)(?:[eE][+-]?[0-9][0-9_]*)?|[0-9][0-9_]*[eE][+-]?[0-9][0-9_]*)${E}`,
        },
        { name: "constant.numeric.float.decimal.jai", match: `(?<![${ID_CHAR}.)\\]])\\.[0-9][0-9_]*(?:[eE][+-]?[0-9][0-9_]*)?${E}` },
        { name: "constant.numeric.integer.decimal.jai", match: `${B}[0-9][0-9_]*${E}` },
      ],
    },

    // `for x: xs`, `for <*x, i: xs`: the loop variables, so the `:` is not a type annotation.
    "for-header": {
      match: `${B}(for)${E}(?:\\s*(<))?(?:\\s*(\\*))?(?:\\s*(${ID})\\s*(?:(,)\\s*(${ID})\\s*)?(:)(?![:=]))?`,
      captures: {
        1: { name: "keyword.control.loop.jai" },
        2: { name: "keyword.operator.reverse.jai" },
        3: { name: "keyword.operator.pointer.jai" },
        4: { name: "variable.other.declaration.jai" },
        5: { name: "punctuation.separator.comma.jai" },
        6: { name: "variable.other.declaration.jai" },
        7: { name: "punctuation.separator.colon.jai" },
      },
    },

    // `name :: (args) -> ret {`, `name :: inline (...)`. A parenthesised constant
    // (`N :: (A + B) * 2;`) is told apart by what follows the closing parenthesis.
    "procedure-declaration": {
      match: `(${ID})\\s*(::)\\s*(?=(?:(?:inline|no_inline)\\s+)?\\((?:[^()]|\\((?:[^()]|\\([^()]*\\))*\\))*\\)\\s*(?:->|\\{|#|@|$|//|/\\*))`,
      captures: {
        1: { name: "entity.name.function.jai" },
        2: { name: "keyword.operator.declaration.constant.jai" },
      },
    },

    // `Name :: struct`, `Name :: enum u8`, `Name :: #type (...)`.
    "type-declaration": {
      match: `(${ID})\\s*(::)\\s*(?=(?:struct|union|enum|enum_flags)${E}|#type${E})`,
      captures: {
        1: { name: "entity.name.type.jai" },
        2: { name: "keyword.operator.declaration.constant.jai" },
      },
    },

    // Names declared with `::`, `:=` or `: Type`, one or several (`a, b := f();`).
    declaration: {
      patterns: [
        { name: "variable.other.constant.declaration.jai", match: `${B}(?!(?:${CONTROL.join("|")}|using)${E})${ID}${namesThen("::")}` },
        { name: "variable.other.declaration.jai", match: `${B}(?!(?:${CONTROL.join("|")}|using)${E})${ID}${namesThen(":=")}` },
        { name: "variable.other.declaration.jai", match: `${B}(?!(?:${CONTROL.join("|")}|using)${E})${ID}${namesThen(":(?![:=])")}` },
      ],
    },

    // `: Type`, `: *[..] Type`, `: $T`: the type named after a declaration's colon.
    "type-annotation": typeAfter("(:)(?![:=])", "punctuation.separator.colon.jai"),
    // `-> Type`: the (first) return type.
    // Not a named result (`-> ok: bool`), whose name the declaration rule takes.
    "return-type": typeAfter(`(->)(?!\\s*${ID}\\s*:(?![:=]))`, "keyword.operator.arrow.jai"),
    "type-prefix": {
      patterns: [{ name: "keyword.operator.pointer.jai", match: "\\*" }, { include: "#code" }],
    },

    keyword: {
      patterns: [
        { name: "keyword.control.jai", match: words(CONTROL) },
        { name: "keyword.other.using.jai", match: words(["using"]) },
        {
          // `cast(T) x`, `cast,no_check(T) x`, `xx x`, `xx,trunc x`.
          match: `${B}(cast|xx)${E}((?:\\s*,\\s*(?:${CAST_FLAGS.join("|")})${E})*)`,
          captures: {
            1: { name: "keyword.operator.cast.jai" },
            2: { name: "storage.modifier.cast.jai" },
          },
        },
        { name: "storage.type.jai", match: words(["struct", "union", "enum", "enum_flags"]) },
        { name: "storage.type.function.operator.jai", match: words(["operator"]) },
        { name: "storage.modifier.jai", match: words(["inline", "no_inline", "interface"]) },
        { name: "variable.language.it.jai", match: words(["it", "it_index"]) },
        { name: "variable.language.context.jai", match: words(["context"]) },
        { name: "constant.language.jai", match: words(["null", "true", "false"]) },
        { name: "support.type.builtin.jai", match: words(BUILTIN_TYPES) },
        { name: "support.function.builtin.jai", match: `${words(BUILTIN_FUNCTIONS)}(?=\\s*\\()` },
      ],
    },

    // `$T` introduces a type variable, `$$x` a value that is baked when constant.
    polymorph: {
      patterns: [
        {
          match: `(\\$\\$)(${ID})`,
          captures: {
            1: { name: "punctuation.definition.polymorph.jai" },
            2: { name: "variable.parameter.baked.jai" },
          },
        },
        {
          match: `(\\$)(${ID})`,
          captures: {
            1: { name: "punctuation.definition.polymorph.jai" },
            2: { name: "entity.name.type.parameter.jai" },
          },
        },
      ],
    },

    call: {
      match: `(${ID})(?=\\s*\\()`,
      captures: { 1: { name: "entity.name.function.call.jai" } },
    },

    // `.member`, `.ENUM_VALUE`; `.{` and `.[` literals are punctuation.
    member: {
      match: `(\\.)(${ID})`,
      captures: {
        1: { name: "punctuation.accessor.jai" },
        2: { name: "variable.other.member.jai" },
      },
    },

    operator: {
      patterns: [
        { name: "constant.language.uninitialized.jai", match: "---" },
        { name: "keyword.operator.declaration.constant.jai", match: "::" },
        { name: "keyword.operator.declaration.jai", match: ":=" },
        { name: "keyword.operator.arrow.jai", match: "->|=>" },
        { name: "keyword.operator.range.jai", match: "\\.\\." },
        { name: "keyword.operator.dereference.jai", match: "\\.\\*" },
        { name: "keyword.operator.backtick.jai", match: "`" },
        { name: "keyword.operator.comparison.jai", match: "==|!=|<=|>=" },
        { name: "keyword.operator.logical.jai", match: "&&|\\|\\||!" },
        { name: "keyword.operator.assignment.compound.jai", match: "(?:<<<|>>>|<<|>>|[-+*/%&|^~])=" },
        { name: "keyword.operator.bitwise.shift.jai", match: "<<<|>>>|<<|>>" },
        { name: "keyword.operator.comparison.jai", match: "<|>" },
        { name: "keyword.operator.assignment.jai", match: "=" },
        { name: "keyword.operator.arithmetic.jai", match: "[-+*/%]" },
        { name: "keyword.operator.bitwise.jai", match: "[&|^~]" },
      ],
    },

    punctuation: {
      patterns: [
        { name: "punctuation.terminator.statement.jai", match: ";" },
        { name: "punctuation.separator.comma.jai", match: "," },
        { name: "punctuation.separator.colon.jai", match: ":" },
        { name: "punctuation.accessor.jai", match: "\\." },
      ],
    },
  },
};

// package.json maps each embedded body's scope to its language ID.
const manifestPath = join(root, "package.json");
const manifestText = readFileSync(manifestPath, "utf8");
const manifest = JSON.parse(manifestText);
manifest.contributes.grammars.find((g) => g.scopeName === "source.jai").embeddedLanguages = Object.fromEntries(
  EMBEDDED_LANGUAGES.map(({ name, language }) => [`meta.embedded.block.${name}`, language]),
);
const manifestOut = JSON.stringify(manifest, null, 2) + "\n";

const text = JSON.stringify(grammar, null, 2) + "\n";
if (process.argv.includes("--check")) {
  let current = "";
  try {
    current = readFileSync(output, "utf8");
  } catch {}
  if (current !== text) {
    console.error("syntaxes/jai.tmLanguage.json is stale: run `node scripts/build-grammar.mjs`");
    process.exit(1);
  }
  if (manifestText !== manifestOut) {
    console.error("package.json's embeddedLanguages are stale: run `node scripts/build-grammar.mjs`");
    process.exit(1);
  }
} else {
  writeFileSync(output, text);
  writeFileSync(manifestPath, manifestOut);
}
