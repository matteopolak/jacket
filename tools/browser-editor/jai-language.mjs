// Stateful lexical coloring. Source always stays in CodeMirror text nodes.
import { StreamLanguage } from "@codemirror/language";
import { tags } from "@lezer/highlight";
const keywords = new Set("if else ifx for while break continue return defer using struct union enum operator cast remove inline no_inline interface push_context pop_context null true false new delete case switch then it_index it".split(" "));
const types = new Set("void bool int s8 s16 s32 s64 u8 u16 u32 u64 float float32 float64 string Type Any Code Context".split(" "));
const identifier = /^[_\p{L}][_\p{L}\p{N}]*/u;
function comment(stream, state) {
  while (!stream.eol()) {
    if (stream.match("/*")) state.commentDepth++;
    else if (stream.match("*/")) { if (--state.commentDepth === 0) break; }
    else stream.next();
  }
  return "comment";
}
function string(stream, state) {
  while (!stream.eol()) {
    const ch = stream.next();
    if (ch === "\\") stream.next();
    else if (ch === '"') { state.string = false; break; }
  }
  return "string";
}
export const jaiTokenizer = {
  startState: () => ({ commentDepth: 0, string: false, hereTag: null, depth: 0 }),
  copyState: state => ({ ...state }),
  token(stream, state) {
    if (state.hereTag) {
      if (stream.sol()) {
        const text = stream.string.trimStart();
        if (text.startsWith(state.hereTag) && !/[_\p{L}\p{N}]/u.test(text[state.hereTag.length] ?? "")) {
          stream.pos = stream.string.length - text.length + state.hereTag.length;
          state.hereTag = null;
          return "string";
        }
      }
      stream.skipToEnd(); return "string";
    }
    if (state.commentDepth) return comment(stream, state);
    if (state.string) return string(stream, state);
    if (stream.eatSpace()) return null;
    if (stream.match("//")) { stream.skipToEnd(); return "comment"; }
    if (stream.match("/*")) { state.commentDepth = 1; return comment(stream, state); }
    if (stream.match('"')) { state.string = true; return string(stream, state); }
    if (stream.match(/^#string\b/u)) {
      // Original lexer permits comma modifiers, including escaped characters.
      const header = stream.match(/^(?:\s*,\s*(?:\\\S|[_\p{L}][_\p{L}\p{N}]*))*\s+([_\p{L}][_\p{L}\p{N}]*)/u);
      if (header) { state.hereTag = header[1]; stream.skipToEnd(); }
      return "directive";
    }
    if (stream.match(/^#[_\p{L}][_\p{L}\p{N}]*/u)) return "directive";
    if (stream.match(/^@[_\p{L}][_\p{L}\p{N}]*/u)) return "note";
    if (stream.match(/^(?:0[xX][\da-fA-F_]+|0[bB][01_]+|0[hH][\da-fA-F_]+|(?:\d[\d_]*(?:\.(?!\.)[\d_]*)?|\.\d[\d_]*)(?:[eE][+-]?[\d_]+)?)/u)) return "number";
    const name = stream.match(identifier);
    if (name) {
      if (types.has(name[0])) return "typeName";
      if (keywords.has(name[0])) return "keyword";
      if (/^\s*::\s*(?:#type\s*)?\(/u.test(stream.string.slice(stream.pos))) return "procedureName";
      if (/^\s*::\s*(?:struct|union|enum|#type)\b/u.test(stream.string.slice(stream.pos))) return "typeName";
      return "variableName";
    }
    const ch = stream.next();
    if ("({[".includes(ch)) state.depth++;
    if (")}]".includes(ch)) state.depth = Math.max(0, state.depth - 1);
    return "{}()[],;".includes(ch) ? "punctuation" : "operator";
  },
  indent(state, textAfter, context) {
    if (state.commentDepth || state.string || state.hereTag) return null;
    return Math.max(0, state.depth - (/^\s*[}\])]/u.test(textAfter) ? 1 : 0)) * context.unit;
  },
  languageData: { commentTokens: { line: "//", block: { open: "/*", close: "*/" } }, closeBrackets: { brackets: ["(", "[", "{", '"'] } },
  tokenTable: { procedureName: tags.function(tags.variableName), typeName: tags.typeName, directive: tags.processingInstruction, note: tags.annotation },
};
export const jaiLanguage = StreamLanguage.define(jaiTokenizer);
