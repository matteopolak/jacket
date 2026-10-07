// jailsp's own commands: `jai.showExpansion` (a code action for `#insert`, `#run` and macro calls)
// returns the generated code, which opens as a read-only `jai-expansion:` document, and
// `jai.showPolymorphs` (a code lens) returns the instances of a polymorphic procedure.
import * as vscode from "vscode";
import type { LanguageClient } from "vscode-languageclient/node";

export const EXPANSION_SCHEME = "jai-expansion";

interface Expansion {
  uri: string;
  kind: string;
  text: string;
}

export class ExpansionDocuments implements vscode.TextDocumentContentProvider {
  /** Texts from `jai.showExpansion` results, by `Uri.toString()`. */
  private readonly texts = new Map<string, { original: string; text: string }>();
  private readonly changed = new vscode.EventEmitter<vscode.Uri>();
  readonly onDidChange = this.changed.event;

  constructor(private readonly client: () => LanguageClient | undefined) {}

  async provideTextDocumentContent(uri: vscode.Uri): Promise<string> {
    const known = this.texts.get(uri.toString());
    const client = this.client();
    if (client?.isRunning()) {
      // Recompute: the source may have changed since the command ran.
      const original = known?.original ?? `${EXPANSION_SCHEME}://${uri.path}?${uri.query}`;
      const text = await client.sendRequest<string | null>("jai/source", { uri: original }).catch(() => null);
      if (typeof text === "string") return text;
    }
    return known?.text ?? "// The expansion is no longer available: the language server is not running.\n";
  }

  /** Middleware for `workspace/executeCommand` results. */
  async handle(command: string, result: unknown): Promise<unknown> {
    if (command === "jai.showExpansion" && result && typeof result === "object") {
      const expansion = result as Expansion;
      const uri = vscode.Uri.parse(expansion.uri);
      this.texts.set(uri.toString(), { original: expansion.uri, text: expansion.text });
      this.changed.fire(uri);
      const document = await vscode.workspace.openTextDocument(uri);
      if (document.languageId !== "jai") await vscode.languages.setTextDocumentLanguage(document, "jai");
      await vscode.window.showTextDocument(document, { preview: true, viewColumn: vscode.ViewColumn.Beside });
    } else if (command === "jai.showExpansion") {
      void vscode.window.showInformationMessage("Jai: there is no expansion here.");
    } else if (command === "jai.showPolymorphs" && Array.isArray(result)) {
      const instances = result.map(String);
      if (instances.length === 0) {
        void vscode.window.showInformationMessage("Jai: nothing has instantiated this procedure yet.");
      } else {
        await vscode.window.showQuickPick(instances, {
          title: `${instances.length} polymorph instance${instances.length === 1 ? "" : "s"}`,
          canPickMany: false,
        });
      }
    }
    return result;
  }
}
