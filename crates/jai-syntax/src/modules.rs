//! Per-file module syntax. This API does not imply executable module support.
use super::file_conditional_bodies::FileItemCount;
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
pub struct ScopedImportDeclaration {
    pub namespace: Option<Symbol>,
    pub using: bool,
    pub mode: ImportMode,
    pub target: String,
    pub arguments: ImportArguments,
    pub span: Span,
}
#[derive(Clone, Debug)]
pub struct LoadDirective {
    pub target: String,
    pub location: SourceSpan,
}
#[derive(Clone, Debug)]
pub enum ModuleParameterType {
    Inferred,
    Interface {
        variable: Symbol,
        constraint: TypeSyntax,
    },
    Unresolved(TypeSyntax),
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
    pub declarations: Vec<FileItem>,
    pub location: SourceSpan,
}
#[derive(Clone, Debug)]
pub enum FileDeclarationKind {
    Placeholder(crate::placeholders::PlaceholderDeclaration),
    Library(LibraryDeclaration),
    Procedure(Procedure),
    OperatorAlias(OperatorAlias),
    ProcedurePrototype(ProcedurePrototype),
    Global(GlobalDeclaration),
    Constant(ConstantDeclaration),
    Record(RecordDeclaration),
    Enum(EnumDeclaration),
    TypeAlias(TypeAliasDeclaration),
}
#[derive(Clone, Debug)]
pub struct FileDeclaration {
    pub program_export: Option<ProgramExport>,
    pub visibility: Visibility,
    pub kind: FileDeclarationKind,
    pub location: SourceSpan,
}
#[derive(Clone, Debug)]
pub enum ContextFieldDeclaration {
    Field(FieldDeclaration),
    Variable(GlobalDeclaration),
    Constant(ConstantDeclaration),
}
impl ContextFieldDeclaration {
    pub fn name(&self) -> Symbol {
        match self {
            Self::Field(field) => field.name,
            Self::Variable(global) => global.declaration.name(),
            Self::Constant(constant) => constant.name,
        }
    }
}
#[derive(Clone, Debug)]
pub enum FileItem {
    Insert {
        directive: InsertDirective,
        location: SourceSpan,
    },
    ContextField {
        declaration: ContextFieldDeclaration,
        location: SourceSpan,
    },
    Scope {
        visibility: Visibility,
        location: SourceSpan,
    },
    Import(ImportDeclaration),
    Using {
        directive: UsingDirective,
        visibility: Visibility,
        location: SourceSpan,
    },
    UsingDeclaration {
        declaration: FileDeclaration,
        selection: UsingSelection,
        target_span: Span,
        location: SourceSpan,
    },
    Load(LoadDirective),
    Run(RunDirective),
    Parameters(ModuleParameters),
    Assert {
        condition: Expression,
        message: Option<Expression>,
        location: SourceSpan,
    },
    CompileTimeCases {
        cases: CompileTimeCases<FileItem>,
        location: SourceSpan,
    },
    Conditional {
        condition: Expression,
        then_items: Vec<FileItem>,
        else_items: Vec<FileItem>,
        location: SourceSpan,
    },
    Declaration(FileDeclaration),
}
#[derive(Clone, Debug)]
pub struct RunDirective {
    pub flags: RunFlags,
    pub body: CompileTimeBody,
    pub location: SourceSpan,
}
#[derive(Clone, Debug)]
pub struct ParsedFile {
    source: SourceId,
    items: Vec<FileItem>,
}
impl ParsedFile {
    pub(super) fn retained_items(&self) -> &Vec<FileItem> {
        &self.items
    }

