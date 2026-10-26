// The test explorer: tests found by `jacket test --list --format json`, run with
// `jacket test --format json`, failures shown in the explorer and as problems. The lens commands
// `jai.runTest` and `jai.debugTest` (from jailsp's "Run test" and "Debug test" lenses) go through
// the same runner.
import * as fs from "node:fs";
import * as path from "node:path";
import * as vscode from "vscode";
import { type Finished, type Listed, type Message, located, outcomeOf, subtestVerdict, terminalText } from "./testEvents";
import { listArguments, runArguments, runJacket } from "./testProcess";
import { type FileEntry, type TestEntry, buildTree, testId } from "./testTree";

export interface JacketTool {
  path: string;
  env: Record<string, string>;
}

export interface TestHost {
  log: vscode.LogOutputChannel;
  /** The `jacket` to run and its environment; undefined (after telling the user) when there is none. */
  jacket(): JacketTool | undefined;
}

/** The argument of a lens command (`crates/jai-language-server/src/test_lenses.rs`). */
export interface LensArgument {
  uri: string;
  file: string;
  name: string;
  kind: string;
  projectRoot: string;
  testRoot: string | null;
  filter: string[];
}

/** Where `jacket test` runs for an item, and the flags that pick its tests there. */
interface Origin {
  cwd: string;
  flags: string[];
}

interface Meta {
  origin: Origin;
  /** The name `jacket test` knows the test by; none for a file. */
  name?: string;
}

/** The items run by one `jacket test` process. */
interface Batch {
  origin: Origin;
  /** Everything the origin finds: no names are passed. */
  everything: boolean;
  items: vscode.TestItem[];
}

const SOURCE = "jacket test";
const DEBUG_UNAVAILABLE =
  "Jai: debugging tests is not available: the extension has no debug configuration for Jai programs yet. Use Run Test.";
const REFRESH_DELAY_MS = 500;

/** The path as the filesystem spells it, so jacket's paths and the editor's meet. */
function canonical(file: string): string {
  try {
    return fs.realpathSync.native(file);
  } catch {
    return file;
  }
}

function lineRange(line: number): vscode.Range {
  const at = Math.max(line - 1, 0);
  return new vscode.Range(at, 0, at, 0);
}

export class JacketTests implements vscode.Disposable {
  private readonly controller = vscode.tests.createTestController("jacket", "Jacket");
  private readonly problems = vscode.languages.createDiagnosticCollection(SOURCE);
  private readonly metas = new WeakMap<vscode.TestItem, Meta>();
  private readonly refreshes = new Map<string, NodeJS.Timeout>();
  private readonly subscriptions: vscode.Disposable[] = [];
  /** Discoveries and runs never overlap: both build the tests in the same folders. */
  private queue: Promise<void> = Promise.resolve();

  constructor(private readonly host: TestHost) {
    this.controller.resolveHandler = async (item) => {
      if (!item) await this.discoverAll();
    };
    this.controller.refreshHandler = () => this.discoverAll();
    this.controller.createRunProfile("Run", vscode.TestRunProfileKind.Run, (request, token) => this.run(request, token), true);
    this.controller.createRunProfile("Debug", vscode.TestRunProfileKind.Debug, () => this.debugUnavailable(), false);
    this.subscriptions.push(
      this.controller,
      this.problems,
      vscode.workspace.onDidSaveTextDocument((document) => this.saved(document)),
      vscode.workspace.onDidGrantWorkspaceTrust(() => void this.discoverAll()),
    );
  }

  dispose(): void {
    for (const timer of this.refreshes.values()) clearTimeout(timer);
    for (const subscription of this.subscriptions) subscription.dispose();
  }

  // --- Discovery ------------------------------------------------------------------------

  /** Finds the tests of every workspace folder again. */
  discoverAll(): Promise<void> {
    return this.enqueue(async () => {
      for (const folder of vscode.workspace.workspaceFolders ?? []) await this.discover(folder.uri.fsPath);
    });
  }

