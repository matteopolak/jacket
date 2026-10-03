import { Workspace } from "./workspace.mjs";
import { createEditor } from "./editor.bundle.mjs";
import { LanguageClient, pathFromUri, offsetAt } from "./lsp-client.mjs";
const $ = selector => document.querySelector(selector);
const result = $("#result"), run = $("#run"), cancel = $("#cancel");
const workspace = new Workspace('main :: () -> int {\n    // This source runs in the browser interpreter.\n    return 42;\n}\n');
const states = new Map(), diagnostics = new Map(), folders = new Map();
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
    workspace.edit(text); diagnostics.delete(workspace.selected.path.name); editor.diagnostics([], text); renderProblems();
    clearTimeout(syncTimer); syncTimer = setTimeout(() => language?.sync(workspace.documents), 120);
  },
});
function saveState() { states.set(workspace.selected.path.name, editor.view.state); }
function selectedChanged() {
  const selected = workspace.selected;
  editor.setState(states.get(selected.path.name) ?? editor.createState(selected.text));
  $("#selected-name").textContent = selected.path.name;
  $("#remove-file").disabled = !workspace.canRemoveSelected;
  editor.diagnostics(diagnostics.get(selected.path.name)?.diagnostics ?? [], selected.text);
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
  for (const [path, entry] of diagnostics) for (const diagnostic of entry.diagnostics) {
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
function connectRunWorker(worker) {
  worker.onmessage = ({ data }) => {
    if (data.type !== "run" || data.id !== job) return;
    result.textContent = data.error ?? `Exit code: ${data.result.exitCode}\nInterpreter steps: ${data.result.steps}`;
    if (data.consoleText) result.textContent = `${data.consoleText}\n${result.textContent}`;
    idle();
  };
  worker.onerror = event => { result.textContent = event.message || "Execution worker failed."; worker.terminate(); if (runWorker === worker) runWorker = undefined; idle(); };
  runWorker = worker;
}
run.onclick = async () => {
  let currentJob;
  try {
    const args = JSON.parse($("#arguments").value); if (!Array.isArray(args) || args.some(arg => typeof arg !== "string")) throw new Error("Arguments must be a JSON array of strings.");
    const fuel = Number($("#fuel").value); if (!Number.isInteger(fuel) || fuel < 0 || fuel > 0xffffffff) throw new Error("The execution step limit must be an integer from 0 to 4,294,967,295.");
    const snapshot = workspace.snapshot(); currentJob = ++job;
    run.disabled = true; cancel.disabled = false; outputPanel("output"); result.textContent = "Checking and running…";
    if (!runWorker) { pendingRun = new AbortController(); const initialized = await initWorker(pendingRun.signal); pendingRun = undefined; if (currentJob !== job) { initialized.worker.terminate(); return; } connectRunWorker(initialized.worker); }
    runWorker.postMessage({ type: "run", id: currentJob, source: snapshot.source, options: { arguments: args, fuel, files: snapshot.files } });
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
      try { const path = pathFromUri(params.uri); const document = workspace.documents.find(item => item.path === path); if (!document || document.version !== params.version) return; diagnostics.set(path, params); if (path === workspace.selected.path.name) editor.diagnostics(params.diagnostics, document.text); renderProblems(); } catch { /* Unadmitted URI cannot modify the workspace. */ }
    }, failure: message => { language = undefined; $("#language-status").textContent = "Jai · language service unavailable"; result.textContent = message; } });
    await language.initialize(); language.sync(workspace.documents); $("#language-status").textContent = "Jai · language service connected";
  }
  report("ready");
} catch (error) { compilerReady = false; runWorker?.terminate(); runWorker = undefined; languageWorker?.terminate(); language?.dispose(); language = undefined; idle(); $("#runtime-status").textContent = "Compiler unavailable"; result.textContent = error.message; report("error", error.message); }
