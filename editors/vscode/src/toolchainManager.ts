// The VS Code side of the downloadable toolchain: asking before a download, progress, and
// remembering answers. The download itself is in toolchain.ts.
import * as fs from "node:fs";
import * as path from "node:path";
import * as vscode from "vscode";
import {
  type PinnedChecksums,
  RELEASES_URL,
  REPOSITORY,
  ToolchainError,
  ToolchainStore,
  assetFor,
  install,
  platformName,
  unsupportedMessage,
} from "./toolchain";

const DECLINED_KEY = "jai.toolchain.declinedVersion";
export type AutoDownload = "prompt" | "always" | "never";

export interface Installed {
  version: string;
  dir: string;
}

export class ToolchainManager {
  readonly version: string;
  readonly asset: string | undefined;
  readonly store: ToolchainStore;
  private downloading: Promise<Installed | undefined> | undefined;

  constructor(
    private readonly context: vscode.ExtensionContext,
    private readonly log: vscode.LogOutputChannel,
  ) {
    this.version = context.extension.packageJSON.version as string;
    this.asset = assetFor(process.platform, process.arch);
    this.store = new ToolchainStore(context.globalStorageUri.fsPath);
  }

  get autoDownload(): AutoDownload {
    return vscode.workspace.getConfiguration("jai").get<AutoDownload>("toolchain.autoDownload", "prompt");
  }

  /** Checksums pinned into this build of the extension by the release workflow. */
  pinned(): PinnedChecksums | undefined {
    try {
      const file = path.join(this.context.extensionPath, "toolchain-checksums.json");
      return JSON.parse(fs.readFileSync(file, "utf8")) as PinnedChecksums;
    } catch {
      return undefined;
    }
  }

  /** The installed toolchain: this extension's version if present, else the newest other one. */
  async installed(): Promise<Installed | undefined> {
    if (!this.asset) return undefined;
    if (this.store.isInstalled(this.version, this.asset)) {
      return { version: this.version, dir: this.store.binDir(this.version, this.asset) };
    }
    for (const version of await this.store.installedVersions()) {
      if (this.store.isInstalled(version, this.asset)) {
        return { version, dir: this.store.binDir(version, this.asset) };
      }
    }
    return undefined;
  }

  /**
   * Called when no usable jailsp was found (`installed` undefined) or only an older downloaded
   * one. Follows `jai.toolchain.autoDownload`: asks first unless it is `always`, never downloads
   * when it is `never`. Returns the new installation, or undefined when nothing was downloaded.
   */
  async offer(current: Installed | undefined): Promise<Installed | undefined> {
    if (!this.asset) {
      if (!current) this.showUnsupported();
      return undefined;
    }
    const mode = this.autoDownload;
    if (mode === "never") {
      if (!current) await this.showNotFound(false);
      return undefined;
    }
    if (mode === "prompt") {
      if (this.context.globalState.get<string>(DECLINED_KEY) === this.version) {
        if (!current) await this.showNotFound(true);
        return undefined;
      }
      const question = current
        ? `Download jaic ${this.version} from github.com/${REPOSITORY} to replace the downloaded ${current.version}?`
        : `Jai: no jailsp was found on PATH or in the settings. Download jaic ${this.version} (jaic, jailsp, jailint, jaifmt and the stdlib) from github.com/${REPOSITORY}?`;
      const choice = await vscode.window.showInformationMessage(question, "Download", "Cancel", "Don't Ask Again");
      if (choice === "Don't Ask Again") {
        await vscode.workspace.getConfiguration("jai").update("toolchain.autoDownload", "never", vscode.ConfigurationTarget.Global);
        return undefined;
      }
      if (choice !== "Download") {
        await this.context.globalState.update(DECLINED_KEY, this.version);
        return undefined;
      }
    }
    return this.download();
  }

  /** Downloads this extension's toolchain version now (the "Download Toolchain" command). */
  download(): Promise<Installed | undefined> {
    this.downloading ??= this.downloadOnce().finally(() => (this.downloading = undefined));
    return this.downloading;
  }

  private async downloadOnce(): Promise<Installed | undefined> {
    const asset = this.asset;
    if (!asset) {
      this.showUnsupported();
      return undefined;
    }
    const title = `Downloading jaic ${this.version} for ${platformName(process.platform, process.arch)}`;
    try {
      const dir = await vscode.window.withProgress(
        { location: vscode.ProgressLocation.Notification, title, cancellable: true },
        async (progress, token) => {
          const abort = new AbortController();
          token.onCancellationRequested(() => abort.abort());
          let reported = 0;
          return install({
            version: this.version,
            asset,
            store: this.store,
            fetcher: (url, init) => fetch(url, init),
            pinned: this.pinned(),
            log: (line) => this.log.info(line),
            signal: abort.signal,
            progress: (received, total) => {
              if (!total) {
                progress.report({ message: `${(received / 1048576).toFixed(1)} MiB` });
                return;
              }
              const percent = Math.floor((received / total) * 100);
              progress.report({
                increment: percent - reported,
                message: `${(received / 1048576).toFixed(1)} of ${(total / 1048576).toFixed(1)} MiB`,
              });
              reported = percent;
            },
          });
        },
      );
      await this.context.globalState.update(DECLINED_KEY, undefined);
      this.log.info(`installed jaic ${this.version} in ${dir}`);
      return { version: this.version, dir };
    } catch (error) {
      this.report(error);
      return undefined;
    }
  }

  private report(error: unknown): void {
    const message = error instanceof Error ? error.message : String(error);
    this.log.error(`toolchain download: ${message}`);
    if (error instanceof ToolchainError && error.kind === "cancelled") return;
    if (error instanceof ToolchainError && error.kind === "unsupported") {
      this.showUnsupported();
      return;
    }
    const retry = "Retry";
    const releases = "Open Releases";
    void vscode.window
      .showErrorMessage(`Jai: could not install the toolchain: ${message}`, retry, releases)
      .then((choice) => {
        if (choice === retry) void vscode.commands.executeCommand("jai.downloadToolchain");
        if (choice === releases) void vscode.env.openExternal(vscode.Uri.parse(RELEASES_URL));
      });
  }

  showUnsupported(): void {
    const message = unsupportedMessage(process.platform, process.arch);
    this.log.warn(message);
    void vscode.window.showWarningMessage(`Jai: ${message}`, "Open Settings").then((choice) => {
      if (choice) void vscode.commands.executeCommand("workbench.action.openSettings", "jai.server.path");
    });
  }

  async showNotFound(offerDownload: boolean): Promise<void> {
    const actions = offerDownload ? ["Download Toolchain", "Open Releases", "Open Settings"] : ["Open Releases", "Open Settings"];
    const choice = await vscode.window.showWarningMessage(
      "Jai: jailsp (the Jai language server) was not found, so there is no checking, completion or linting. " +
        `Install the toolchain from ${RELEASES_URL} and put it on PATH, or set jai.server.path.`,
      ...actions,
    );
    if (choice === "Download Toolchain") await vscode.commands.executeCommand("jai.downloadToolchain");
    if (choice === "Open Releases") await vscode.env.openExternal(vscode.Uri.parse(RELEASES_URL));
    if (choice === "Open Settings") await vscode.commands.executeCommand("workbench.action.openSettings", "@ext:matteopolak.jai");
  }
}
