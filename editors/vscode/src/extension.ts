import * as os from "node:os";
import * as path from "node:path";
import * as vscode from "vscode";
import { LanguageClient, type LanguageClientOptions, type ServerOptions, TransportKind } from "vscode-languageclient/node";
import { type JaicCommand, runJaic } from "./commands";
import { MissingConfiguredTool, type Settings, findCompiler, findFormatter, findServer, nodeEnvironment } from "./discovery";
import { EXPANSION_SCHEME, ExpansionDocuments } from "./expansion";
import { CONFIG_FILE, findConfig, ignoreGlobs, isIgnored, minimalReplacement, runJaifmt } from "./jaifmt";
import { ToolchainManager, type Installed } from "./toolchainManager";

/** Settings files jailsp reads as open documents, so unsaved edits apply at once. */
const SETTINGS_FILES = ["jailint.toml", "jai.toml"];
/** The extension's ID up to 0.4.1, before the Marketplace's name clash forced a new one. */
const OLD_EXTENSION_ID = "matteopolak.jai";

let client: LanguageClient | undefined;
let log: vscode.LogOutputChannel;
let serverOutput: vscode.LogOutputChannel;
let toolchain: ToolchainManager;
let installed: Installed | undefined;
let starting: Promise<void> | undefined;

function settings(): Settings {
  const config = vscode.workspace.getConfiguration("jai");
  return {
    serverPath: config.get<string>("server.path"),
    compilerPath: config.get<string>("compiler.path"),
    formatterPath: config.get<string>("formatter.path"),
  };
}

/** Environment for jailsp and jaic: `JAIC_STDLIB` from `jai.stdlib.path`. */
function toolEnvironment(): Record<string, string> {
  const env: Record<string, string> = {};
  for (const [key, value] of Object.entries(process.env)) if (value !== undefined) env[key] = value;
  const stdlib = vscode.workspace.getConfiguration("jai").get<string>("stdlib.path")?.trim();
  if (stdlib) env.JAIC_STDLIB = stdlib.replace(/^~(?=$|[\\/])/, os.homedir());
  return env;
}

function locate<T>(find: () => T): T | undefined {
  try {
    return find();
  } catch (error) {
    if (error instanceof MissingConfiguredTool) {
      log.error(error.message);
      void vscode.window.showErrorMessage(`Jai: ${error.message}.`, "Open Settings").then((choice) => {
        if (choice) void vscode.commands.executeCommand("workbench.action.openSettings", error.setting);
      });
      return undefined;
    }
    throw error;
  }
}

export async function activate(context: vscode.ExtensionContext): Promise<{ client: () => LanguageClient | undefined }> {
  // Both register the same command IDs and whichever activates second fails, so this one
  // stands aside until the old one is uninstalled.
  if (vscode.extensions.getExtension(OLD_EXTENSION_ID)) {
    void offerToUninstallOldExtension();
    return { client: () => undefined };
  }
  log = vscode.window.createOutputChannel("Jai", { log: true });
  serverOutput = vscode.window.createOutputChannel("Jai Language Server", { log: true });
  toolchain = new ToolchainManager(context, log);
  const expansions = new ExpansionDocuments(() => client);

  context.subscriptions.push(
    log,
    serverOutput,
    vscode.workspace.registerTextDocumentContentProvider(EXPANSION_SCHEME, expansions),
    vscode.commands.registerCommand("jai.restartServer", () => restart(expansions)),
    vscode.commands.registerCommand("jai.showOutput", () => log.show()),
    vscode.commands.registerCommand("jai.downloadToolchain", async () => {
      const result = await toolchain.download();
      if (result) {
        installed = result;
        const server = locate(() => findServer(settings(), nodeEnvironment(), installed?.dir));
        if (server && server.source !== "toolchain") {
          void vscode.window.showInformationMessage(
            `Jai: downloaded jaic ${result.version}. The ${server.source === "setting" ? "configured" : "PATH"} jailsp (${server.path}) still takes precedence.`,
          );
        }
        await restart(expansions);
      }
    }),
    ...(["run", "build", "check"] as JaicCommand[]).map((command) =>
      vscode.commands.registerCommand(`jai.${command}File`, async () => {
        const compiler = locate(() => findCompiler(settings(), nodeEnvironment(), installed?.dir));
        await runJaic(command, compiler?.path, toolEnvironment());
      }),
    ),
    vscode.languages.registerDocumentFormattingEditProvider({ language: "jai" }, { provideDocumentFormattingEdits: formatDocument }),
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (["jai.server.path", "jai.compiler.path", "jai.stdlib.path", "jai.trace.server"].some((s) => event.affectsConfiguration(s))) {
        void restart(expansions);
      }
    }),
    ...syncSettingsFiles(),
    { dispose: () => void client?.stop() },
  );

  installed = await toolchain.installed();
  void start(expansions);
  return { client: () => client };
}

