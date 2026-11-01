// The Markdown subset sitegen understands, turned into HTML.
//
// Blocks: headings (`#` to `######`), paragraphs, fenced code, block quotes, bullet and numbered
// lists (nested by indenting), and horizontal rules. Inline: `code`, `*emphasis*`, `**strong**`,
// links and images. Raw HTML is not passed through: `<`, `>` and `&` are always escaped.
//
// `render_markdown` is the one entry point; the rest of this folder is its two halves.
// `blocks.jai` finds the structure line by line and `inline.jai` handles the text inside it.
// The module knows nothing about sites, so the tests use it on its own.

#import "Basic";
#import "String";

#load "escape.jai";
#load "inline.jai";
#load "blocks.jai";
