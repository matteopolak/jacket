// Code a build metaprogram generates (`add_build_string`, `#import,string`) has no file. `jailsp`
// points definitions and related notes at `jai-generated:` URIs and serves their text through
// `jai/source`; they open as read-only documents.
import * as vscode from "vscode";
import type { LanguageClient } from "vscode-languageclient/node";

export const GENERATED_SCHEME = "jai-generated";

export class GeneratedDocuments implements vscode.TextDocumentContentProvider {
  constructor(private readonly client: () => LanguageClient | undefined) {}

  async provideTextDocumentContent(uri: vscode.Uri): Promise<string> {
    const client = this.client();
    if (client?.isRunning()) {
      const text = await client.sendRequest<string | null>("jai/source", { uri: uri.toString() }).catch(() => null);
      if (typeof text === "string") return text;
    }
    return "// The generated code is no longer available: the language server is not running.\n";
  }

  /** Gives the document the Jai language once it opens. */
  static async onOpen(document: vscode.TextDocument): Promise<void> {
    if (document.uri.scheme === GENERATED_SCHEME && document.languageId !== "jai") {
      await vscode.languages.setTextDocumentLanguage(document, "jai");
    }
  }
}
