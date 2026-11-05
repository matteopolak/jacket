/** Match the existing non-Jai fence aliases; unknown languages remain escaped plain text. */
export function codeLanguage(language: string): 'shellscript' | 'powershell' | 'toml' | undefined {
  switch (language.toLowerCase()) {
    case 'sh': case 'bash': case 'shell': case 'zsh': case 'console': return 'shellscript';
    case 'powershell': case 'ps1': case 'pwsh': return 'powershell';
    case 'toml': return 'toml';
    default: return undefined;
  }
}

/** Highlighter-owned spans use the same live light/dark palette as compiler-classified Jai. */
export const codeTheme = {
  name: 'jacket-palette',
  type: 'dark' as const,
  colors: { 'editor.background': '#00000000', 'editor.foreground': 'var(--fg)' },
  tokenColors: [
    { scope: ['comment', 'punctuation.definition.comment'], settings: { foreground: 'var(--muted)', fontStyle: 'italic' } },
    { scope: ['keyword', 'storage', 'support.function', 'entity.name.function'], settings: { foreground: 'var(--kw)' } },
    { scope: ['string', 'constant.numeric', 'constant.language'], settings: { foreground: 'var(--str)' } },
    { scope: ['entity.name.type', 'support.type', 'support.class'], settings: { foreground: 'var(--type)' } },
  ],
};

/** Keep the existing pre element's CSS in charge of its background, wrapping and focus. */
export const codePresentation = [{
  pre(node: {properties: Record<string, unknown>}) {
    delete node.properties.style;
    delete node.properties.tabindex;
    delete node.properties.tabIndex;
    node.properties.class = 'code wrap';
  },
}];
