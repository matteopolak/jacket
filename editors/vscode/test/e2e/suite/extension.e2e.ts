// Runs inside VS Code's extension host, against a real jailsp (see ../run.ts).
import assert from "node:assert/strict";
import * as path from "node:path";
import * as vscode from "vscode";

const workspace = () => vscode.workspace.workspaceFolders![0].uri.fsPath;

async function open(name: string): Promise<vscode.TextDocument> {
  const document = await vscode.workspace.openTextDocument(path.join(workspace(), name));
  await vscode.window.showTextDocument(document);
  return document;
}

/** Waits until `predicate` holds for the document's diagnostics. */
async function diagnostics(document: vscode.TextDocument, predicate: (d: vscode.Diagnostic[]) => boolean, timeout = 60_000): Promise<vscode.Diagnostic[]> {
  const deadline = Date.now() + timeout;
  for (;;) {
    const current = vscode.languages.getDiagnostics(document.uri);
    if (predicate(current)) return current;
    if (Date.now() > deadline) {
      assert.fail(`timed out waiting for diagnostics on ${path.basename(document.fileName)}; have ${JSON.stringify(current.map((d) => d.message))}`);
    }
    await new Promise((resolve) => setTimeout(resolve, 200));
  }
}

const labelOf = (item: vscode.CompletionItem) => (typeof item.label === "string" ? item.label : item.label.label);