  private saved(document: vscode.TextDocument): void {
    const file = canonical(document.uri.fsPath);
    const folder = vscode.workspace.getWorkspaceFolder(document.uri)?.uri.fsPath;
    // A saved file can add or remove tests when it is a test file, or the settings that say where tests are.
    const testFile = this.controller.items.get(file) !== undefined || /[\\/]tests[\\/]/.test(file);
    if (!folder || !(path.basename(file) === "jacket.toml" || (document.languageId === "jai" && testFile))) return;
    clearTimeout(this.refreshes.get(folder));
    this.refreshes.set(
      folder,
      setTimeout(() => {
        this.refreshes.delete(folder);
        void this.enqueue(() => this.discover(folder));
      }, REFRESH_DELAY_MS),
    );
  }

  private enqueue(action: () => Promise<void>): Promise<void> {
    this.queue = this.queue.then(action).catch((error: unknown) => {
      this.host.log.error(error instanceof Error ? (error.stack ?? error.message) : String(error));
    });
    return this.queue;
  }

  private extraArguments(uri: vscode.Uri | undefined): string[] {
    return vscode.workspace.getConfiguration("jai", uri).get<string[]>("tests.arguments", []);
  }

  private async discover(folder: string): Promise<void> {
    if (!vscode.workspace.isTrusted) return;
    const jacket = this.host.jacket();
    if (!jacket) return;
    const listed: Listed[] = [];
    const finish = await runJacket(jacket.path, listArguments(this.extraArguments(vscode.Uri.file(folder))), {
      cwd: folder,
      env: jacket.env,
      onEvent: (event) => {
        if (event.type === "list") listed.push({ ...event.test, file: canonical(event.test.file) });
      },
    });
    if (finish.code !== 0) {
      // A folder with no tests has none to show; a build error keeps the tests shown last.
      const empty = /no tests found/.test(finish.stderr);
      this.host.log[empty ? "info" : "warn"](`jacket test --list in ${folder}: ${finish.stderr.trim() || `exit status ${String(finish.code)}`}`);
      if (!empty) {
        vscode.window.setStatusBarMessage("$(warning) jacket: could not list the tests; see the Jai output", 8000);
        return;
      }
    }
    const origin: Origin = { cwd: folder, flags: [] };
    const tree = buildTree(listed);
    const wanted = new Set(tree.map((entry) => entry.file));
    for (const item of gather(this.controller.items)) {
      const meta = this.metas.get(item);
      if (meta?.origin.cwd === folder && meta.origin.flags.length === 0 && !wanted.has(item.id)) this.controller.items.delete(item.id);
    }
    for (const entry of tree) this.upsertFile(entry, origin);
  }

  private upsertFile(entry: FileEntry, origin: Origin): vscode.TestItem {
    const uri = vscode.Uri.file(entry.file);
    const item = this.controller.items.get(entry.file) ?? this.controller.createTestItem(entry.file, vscode.workspace.asRelativePath(uri, false), uri);
    this.metas.set(item, { origin });
    this.controller.items.add(item);
    item.children.replace(entry.tests.map((test) => this.testItem(entry.file, test, origin)));
    return item;
  }

  private testItem(file: string, test: TestEntry, origin: Origin): vscode.TestItem {
    const item = this.controller.createTestItem(testId(file, test.name), test.label, vscode.Uri.file(file));
    item.range = lineRange(test.line);
    if (test.kind !== "test" && test.kind !== "subtest") item.description = test.kind.replace("_", " ");
    this.metas.set(item, { origin, name: test.name });
    item.children.replace(test.children.map((child) => this.testItem(file, child, origin)));
    return item;
  }

  // --- Running --------------------------------------------------------------------------

  private debugUnavailable(): void {
    void vscode.window.showWarningMessage(DEBUG_UNAVAILABLE);
  }

