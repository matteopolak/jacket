//! Library declarations are metadata, never expression values or loader requests.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LibraryKind {
    System,
    Local,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LibraryOptions {
    pub link_always: bool,
    pub no_dll: bool,
    pub no_static_library: bool,
}

#[derive(Clone, Debug)]
pub struct LibraryDeclaration {
    pub name: Symbol,
    pub kind: LibraryKind,
    pub target: String,
    pub options: LibraryOptions,
    pub span: Span,
}

impl Parser<'_> {
    pub(super) fn starts_library(&self) -> bool {
        self.token().kind == Kind::Ident
            && self
                .tokens
                .get(self.at + 1)
                .is_some_and(|token| token.kind == Kind::Punctuation(Punct::Constant))
            && self.tokens.get(self.at + 2).is_some_and(|token| {
                matches!(
                    token.kind,
                    Kind::Directive(Directive::Library | Directive::SystemLibrary)
                )
            })
    }

    pub(super) fn library_declaration(&mut self) -> Result<LibraryDeclaration, Diagnostic> {
        let start = self.token().span.start;
        let name = self.name()?;
        self.need(Punct::Constant)?;
        let mut kind = match self.token().kind {
            Kind::Directive(Directive::Library) => LibraryKind::Local,
            Kind::Directive(Directive::SystemLibrary) => LibraryKind::System,
            _ => return Err(self.error("expected #library or #system_library")),
        };
        self.at += 1;
        let mut options = LibraryOptions::default();
        let mut system_modifier = false;
        while self.take(Punct::Comma) {
            let duplicate = match self.text() {
                "system" => {
                    let duplicate = system_modifier || kind == LibraryKind::System;
                    system_modifier = true;
                    kind = LibraryKind::System;
                    duplicate
                }
                "link_always" => std::mem::replace(&mut options.link_always, true),
                "no_dll" => std::mem::replace(&mut options.no_dll, true),
                "no_static_library" => std::mem::replace(&mut options.no_static_library, true),
                _ => return Err(self.error("unsupported library modifier")),
            };
            if duplicate {
                return Err(self.error("duplicate library modifier"));
            }
            self.at += 1;
        }
        if options.no_dll && options.no_static_library {
            return Err(self.error("library cannot disable both dynamic and static linking"));
        }
        let target = self.module_string()?;
        if target.is_empty() || target.contains('\0') {
            return Err(self.error("library target must be nonempty and contain no NUL"));
        }
        self.need(Punct::Semicolon)?;
        Ok(LibraryDeclaration {
            name,
            kind,
            target,
            options,
            span: Span::new(start, self.tokens[self.at - 1].span.end),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::SourceMap;

    fn parse(text: &str) -> Result<LibraryDeclaration, String> {
        let mut sources = SourceMap::default();
        let source = sources.insert("fixture.jai".into(), text.into());
        let file = parse_file(sources.get(source).unwrap(), &mut Symbols::default())
            .map_err(|error| error.to_string())?;
        let FileItem::Declaration(declaration) = &file.items()[0] else {
            panic!("expected declaration");
        };
        let FileDeclarationKind::Library(library) = &declaration.kind else {
            panic!("expected library declaration");
        };
        Ok(library.clone())
    }

    #[test]
    fn equivalent_system_spellings_preserve_actual_target() {
        for text in [
            "alias :: #system_library \"libc\";",
            "alias :: #library,system \"libc\";",
            "alias :: #foreign_library,system \"libc\";",
        ] {
            let library = parse(text).unwrap();
            assert_eq!(library.kind, LibraryKind::System);
            assert_eq!(library.target, "libc");
        }
    }

    #[test]
    fn retains_local_and_link_options() {
        let library = parse("own :: #library,no_dll,link_always \"native/own\";").unwrap();
        assert_eq!(library.kind, LibraryKind::Local);
        assert!(library.options.no_dll && library.options.link_always);
        let legacy = parse("own :: #foreign_library,no_dll,link_always \"native/own\";").unwrap();
        assert_eq!(legacy.kind, library.kind);
        assert_eq!(legacy.target, library.target);
        assert_eq!(legacy.options, library.options);
        let library = parse("cpp :: #system_library,link_always \"libstdc++.so.6\";").unwrap();
        assert_eq!(library.target, "libstdc++.so.6");
    }

    #[test]
    fn rejects_unknown_duplicate_and_impossible_options() {
        for text in [
            "bad :: #library,magic \"x\";",
            "bad :: #foreign_library,magic \"x\";",
            "bad :: #library,no_dll,no_dll \"x\";",
            "bad :: #library,no_dll,no_static_library \"x\";",
            "bad :: #system_library \"\";",
        ] {
            assert!(parse(text).is_err(), "{text}");
        }
    }
}
