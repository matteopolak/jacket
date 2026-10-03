//! LLVM locations and variables driven by checked source provenance.
#[cfg(test)]
mod artificial_calls_tests;
mod procedure;
mod types;
mod variables;
use crate::optimization::{BitcodeOptimization, Optimization};
use inkwell::{
    builder::Builder,
    context::{Context, ContextRef},
    module::Module,
    values::FunctionValue,
};
use jai_ir::{DebugSourceLocation, DebugSources, ProcedureId};
use jai_llvm::{DebugEmission, DebugSession, DebugSource};
pub use jai_types::DebugInformation;
pub(crate) use procedure::State;
use std::{fmt, path::Path};
pub use variables::{Scope, VariableKind, VariableRecord};

pub struct LineTables<'ctx, 'module> {
    session: DebugSession<'module, 'ctx>,
    information: DebugInformation,
    policy: jai_types::LayoutPolicy,
    runtime_types:
        std::cell::RefCell<std::collections::HashMap<jai_types::TypeId, jai_llvm::DebugType>>,
    runtime_signatures:
        std::cell::RefCell<std::collections::HashMap<jai_types::TypeId, jai_llvm::DebugType>>,
    context: ContextRef<'ctx>,
    procedures: Option<std::collections::HashSet<ProcedureId>>,
}
impl<'ctx, 'module> LineTables<'ctx, 'module> {
    pub fn new(
        module: &'module Module<'ctx>,
        context: &'ctx Context,
        sources: &DebugSources,
        optimization: Optimization,
    ) -> Result<Option<Self>, Error> {
        Self::new_with_information(
            module,
            context,
            sources,
            optimization,
            DebugInformation::LineTables,
        )
    }
    pub fn new_with_information(
        module: &'module Module<'ctx>,
        context: &'ctx Context,
        sources: &DebugSources,
        optimization: Optimization,
        information: DebugInformation,
    ) -> Result<Option<Self>, Error> {
        Self::new_with_policy(
            module,
            context,
            sources,
            optimization,
            information,
            jai_types::LayoutPolicy::lp64(),
        )
    }
    pub fn new_with_policy(
        module: &'module Module<'ctx>,
        context: &'ctx Context,
        sources: &DebugSources,
        optimization: Optimization,
        information: DebugInformation,
        policy: jai_types::LayoutPolicy,
    ) -> Result<Option<Self>, Error> {
        Self::new_selected(
            module,
            context,
            sources,
            optimization,
            information,
            policy,
            None,
        )
    }
    /// Select only source procedures whose native bodies will actually be emitted.
    pub fn new_for_procedures(
        module: &'module Module<'ctx>,
        context: &'ctx Context,
        sources: &DebugSources,
        optimization: Optimization,
        information: DebugInformation,
        policy: jai_types::LayoutPolicy,
        procedures: impl IntoIterator<Item = ProcedureId>,
    ) -> Result<Option<Self>, Error> {
        Self::new_selected(
            module,
            context,
            sources,
            optimization,
            information,
            policy,
            Some(procedures.into_iter().collect()),
        )
    }
    fn new_selected(
        module: &'module Module<'ctx>,
        context: &'ctx Context,
        sources: &DebugSources,
        optimization: Optimization,
        information: DebugInformation,
        policy: jai_types::LayoutPolicy,
        procedures: Option<std::collections::HashSet<ProcedureId>>,
    ) -> Result<Option<Self>, Error> {
        if information == DebugInformation::Off {
            return Ok(None);
        }
        if module.get_context() != *context {
            return Err(Error::Ownership);
        }
        let Some((_, source)) = sources
            .procedures()
            .filter(|(id, _)| {
                sources.procedure_policy(*id).emits()
                    && procedures.as_ref().is_none_or(|ids| ids.contains(id))
            })
            .min_by_key(|(id, _)| id.index())
        else {
            return Ok(None);
        };
        let primary = sources
            .primary_path()
            .unwrap_or_else(|| source.location.path());
        let (file, directory) = path_parts(primary)?;
        let optimized = !matches!(
            optimization.bitcode,
            BitcodeOptimization::Unset | BitcodeOptimization::O0
        ) || optimization.machine_level() != inkwell::OptimizationLevel::None;
        module.set_source_file_name(&primary.to_string_lossy());
        let session = DebugSession::new(
            module,
            context,
            file,
            directory,
            "jai-rs",
            optimized,
            match information {
                DebugInformation::Variables => DebugEmission::Variables,
                DebugInformation::LineTables => DebugEmission::LineTables,
                DebugInformation::Off => unreachable!("off mode returns before metadata"),
            },
        )
        .map_err(bridge_error)?;
        Ok(Some(Self {
            session,
            information,
            policy,
            runtime_types: std::cell::RefCell::new(std::collections::HashMap::new()),
            runtime_signatures: std::cell::RefCell::new(std::collections::HashMap::new()),
            context: module.get_context(),
            procedures,
        }))
    }
    pub fn attach(
        &self,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
        function: FunctionValue<'ctx>,
        id: ProcedureId,
        sources: &DebugSources,
    ) -> Result<(), Error> {
        self.attach_scope(context, builder, function, id, sources)
            .map(|_| ())
    }
    pub fn attach_scope(
        &self,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
        function: FunctionValue<'ctx>,
        id: ProcedureId,
        sources: &DebugSources,
    ) -> Result<Option<Scope<'ctx>>, Error> {
        if !sources.procedure_policy(id).emits() {
            return Ok(None);
        }
        if self
            .procedures
            .as_ref()
            .is_some_and(|ids| !ids.contains(&id))
        {
            return Ok(None);
        }
        let Some(source) = sources.procedure(id) else {
            return Ok(None);
        };
        if !self.session.context_matches(context) {
            return Err(Error::Ownership);
        }
        let metadata = self
            .session
            .function_scope(
                builder,
                function,
                source_descriptor(&source.location)?,
                &source.name,
            )
            .map_err(bridge_error)?;
        Ok(Some(Scope {
            metadata,
            source: source.location.span().source,
        }))
    }
    pub(crate) fn attach_typed_scope(
        &self,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
        function: FunctionValue<'ctx>,
        id: ProcedureId,
        sources: &DebugSources,
        signature: (jai_types::TypeId, &dyn jai_types::TypeView),
    ) -> Result<Option<Scope<'ctx>>, Error> {
        let scope = self.attach_scope(context, builder, function, id, sources)?;
        if let Some(scope) = scope {
            self.runtime_type(signature.0, signature.1, Some(sources))?;
            if let Some(&signature) = self.runtime_signatures.borrow().get(&signature.0) {
                self.session
                    .set_function_type(scope.metadata, signature)
                    .map_err(bridge_error)?;
            }
        }
        Ok(scope)
    }
    pub fn finish(self) {
        self.session.finish();
    }
}
fn source_descriptor(source: &DebugSourceLocation) -> Result<DebugSource<'_>, Error> {
    let (file, directory) = path_parts(source.path())?;
    Ok(DebugSource {
        file,
        directory,
        line: source.line(),
        column: source.column(),
    })
}
fn bridge_error(error: jai_llvm::Error) -> Error {
    match error {
        jai_llvm::Error::ContextMismatch | jai_llvm::Error::DebugOwnership => Error::Ownership,
        jai_llvm::Error::Build(inkwell::builder::BuilderError::UnsetPosition) => {
            Error::MissingInsertionBlock
        }
        other => Error::Bridge(other),
    }
}
fn path_parts(path: &Path) -> Result<(&str, &str), Error> {
    let file = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(Error::NonUtf8Path)?;
    let directory = path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_str()
        .ok_or(Error::NonUtf8Path)?;
    if file.contains('\0') || directory.contains('\0') {
        return Err(Error::InvalidPath);
    }
    Ok((file, directory))
}
#[derive(Debug)]
pub enum Error {
    InvalidPath,
    NonUtf8Path,
    NonUtf8Linkage,
    Type(jai_types::TypeError),
    Metadata(String),
    MissingInsertionBlock,
    Ownership,
    Bridge(jai_llvm::Error),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Type(error) => return error.fmt(f),
            Self::Metadata(error) => return f.write_str(error),
            Self::Bridge(error) => return error.fmt(f),
            _ => {}
        }
        f.write_str(match self {
            Self::InvalidPath => "debug source path cannot contain a NUL byte",
            Self::NonUtf8Path => "debug source path must be UTF-8",
            Self::NonUtf8Linkage => "debug linkage name must be UTF-8",
            Self::MissingInsertionBlock => "debug declaration needs an active LLVM block",
            Self::Ownership => "debug metadata belongs to another builder or LLVM context",
            Self::Type(_) | Self::Metadata(_) | Self::Bridge(_) => unreachable!(),
        })
    }
}
impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_ir::{DebugSourceLocation, ProcedureSource};
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };
    pub(super) fn sources() -> DebugSources {
        let mut map = jai_source::SourceMap::default();
        let id = map.insert(
            "/own-source/debug-fixture.jai".into(),
            "// Own debug fixture.\nmain :: () -> int { return 42; }\n".into(),
        );
        let source = map.get(id).unwrap();
        let at = source.text().find("main").unwrap();
        let location = DebugSourceLocation::from_source(
            source,
            jai_source::SourceSpan {
                source: id,
                span: jai_source::Span::new(at, source.text().len()),
            },
        )
        .unwrap();
        let mut debug = DebugSources::default();
        debug.retain_source(source);
        debug.insert(
            ProcedureId::new(27),
            ProcedureSource {
                name: "main".into(),
                location,
            },
        );
        debug
    }
    fn fixture<'ctx>(context: &'ctx Context, sources: &DebugSources) -> Module<'ctx> {
        let module = context.create_module("own.debug.fixture");
        let tables = LineTables::new(&module, context, sources, Optimization::default())
            .unwrap()
            .unwrap();
        let function = module.add_function("jai.p27", context.i32_type().fn_type(&[], false), None);
        let builder = context.create_builder();
        builder.position_at_end(context.append_basic_block(function, "entry"));
        tables
            .attach(context, &builder, function, ProcedureId::new(27), sources)
            .unwrap();
        builder
            .build_return(Some(&context.i32_type().const_int(42, false)))
            .unwrap();
        tables.finish();
        module.verify().unwrap();
        module
    }
    #[test]
    fn source_line_tables_use_source_spelling_and_sparse_procedure_identity() {
        let context = Context::create();
        let sources = sources();
        let module = fixture(&context, &sources);
        let text = module.print_to_string().to_string();
        assert!(text.contains("emissionKind: LineTablesOnly"));
        assert!(text.contains("name: \"main\", linkageName: \"jai.p27\""));
        assert!(text.contains("DILocation(line: 2, column: 1"));
        assert!(text.contains("filename: \"debug-fixture.jai\", directory: \"/own-source\""));
        assert_eq!(
            module.get_source_file_name().to_str().unwrap(),
            "/own-source/debug-fixture.jai"
        );
    }
    #[test]
    fn invalid_debug_source_path_fails_before_llvm_string_conversion() {
        assert!(matches!(
            path_parts(Path::new("own/bad\0source.jai")),
            Err(Error::InvalidPath)
        ));
    }
    #[test]
    fn empty_provenance_emits_no_fabricated_source_debug_unit() {
        let context = Context::create();
        let module = context.create_module("own.empty.debug");
        assert!(
            LineTables::new(
                &module,
                &context,
                &DebugSources::default(),
                Optimization::default()
            )
            .unwrap()
            .is_none()
        );
        assert!(
            !module
                .print_to_string()
                .to_string()
                .contains("DICompileUnit")
        );
    }
    #[test]
    fn suppressed_procedure_provenance_cannot_recreate_debug_metadata() {
        let context = Context::create();
        let module = context.create_module("own.suppressed.debug");
        let mut sources = sources();
        sources.set_procedure_policy(ProcedureId::new(27), jai_ir::DebugPolicy::Suppress);
        assert!(
            LineTables::new(&module, &context, &sources, Optimization::default())
                .unwrap()
                .is_none()
        );
        let visible = sources.procedure(ProcedureId::new(27)).unwrap().clone();
        sources.insert(ProcedureId::new(89), visible);
        assert!(
            LineTables::new_for_procedures(
                &module,
                &context,
                &sources,
                Optimization::default(),
                DebugInformation::Variables,
                jai_types::LayoutPolicy::lp64(),
                [ProcedureId::new(27)],
            )
            .unwrap()
            .is_none(),
            "an unselected visible source cannot create a compile unit"
        );
        let tables = LineTables::new(&module, &context, &sources, Optimization::default())
            .unwrap()
            .unwrap();
        let function =
            module.add_function("suppressed", context.i32_type().fn_type(&[], false), None);
        let builder = context.create_builder();
        builder.position_at_end(context.append_basic_block(function, "entry"));
        assert!(
            tables
                .attach_scope(&context, &builder, function, ProcedureId::new(27), &sources)
                .unwrap()
                .is_none()
        );
        builder
            .build_return(Some(&context.i32_type().const_int(42, false)))
            .unwrap();
        tables.finish();
        module.verify().unwrap();
        assert!(
            !module
                .print_to_string()
                .to_string()
                .contains("DISubprogram")
        );
    }
    #[test]
    fn emitted_object_contains_inspectable_dwarf_line_tables() {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        struct Scratch(PathBuf);
        impl Drop for Scratch {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
        let scratch = Scratch(std::env::temp_dir().join(format!(
            "jai-debug-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )));
        fs::create_dir(&scratch.0).unwrap();
        let context = Context::create();
        let module = fixture(&context, &sources());
        let object = scratch.0.join("debug.o");
        crate::target::NativeTarget::new()
            .unwrap()
            .write_object(&module, &object)
            .unwrap();
        let output = variables::tests::debug_tool_command("llvm-dwarfdump")
            .arg("--debug-line")
            .arg("--debug-info")
            .arg(&object)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let dwarf = String::from_utf8(output.stdout).unwrap();
        assert!(dwarf.contains("debug-fixture.jai"), "{dwarf}");
        assert!(dwarf.contains("/own-source"), "{dwarf}");
        assert!(dwarf.contains("Line table prologue"), "{dwarf}");
        assert!(
            dwarf.lines().any(|line| {
                let columns: Vec<_> = line.split_whitespace().collect();
                columns
                    .first()
                    .is_some_and(|address| address.starts_with("0x"))
                    && columns.get(1) == Some(&"2")
                    && columns.get(2) == Some(&"1")
            }),
            "{dwarf}"
        );
    }
}
