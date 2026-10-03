import { EditorState, Compartment } from "@codemirror/state";
import { EditorView, keymap, lineNumbers, highlightActiveLineGutter, highlightSpecialChars, drawSelection, rectangularSelection, crosshairCursor, highlightActiveLine, hoverTooltip } from "@codemirror/view";
import { defaultKeymap, historyKeymap, history, indentWithTab } from "@codemirror/commands";
import { indentOnInput, bracketMatching, foldGutter, foldKeymap, syntaxHighlighting, HighlightStyle, indentUnit } from "@codemirror/language";
import { closeBrackets, closeBracketsKeymap, autocompletion, completionKeymap } from "@codemirror/autocomplete";
import { searchKeymap, highlightSelectionMatches, openSearchPanel } from "@codemirror/search";
import { lintGutter, setDiagnostics } from "@codemirror/lint";
import { tags } from "@lezer/highlight";
import { jaiLanguage } from "./jai-language.mjs";
import { documentUri, positionAt, offsetAt } from "../../web/scripting-runtime/lsp-client.mjs";
const colors = HighlightStyle.define([
  { tag: tags.keyword, color: "#c4a1ff" }, { tag: [tags.typeName, tags.className], color: "#83d5cc" },
  { tag: tags.function(tags.variableName), color: "#8bbcff" }, { tag: tags.variableName, color: "#d8e1ee" },
  { tag: [tags.string, tags.character], color: "#b8d990" }, { tag: tags.comment, color: "#8093a9", fontStyle: "italic" },
  { tag: tags.number, color: "#efbd8b" }, { tag: [tags.processingInstruction, tags.annotation], color: "#e8c779" },
  { tag: [tags.operator, tags.punctuation], color: "#aabdd3" },
]);
const theme = EditorView.theme({ "&": { height: "100%", fontSize: "14px", color: "#d8e1ee", backgroundColor: "#111a26" }, ".cm-scroller": { overflow: "auto", fontFamily: "'SFMono-Regular', Consolas, 'Liberation Mono', monospace", lineHeight: "1.65" }, ".cm-content": { padding: "16px 0", caretColor: "#8bbcff" }, ".cm-gutters": { backgroundColor: "#111a26", color: "#6f839b", border: "none", paddingRight: "10px" }, ".cm-activeLine,.cm-activeLineGutter": { backgroundColor: "#192639" }, "&.cm-focused .cm-selectionBackground, .cm-selectionBackground": { backgroundColor: "#294568" }, ".cm-matchingBracket": { backgroundColor: "#385374", color: "#fff", outline: "1px solid #7396bd" }, ".cm-tooltip": { backgroundColor: "#1a293b", color: "#e0e9f5", border: "1px solid #40516b" }, ".cm-searchMatch": { backgroundColor: "#685329" }, ".cm-panels": { backgroundColor: "#1a293b", color: "#e0e9f5" } }, { dark: true });
function textContent(value) {
  if (typeof value === "string") return value;
  if (Array.isArray(value)) return value.map(textContent).join("\n\n");
  return typeof value?.value === "string" ? value.value : "";
}
export function createEditor(parent, { text, onChange, onCursor, currentDocument, service }) {
  const editable = new Compartment();
  const completions = async context => {
    const client = service(); if (!client) return null;
    const document = currentDocument(); const version = document.version;
    const controller = new AbortController(); context.addEventListener("abort", () => controller.abort(), { onDocChange: true });
    const word = context.matchBefore(/[_\p{L}\p{N}#]*/u);
    if (!context.explicit && !word?.text) return null;
    try {
      const response = await client.request("textDocument/completion", { textDocument: { uri: documentUri(document.path) }, position: positionAt(context.state.doc.toString(), context.pos) }, controller.signal);
      if (currentDocument().path !== document.path || currentDocument().version !== version) return null;
      const items = Array.isArray(response) ? response : response?.items ?? [];
      return { from: word?.from ?? context.pos, options: items.filter(item => item.insertTextFormat !== 2).map(item => ({ label: item.label, detail: item.detail, info: textContent(item.documentation), type: item.kind === 3 ? "function" : item.kind === 7 ? "class" : item.kind === 14 ? "keyword" : "variable", apply: item.insertText ?? item.label })) };
    } catch { return null; }
  };
  const hover = hoverTooltip(async (view, position) => {
    const client = service(); if (!client) return null;
    const document = currentDocument(); const version = document.version;
    try {
      const result = await client.request("textDocument/hover", { textDocument: { uri: documentUri(document.path) }, position: positionAt(view.state.doc.toString(), position) });
      if (!result || currentDocument().path !== document.path || currentDocument().version !== version) return null;
      const text = textContent(result.contents); if (!text) return null;
      return { pos: result.range ? offsetAt(document.text, result.range.start) : position, end: result.range ? offsetAt(document.text, result.range.end) : undefined, create() { const dom = documentNode("div"); dom.className = "jai-hover"; dom.textContent = text; return { dom }; } };
    } catch { return null; }
  });
  function state(doc) { return EditorState.create({ doc, extensions: [EditorState.lineSeparator.of("\n"), jaiLanguage, syntaxHighlighting(colors), theme, lineNumbers(), highlightActiveLineGutter(), highlightSpecialChars(), history(), drawSelection(), EditorState.allowMultipleSelections.of(true), indentOnInput(), indentUnit.of("    "), bracketMatching(), closeBrackets(), foldGutter(), rectangularSelection(), crosshairCursor(), highlightActiveLine(), highlightSelectionMatches(), lintGutter(), autocompletion({ override: [completions] }), hover, editable.of(EditorView.editable.of(true)), keymap.of([...closeBracketsKeymap, ...defaultKeymap, ...historyKeymap, ...searchKeymap, ...foldKeymap, ...completionKeymap, indentWithTab]), EditorView.contentAttributes.of({ "aria-label": "Jai source editor", spellcheck: "false" }), EditorView.updateListener.of(update => { if (update.docChanged) onChange(update.state.doc.toString()); if (update.docChanged || update.selectionSet) { const cursor = update.state.selection.main.head; const line = update.state.doc.lineAt(cursor); onCursor(line.number, cursor - line.from + 1); } })] }); }
  const view = new EditorView({ state: state(text), parent });
  return { view, createState: state, setState: value => { view.setState(value); const cursor = view.state.selection.main.head; const line = view.state.doc.lineAt(cursor); onCursor(line.number, cursor - line.from + 1); }, diagnostics(values, documentText) { const checked = []; for (const value of values) { try { checked.push({ from: offsetAt(documentText, value.range.start), to: offsetAt(documentText, value.range.end), severity: value.severity === 2 ? "warning" : value.severity === 3 || value.severity === 4 ? "info" : "error", message: String(value.message), source: value.source }); } catch { /* Invalid server ranges never become guessed editor positions. */ } } view.dispatch(setDiagnostics(view.state, checked)); }, find: () => openSearchPanel(view), focus: () => view.focus(), destroy: () => view.destroy() };
}
function documentNode(tag) { return globalThis.document.createElement(tag); }
