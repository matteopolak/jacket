//! Per-file module syntax. This API does not imply executable module support.
use super::*;
use jai_source::{LocatedDiagnostic, SourceId, SourceRecord, SourceSpan};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visibility {
    Export,
    Module,
    File,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamePath {
    pub root: Symbol,
    pub members: Vec<Symbol>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImportMode {
    Search,
    File,
    Directory,
    String,
}
#[derive(Clone, Debug)]
pub struct ImportArgument {
    pub name: Option<Symbol>,
    pub value: ModuleArgumentValue,
}
#[derive(Clone, Debug)]
pub enum ModuleArgumentValue {
    Expression(Expression),
    String(String),
}
#[derive(Clone, Debug, Default)]
pub struct ImportArguments {
    /// None differs syntactically from an explicitly supplied empty list.
    pub instance: Option<Vec<ImportArgument>>,
    pub program: Option<Vec<ImportArgument>>,
}
#[derive(Clone, Debug)]
pub struct ImportDeclaration {
    pub namespace: Option<Symbol>,
    pub using: bool,
    pub visibility: Visibility,
    pub mode: ImportMode,
    pub target: String,
    pub arguments: ImportArguments,
    pub location: SourceSpan,
}
#[derive(Clone, Debug)]
pub struct LoadDirective {
    pub target: String,
    pub location: SourceSpan,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModuleParameterType {
    Inferred,
    Scalar(ScalarType),
    String,
}
#[derive(Clone, Debug)]
pub struct ModuleParameter {
    pub name: Symbol,
    pub ty: ModuleParameterType,
    pub default: Option<ModuleArgumentValue>,
    pub location: SourceSpan,
}
#[derive(Clone, Debug)]
pub struct ModuleParameters {
    pub instance: Vec<ModuleParameter>,
    pub program: Option<Vec<ModuleParameter>>,
    pub location: SourceSpan,
}
#[derive(Clone, Debug)]
pub enum FileDeclarationKind {
    Procedure(Procedure),
    Global(GlobalDeclaration),
    Constant(ConstantDeclaration),
    Record(RecordDeclaration),
    Enum(EnumDeclaration),
}
#[derive(Clone, Debug)]
pub struct FileDeclaration {
    pub visibility: Visibility,
    pub kind: FileDeclarationKind,
    pub location: SourceSpan,
}
#[derive(Clone, Debug)]
pub enum FileItem {
    Scope {
        visibility: Visibility,
        location: SourceSpan,
    },
    Import(ImportDeclaration),
    Load(LoadDirective),
    Parameters(ModuleParameters),
    Declaration(FileDeclaration),
}
#[derive(Clone, Debug)]
pub struct ParsedFile {
    source: SourceId,
    items: Vec<FileItem>,
}
impl ParsedFile {
    pub fn source(&self) -> SourceId {
        self.source
    }
    pub fn items(&self) -> &[FileItem] {
        &self.items
    }
    /// Nested AST spans remain offsets within this file, never concatenated input.
    pub fn location(&self, span: Span) -> SourceSpan {
        SourceSpan {
            source: self.source,
            span,
        }
    }
}
/// Share one spelling interner across independently parsed files. Even a failed parse
/// restores the interner, so previously allocated spelling identities remain valid.
pub fn parse_file(
    source: &SourceRecord,
    symbols: &mut Symbols,
) -> Result<ParsedFile, LocatedDiagnostic> {
    let tokens = lex(source.text()).map_err(|d| LocatedDiagnostic::new(source.id(), d))?;
    let mut parser = Parser {
        source: source.text(),
        tokens,
        at: 0,
        symbols: std::mem::take(symbols),
        allow_qualified: true,
    };
    let result = parser.file_items(source.id());
    *symbols = parser.symbols;
    result.map_err(|d| LocatedDiagnostic::new(source.id(), d))
}
impl Parser<'_> {
    fn location_from(&self, source: SourceId, start: usize) -> SourceSpan {
        SourceSpan {
            source,
            span: Span::new(start, self.tokens[self.at.saturating_sub(1)].span.end),
        }
    }
    fn file_items(&mut self, source: SourceId) -> Result<ParsedFile, Diagnostic> {
        let mut visibility = Visibility::Export;
        let mut items = Vec::new();
        let mut saw_parameters = false;
        while self.token().kind != Kind::Eof {
            let start = self.token().span.start;
            let scope = match self.token().kind {
                Kind::Directive(Directive::ScopeExport) => Some(Visibility::Export),
                Kind::Directive(Directive::ScopeModule) => Some(Visibility::Module),
                Kind::Directive(Directive::ScopeFile) => Some(Visibility::File),
                _ => None,
            };
            if let Some(value) = scope {
                self.at += 1;
                self.take(Punct::Semicolon);
                visibility = value;
                items.push(FileItem::Scope {
                    visibility,
                    location: self.location_from(source, start),
                });
                continue;
            }
            if self.token().kind == Kind::Directive(Directive::Load) {
                self.at += 1;
                let target = self.module_string()?;
                self.need(Punct::Semicolon)?;
                items.push(FileItem::Load(LoadDirective {
                    target,
                    location: self.location_from(source, start),
                }));
                continue;
            }
            if self.token().kind == Kind::Directive(Directive::ModuleParameters) {
                if saw_parameters {
                    return Err(self.error("duplicate #module_parameters directive"));
                }
                saw_parameters = true;
                self.at += 1;
                let instance = self.module_parameters(source)?;
                let program = if self.is(Punct::OpenParen) {
                    Some(self.module_parameters(source)?)
                } else {
                    None
                };
                if self.is(Punct::OpenBrace) {
                    return Err(
                        self.error("module parameter declaration blocks are not implemented")
                    );
                }
                self.need(Punct::Semicolon)?;
                items.push(FileItem::Parameters(ModuleParameters {
                    instance,
                    program,
                    location: self.location_from(source, start),
                }));
                continue;
            }
            if self.import_prefix() {
                items.push(FileItem::Import(
                    self.import_declaration(source, visibility)?,
                ));
                continue;
            }
            let kind = if matches!(
                self.nominal_prefix(),
                Some(Keyword::Struct | Keyword::Union)
            ) {
                FileDeclarationKind::Record(self.record_declaration()?)
            } else if matches!(
                self.nominal_prefix(),
                Some(Keyword::Enum | Keyword::EnumFlags)
            ) {
                FileDeclarationKind::Enum(self.enum_declaration()?)
            } else if self.starts_procedure() {
                FileDeclarationKind::Procedure(self.procedure()?)
            } else {
                let span = self.token().span;
                let name = self.name()?;
                match self.data_declaration(name, span)? {
                    Statement::Declare(declaration) => {
                        FileDeclarationKind::Global(GlobalDeclaration { declaration, span })
                    }
                    Statement::Constant(declaration) => FileDeclarationKind::Constant(declaration),
                    _ => unreachable!("data declaration produces declaration"),
                }
            };
            items.push(FileItem::Declaration(FileDeclaration {
                visibility,
                kind,
                location: self.location_from(source, start),
            }));
        }
        Ok(ParsedFile { source, items })
    }
    fn import_prefix(&self) -> bool {
        let offset = usize::from(self.token().kind == Kind::Keyword(Keyword::Using));
        let Some(t) = self.tokens.get(self.at + offset) else {
            return false;
        };
        t.kind == Kind::Directive(Directive::Import)
            || (t.kind == Kind::Ident
                && self
                    .tokens
                    .get(self.at + offset + 1)
                    .is_some_and(|t| t.kind == Kind::Punctuation(Punct::Constant))
                && self
                    .tokens
                    .get(self.at + offset + 2)
                    .is_some_and(|t| t.kind == Kind::Directive(Directive::Import)))
    }
    fn import_declaration(
        &mut self,
        source: SourceId,
        visibility: Visibility,
    ) -> Result<ImportDeclaration, Diagnostic> {
        let start = self.token().span.start;
        let using = self.keyword(Keyword::Using);
        let namespace = if self.token().kind == Kind::Ident {
            let name = self.name()?;
            self.need(Punct::Constant)?;
            Some(name)
        } else {
            None
        };
        if using && namespace.is_none() {
            return Err(self.error("using import requires a namespace binding"));
        }
        if self.token().kind != Kind::Directive(Directive::Import) {
            return Err(self.error("expected #import"));
        }
        self.at += 1;
        let mode = if self.take(Punct::Comma) {
            let mode = match self.text() {
                "file" => ImportMode::File,
                "dir" => ImportMode::Directory,
                "string" => ImportMode::String,
                _ => return Err(self.error("unsupported import modifier")),
            };
            self.at += 1;
            mode
        } else {
            ImportMode::Search
        };
        let target = self.module_string()?;
        let instance = if self.is(Punct::OpenParen) {
            Some(self.import_arguments()?)
        } else {
            None
        };
        let program = if self.is(Punct::OpenParen) {
            Some(self.import_arguments()?)
        } else {
            None
        };
        self.need(Punct::Semicolon)?;
        Ok(ImportDeclaration {
            namespace,
            using,
            visibility,
            mode,
            target,
            arguments: ImportArguments { instance, program },
            location: self.location_from(source, start),
        })
    }
    fn module_string(&mut self) -> Result<String, Diagnostic> {
        if self.token().kind != Kind::String {
            return Err(self.error("expected literal module source or path string"));
        }
        let raw = self.text();
        let value = &raw[1..raw.len() - 1];
        if value.contains('\\') {
            return Err(self.error("escaped module strings are not implemented"));
        }
        let value = value.to_owned();
        self.at += 1;
        Ok(value)
    }
    fn module_value(&mut self) -> Result<ModuleArgumentValue, Diagnostic> {
        if self.token().kind == Kind::String {
            Ok(ModuleArgumentValue::String(self.module_string()?))
        } else {
            Ok(ModuleArgumentValue::Expression(self.expression(0)?))
        }
    }
    fn import_arguments(&mut self) -> Result<Vec<ImportArgument>, Diagnostic> {
        self.need(Punct::OpenParen)?;
        let mut values = Vec::new();
        if self.take(Punct::CloseParen) {
            return Ok(values);
        }
        loop {
            let name = if self.named_prefix(Punct::Assign) {
                let name = self.name()?;
                self.need(Punct::Assign)?;
                Some(name)
            } else {
                None
            };
            values.push(ImportArgument {
                name,
                value: self.module_value()?,
            });
            if self.take(Punct::CloseParen) {
                break;
            }
            self.need(Punct::Comma)?;
            if self.take(Punct::CloseParen) {
                break;
            }
        }
        Ok(values)
    }
    fn module_parameters(&mut self, source: SourceId) -> Result<Vec<ModuleParameter>, Diagnostic> {
        self.need(Punct::OpenParen)?;
        let mut values = Vec::new();
        if self.take(Punct::CloseParen) {
            return Ok(values);
        }
        loop {
            let start = self.token().span.start;
            let name = self.name()?;
            let (ty, default) = if self.take(Punct::Infer) {
                (ModuleParameterType::Inferred, Some(self.module_value()?))
            } else {
                self.need(Punct::Colon)?;
                let ty = if self.text() == "string" {
                    self.at += 1;
                    ModuleParameterType::String
                } else {
                    ModuleParameterType::Scalar(self.scalar_type()?)
                };
                let default = if self.take(Punct::Assign) {
                    Some(self.module_value()?)
                } else {
                    None
                };
                (ty, default)
            };
            values.push(ModuleParameter {
                name,
                ty,
                default,
                location: self.location_from(source, start),
            });
            if self.take(Punct::CloseParen) {
                break;
            }
            self.need(Punct::Comma)?;
            if self.take(Punct::CloseParen) {
                break;
            }
        }
        Ok(values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::SourceMap;
    fn file(text: &str, symbols: &mut Symbols) -> Result<ParsedFile, LocatedDiagnostic> {
        let mut sources = SourceMap::default();
        let id = sources.insert("fixture.jai".into(), text.into());
        parse_file(sources.get(id).unwrap(), symbols)
    }
    #[test]
    fn independent_files_share_spellings_and_preserve_sources() {
        let mut sources = SourceMap::default();
        let a = sources.insert("a.jai".into(), "same :: 1;".into());
        let b = sources.insert("b.jai".into(), "same :: 2;".into());
        let mut symbols = Symbols::default();
        let a = parse_file(sources.get(a).unwrap(), &mut symbols).unwrap();
        let name = symbols.find("same").unwrap();
        let b = parse_file(sources.get(b).unwrap(), &mut symbols).unwrap();
        assert_ne!(a.source(), b.source());
        for parsed in [&a, &b] {
            let FileItem::Declaration(declaration) = &parsed.items()[0] else {
                panic!()
            };
            let FileDeclarationKind::Constant(constant) = &declaration.kind else {
                panic!()
            };
            assert_eq!(constant.name, name);
            assert_eq!(declaration.location.source, parsed.source());
            assert_eq!(
                parsed.location(constant.initializer.span).source,
                parsed.source()
            );
        }
    }
    #[test]
    fn visibility_switches_and_load_do_not_erase_file_boundary() {
        let parsed = file("public :: 1; #scope_file private :: 2; #load \"other.jai\"; #scope_module; hidden :: 3; #scope_export public_again :: 4;", &mut Symbols::default()).unwrap();
        let visibilities: Vec<_> = parsed
            .items()
            .iter()
            .filter_map(|item| match item {
                FileItem::Declaration(d) => Some(d.visibility),
                _ => None,
            })
            .collect();
        assert_eq!(
            visibilities,
            [
                Visibility::Export,
                Visibility::File,
                Visibility::Module,
                Visibility::Export
            ]
        );
        assert!(matches!(&parsed.items()[3], FileItem::Load(load) if load.target == "other.jai"));
        assert!(parse("#scope_file private :: 2; main :: () {}").is_err());
    }
    #[test]
    fn import_modes_namespaces_and_two_argument_lists() {
        let mut symbols = Symbols::default();
        let parsed = file("#scope_file; using Basic :: #import \"Basic\"()(MEMORY_DEBUGGER=true); Specific :: #import,file \"local.jai\"; #import,dir \"local\"; #import,string \"answer :: 42;\";", &mut symbols).unwrap();
        let imports: Vec<_> = parsed
            .items()
            .iter()
            .filter_map(|item| {
                if let FileItem::Import(i) = item {
                    Some(i)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(imports.len(), 4);
        assert_eq!(imports[0].namespace, symbols.find("Basic"));
        assert!(imports[0].using);
        assert_eq!(imports[0].visibility, Visibility::File);
        assert!(imports[0].arguments.instance.as_ref().unwrap().is_empty());
        assert_eq!(
            imports[0].arguments.program.as_ref().unwrap()[0].name,
            symbols.find("MEMORY_DEBUGGER")
        );
        assert_eq!(imports[1].mode, ImportMode::File);
        assert_eq!(imports[2].mode, ImportMode::Directory);
        assert_eq!(imports[3].mode, ImportMode::String);
        assert!(parse("#import \"Basic\";").is_err());
    }
    #[test]
    fn parameter_defaults_required_values_and_qualified_calls() {
        let mut symbols = Symbols::default();
        let parsed = file("#module_parameters(Count: int = 4, Name: string = \"fallback\", Required: string)(DEBUG := false); Math :: #import \"Math\"; answer :: () -> int { return Math.inner.answer(Count); }", &mut symbols).unwrap();
        let FileItem::Parameters(parameters) = &parsed.items()[0] else {
            panic!()
        };
        assert_eq!(parameters.instance.len(), 3);
        assert_eq!(parameters.instance[1].ty, ModuleParameterType::String);
        assert!(parameters.instance[2].default.is_none());
        assert_eq!(
            parameters.program.as_ref().unwrap()[0].ty,
            ModuleParameterType::Inferred
        );
        let FileItem::Declaration(d) = &parsed.items()[2] else {
            panic!()
        };
        let FileDeclarationKind::Procedure(procedure) = &d.kind else {
            panic!()
        };
        let Statement::Return(Some(expression)) = &procedure.body[0] else {
            panic!()
        };
        let ExpressionKind::QualifiedCall(path, arguments) = &expression.kind else {
            panic!()
        };
        assert_eq!(path.root, symbols.find("Math").unwrap());
        assert_eq!(
            path.members,
            [
                symbols.find("inner").unwrap(),
                symbols.find("answer").unwrap()
            ]
        );
        assert_eq!(arguments.len(), 1);
        assert!(parse("main :: () -> int { return Math.answer(); }").is_err());
    }
    #[test]
    fn failures_are_located_and_restore_interner() {
        let mut sources = SourceMap::default();
        let id = sources.insert("bad.jai".into(), "#import \"Math\"(known=true;".into());
        let mut symbols = Symbols::default();
        let existing = symbols.intern("existing");
        let error = parse_file(sources.get(id).unwrap(), &mut symbols).unwrap_err();
        assert_eq!(error.location.source, id);
        assert!(error.render(&sources).contains("bad.jai:1:"));
        assert_eq!(symbols.find("existing"), Some(existing));
        assert!(symbols.find("known").is_some());
        for unsupported in [
            "#import,mystery \"Math\";",
            "#module_parameters(x := true) { X :: enum { A; } };",
            "#module_parameters(x := true); #module_parameters(y := false);",
            "#import \"Math\"()()();",
            "main :: () { #import \"Math\"; }",
        ] {
            assert!(file(unsupported, &mut symbols).is_err(), "{unsupported}");
        }
    }
}
