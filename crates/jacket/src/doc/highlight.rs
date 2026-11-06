//! Compiler-resolved destinations over lexical spans from one clean-room coloring spec.
#[path = "coloring.rs"]
mod coloring;

/// Which names the semantic span overlay offers to its resolver.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Names {
    /// Those a call is made to.
    Called,
    /// All of them as paths, except the ones the code declares.
    Paths,
    /// Compiler facts decide eligibility at each UTF-8 offset; the scanner only excludes literals/comments/notes.
    ResolvedPaths,
}

/// A lexical span with optional compiler-resolved destination; offsets are UTF-8 bytes.
#[derive(Clone, Debug)]
pub(super) struct Token {
    pub start: usize,
    pub end: usize,
    pub role: Option<&'static str>,
    pub target: Option<String>,
}

/// Resolve a UTF-8 offset and path to its matched byte length and canonical destination.
type ResolveAt<'a> = dyn Fn(usize, &str) -> Option<(usize, String)> + 'a;

/// Tokenize a declaration without producing markup; declared identifiers remain unlinked.
pub(super) fn linked_tokens(
    code: &str,
    link: &dyn Fn(&str) -> Option<(usize, String)>,
) -> Vec<Token> {
    lexed(code, &|_, path| link(path), Names::Paths)
}

/// Color a complete formatted definition, resolving only the compiler-authorized occurrences.
pub(super) fn resolved_tokens(code: &str, link: &ResolveAt<'_>) -> Vec<Token> {
    lexed(code, link, Names::ResolvedPaths)
}

/// Example call destinations use the same scanner as highlighted declarations.
pub(super) fn called_tokens(code: &str, link: &dyn Fn(&str) -> Option<String>) -> Vec<Token> {
    lexed(
        code,
        &|_, name| link(name).map(|href| (name.len(), href)),
        Names::Called,
    )
}