  /**
   * The items a lens is over: the explorer's own when discovery found the test, else a new one
   * that runs from the lens' project with the flags it carries.
   */
  private lensItem(argument: LensArgument): vscode.TestItem {
    const file = canonical(argument.file);
    const known = this.controller.items.get(file)?.children.get(testId(file, argument.name));
    if (known) return known;
    const origin: Origin = { cwd: argument.projectRoot, flags: [...(argument.testRoot ? ["--path", argument.testRoot] : []), ...argument.filter] };
    const entry: TestEntry = { name: argument.name, kind: argument.kind, line: 1, label: argument.name, children: [] };
    const parent = this.controller.items.get(file) ?? this.upsertFile({ file, tests: [] }, origin);
    const item = this.testItem(file, entry, origin);
    parent.children.add(item);
    return item;
  }

  /** The lens commands: run `argument`'s test through the explorer, or say why it cannot be debugged. */
  async runLens(argument: LensArgument, debug: boolean): Promise<void> {
    if (debug) return this.debugUnavailable();
    const cancellation = new vscode.CancellationTokenSource();
    try {
      await this.run(new vscode.TestRunRequest([this.lensItem(argument)]), cancellation.token);
    } finally {
      cancellation.dispose();
    }
  }

  private run(request: vscode.TestRunRequest, token: vscode.CancellationToken): Promise<void> {
    if (!vscode.workspace.isTrusted) {
      void vscode.window.showWarningMessage("Jai: trust this workspace to run tests: they execute its code at compile time and run time.");
      return Promise.resolve();
    }
    const jacket = this.host.jacket();
    if (!jacket) return Promise.resolve();
    const run = this.controller.createTestRun(request);
    return this.enqueue(async () => {
      try {
        for (const batch of this.batches(request)) await this.runBatch(run, batch, jacket, token);
      } finally {
        run.end();
      }
    });
  }

  /** The tests the request names (a file means its tests; a test with cases, all of them), grouped by how jacket is run for them. */
  private batches(request: vscode.TestRunRequest): Batch[] {
    const excluded = new Set(request.exclude ?? []);
    const chosen = new Map<string, vscode.TestItem>();
    const add = (item: vscode.TestItem) => {
      if (excluded.has(item)) return;
      if (this.metas.get(item)?.name === undefined) item.children.forEach(add);
      else chosen.set(item.id, item);
    };
    const everything = request.include === undefined && excluded.size === 0;
    (request.include ?? gather(this.controller.items)).forEach(add);
    // A test with cases runs them all; running them one by one too would run each twice.
    const items = [...chosen.values()].filter((item) => !(item.parent && chosen.has(item.parent.id)));
    const batches = new Map<string, Batch>();
    for (const item of items) {
      const origin = this.metas.get(item)?.origin;
      if (!origin) continue;
      const key = JSON.stringify(origin);
      const batch = batches.get(key) ?? { origin, everything: everything && origin.flags.length === 0, items: [] };
      batch.items.push(item);
      batches.set(key, batch);
    }
    return [...batches.values()];
  }

  private async runBatch(run: vscode.TestRun, batch: Batch, jacket: JacketTool, token: vscode.CancellationToken): Promise<void> {
    const byName = new Map<string, vscode.TestItem>();
    const register = (item: vscode.TestItem) => {
      const name = this.metas.get(item)?.name;
      if (name !== undefined) byName.set(name, item);
      run.enqueued(item);
      item.children.forEach(register);
    };
    batch.items.forEach(register);
    const files = new Set(batch.items.map((item) => item.uri?.fsPath ?? ""));
    // A run replaces the problems of the files it covers.
    for (const file of files) this.problems.delete(vscode.Uri.file(file));
    const found = new Map<string, vscode.Diagnostic[]>();
    const finished = new Set<string>();
    const abort = new AbortController();
    const cancel = token.onCancellationRequested(() => abort.abort());
    // A lens' flags pick their one test; everything else is picked by name.
    const names = batch.everything || batch.origin.flags.length > 0 ? [] : batch.items.map((item) => this.metas.get(item)?.name ?? "");
    try {
      const args = runArguments({ flags: batch.origin.flags, names }, this.extraArguments(batch.items[0]?.uri));
      this.host.log.info(`jacket ${args.join(" ")} (in ${batch.origin.cwd})`);
      const result = await runJacket(jacket.path, args, {
        cwd: batch.origin.cwd,
        env: jacket.env,
        signal: abort.signal,
        onEvent: (event) => {
          if (event.type === "started") {
            const item = byName.get(event.name);
            if (item) run.started(item);
          } else if (event.type === "finished") {
            finished.add(event.test.name);
            for (const sub of event.test.subtests) finished.add(sub.name);
            this.report(run, byName, event.test, found);
          }
        },
      });
      if (result.stderr.trim()) run.appendOutput(terminalText(result.stderr.trimEnd() + "\n"));
      if (!token.isCancellationRequested) this.unreported(run, byName, finished, result.code, result.stderr);
    } catch (error) {
      this.host.log.error(`running jacket: ${String(error)}`);
      void vscode.window.showErrorMessage(`Jai: could not run jacket (${jacket.path}): ${String(error)}`);
    } finally {
      cancel.dispose();
    }
  }