    /// Wrap original typed source items in a new expansion instance without
    /// reparsing text or changing their source coordinates.
    pub fn from_items(source: SourceId, items: Vec<FileItem>) -> Self {
        Self {
            source,
            items,
        }
    }
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
        record_conditional_depth: 0,
        file_conditional_depth: 0,
    };
    let result = parser.file_items(source.id());
    *symbols = parser.symbols;
    result.map_err(|d| LocatedDiagnostic::new(source.id(), d))
}
impl Parser<'_> {
    pub(super) fn location_from(&self, source: SourceId, start: usize) -> SourceSpan {
        SourceSpan {
            source,
            span: Span::new(start, self.tokens[self.at.saturating_sub(1)].span.end),
        }
    }
    fn file_items(&mut self, source: SourceId) -> Result<ParsedFile, Diagnostic> {
        let items = self.file_item_list(source, Visibility::Export, false)?;
        Ok(ParsedFile {
            source,
            items,
        })
    }
    fn file_item_list(
        &mut self,
        source: SourceId,
        visibility: Visibility,
        nested: bool,
    ) -> Result<Vec<FileItem>, Diagnostic> {
        self.file_item_list_with_limit(source, visibility, nested, FileItemCount::Any)
    }
    pub(super) fn file_item_list_with_limit(
        &mut self,
        source: SourceId,
        mut visibility: Visibility,
        nested: bool,
        count: FileItemCount,
    ) -> Result<Vec<FileItem>, Diagnostic> {
        let mut items = Vec::new();
        let mut saw_parameters = false;
        while self.token().kind != Kind::Eof {
            if count == FileItemCount::One && !items.is_empty() {
                return Ok(items);
            }
            if nested && self.take(Punct::CloseBrace) {
                return Ok(items);
            }
            let start = self.token().span.start;
            if self.token().kind == Kind::UnknownDirective && self.text() == "#placeholder" {
                let marker = self.placeholder_declaration()?;
                items.push(FileItem::Declaration(FileDeclaration {
                    program_export: None,
                    visibility,
                    kind: FileDeclarationKind::Placeholder(marker),
                    location: self.location_from(source, start),
                }));
                continue;
            }
            if self.token().kind == Kind::UnknownDirective {
                return Err(self.error(format!("unknown directive '{}'", self.text())));
            }
            let program_export = if self.token().kind == Kind::Directive(Directive::ProgramExport) {
                Some(self.program_export()?)
            } else {
                None
            };
            if program_export.is_some() && self.import_prefix() {
                return Err(self.error("#program_export cannot annotate an import"));
            }
            if self.token().kind == Kind::Directive(Directive::Insert) {
                let directive = self.insert_directive(0)?;
                self.insert_terminator(&directive)?;
                items.push(FileItem::Insert {
                    directive,
                    location: self.location_from(source, start),
                });
                continue;
            }
            if self.token().kind == Kind::Directive(Directive::AddContext) {
                let declaration = self.context_field_declaration()?;
                items.push(FileItem::ContextField {
                    declaration,
                    location: self.location_from(source, start),
                });
                continue;
            }
            if self.token().kind == Kind::Directive(Directive::Run) {
                let expression = self.compile_time()?;
                let ExpressionKind::CompileTime(CompileTimeRun {
                    flags,
                    body,
                }) = expression.kind
                else {
                    unreachable!("compile-time parser returns a run request");
                };
                if matches!(body, CompileTimeBody::Expression(_)) {
                    self.need(Punct::Semicolon)?;
                } else {
                    self.take(Punct::Semicolon);
                }
                items.push(FileItem::Run(RunDirective {
                    flags,
                    body,
                    location: self.location_from(source, start),
                }));
                continue;
            }
            if self.token().kind == Kind::Directive(Directive::Assert) {
                let (condition, message) = self.assertion_arguments()?;
                items.push(FileItem::Assert {
                    condition,
                    message,
                    location: self.location_from(source, start),
                });
                continue;
            }
            if self.token().kind == Kind::Directive(Directive::If) {
                items.push(self.file_conditional(source, visibility)?);
                continue;
            }
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
                let declarations = if self.take(Punct::OpenBrace) {
                    let declarations = self.file_item_list(source, Visibility::Module, true)?;
                    self.take(Punct::Semicolon);
                    declarations
                } else {
                    self.need(Punct::Semicolon)?;
                    Vec::new()
                };
                items.push(FileItem::Parameters(ModuleParameters {
                    instance,
                    program,
                    declarations,
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
            let using_declaration = if self.token().kind == Kind::Keyword(Keyword::Using) {
                self.at += 1;
                let selection = self.using_selection()?;
                if self.using_declaration_prefix() {
                    Some((selection, self.token().span))
                } else {
                    if program_export.is_some() {
                        return Err(self.error("#program_export requires a declaration"));
                    }
                    let directive = self.using_target(start, selection)?;
                    items.push(FileItem::Using {
                        directive,
                        visibility,
                        location: self.location_from(source, start),
                    });
                    continue;
                }
            } else {
                None
            };
            let declaration_start = self.token().span.start;
            if self.token().kind == Kind::Ident
                && self
                    .tokens
                    .get(self.at + 1)
                    .is_some_and(|token| token.kind == Kind::Punctuation(Punct::Comma))
            {
                if using_declaration.is_some() || program_export.is_some() {
                    return Err(
                        self.error("using and #program_export require an individual declaration")
                    );
                }
                for member in self.file_data_declarations()? {
                    let (kind, span) = match member {
                        declaration_lists::GlobalOrConstant::Global(global) => {
                            let span = global.span;
                            (FileDeclarationKind::Global(global), span)
                        }
                        declaration_lists::GlobalOrConstant::Constant(constant) => {
                            let span = constant.span;
                            (FileDeclarationKind::Constant(constant), span)
                        }
                    };
                    items.push(FileItem::Declaration(FileDeclaration {
                        program_export: None,
                        visibility,
                        kind,
                        location: SourceSpan {
                            source,
                            span,
                        },
                    }));
                }
                continue;
            }
            let kind = if self.starts_library() {
                FileDeclarationKind::Library(self.library_declaration()?)
            } else if matches!(
                self.nominal_prefix(),
                Some(Keyword::Struct | Keyword::Union)
            ) {
                FileDeclarationKind::Record(self.record_declaration()?)
            } else if matches!(
                self.nominal_prefix(),
                Some(Keyword::Enum | Keyword::EnumFlags)
            ) {
                FileDeclarationKind::Enum(self.enum_declaration()?)
            } else if self.type_alias_prefix() {
                FileDeclarationKind::TypeAlias(self.type_alias_declaration()?)
            } else if self.starts_procedure() {
                self.procedure_declaration()?
            } else {
                let span = self.token().span;
                let name = self.name()?;
                match self.data_declaration(name, span)?.kind {
                    StatementKind::Declare(declaration) => {
                        FileDeclarationKind::Global(GlobalDeclaration {
                            declaration,
                            span,
                        })
                    }
                    StatementKind::Constant(declaration) => {
                        FileDeclarationKind::Constant(declaration)
                    }
                    _ => unreachable!("data declaration produces declaration"),
                }
            };
            let declaration = FileDeclaration {
                program_export: match (&kind, program_export) {
                    (
                        FileDeclarationKind::Procedure(_) | FileDeclarationKind::Global(_),
                        annotation,
                    ) => annotation,
                    (_, Some(annotation)) => {
                        return Err(Diagnostic::new(
                            annotation.span,
                            "#program_export requires a procedure or global definition",
                        ));
                    }
                    (_, None) => None,
                },
                visibility,
                kind,
                location: self.location_from(
                    source,
                    if using_declaration.is_some() {
                        declaration_start
                    } else {
                        start
                    },
                ),
            };
            if let Some((selection, target_span)) = using_declaration {
                items.push(FileItem::UsingDeclaration {
                    declaration,
                    selection,
                    target_span,
                    location: self.location_from(source, start),
                });
            } else {
                items.push(FileItem::Declaration(declaration));
            }
        }
        if nested {
            return Err(self.error("unterminated file declaration block"));
        }
        if count == FileItemCount::One && items.is_empty() {
            return Err(self.error("expected a file declaration or directive in conditional body"));
        }
        Ok(items)
    }
    pub(super) fn import_prefix(&self) -> bool {
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
        let import = self.scoped_import_declaration()?;
        Ok(ImportDeclaration {
            namespace: import.namespace,
            using: import.using,
            visibility,
            mode: import.mode,
            target: import.target,
            arguments: import.arguments,
            location: SourceSpan {
                source,
                span: import.span,
            },
        })
    }
    pub(super) fn scoped_import_declaration(
        &mut self,
    ) -> Result<ScopedImportDeclaration, Diagnostic> {
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
        Ok(ScopedImportDeclaration {
            namespace,
            using,
            mode,
            target,
            arguments: ImportArguments {
                instance,
                program,
            },
            span: Span::new(start, self.tokens[self.at - 1].span.end),
        })
    }
    pub(super) fn module_string(&mut self) -> Result<String, Diagnostic> {
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
        if (self.token().kind == Kind::Ident
            && BuiltinType::from_spelling(&self.token().spelling(self.source)).is_some())
            || self.is(Punct::OpenBracket)
        {
            let start = self.token().span.start;
            let ty = self.type_syntax()?;
            return Ok(ModuleArgumentValue::Expression(Expression {
                kind: ExpressionKind::Type(ty),
                span: Span::new(start, self.tokens[self.at.saturating_sub(1)].span.end),
            }));
        }
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
                let syntax = self.type_syntax()?;
                let ty = if let TypeSyntax::Restricted {
                    variable,
                    restriction: TypeRestrictionSyntax::Interface(constraint),
                    ..
                } = &syntax
                {
                    ModuleParameterType::Interface {
                        variable: *variable,
                        constraint: (**constraint).clone(),
                    }
                } else if let TypeSyntax::Variable(variable) = &syntax {
                    if self.take(Punct::Div) {
                        if !self.keyword(Keyword::Interface) {
                            return Err(self.error(
                                "expected interface constraint after module type variable",
                            ));
                        }
                        ModuleParameterType::Interface {
                            variable: *variable,
                            constraint: self.type_syntax()?,
                        }
                    } else {
                        ModuleParameterType::Unresolved(syntax)
                    }
                } else {
                    match syntax {
                        TypeSyntax::Builtin(BuiltinType::String) => ModuleParameterType::String,
                        TypeSyntax::Builtin(BuiltinType::Scalar(ty)) => {
                            ModuleParameterType::Scalar(ty)
                        }
                        syntax => ModuleParameterType::Unresolved(syntax),
                    }
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

    #[test]
    fn local_imports_retain_lexical_structure_and_real_statement_ranges() {
        let text = "main :: () { #if ENABLE_BACKTRACE_ON_CRASH { Handler :: #import \"Runtime_Support_Crash_Handler\"; Handler.init(); } using Basic :: #import \"Basic\"()(ENABLE_ASSERT=true); }";
        let mut symbols = Symbols::default();
        let parsed = file(text, &mut symbols).unwrap();
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Procedure(procedure),
            ..
        }) = &parsed.items()[0]
        else {
            panic!("expected procedure")
        };
        let StatementKind::CompileTimeIf {
            then_body, ..
        } = &procedure.body[0].kind
        else {
            panic!("expected conditional")
        };
        let statement = &then_body[0];
        let StatementKind::Import(import) = &statement.kind else {
            panic!("expected local import")
        };
        assert_eq!(symbols.name(import.namespace.unwrap()), "Handler");
        assert_eq!(import.target, "Runtime_Support_Crash_Handler");
        assert_eq!(import.span, statement.span);
        assert_eq!(
            statement.span.text(text),
            "Handler :: #import \"Runtime_Support_Crash_Handler\";"
        );
        let StatementKind::Import(import) = &procedure.body[1].kind else {
            panic!("expected using import")
        };
        assert!(import.using);
        assert_eq!(import.arguments.instance.as_ref().unwrap().len(), 0);
        assert_eq!(import.arguments.program.as_ref().unwrap().len(), 1);
        assert_eq!(
            parse("main :: () { Handler :: #import \"Handler\"; }")
                .unwrap_err()
                .message,
            "local imports require module and lexical scope resolution"
        );
    }
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
    fn context_fields_and_top_level_insert_are_distinct_file_items() {
        let text = "#scope_file; #add_context count: int = 33; #add_context callback :: MyCallback; #add_context nested: #Context; #insert,scope() generated; #insert -> string { return \"answer :: 42;\"; }";
        let mut symbols = Symbols::default();
        let parsed = file(text, &mut symbols).unwrap();
        let FileItem::ContextField {
            declaration: ContextFieldDeclaration::Variable(variable),
            location,
        } = &parsed.items()[1]
        else {
            panic!("expected context variable")
        };
        assert_eq!(symbols.name(variable.declaration.name()), "count");
        assert_eq!(location.span.text(text), "#add_context count: int = 33;");
        let FileItem::ContextField {
            declaration: ContextFieldDeclaration::Constant(constant),
            ..
        } = &parsed.items()[2]
        else {
            panic!("expected context constant")
        };
        assert_eq!(symbols.name(constant.name), "callback");
        assert!(matches!(
            &parsed.items()[3],
            FileItem::ContextField {
                declaration: ContextFieldDeclaration::Variable(GlobalDeclaration {
                    declaration: Declaration::UnresolvedExplicit {
                        ty: TypeSyntax::Builtin(BuiltinType::Context),
                        ..
                    },
                    ..
                }),
                ..
            }
        ));
        assert!(matches!(
            &parsed.items()[4],
            FileItem::Insert {
                directive: InsertDirective {
                    scope: InsertScope::Current,
                    ..
                },
                ..
            }
        ));
        assert!(matches!(
            &parsed.items()[5],
            FileItem::Insert {
                directive: InsertDirective {
                    value: Expression {
                        kind: ExpressionKind::CompileTime(CompileTimeRun {
                            body: CompileTimeBody::Procedure { .. },
                            ..
                        }),
                        ..
                    },
                    ..
                },
                ..
            }
        ));
        assert_eq!(
            parsed
                .items()
                .iter()
                .filter(|item| matches!(item, FileItem::Declaration(_)))
                .count(),
            0
        );
    }

    #[test]
    fn unknown_file_directive_is_a_specific_located_error() {
        let text = "#invented_file_directive;";
        let error = file(text, &mut Symbols::default()).unwrap_err();
        assert_eq!(
            error.message,
            "unknown directive '#invented_file_directive'"
        );
        assert_eq!(error.location.span.text(text), "#invented_file_directive");
    }

    #[test]
    fn parameter_defaults_required_values_and_qualified_calls() {
        let mut symbols = Symbols::default();
        let parsed = file("#module_parameters(Count: int = 4, Name: string = \"fallback\", Required: string)(DEBUG := false); Math :: #import \"Math\"; answer :: () -> int { return Math.inner.answer(Count); }", &mut symbols).unwrap();
        let FileItem::Parameters(parameters) = &parsed.items()[0] else {
            panic!()
        };
        assert_eq!(parameters.instance.len(), 3);
        assert!(matches!(
            parameters.instance[1].ty,
            ModuleParameterType::String
        ));
        assert!(parameters.instance[2].default.is_none());
        assert!(matches!(
            parameters.program.as_ref().unwrap()[0].ty,
            ModuleParameterType::Inferred
        ));
        let FileItem::Declaration(d) = &parsed.items()[2] else {
            panic!()
        };
        let FileDeclarationKind::Procedure(procedure) = &d.kind else {
            panic!()
        };
        let StatementKind::Return(Some(expression)) = &procedure.body[0].kind else {
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
            "#module_parameters(x := true); #module_parameters(y := false);",
            "#import \"Math\"()()();",
            "main :: () { #import 7; }",
        ] {
            assert!(file(unsupported, &mut symbols).is_err(), "{unsupported}");
        }
    }
}
