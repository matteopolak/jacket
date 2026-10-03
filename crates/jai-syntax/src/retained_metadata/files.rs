//! Visit actual file owners; independently cloned declarations are separate roots.
use super::*;

impl<A> Visitor<'_, A> {
    pub(super) fn file_declaration<E>(
        &mut self,
        source: &FileDeclaration,
        depth: usize,
    ) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.node(depth)?;
        if let Some(export) = &source.program_export
            && let Some(symbol) = &export.symbol
        {
            self.text(symbol)?;
        }
        match &source.kind {
            FileDeclarationKind::Placeholder(_) => Ok(()),
            FileDeclarationKind::Library(value) => self.text(&value.target),
            FileDeclarationKind::Procedure(value) => {
                self.source_procedure(&value.source, depth + 1)
            }
            FileDeclarationKind::OperatorAlias(value) => {
                self.path(&value.target_namespace, depth + 1)
            }
            FileDeclarationKind::ProcedurePrototype(value) => self.prototype(value, depth + 1),
            FileDeclarationKind::Global(value) => self.declaration(&value.declaration, depth + 1),
            FileDeclarationKind::Constant(value) => self.constant(value, depth + 1),
            FileDeclarationKind::Record(value) => self.record(value, depth + 1),
            FileDeclarationKind::Enum(value) => self.enumeration(value, depth + 1),
            FileDeclarationKind::TypeAlias(value) => self.ty(&value.ty, depth + 1),
        }
    }

    pub(super) fn file_item<E>(&mut self, source: &FileItem, depth: usize) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.node(depth)?;
        match source {
            FileItem::Scope {
                ..
            } => Ok(()),
            FileItem::Insert {
                directive, ..
            } => self.insert(directive, depth + 1),
            FileItem::ContextField {
                declaration, ..
            } => self.context_field(declaration, depth + 1),
            FileItem::Import(value) => {
                self.text(&value.target)?;
                self.import_arguments(&value.arguments, depth + 1)
            }
            FileItem::Using {
                directive, ..
            } => {
                self.expression(&directive.target, depth + 1)?;
                self.selection(&directive.selection, depth + 1)
            }
            FileItem::UsingDeclaration {
                declaration,
                selection,
                ..
            } => {
                self.file_declaration(declaration, depth + 1)?;
                self.selection(selection, depth + 1)
            }
            FileItem::Load(value) => self.text(&value.target),
            FileItem::Run(value) => self.run_body(&value.body, depth + 1),
            FileItem::Parameters(value) => {
                self.sequence(&value.instance, depth, Self::module_parameter)?;
                if let Some(program) = &value.program {
                    self.sequence(program, depth, Self::module_parameter)?;
                }
                self.sequence(&value.declarations, depth, Self::file_item)
            }
            FileItem::Assert {
                condition,
                message,
                ..
            } => self.assertion(condition, message, depth),
            FileItem::CompileTimeCases {
                cases, ..
            } => self.source_cases(cases, depth, Self::file_item),
            FileItem::Conditional {
                condition,
                then_items,
                else_items,
                ..
            } => {
                self.expression(condition, depth + 1)?;
                self.sequence(then_items, depth, Self::file_item)?;
                self.sequence(else_items, depth, Self::file_item)
            }
            FileItem::Declaration(value) => self.file_declaration(value, depth + 1),
        }
    }

    fn module_parameter<E>(&mut self, source: &ModuleParameter, depth: usize) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.node(depth)?;
        match &source.ty {
            ModuleParameterType::Inferred
            | ModuleParameterType::Scalar(_)
            | ModuleParameterType::String => {}
            ModuleParameterType::Interface {
                constraint, ..
            }
            | ModuleParameterType::Unresolved(constraint) => self.ty(constraint, depth + 1)?,
        }
        if let Some(default) = &source.default {
            match default {
                ModuleArgumentValue::Expression(value) => self.expression(value, depth + 1)?,
                ModuleArgumentValue::String(value) => self.text(value)?,
            }
        }
        Ok(())
    }
}

impl ParsedFile {
    /// Count the real allocated item vector and every owned descendant.
    pub fn visit_retained_metadata<E>(
        &self,
        admit: &mut impl FnMut(usize, usize) -> std::result::Result<(), E>,
    ) -> std::result::Result<(), SourceMetadataError<E>> {
        Visitor {
            admit,
        }
        .sequence(self.retained_items(), 0, Visitor::file_item)
    }
}

macro_rules! metadata_entry {
    ($ty:ty, $method:ident) => {
        impl $ty {
            /// Visit owned descendants; the containing allocation owns this inline row.
            pub fn visit_retained_metadata<E>(
                &self,
                admit: &mut impl FnMut(usize, usize) -> std::result::Result<(), E>,
            ) -> std::result::Result<(), SourceMetadataError<E>> {
                Visitor {
                    admit,
                }
                .$method(self, 0)
            }
        }
    };
}
metadata_entry!(FileDeclaration, file_declaration);
metadata_entry!(FileItem, file_item);
metadata_entry!(ContextFieldDeclaration, context_field);
metadata_entry!(SourceProcedureSyntax, source_procedure);
metadata_entry!(ProcedurePrototype, prototype);
metadata_entry!(Expression, expression);
metadata_entry!(CodeBody, code);
metadata_entry!(Statement, statement);
metadata_entry!(RecordMember, record_member);
metadata_entry!(RecordDeclaration, record);
metadata_entry!(RecordTypeSyntax, inline_record);
metadata_entry!(FieldDeclaration, field);
metadata_entry!(TypeSyntax, ty);
metadata_entry!(ConstantDeclaration, constant);
metadata_entry!(Declaration, declaration);
metadata_entry!(InsertDirective, insert);

impl Procedure {
    pub fn visit_retained_metadata<E>(
        &self,
        admit: &mut impl FnMut(usize, usize) -> std::result::Result<(), E>,
    ) -> std::result::Result<(), SourceMetadataError<E>> {
        self.source.visit_retained_metadata(admit)
    }
}
impl RunDirective {
    pub fn visit_retained_metadata<E>(
        &self,
        admit: &mut impl FnMut(usize, usize) -> std::result::Result<(), E>,
    ) -> std::result::Result<(), SourceMetadataError<E>> {
        Visitor {
            admit,
        }
        .run_body(&self.body, 0)
    }
}