  /** What the tests the process never reported on become: errors when it failed, else skipped. */
  private unreported(run: vscode.TestRun, byName: Map<string, vscode.TestItem>, finished: Set<string>, code: number | null, stderr: string): void {
    for (const [name, item] of byName) {
      if (finished.has(name)) continue;
      if (code === 0) run.skipped(item);
      else run.errored(item, new vscode.TestMessage(stderr.trim() || `jacket exited with status ${String(code)} before running this test`));
    }
  }

  private report(run: vscode.TestRun, byName: Map<string, vscode.TestItem>, test: Finished, found: Map<string, vscode.Diagnostic[]>): void {
    // Running one case runs its test, which reports on that case alone.
    const item = byName.get(test.name);
    const outcome = outcomeOf(test);
    const output = [test.stdout, test.stderr].filter((text) => text.trim()).join("");
    if (output && item) run.appendOutput(terminalText(`${test.name}\n${output.trimEnd()}\n`), undefined, item);
    for (const sub of test.subtests) {
      const child = byName.get(sub.name);
      if (!child) continue;
      const verdict = subtestVerdict(sub.status);
      const own = outcome.messages.filter((m) => m.test === sub.name);
      this.finish(run, child, verdict, sub.duration * 1000, own);
    }
    if (item) {
      // Cases the test never reached (it was skipped, or crashed) share its verdict.
      const reported = new Set(test.subtests.map((sub) => sub.name));
      item.children.forEach((child) => {
        if (!reported.has(this.metas.get(child)?.name ?? "")) this.finish(run, child, outcome.verdict, 0, []);
      });
      const shared = outcome.messages.filter((m) => m.test === undefined || !byName.has(m.test));
      this.finish(run, item, outcome.verdict, outcome.durationMs, shared);
    }
    for (const message of located(outcome.messages)) {
      const uri = vscode.Uri.file(canonical(message.file));
      const diagnostic = new vscode.Diagnostic(lineRange(message.line), message.text, vscode.DiagnosticSeverity.Error);
      diagnostic.source = SOURCE;
      diagnostic.code = test.name;
      const list = [...(found.get(uri.fsPath) ?? []), diagnostic];
      found.set(uri.fsPath, list);
      this.problems.set(uri, list);
    }
  }

  private finish(run: vscode.TestRun, item: vscode.TestItem, verdict: "passed" | "failed" | "skipped", durationMs: number, messages: Message[]): void {
    if (verdict === "passed") run.passed(item, durationMs);
    else if (verdict === "skipped") run.skipped(item);
    else run.failed(item, messages.length > 0 ? messages.map((m) => testMessage(m, item)) : new vscode.TestMessage("failed"), durationMs);
  }
}

function gather(collection: vscode.TestItemCollection): vscode.TestItem[] {
  const items: vscode.TestItem[] = [];
  collection.forEach((item) => items.push(item));
  return items;
}

function testMessage(message: Message, item: vscode.TestItem): vscode.TestMessage {
  const result = new vscode.TestMessage(message.text);
  if (message.file !== undefined && message.line !== undefined) {
    result.location = new vscode.Location(vscode.Uri.file(canonical(message.file)), lineRange(message.line));
  } else if (item.uri && item.range) {
    result.location = new vscode.Location(item.uri, item.range);
  }
  return result;
}
