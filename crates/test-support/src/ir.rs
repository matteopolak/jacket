//! Assertions over textual LLVM IR (what `emit_ir` writes).
//!
//! They fail with the whole IR in the message, so a failing test shows what was emitted.

/// The text of the function `@name`, from its `define` line to the closing `}`.
#[must_use]
pub fn function<'a>(ir: &'a str, name: &str) -> Option<&'a str> {
    let needle = format!("@{name}(");
    let start = ir
        .match_indices(&needle)
        .map(|(at, _)| ir[..at].rfind('\n').map_or(0, |n| n + 1))
        .find(|&line| ir[line..].starts_with("define "))?;
    let end = ir[start..].find("\n}").map_or(ir.len(), |n| start + n + 2);
    Some(&ir[start..end])
}

/// [`function`], panicking with the whole IR when there is no such function.
///
/// # Panics
/// When `ir` has no function `@name`.
#[must_use]
pub fn body<'a>(ir: &'a str, name: &str) -> &'a str {
    function(ir, name).unwrap_or_else(|| panic!("no function @{name} in\n{ir}"))
}

/// Whether some line of `ir` contains every one of `parts`.
#[must_use]
pub fn has_line(ir: &str, parts: &[&str]) -> bool {
    ir.lines()
        .any(|line| parts.iter().all(|part| line.contains(part)))
}

/// Requires `ir` to contain `text`.
pub fn assert_contains(ir: &str, text: &str) {
    assert!(ir.contains(text), "the IR lacks {text:?}:\n{ir}");
}

/// Requires `ir` not to contain `text`.
pub fn assert_lacks(ir: &str, text: &str) {
    assert!(!ir.contains(text), "the IR has {text:?}:\n{ir}");
}

/// Requires a line of `ir` that contains every one of `parts`.
pub fn assert_line(ir: &str, parts: &[&str]) {
    assert!(
        has_line(ir, parts),
        "no IR line has all of {parts:?}:\n{ir}"
    );
}

/// Requires the body of `@name` to contain `text`.
pub fn assert_function_contains(ir: &str, name: &str, text: &str) {
    let body = function(ir, name).unwrap_or_else(|| panic!("no function @{name}:\n{ir}"));
    assert!(body.contains(text), "@{name} lacks {text:?}:\n{body}");
}

#[cfg(test)]
mod tests {
    use super::*;

    const IR: &str = "declare void @ext()\n\ndefine i8 @flag() {\nentry:\n  ret i8 0\n}\n\ndefine i32 @main() {\n  call void @flag()\n  ret i32 1\n}\n";

    #[test]
    fn extracts_a_function() {
        assert_eq!(
            function(IR, "flag"),
            Some("define i8 @flag() {\nentry:\n  ret i8 0\n}")
        );
        assert!(function(IR, "main").unwrap().contains("ret i32 1"));
        assert_eq!(function(IR, "ext"), None);
        assert_eq!(function(IR, "absent"), None);
    }

    #[test]
    fn lines_and_text() {
        assert!(has_line(IR, &["call", "@flag"]));
        assert!(!has_line(IR, &["ret i8", "ret i32"]));
        assert_contains(IR, "ret i8 0");
        assert_lacks(IR, "unreachable");
        assert_function_contains(IR, "flag", "ret i8 0");
    }

    #[test]
    #[should_panic(expected = "lacks")]
    fn a_missing_function_text_fails() {
        assert_function_contains(IR, "flag", "ret i32");
    }
}
