// A stand-in for `jaifmt --stdin` in the integration tests when no real jaifmt is built:
// strips trailing whitespace, ends the file with one newline, and refuses unbalanced braces
// with exit status 2, as jaifmt does. It records its working directory in a comment so the
// test can check the extension starts it next to the document.
import { readFileSync } from "node:fs";

if (!process.argv.includes("--stdin")) {
  process.stderr.write("fake-jaifmt: expected --stdin\n");
  process.exit(2);
}
const input = readFileSync(0, "utf8");
let depth = 0;
for (const c of input) {
  if (c === "{") depth++;
  if (c === "}") depth--;
}
if (depth !== 0) {
  process.stderr.write("jaifmt: <stdin>:1:1: error: unbalanced `{`\n");
  process.exit(2);
}
const lines = input.split("\n").map((line) => line.replace(/[ \t]+$/, ""));
while (lines.length && lines[lines.length - 1] === "") lines.pop();
process.stdout.write(lines.join("\n") + "\n");
