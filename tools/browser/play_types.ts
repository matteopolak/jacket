// Types for the browser compiler (crates/jai-wasm/js/engine.mjs), which ships as plain JavaScript,
// and for the playground case catalogue (tools/browser/playground_policy.py). Import them with
// `import type`.

export interface Diagnostic {
  severity: string;
  code?: string;
  message: string;
}

/** Workspace files by relative path, as text or bytes. */
export type PlayFiles = Record<string, string | Uint8Array>;

export interface PlayOptions {
  budget?: number;
  args?: string[];
  stdin?: string;
  libraryRoot?: string;
}

/** What `engine.play` returns, as far as the checks read it. */
export interface PlayResult {
  exitCode?: number | null;
  diagnostics?: Diagnostic[];
  stdout?: string;
  stderr?: string;
  rendered?: string;
  output?: { stream: 'stdout' | 'stderr'; text: string }[];
  failurePhase?: string | null;
}

/** The program's memory as `engine.mjs` hands it to host functions. */
export interface HostMemory {
  readonly buffer: ArrayBuffer;
  alloc(size: number): number;
  free(ptr: number, size: number): void;
}

/** A host function the page supplies; `args` are the raw wasm arguments. */
export type HostFunction = (args: bigint[], memory: HostMemory) => unknown;

export interface EngineOptions {
  host?: { functions?: Record<string, HostFunction> };
  assets?: { base: string; limit?: number };
}

export interface LspMessage {
  id?: number;
  method?: string;
  error?: unknown;
  result?: unknown;
  params?: { uri?: string; version?: number; diagnostics?: Diagnostic[] };
}

export interface Engine {
  play(files: PlayFiles, main: string, options?: PlayOptions): Required<PlayResult>;
  playAsync(files: PlayFiles, main: string, options?: PlayOptions): Promise<Required<PlayResult>>;
  jspi: boolean;
  lsp?(message: unknown): LspMessage[];
}

/** A staged `engine.mjs`, imported by path. */
export interface EngineModule {
  createEngine(wasm: Uint8Array, options?: EngineOptions): Promise<Engine>;
}

/** The byte counts of one playground request. */
export interface RequestSize {
  fileCount: number;
  pathBytes: number;
  sourceBytes: number;
  mainBytes: number;
  totalBytes: number;
}

/** One expected-result record from the playground catalogue. */
export interface CatalogueCase {
  id: string;
  path: string;
  mode?: string;
  diagnostic_contains?: string;
  error_contains?: string;
  exit_code?: number;
  stdout?: string;
  stderr?: string;
  positive_baseline?: boolean;
  positive_baseline_id?: string;
  browser_unavailable_reason?: string | null;
}

/** What a worker sends back: a play result, or why the request failed. */
export interface CaseResult extends PlayResult {
  requestFailure?: string;
  compilerCrashed?: boolean;
  request?: RequestSize;
}

/** One entry of tests/corpus/examples.json. */
export interface ExampleCase {
  id: string;
  main: string;
  directory: string;
  budget?: number;
  bundle?: boolean;
  stdout_contains: string[];
  stdout_excludes?: string[];
}
