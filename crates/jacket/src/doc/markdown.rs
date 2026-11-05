//! Markdown validation for doc comments; semantic parsing belongs to `export::document`.
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};

/// The first line of each indented block, which cannot be run as a fenced Jai example.
pub fn indented_blocks(md: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut open = false;
    for event in Parser::new_ext(md, Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH) {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Indented)) => open = true,
            Event::End(TagEnd::CodeBlock) => open = false,
            Event::Text(t) if open => {
                out.push(t.lines().next().unwrap_or("").trim().to_owned());
                open = false;
            }
            _ => {}
        }
    }
    out
}


#[cfg(test)]
mod tests {
    use super::indented_blocks;

    #[test]
    fn indented_blocks_are_found_and_fenced_ones_are_not() {
        let md =
            "Text.\n\n    a := 1;\n    b := 2;\n\n```jai\nc := 3;\n```\n\n- item\n\n  continued\n";
        assert_eq!(indented_blocks(md), ["a := 1;"]);
    }
}
