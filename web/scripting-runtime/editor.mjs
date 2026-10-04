import { Workspace } from "./workspace.mjs";
import { createEditor } from "./editor.bundle.mjs";
import { LanguageClient, pathFromUri, offsetAt } from "./lsp-client.mjs";
const $ = selector => document.querySelector(selector);
const result = $("#result"), run = $("#run"), cancel = $("#cancel");
const workspace = new Workspace('#import "Basic";\n\nmain :: () {\n    // This source runs in the browser interpreter.\n    print("Hello, %!\\n", 42);\n}\n');
const states = new Map(), diagnostics = new Map(), runDiagnostics = new Map(), folders = new Map();
const diagnosticsFor = path => [...(diagnostics.get(path)?.diagnostics ?? []), ...(runDiagnostics.get(path) ?? [])];
let language, languageWorker, runWorker, compilerReady = false, job = 0, syncTimer, pendingRun;
const revision = window.JAI_PLAYGROUND_BOOT?.revision;
const embedded = new URLSearchParams(location.search).get("embed") === "1";
function report(state, message) {
  if (embedded) parent.postMessage({ type: "jai-playground", state, revision: revision ?? "", ...(message ? { message } : {}) }, location.origin);
}
const editor = createEditor($("#source"), {
  text: workspace.selected.text,
  currentDocument: () => ({ path: workspace.selected.path.name, text: workspace.selected.text, version: workspace.selected.version }),
  service: () => { clearTimeout(syncTimer); language?.sync(workspace.documents); return language; },
  onCursor: (line, column) => { $("#editor-position").textContent = `Ln ${line}, Col ${column}`; },
  onChange: text => {
    workspace.edit(text); diagnostics.delete(workspace.selected.path.name); runDiagnostics.delete(workspace.selected.path.name); editor.diagnostics([], text); renderProblems();
    clearTimeout(syncTimer); syncTimer = setTimeout(() => language?.sync(workspace.documents), 120);
  },
});
function saveState() { states.set(workspace.selected.path.name, editor.view.state); }
function selectedChanged() {
  const selected = workspace.selected;
  editor.setState(states.get(selected.path.name) ?? editor.createState(selected.text));
  $("#selected-name").textContent = selected.path.name;
  $("#remove-file").disabled = !workspace.canRemoveSelected;
  editor.diagnostics(diagnosticsFor(selected.path.name), selected.text);
  renderFiles(); editor.focus();
}
function select(path) { saveState(); workspace.select(path); selectedChanged(); }
function renderFiles() {
  const build = node => {
    if (node.kind === "file") {
      const button = document.createElement("button"); button.type = "button"; button.className = "file-button"; button.title = node.path;
      if (node.path === workspace.selected.path.name) button.setAttribute("aria-current", "page");
      const icon = document.createElement("span"); icon.className = "file-symbol"; icon.textContent = "J"; icon.setAttribute("aria-hidden", "true");
      const label = document.createElement("span"); label.textContent = node.name;
      button.append(icon, label); button.onclick = () => select(node.path); return button;
    }
    const details = document.createElement("details"); details.open = folders.get(node.path) ?? true;
    const summary = document.createElement("summary"); const icon = document.createElement("span"); icon.className = "folder-icon"; icon.textContent = "▱"; icon.setAttribute("aria-hidden", "true"); const label = document.createElement("span"); label.textContent = node.name; summary.append(icon, label);
    const children = document.createElement("div"); children.className = "tree-children"; children.append(...node.children.map(build)); details.append(summary, children); details.ontoggle = () => folders.set(node.path, details.open); return details;
  };
  $("#files").replaceChildren(...workspace.tree.children.map(build));
}
function renderProblems() {
  const rows = [];
  for (const path of new Set([...diagnostics.keys(), ...runDiagnostics.keys()])) for (const diagnostic of diagnosticsFor(path)) {
    const row = document.createElement("button"); row.type = "button"; row.className = "problem-row";
    const severity = document.createElement("span"); severity.className = "problem-severity"; severity.textContent = diagnostic.severity === 2 ? "△" : "●";
    const message = document.createElement("span"); message.className = "problem-message"; message.textContent = diagnostic.message;
    const location = document.createElement("span"); location.className = "problem-location"; location.textContent = `${path}:${diagnostic.range.start.line + 1}:${diagnostic.range.start.character + 1}`;
    row.append(severity, message, location); row.onclick = () => { select(path); try { const from = offsetAt(workspace.selected.text, diagnostic.range.start); editor.view.dispatch({ selection: { anchor: from }, scrollIntoView: true }); } catch { /* Do not guess a malformed server position. */ } }; rows.push(row);
  }
  $("#problem-count").textContent = String(rows.length);
  if (!rows.length) { const empty = document.createElement("p"); empty.className = "muted"; empty.textContent = language ? "No diagnostics." : "Syntax diagnostics become available when the language service connects."; rows.push(empty); }
  $("#problems").replaceChildren(...rows);
}
$("#new-file").onsubmit = event => {
  event.preventDefault();
  try { saveState(); workspace.add($("#file-name").value); $("#file-name").value = ""; selectedChanged(); language?.sync(workspace.documents); }
  catch (error) { result.textContent = error.message; $("#file-name").focus(); }
};
$("#remove-file").onclick = () => { const path = workspace.selected.path.name; workspace.removeSelected(); states.delete(path); diagnostics.delete(path); selectedChanged(); renderProblems(); language?.sync(workspace.documents); };
$("#find").onclick = () => editor.find();
function outputPanel(name) { for (const key of ["output", "problems"]) { $("#" + key + "-tab").setAttribute("aria-selected", String(key === name)); $("#" + key + "-panel").hidden = key !== name; } }
$("#output-tab").onclick = () => outputPanel("output"); $("#problems-tab").onclick = () => outputPanel("problems");
function idle() { run.disabled = !compilerReady; cancel.disabled = true; }
function initWorker(signal) {
  const worker = new Worker(new URL("./worker.mjs", import.meta.url), { type: "module" });
  return new Promise((resolve, reject) => {
    const aborted = () => { clearTimeout(timer); worker.terminate(); reject(new DOMException("Initialization cancelled", "AbortError")); };
    const timer = setTimeout(() => { worker.terminate(); reject(new Error("Compiler initialization timed out.")); }, 20000);
    signal?.addEventListener("abort", aborted, { once: true });
    if (signal?.aborted) { aborted(); return; }
    const done = event => { if (event.data.type !== "init") return; clearTimeout(timer); signal?.removeEventListener("abort", aborted); worker.removeEventListener("message", done); event.data.error ? (worker.terminate(), reject(new Error(event.data.error))) : resolve({ worker, capabilities: event.data.capabilities }); };
    worker.addEventListener("message", done);
    worker.addEventListener("error", event => { clearTimeout(timer); signal?.removeEventListener("abort", aborted); worker.terminate(); reject(new Error(event.message || "Compiler worker failed.")); }, { once: true });
    worker.postMessage({ type: "init" });
  });
}
function lspDiagnostic(d) {
  const line = Math.max(0, d.line - 1), character = Math.max(0, d.column - 1);
  return { severity: d.severity === "warning" ? 2 : d.severity === "note" ? 3 : 1, message: d.message, range: { start: { line, character }, end: { line, character: character + 1 } } };
}
function showPlay(play) {
  runDiagnostics.clear();
  const byFile = new Map();
  for (const d of play.diagnostics) if (d.file && d.line > 0 && workspace.documents.some(item => item.path === d.file)) byFile.set(d.file, [...(byFile.get(d.file) ?? []), lspDiagnostic(d)]);
  for (const [path, list] of byFile) runDiagnostics.set(path, list);
  const selected = workspace.selected;
  editor.diagnostics(diagnosticsFor(selected.path.name), selected.text);
  renderProblems();
  const parts = [];
  if (play.stdout) parts.push(play.stdout);
  if (play.stderr) parts.push(`[stderr]\n${play.stderr}`);
  if (play.rendered) parts.push(play.rendered);
  else for (const d of play.diagnostics) if (!d.file) parts.push(`${d.severity}: ${d.message}`);
  parts.push(play.exitCode === null ? "Compilation failed." : `Exit code: ${play.exitCode}`);
  result.textContent = parts.join(parts.length > 1 && !(play.stdout?.endsWith("\n") ?? true) ? "\n" : "");
  if (play.diagnostics.length) outputPanel(play.exitCode === null ? "problems" : "output");
}
function connectRunWorker(worker) {
  worker.onmessage = ({ data }) => {
    if (data.type !== "run" || data.id !== job) return;
    if (data.play) showPlay(data.play);
    else result.textContent = data.error ?? "The compiler returned no result.";
    idle();
  };
  worker.onerror = event => { result.textContent = event.message || "Execution worker failed."; worker.terminate(); if (runWorker === worker) runWorker = undefined; idle(); };
  runWorker = worker;
}
run.onclick = async () => {
  let currentJob;
  try {
    const snapshot = workspace.snapshot(); currentJob = ++job;
    run.disabled = true; cancel.disabled = false; outputPanel("output"); result.textContent = "Checking and running…";
    if (!runWorker) { pendingRun = new AbortController(); const initialized = await initWorker(pendingRun.signal); pendingRun = undefined; if (currentJob !== job) { initialized.worker.terminate(); return; } connectRunWorker(initialized.worker); }
    runWorker.postMessage({ type: "run", id: currentJob, source: snapshot.source, options: { files: snapshot.files } });
  } catch (error) { if (currentJob !== undefined && currentJob !== job) return; result.textContent = error.message; idle(); }
};
cancel.onclick = () => { ++job; pendingRun?.abort(); pendingRun = undefined; runWorker?.terminate(); runWorker = undefined; result.textContent = "Cancelled."; idle(); };
document.addEventListener("keydown", event => { if ((event.ctrlKey || event.metaKey) && event.key === "Enter" && !run.disabled) { event.preventDefault(); run.click(); } });
window.addEventListener("pagehide", () => { clearTimeout(syncTimer); pendingRun?.abort(); language?.dispose(); languageWorker?.terminate(); runWorker?.terminate(); editor.destroy(); });
renderFiles(); renderProblems();
try {
  if (revision) $("#revision").textContent = revision.slice(0, 12);
  const initialized = await initWorker(); connectRunWorker(initialized.worker); compilerReady = true; $("#runtime-status").textContent = "Compiler ready"; idle();
  if (initialized.capabilities?.languageServer) {
    const server = await initWorker(); languageWorker = server.worker;
    language = new LanguageClient(languageWorker, { diagnostics: params => {
      try { const path = pathFromUri(params.uri); const document = workspace.documents.find(item => item.path === path); if (!document || document.version !== params.version) return; diagnostics.set(path, params); if (path === workspace.selected.path.name) editor.diagnostics(diagnosticsFor(path), document.text); renderProblems(); } catch { /* Unadmitted URI cannot modify the workspace. */ }
    }, failure: message => { language = undefined; $("#language-status").textContent = "Jai · language service unavailable"; result.textContent = message; } });
    await language.initialize(); language.sync(workspace.documents); $("#language-status").textContent = "Jai · language service connected";
  }
  report("ready");
} catch (error) { compilerReady = false; runWorker?.terminate(); runWorker = undefined; languageWorker?.terminate(); language?.dispose(); language = undefined; idle(); $("#runtime-status").textContent = "Compiler unavailable"; result.textContent = error.message; report("error", error.message); }
