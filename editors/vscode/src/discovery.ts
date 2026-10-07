// Finding jailsp, jaic and jaifmt. Pure functions over an injectable file system and
// environment, so the unit tests can describe any machine.
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";
import { executableName, type Tool } from "./toolchain";

export interface Environment {
  platform: NodeJS.Platform;
  /** `PATH` (or `Path` on Windows). */
  pathVariable: string | undefined;
  /** `PATHEXT` on Windows. */
  pathExt?: string;
  home: string;
  isFile(file: string): boolean;
}

export function nodeEnvironment(): Environment {
  return {
    platform: process.platform,
    pathVariable: process.env.PATH ?? process.env.Path,
    pathExt: process.env.PATHEXT,
    home: os.homedir(),
    isFile: (file) => {
      try {
        return fs.statSync(file).isFile();
      } catch {
        return false;
      }
    },
  };
}

/** Where a tool was found, for the output channel and messages. */
export type Source = "setting" | "path" | "next to jaic" | "next to jailsp" | "toolchain";

export interface Found {
  path: string;
  source: Source;
}

/** Expands `~` and `${env:NAME}` in a configured path. */
export function expandPath(value: string, env: Environment, variables: NodeJS.ProcessEnv = process.env): string {
  let result = value.trim().replace(/\$\{env:([^}]+)\}/g, (_, name: string) => variables[name] ?? "");
  if (result === "~" || result.startsWith("~/") || result.startsWith("~\\")) {
    result = path.join(env.home, result.slice(1));
  }
  return result;
}

/** The first `name` on `PATH`, trying `PATHEXT` extensions on Windows. */
export function onPath(name: string, env: Environment): string | undefined {
  if (!env.pathVariable) return undefined;
  const separator = env.platform === "win32" ? ";" : ":";
  const join = env.platform === "win32" ? path.win32.join : path.posix.join;
  const extensions =
    env.platform === "win32" ? ["", ...(env.pathExt ?? ".EXE;.CMD;.BAT").split(";").map((e) => e.toLowerCase())] : [""];
  for (const dir of env.pathVariable.split(separator)) {
    if (!dir) continue;
    for (const extension of extensions) {
      const candidate = join(dir.replace(/^"(.*)"$/, "$1"), name + extension);
      if (env.isFile(candidate)) return candidate;
    }
  }
  return undefined;
}

/** `tool` in the same folder as `sibling` (an executable path). */
export function nextTo(sibling: string, tool: Tool, env: Environment): string | undefined {
  const dirname = env.platform === "win32" ? path.win32.dirname : path.posix.dirname;
  const join = env.platform === "win32" ? path.win32.join : path.posix.join;
  const candidate = join(dirname(sibling), executableName(tool, env.platform));
  return env.isFile(candidate) ? candidate : undefined;
}

export interface Settings {
  serverPath?: string;
  compilerPath?: string;
  formatterPath?: string;
}

/** A configured path that does not exist: an error, never a silent fallback. */
export class MissingConfiguredTool extends Error {
  constructor(
    readonly setting: string,
    readonly configured: string,
  ) {
    super(`${setting} is set to ${configured}, which does not exist`);
  }
}

function configured(value: string | undefined, setting: string, env: Environment): Found | undefined {
  if (!value?.trim()) return undefined;
  const expanded = expandPath(value, env);
  if (!env.isFile(expanded)) throw new MissingConfiguredTool(setting, expanded);
  return { path: expanded, source: "setting" };
}

/**
 * The compiler: `jai.compiler.path`, else `jaic` on PATH, else in the downloaded toolchain
 * (`toolchainDir`).
 */
export function findCompiler(settings: Settings, env: Environment, toolchainDir?: string): Found | undefined {
  const fromSetting = configured(settings.compilerPath, "jai.compiler.path", env);
  if (fromSetting) return fromSetting;
  const fromPath = onPath("jaic", env);
  if (fromPath) return { path: fromPath, source: "path" };
  return inToolchain("jaic", env, toolchainDir);
}

/**
 * The language server: `jai.server.path`, else `jailsp` on PATH, else next to the compiler from
 * `jai.compiler.path` or PATH, else the downloaded toolchain.
 */
export function findServer(settings: Settings, env: Environment, toolchainDir?: string): Found | undefined {
  const fromSetting = configured(settings.serverPath, "jai.server.path", env);
  if (fromSetting) return fromSetting;
  const fromPath = onPath("jailsp", env);
  if (fromPath) return { path: fromPath, source: "path" };
  const compiler = findCompiler(settings, env);
  const sibling = compiler && nextTo(compiler.path, "jailsp", env);
  if (sibling) return { path: sibling, source: "next to jaic" };
  return inToolchain("jailsp", env, toolchainDir);
}

/**
 * The formatter: `jai.formatter.path`, else `jaifmt` on PATH, else next to the compiler, else
 * next to the server, else the downloaded toolchain.
 */
export function findFormatter(settings: Settings, env: Environment, toolchainDir?: string): Found | undefined {
  const fromSetting = configured(settings.formatterPath, "jai.formatter.path", env);
  if (fromSetting) return fromSetting;
  const fromPath = onPath("jaifmt", env);
  if (fromPath) return { path: fromPath, source: "path" };
  const compiler = findCompiler(settings, env);
  const besideCompiler = compiler && nextTo(compiler.path, "jaifmt", env);
  if (besideCompiler) return { path: besideCompiler, source: "next to jaic" };
  const server = configured(settings.serverPath, "jai.server.path", env) ?? optional(onPath("jailsp", env));
  const besideServer = server && nextTo(server.path, "jaifmt", env);
  if (besideServer) return { path: besideServer, source: "next to jailsp" };
  return inToolchain("jaifmt", env, toolchainDir);
}

function optional(file: string | undefined): Found | undefined {
  return file ? { path: file, source: "path" } : undefined;
}

function inToolchain(tool: Tool, env: Environment, dir: string | undefined): Found | undefined {
  if (!dir) return undefined;
  const join = env.platform === "win32" ? path.win32.join : path.posix.join;
  const candidate = join(dir, executableName(tool, env.platform));
  return env.isFile(candidate) ? { path: candidate, source: "toolchain" } : undefined;
}
