//! A file conditional owns either a braced list or exactly one original item.
use super::*;
use jai_source::SourceId;

const MAX_FILE_CONDITIONAL_DEPTH: usize = 64;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum FileItemCount {
    Any,
    One,
}

impl Parser<'_> {
    pub(super) fn file_conditional(
        &mut self,
        source: SourceId,
        visibility: Visibility,
    ) -> Result<FileItem, Diagnostic> {
        if self.file_conditional_depth >= MAX_FILE_CONDITIONAL_DEPTH {
            return Err(self.error("file conditional nesting exceeds the parser limit of 64"));
        }
        self.file_conditional_depth += 1;
        let result = self.file_conditional_inner(source, visibility);
        self.file_conditional_depth -= 1;
        result
    }

    fn file_conditional_inner(
        &mut self,
        source: SourceId,
        visibility: Visibility,
    ) -> Result<FileItem, Diagnostic> {
        let start = self.token().span.start;
        self.at += 1;
        let (condition, operator, complete) = self.source_conditional_header()?;
        if let Some(operator) = operator {
            let cases = self.source_cases(start, condition, operator, complete, |parser| {
                parser.source_case_file_items(source, visibility)
            })?;
            return Ok(FileItem::CompileTimeCases {
                cases,
                location: self.location_from(source, start),
            });
        }
        let then_items = self.file_conditional_body(source, visibility)?;
        let else_items = if self.keyword(Keyword::Else) {
            self.file_conditional_body(source, visibility)?
        } else {
            Vec::new()
        };
        Ok(FileItem::Conditional {
            condition,
            then_items,
            else_items,
            location: self.location_from(source, start),
        })
    }

    fn file_conditional_body(
        &mut self,
        source: SourceId,
        visibility: Visibility,
    ) -> Result<Vec<FileItem>, Diagnostic> {
        let block = self.take(Punct::OpenBrace);
        self.file_item_list_with_limit(
            source,
            visibility,
            block,
            if block {
                FileItemCount::Any
            } else {
                FileItemCount::One
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::{SourceMap, Symbols};

    fn parse(text: &str) -> Result<ParsedFile, jai_source::LocatedDiagnostic> {
        let mut sources = SourceMap::default();
        let id = sources.insert("conditional.jai".into(), text.to_owned());
        parse_file(sources.get(id).unwrap(), &mut Symbols::default())
    }

    #[test]
    fn single_load_ends_at_its_semicolon_and_preserves_original_span() {
        let text = "#if true #load \"first.jai\"; FOLLOW::42;";
        let parsed = parse(text).unwrap();
        let [
            FileItem::Conditional {
                then_items,
                else_items,
                location,
                ..
            },
            FileItem::Declaration(_),
        ] = parsed.items()
        else {
            panic!("one conditional and following declaration expected");
        };
        assert!(else_items.is_empty());
        let [FileItem::Load(load)] = then_items.as_slice() else {
            panic!("one original load expected");
        };
        assert_eq!(load.target, "first.jai");
        assert_eq!(
            &text[load.location.span.start..load.location.span.end],
            "#load \"first.jai\";"
        );
        assert_eq!(
            &text[location.span.start..location.span.end],
            "#if true #load \"first.jai\";"
        );
    }

    #[test]
    fn chains_accept_original_import_and_declaration_items_with_visibility() {
        let parsed = parse("#scope_module #if true #load \"a\"; else #if false #import \"Other\"; else VALUE::42; AFTER::1;").unwrap();
        let [
            FileItem::Scope { .. },
            FileItem::Conditional { else_items, .. },
            FileItem::Declaration(after),
        ] = parsed.items()
        else {
            panic!("conditional must consume one item per branch");
        };
        assert_eq!(after.visibility, Visibility::Module);
        let [
            FileItem::Conditional {
                then_items,
                else_items,
                ..
            },
        ] = else_items.as_slice()
        else {
            panic!("else-if expected");
        };
        let [FileItem::Import(import)] = then_items.as_slice() else {
            panic!("original import expected");
        };
        assert_eq!(import.visibility, Visibility::Module);
        let [FileItem::Declaration(value)] = else_items.as_slice() else {
            panic!("original declaration expected");
        };
        assert_eq!(value.visibility, Visibility::Module);
    }

    #[test]
    fn dangling_else_belongs_to_the_nearest_conditional() {
        let parsed = parse("#if true #if false #load \"a\"; else #load \"b\";").unwrap();
        let [
            FileItem::Conditional {
                then_items,
                else_items,
                ..
            },
        ] = parsed.items()
        else {
            panic!("outer conditional expected");
        };
        assert!(else_items.is_empty());
        let [FileItem::Conditional { else_items, .. }] = then_items.as_slice() else {
            panic!("inner conditional expected");
        };
        assert!(matches!(else_items.as_slice(), [FileItem::Load(_)]));
    }

    #[test]
    fn malformed_single_bodies_do_not_consume_following_items() {
        assert!(parse("#if true").is_err());
        assert!(parse("#if true #load \"a\" else #load \"b\";").is_err());
        assert!(parse("#if true {} else").is_err());
    }

    #[test]
    fn braced_and_unbraced_nesting_is_bounded_on_a_two_mib_stack() {
        std::thread::Builder::new()
            .stack_size(2 * 1024 * 1024)
            .spawn(|| {
                for braced in [false, true] {
                    for depth in [64, 65, 130] {
                        let text = if braced {
                            format!(
                                "{}#load \"a\";{}",
                                "#if true {".repeat(depth),
                                "}".repeat(depth)
                            )
                        } else {
                            format!("{}#load \"a\";", "#if true ".repeat(depth))
                        };
                        let result = parse(&text);
                        if depth == 64 {
                            assert!(result.is_ok());
                        } else {
                            assert!(result.unwrap_err().message.contains(
                                "file conditional nesting exceeds the parser limit of 64"
                            ));
                        }
                    }
                }
                let siblings = format!(
                    "{}#load \"a\"; {}#load \"b\";",
                    "#if true ".repeat(64),
                    "#if true ".repeat(64)
                );
                assert!(parse(&siblings).is_ok());
            })
            .unwrap()
            .join()
            .unwrap();
    }
}
