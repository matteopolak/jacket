// Tokenizes real Jai files with the grammar and fails when one ends inside a string, comment or
// here-string (an unbalanced begin/end rule), or has an `invalid` token. This catches rules that
// swallow the rest of a file, which the hand-written assertions in test/grammar cannot.
//
//   node scripts/tokenize-corpus.mjs [files or directories...]
// Default: the repository's examples/ and stdlib/.
import { readFileSync, readdirSync, statSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);
const vsctm = require("vscode-textmate");
const oniguruma = require("vscode-oniguruma");

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const repository = join(root, "..", "..");

const wasm = readFileSync(require.resolve("vscode-oniguruma/release/onig.wasm")).buffer;
await oniguruma.loadWASM(wasm);
const registry = new vsctm.Registry({
  onigLib: Promise.resolve({
    createOnigScanner: (patterns) => new oniguruma.OnigScanner(patterns),
    createOnigString: (text) => new oniguruma.OnigString(text),
  }),
  // The extension's own grammars (the bundled WGSL one highlights `#string WGSL` bodies); the
  // ones VS Code ships are not here, so those here-strings stay plain strings.
  loadGrammar: async (scopeName) => {
    const manifest = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
    const entry = manifest.contributes.grammars.find((g) => g.scopeName === scopeName);
    if (!entry) return null;
    const path = join(root, entry.path);
    return vsctm.parseRawGrammar(readFileSync(path, "utf8"), path);
  },
});
const grammar = await registry.loadGrammar("source.jai");

function* jaiFiles(path) {
  if (statSync(path).isDirectory()) {
    for (const entry of readdirSync(path).toSorted()) {
      if (entry.startsWith(".")) continue;
      yield* jaiFiles(join(path, entry));
    }
  } else if (path.endsWith(".jai")) {
    yield path;
  }
}

const args = process.argv.slice(2);
const roots = args.length ? args : [join(repository, "examples"), join(repository, "stdlib")];
let files = 0;
let lines = 0;
const failures = [];
for (const top of roots) {
  for (const file of jaiFiles(top)) {
    // Formatter golden inputs and deliberately malformed cases are not real programs.
    if (/Jai_Format[\\/]tests[\\/]cases/.test(file)) continue;
    files += 1;
    const text = readFileSync(file, "utf8").split(/\r?\n/);
    let stack = vsctm.INITIAL;
    const name = relative(repository, file);
    text.forEach((line, index) => {
      lines += 1;
      const result = grammar.tokenizeLine(line, stack, 1000);
      for (const token of result.tokens) {
        if (token.scopes.some((s) => s.startsWith("invalid"))) {
          failures.push(`${name}:${index + 1}:${token.startIndex + 1}: invalid token \`${line.slice(token.startIndex, token.endIndex)}\``);
        }
      }
      stack = result.ruleStack;
    });
    if (stack.depth > 1) {
      // Name the scope the file ended in, to find the rule that never closed.
      const last = grammar.tokenizeLine("", stack).tokens.at(-1)?.scopes.at(-1);
      failures.push(`${name}: ends inside \`${last}\``);
    }
  }
}
if (failures.length) {
  console.error(failures.slice(0, 50).join("\n"));
  console.error(`${failures.length} problem(s) in ${files} files`);
  process.exit(1);
}
console.log(`grammar tokenized ${files} files (${lines} lines) cleanly`);