describe("Jai extension", () => {
  let api: { client(): { isRunning(): boolean } | undefined };

  before(async () => {
    const extension = vscode.extensions.getExtension("matteopolak.jai-toolchain");
    assert.ok(extension, "the extension is installed");
    await open("broken.jai");
    api = await extension.activate();
    const deadline = Date.now() + 30_000;
    while (!api.client()?.isRunning()) {
      if (Date.now() > deadline) assert.fail("jailsp did not start");
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
  });

  it("registers the commands", async () => {
    const commands = await vscode.commands.getCommands(true);
    for (const command of ["jai.restartServer", "jai.runFile", "jai.buildFile", "jai.checkFile", "jai.downloadToolchain"]) {
      assert.ok(commands.includes(command), command);
    }
  });

  it("shows jaic's errors as diagnostics", async () => {
    const document = await open("broken.jai");
    const found = await diagnostics(document, (d) => d.some((x) => x.severity === vscode.DiagnosticSeverity.Error));
    const error = found.find((x) => x.severity === vscode.DiagnosticSeverity.Error)!;
    assert.equal(error.range.start.line, 1, JSON.stringify(found.map((d) => d.message)));
  });

  it("shows jailint findings and fixes them all with source.fixAll.jailint", async () => {
    const document = await open("lint.jai");
    const found = await diagnostics(document, (d) => d.some((x) => x.source === "jailint"));
    const lint = found.find((x) => x.source === "jailint")!;
    const code = typeof lint.code === "object" ? lint.code.value : lint.code;
    assert.equal(code, "bool_comparison");

    const actions = await vscode.commands.executeCommand<vscode.CodeAction[]>(
      "vscode.executeCodeActionProvider",
      document.uri,
      new vscode.Range(0, 0, document.lineCount, 0),
      "source.fixAll.jailint",
    );
    const fixAll = actions.find((a) => a.kind?.value === "source.fixAll.jailint");
    assert.ok(fixAll?.edit, `a fix-all action with an edit; got ${actions.map((a) => a.title)}`);
    assert.ok(await vscode.workspace.applyEdit(fixAll.edit));
    assert.match(document.getText(), /if ready print/);
    await vscode.commands.executeCommand("workbench.action.files.revert");
  });

  it("applies unsaved jailint.toml edits", async () => {
    const lint = await open("lint.jai");
    await diagnostics(lint, (d) => d.some((x) => x.source === "jailint"));
    const settings = await open("jailint.toml");
    assert.equal(settings.languageId, "toml");
    const edit = new vscode.WorkspaceEdit();
    edit.insert(settings.uri, new vscode.Position(1, 0), 'bool_comparison = "allow"\n');
    assert.ok(await vscode.workspace.applyEdit(edit));
    await open("lint.jai");
    await diagnostics(lint, (d) => !d.some((x) => x.source === "jailint"));
    await open("jailint.toml");
    await vscode.commands.executeCommand("workbench.action.files.revert");
    await vscode.commands.executeCommand("workbench.action.closeActiveEditor");
  });

  it("completes a name from a module the file does not import, adding the #import", async () => {
    const document = await open("auto-import.jai");
    const list = await vscode.commands.executeCommand<vscode.CompletionList>("vscode.executeCompletionItemProvider", document.uri, new vscode.Position(1, 8));
    const print = list.items.find((item) => labelOf(item) === "print" && item.additionalTextEdits?.length);
    assert.ok(print, JSON.stringify(list.items.slice(0, 20).map(labelOf)));
    assert.equal(typeof print.label === "object" ? print.label.description : undefined, "Basic");
    const edit = print.additionalTextEdits![0];
    assert.equal(edit.newText, '#import "Basic";\n\n');
    assert.equal(edit.range.start.line, 0);
  });

  it("provides semantic tokens", async () => {
    const document = await open("lint.jai");
    const tokens = await vscode.commands.executeCommand<vscode.SemanticTokens>("vscode.provideDocumentSemanticTokens", document.uri);
    assert.ok(tokens && tokens.data.length > 0);
  });

  it("formats with jaifmt and applies the edit", async () => {
    const document = await open("unformatted.jai");
    const edits = await vscode.commands.executeCommand<vscode.TextEdit[]>("vscode.executeFormatDocumentProvider", document.uri, {
      tabSize: 4,
      insertSpaces: true,
    });
    assert.ok(edits && edits.length > 0, "the formatter changed something");
    const edit = new vscode.WorkspaceEdit();
    edit.set(document.uri, edits);
    assert.ok(await vscode.workspace.applyEdit(edit));
    assert.equal(document.getText(), "main :: () {\n    x := 1;\n    bump(*x);\n}\n\nbump :: (x: *int) {\n    x.* += 1;\n}\n");
    await vscode.commands.executeCommand("workbench.action.files.revert");
  });

  it("leaves files jaifmt.toml ignores alone", async () => {
    const document = await open(path.join("sub", "ignored.jai"));
    const edits = await vscode.commands.executeCommand<vscode.TextEdit[]>("vscode.executeFormatDocumentProvider", document.uri, {
      tabSize: 4,
      insertSpaces: true,
    });
    assert.equal(edits?.length ?? 0, 0);
  });

  it("highlights with the TextMate grammar", async () => {
    const document = await open("broken.jai");
    assert.equal(document.languageId, "jai");
  });

  it("highlights a language-tagged here-string body with that language's grammar", async () => {
    // VS Code's own grammars (SQL, Python) and the bundled WGSL one, as the editor applies them.
    const file = vscode.Uri.file(path.join(workspace(), "here-strings.jai"));
    const tokens = await vscode.commands.executeCommand<{ c: string; t: string }[]>("_workbench.captureSyntaxTokens", file);
    const scopes = (text: string) => tokens.find((token) => token.c === text)?.t ?? `no token ${text}`;
    assert.match(scopes("SELECT"), /meta\.embedded\.block\.sql .*keyword\.other\.DML\.sql/);
    assert.match(scopes("def"), /meta\.embedded\.block\.python .*storage\.type\.function\.python/);
    assert.match(scopes("vertex"), /meta\.embedded\.block\.wgsl .*entity\.name\.function\.decorator\.wgsl/);
    assert.match(scopes("plain text"), /string\.unquoted\.here-string\.body\.jai/);
    assert.doesNotMatch(scopes("after"), /here-string/);
  });

  it("restarts the language server", async () => {
    await vscode.commands.executeCommand("jai.restartServer");
    assert.ok(api.client()?.isRunning());
    const document = await open("broken.jai");
    await diagnostics(document, (d) => d.length > 0);
  });
});
