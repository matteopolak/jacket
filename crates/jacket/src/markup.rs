//! Escaping text for HTML and XML, the one rule for the doc site, the coverage page and `JUnit`
//! reports.

/// `text` safe inside an HTML or XML element or a double-quoted attribute: `&`, `<`, `>` and `"`
/// become entities, and a control character other than tab and line breaks (which XML 1.0 does
/// not allow, and HTML does not show) becomes `?`.
///
/// ```ignore
/// assert_eq!(escape("a < \"b\"\u{7}"), "a &lt; &quot;b&quot;?");
/// ```
pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());

    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            c if c.is_ascii_control() && !matches!(c, '\t' | '\n' | '\r') => out.push('?'),
            c => out.push(c),
        }
    }

    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn markup_and_control_characters_are_escaped() {
        assert_eq!(
            super::escape("a < \"b\" & c > d\u{7}\u{7f}"),
            "a &lt; &quot;b&quot; &amp; c &gt; d??"
        );
        assert_eq!(super::escape("tab\tline\r\n'é'"), "tab\tline\r\n'é'");
    }
}