async function offerToUninstallOldExtension(): Promise<void> {
  const uninstall = `Uninstall ${OLD_EXTENSION_ID}`;
  const choice = await vscode.window.showWarningMessage(
    `Jai Toolchain is now matteopolak.jai-toolchain, but the old ${OLD_EXTENSION_ID} is still installed. Both provide the same commands, so this one stays off until the old one is uninstalled.`,
    uninstall,
  );
  if (choice !== uninstall) return;
  await vscode.commands.executeCommand("workbench.extensions.uninstallExtension", OLD_EXTENSION_ID);
  const reload = "Reload Window";
  if ((await vscode.window.showInformationMessage(`Uninstalled ${OLD_EXTENSION_ID}. Reload the window to start Jai Toolchain.`, reload)) === reload) {
    await vscode.commands.executeCommand("workbench.action.reloadWindow");
  }
}

export async function deactivate(): Promise<void> {
  await client?.stop();
  client = undefined;
}

async function restart(expansions: ExpansionDocuments): Promise<void> {
  await starting;
  const old = client;
  client = undefined;
  if (old) {
    try {
      await old.stop(5000);
    } catch (error) {
      log.warn(`stopping jailsp: ${String(error)}`);
    }
    await old.dispose();
  }
  await start(expansions);
}

function start(expansions: ExpansionDocuments): Promise<void> {
  starting = startServer(expansions).finally(() => (starting = undefined));
  return starting;
}

async function startServer(expansions: ExpansionDocuments): Promise<void> {
  const env = nodeEnvironment();
  let misconfigured = false;
  let server = locate(() => {
    try {
      return findServer(settings(), env, installed?.dir);
    } catch (error) {
      misconfigured = error instanceof MissingConfiguredTool;
      throw error;
    }
  });
  if (server === undefined && !misconfigured) {
    // Nothing configured or on PATH: offer the pinned toolchain release.
    const result = await toolchain.offer(installed);
    if (result) installed = result;
    server = locate(() => findServer(settings(), env, installed?.dir));
  } else if (server?.source === "toolchain" && installed && installed.version !== toolchain.version) {
    // An older download: offer this extension's version; keep using the old one meanwhile.
    void toolchain.offer(installed).then(async (result) => {
      if (result) {
        installed = result;
        await restart(expansions);
      }
    });
  }
  if (!server) {
    log.warn("jailsp not found; the language server is not running");
    return;
  }
  log.info(`starting ${server.path} (${server.source})`);

  const serverOptions: ServerOptions = {
    command: server.path,
    transport: TransportKind.stdio,
    options: { env: toolEnvironment() },
  };
  const clientOptions: LanguageClientOptions = {
    // jailsp needs `file://` URIs: unsaved and virtual documents are not sent.
    documentSelector: [{ scheme: "file", language: "jai" }],
    outputChannel: serverOutput,
    traceOutputChannel: serverOutput,
    // The stdlib comes from `JAIC_STDLIB`, the project's entry files and module folders from
    // the nearest jai.toml (else inferred), lint levels from the nearest jailint.toml. Editor
    // settings go as initialization options and, when they change, as
    // `workspace/didChangeConfiguration` with the whole `jai` section.
    initializationOptions: () => serverSettings(),
    synchronize: {
      configurationSection: "jai",
      // Auto-import completion indexes the project's files: tell jailsp when they change.
      fileEvents: vscode.workspace.createFileSystemWatcher("**/{*.jai,jai.toml}"),
    },
    middleware: {
      executeCommand: async (command, args, next) => expansions.handle(command, await next(command, args)),
    },
  };
  const next = new LanguageClient("jai", "Jai Language Server", serverOptions, clientOptions);
  client = next;
  try {
    await next.start();
    openSettingsFiles(next);
  } catch (error) {
    log.error(`jailsp did not start: ${String(error)}`);
    void vscode.window.showErrorMessage(`Jai: the language server (${server.path}) did not start: ${String(error)}`, "Show Output").then((choice) => {
      if (choice) serverOutput.show();
    });
  }
}

// --- jailint.toml and jai.toml ------------------------------------------------------------
// jailsp reads lint settings from the nearest jailint.toml and project settings from the
// nearest jai.toml on disk, and also accepts either as an open document, so unsaved edits
// apply at once. Only the text is synchronised: the document selector stays Jai-only, so no
// other request is sent for the TOML files.

