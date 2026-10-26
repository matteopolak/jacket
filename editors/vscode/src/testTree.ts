// The test explorer's tree, from `jacket test --list --format json`: files, their tests, and under a
// `@cases` test the subtest of each case. Pure.
import type { Listed } from "./testEvents";

export interface TestEntry {
  /** The name `jacket test` knows it by (`--exact NAME`). */
  name: string;
  kind: string;
  /** 1-based line of the declaration. */
  line: number;
  /** What the case is called, for a subtest. */
  label: string;
  children: TestEntry[];
}

export interface FileEntry {
  file: string;
  tests: TestEntry[];
}

/** Kinds the explorer cannot run: `jacket bench` only lists benchmarks for now. */
const NOT_RUNNABLE = new Set(["bench"]);

/** Files in path order, each with its tests in source order. */
export function buildTree(listed: readonly Listed[]): FileEntry[] {
  const files = new Map<string, TestEntry[]>();
  const subtests: Listed[] = [];
  for (const item of listed) {
    if (NOT_RUNNABLE.has(item.kind)) continue;
    if (item.kind === "subtest") {
      subtests.push(item);
      continue;
    }
    const tests = files.get(item.file) ?? [];
    tests.push({ name: item.name, kind: item.kind, line: item.line, label: item.name, children: [] });
    files.set(item.file, tests);
  }
  for (const sub of subtests) {
    // A case lies on its test's declaration line; its name is `test/case` and the case may hold `/`.
    const parent = files.get(sub.file)?.find((t) => t.line === sub.line && sub.name.startsWith(`${t.name}/`));
    if (!parent) continue;
    parent.children.push({ name: sub.name, kind: sub.kind, line: sub.line, label: sub.name.slice(parent.name.length + 1), children: [] });
  }
  return [...files.entries()]
    .toSorted(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0))
    .map(([file, tests]) => ({ file, tests: tests.toSorted((a, b) => a.line - b.line) }));
}

/** The id of the explorer item for `name` in `file`. */
export function testId(file: string, name: string): string {
  return `${file}#${name}`;
}