fn lexed(code: &str, link: &ResolveAt<'_>, names: Names) -> Vec<Token> {
    let lexical = coloring::scan(code);
    let mut out = Vec::with_capacity(lexical.len());
    let mut index = 0;
    while index < lexical.len() {
        let first = &lexical[index];
        let mut end = first.end;
        let mut last = index;
        if first.kind == coloring::Kind::Identifier && first.role.is_none() {
            while matches!(names, Names::Paths | Names::ResolvedPaths)
                && lexical
                    .get(last + 1)
                    .is_some_and(|token| &code[token.start..token.end] == ".")
                && lexical
                    .get(last + 2)
                    .is_some_and(|token| token.kind == coloring::Kind::Identifier)
            {
                last += 2;
                end = lexical[last].end;
            }
            let path = &code[first.start..end];
            let declared = code[end..].trim_start_matches([' ', '\t']).starts_with(':');
            let wanted = match names {
                Names::Called => code.as_bytes().get(end) == Some(&b'('),
                Names::Paths => !declared,
                Names::ResolvedPaths => true,
            };
            if let Some((len, href)) = wanted
                .then(|| link(first.start, path))
                .flatten()
                .filter(|(len, _)| *len > 0 && *len <= path.len() && path.is_char_boundary(*len))
            {
                out.push(Token {
                    start: first.start,
                    end: first.start + len,
                    role: None,
                    target: Some(href),
                });
                if first.start + len < end {
                    out.push(Token {
                        start: first.start + len,
                        end,
                        role: None,
                        target: None,
                    });
                }
            } else {
                out.push(Token {
                    start: first.start,
                    end,
                    role: first.role,
                    target: None,
                });
            }
        } else {
            out.push(Token {
                start: first.start,
                end,
                role: first.role,
                target: None,
            });
        }
        index = last + 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formatted_definition_links_use_compiler_occurrence_offsets_and_note_opacity() {
        let code = "Collision, other: Collision; @note(Collision) value: Collision;";
        let occurrences = code
            .match_indices("Collision")
            .map(|(start, _)| start)
            .collect::<Vec<_>>();
        let permitted = [occurrences[1], occurrences[3]];
        let offered = std::cell::RefCell::new(Vec::new());
        let tokens = resolved_tokens(code, &|start, path| {
            offered.borrow_mut().push(start);
            permitted
                .contains(&start)
                .then(|| (path.len(), "/api/Type/Collision".into()))
        });
        assert_eq!(
            tokens
                .iter()
                .filter(|token| token.target.is_some())
                .map(|token| token.start)
                .collect::<Vec<_>>(),
            permitted
        );
        assert!(
            !offered.borrow().contains(&occurrences[2]),
            "notes must not offer their arguments as semantic references"
        );
        assert_eq!(
            tokens
                .iter()
                .map(|token| &code[token.start..token.end])
                .collect::<String>(),
            code
        );
    }

    #[test]
    fn note_color_parts_never_expose_note_names_to_semantic_links() {
        let source = "@named(Resolve()) Resolve()";
        let tokens = called_tokens(source, &|name| {
            (name == "Resolve").then(|| "/resolved".into())
        });
        let links: Vec<_> = tokens
            .iter()
            .filter(|token| token.target.is_some())
            .collect();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].start, source.rfind("Resolve").unwrap());
    }

    #[test]
    fn shared_coloring_fixtures() {
        let cases: serde_json::Value =
            serde_json::from_str(include_str!("coloring/fixtures.json")).unwrap();
        for case in cases.as_array().unwrap() {
            let expected: Vec<(Option<&str>, &str)> = case["segments"]
                .as_array()
                .unwrap()
                .iter()
                .map(|part| (part[0].as_str(), part[1].as_str().unwrap()))
                .collect();
            let source: String = expected.iter().map(|part| part.1).collect();
            let mut actual: Vec<(Option<&str>, String)> = Vec::new();
            for token in called_tokens(&source, &|_| None) {
                let text = &source[token.start..token.end];
                if let Some(previous) = actual.last_mut()
                    && previous.0 == token.role
                {
                    previous.1.push_str(text);
                } else {
                    actual.push((token.role, text.into()));
                }
            }
            let expected: Vec<_> = expected
                .into_iter()
                .map(|(role, text)| (role, text.to_owned()))
                .collect();
            assert_eq!(actual, expected, "fixture {}", case["id"]);
        }
    }

    #[test]
    fn semantic_tokens_link_written_paths_but_not_the_names_they_declare() {
        let code = "Box :: struct(T: Type) {\n    next: *Box(T);\n    table: Hash.Table(int, T);\n    tag: Hash.Kind.ONE;\n}";
        let tokens = linked_tokens(code, &|path| match path {
            "Box" => Some((3, "/Box/".into())),
            "Hash.Table" => Some((10, "/Table/".into())),
            "Hash.Kind.ONE" => Some((9, "/Kind/".into())),
            _ => None,
        });
        let linked = tokens
            .iter()
            .filter_map(|token| {
                token
                    .target
                    .as_deref()
                    .map(|target| (&code[token.start..token.end], target))
            })
            .collect::<Vec<_>>();
        assert_eq!(
            linked,
            [
                ("Box", "/Box/"),
                ("Hash.Table", "/Table/"),
                ("Hash.Kind", "/Kind/")
            ]
        );
        assert_eq!(
            tokens
                .iter()
                .map(|token| &code[token.start..token.end])
                .collect::<String>(),
            code
        );
        assert!(
            tokens
                .iter()
                .any(|token| token.role == Some("k") && &code[token.start..token.end] == "struct")
        );
    }

    #[test]
    fn semantic_example_tokens_keep_literal_bytes_roles_and_call_destinations() {
        let code = "found := trim(\"a<b\"); // note\nx: int = 0x1F; #char \"=\";";
        let tokens = called_tokens(code, &|name| (name == "trim").then(|| "/trim.html".into()));
        assert_eq!(
            tokens
                .iter()
                .filter_map(|token| token
                    .target
                    .as_deref()
                    .map(|target| (&code[token.start..token.end], target)))
                .collect::<Vec<_>>(),
            [("trim", "/trim.html")]
        );
        for (role, text) in [
            ("s", "\"a<b\""),
            ("c", "// note"),
            ("t", "int"),
            ("n", "0x1F"),
            ("d", "#char"),
        ] {
            assert!(
                tokens
                    .iter()
                    .any(|token| token.role == Some(role) && &code[token.start..token.end] == text),
                "missing {role}: {text}"
            );
        }
        assert_eq!(
            tokens
                .iter()
                .map(|token| &code[token.start..token.end])
                .collect::<String>(),
            code
        );
    }
}