function isSettingsFile(document: vscode.TextDocument): boolean {
  return document.uri.scheme === "file" && SETTINGS_FILES.includes(path.basename(document.uri.fsPath));
}

/** The settings jailsp reads, as `initializationOptions` (the shape of the `jai` section). */
function serverSettings(): { completion: { autoImport: boolean } } {
  return { completion: { autoImport: vscode.workspace.getConfiguration("jai").get<boolean>("completion.autoImport", true) } };
}

function openSettingsFiles(target: LanguageClient): void {
  for (const document of vscode.workspace.textDocuments) if (isSettingsFile(document)) didOpen(target, document);
}

function didOpen(target: LanguageClient, document: vscode.TextDocument): void {
  void target.sendNotification("textDocument/didOpen", {
    textDocument: { uri: document.uri.toString(), languageId: "toml", version: document.version, text: document.getText() },
  });
}

function runningClient(): LanguageClient | undefined {
  return client?.isRunning() ? client : undefined;
}

function syncSettingsFiles(): vscode.Disposable[] {
  return [
    vscode.workspace.onDidOpenTextDocument((document) => {
      const target = runningClient();
      if (target && isSettingsFile(document)) didOpen(target, document);
    }),
    vscode.workspace.onDidChangeTextDocument((event) => {
      const target = runningClient();
      if (!target || !isSettingsFile(event.document) || event.contentChanges.length === 0) return;
      void target.sendNotification("textDocument/didChange", {
        textDocument: { uri: event.document.uri.toString(), version: event.document.version },
        contentChanges: [{ text: event.document.getText() }],
      });
    }),
    vscode.workspace.onDidCloseTextDocument((document) => {
      const target = runningClient();
      if (target && isSettingsFile(document)) {
        void target.sendNotification("textDocument/didClose", { textDocument: { uri: document.uri.toString() } });
      }
    }),
  ];
}

// --- Formatting -----------------------------------------------------------------------------
// jailsp has no textDocument/formatting, so the extension runs `jaifmt --stdin` itself.

async function formatDocument(document: vscode.TextDocument, _options: vscode.FormattingOptions, token: vscode.CancellationToken): Promise<vscode.TextEdit[]> {
  const formatter = locate(() => findFormatter(settings(), nodeEnvironment(), installed?.dir));
  if (!formatter) {
    void vscode.window.showWarningMessage("Jai: jaifmt was not found. Set jai.formatter.path, put jaifmt on PATH, or run \"Jai: Download Toolchain\".");
    return [];
  }
  const fileDir =
    document.uri.scheme === "file"
      ? path.dirname(document.uri.fsPath)
      : (vscode.workspace.workspaceFolders?.[0]?.uri.fsPath ?? os.homedir());
  if (document.uri.scheme === "file") {
    const config = findConfig(fileDir);
    if (config) {
      try {
        const text = Buffer.from(await vscode.workspace.fs.readFile(vscode.Uri.file(config))).toString("utf8");
        if (isIgnored(config, ignoreGlobs(text), document.uri.fsPath)) {
          log.info(`not formatting ${document.uri.fsPath}: ignored by ${config}`);
          return [];
        }
      } catch (error) {
        log.warn(`reading ${config}: ${String(error)}`);
      }
    }
  }
  const before = document.getText();
  let result;
  try {
    result = await runJaifmt(formatter.path, before, fileDir, token);
  } catch (error) {
    log.error(`running ${formatter.path}: ${String(error)}`);
    void vscode.window.showErrorMessage(`Jai: could not run jaifmt (${formatter.path}): ${String(error)}`);
    return [];
  }
  if (token.isCancellationRequested) return [];
  if (!result.ok) {
    // Unbalanced brackets or text that does not lex: jaifmt leaves it alone and says why.
    const reason = result.stderr.trim().split(/\r?\n/).join(" ") || `exit status ${result.code}`;
    log.warn(`jaifmt: ${reason}`);
    vscode.window.setStatusBarMessage(`$(warning) jaifmt: ${reason.replace(/^(jaifmt: )?(<stdin>:)?/, "")}`, 8000);
    if (/jaifmt\.toml/.test(reason)) void vscode.window.showErrorMessage(`Jai: ${reason} (${CONFIG_FILE})`);
    return [];
  }
  const change = minimalReplacement(before, result.stdout);
  if (!change) return [];
  const range = new vscode.Range(document.positionAt(change.start), document.positionAt(change.end));
  return [vscode.TextEdit.replace(range, change.text)];
}
