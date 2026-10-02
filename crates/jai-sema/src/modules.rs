//! Resolve declaration identities through the defining file's namespace.
use super::*;
use jai_modules::{FileInstanceId, ModuleGraph};
use jai_source::{DeclarationId, LocatedDiagnostic, SourceSpan};
use syntax::{FileDeclarationKind, NamePath};
mod constants;
mod scope;
use constants::Constants;
pub(super) use scope::FileScope;
use scope::{declaration_id, located, path};
#[cfg(test)]
mod tests;

struct ScopedDeclarations<'a> {
    graph: &'a ModuleGraph,
    values: HashMap<DeclarationId, Binding>,
    signatures: HashMap<DeclarationId, Signature>,
}
/// Checked declarations without an executable entry-point requirement.
#[derive(Debug)]
pub struct Library {
    procedures: Vec<Procedure>,
    globals: Vec<Global>,
    types: jai_types::Types,
    procedure_ids: HashMap<DeclarationId, ProcedureId>,
}
impl Library {
    pub fn procedures(&self) -> &[Procedure] {
        &self.procedures
    }
    pub fn globals(&self) -> &[Global] {
        &self.globals
    }
    pub fn types(&self) -> &jai_types::Types {
        &self.types
    }
    pub fn procedure(&self, declaration: DeclarationId) -> Option<&Procedure> {
        self.procedure_ids
            .get(&declaration)
            .and_then(|id| self.procedures.get(id.index()))
    }
}
/// Type-check all reachable declarations without inventing a library entry point.
pub fn resolve_library(graph: &ModuleGraph) -> Result<Library, LocatedDiagnostic> {
    let mut types = TypeRegistry::new();
    let mut constants = Constants::new(graph);
    let mut declarations = ScopedDeclarations {
        graph,
        values: HashMap::new(),
        signatures: HashMap::new(),
    };
    for declaration in graph.declarations() {
        if matches!(
            declaration.syntax().kind,
            FileDeclarationKind::Record(_) | FileDeclarationKind::Enum(_)
        ) {
            return Err(located(
                graph,
                declaration.file(),
                Diagnostic::new(
                    declaration.location().span,
                    "aggregate declaration lowering is not implemented",
                ),
            ));
        }
        if matches!(declaration.syntax().kind, FileDeclarationKind::Constant(_)) {
            let value = constants.value(declaration.id(), declaration.location())?;
            declarations
                .values
                .insert(declaration.id(), Binding::Constant(value));
        }
    }
    let mut globals = Vec::new();
    for declaration in graph.declarations() {
        let FileDeclarationKind::Global(global) = &declaration.syntax().kind else {
            continue;
        };
        let file = declaration.file();
        let value = match &global.declaration {
            syntax::Declaration::Inferred { initializer, .. } => {
                constants.evaluate(file, initializer)?
            }
            syntax::Declaration::Explicit {
                ty, initializer, ..
            } => {
                let value = match initializer {
                    Some(expression) => constants.evaluate(file, expression)?,
                    None => ConstantValue::zero(*ty),
                };
                value
                    .coerce(*ty, global.span)
                    .map_err(|error| located(graph, file, error))?
            }
            #[allow(unreachable_patterns)]
            _ => {
                return Err(located(
                    graph,
                    file,
                    Diagnostic::new(global.span, "global type is not implemented"),
                ));
            }
        };
        let value = value
            .coerce(value.ty(), global.span)
            .map_err(|error| located(graph, file, error))?;
        let initializer = match value {
            ConstantValue::Int(value) => GlobalInitializer::Int(value),
            ConstantValue::Bool(value) => GlobalInitializer::Bool(value),
            ConstantValue::Literal(_) => unreachable!("global initializer is materialized"),
        };
        let global = Global::new(globals.len(), initializer, &types);
        declarations
            .values
            .insert(declaration.id(), Binding::Storage(global.storage()));
        globals.push(global);
    }
    for declaration in graph.declarations() {
        let FileDeclarationKind::Procedure(procedure) = &declaration.syntax().kind else {
            continue;
        };
        let file = declaration.file();
        let parameters = calls::parameters_paths(procedure, |path, span| {
            let id = declaration_id(graph, file, path, span)?;
            constants
                .ready(
                    id,
                    SourceSpan {
                        source: declaration.location().source,
                        span,
                    },
                )
                .map_err(|error| Diagnostic::new(error.location.span, error.message))
        })
        .map_err(|error| located(graph, file, error))?;
        let results = match procedure.return_type {
            ReturnType::Void => vec![],
            ReturnType::Value(ty) => vec![types.scalar(ty)],
        };
        let ty = types
            .procedure(ProcedureType {
                parameters: parameters
                    .iter()
                    .map(|parameter| types.scalar(parameter.ty))
                    .collect(),
                results: results.into_boxed_slice(),
                convention: CallingConvention::Jai,
                context: ContextMode::None,
            })
            .map_err(|error| {
                located(
                    graph,
                    file,
                    Diagnostic::new(procedure.span, error.to_string()),
                )
            })?;
        declarations.signatures.insert(
            declaration.id(),
            Signature {
                id: ProcedureId(declarations.signatures.len()),
                ty,
                parameters,
                result: procedure.return_type,
            },
        );
    }
    let root_file = graph.module(graph.root()).unwrap().entry();
    let empty_signatures = HashMap::new();
    let empty_values = HashMap::new();
    let mut procedures = Vec::new();
    for declaration in graph.declarations() {
        let FileDeclarationKind::Procedure(procedure) = &declaration.syntax().kind else {
            continue;
        };
        let signature = &declarations.signatures[&declaration.id()];
        let file = declaration.file();
        let mut resolver = Resolver {
            procedure: signature.id,
            types: &types,
            signatures: &empty_signatures,
            globals: &empty_values,
            graph_scope: Some(FileScope {
                declarations: &declarations,
                file,
            }),
            symbols: graph.symbols(),
            scopes: vec![HashMap::new()],
            locals: Vec::new(),
            span: procedure.span,
            result: procedure.return_type,
            loops: Vec::new(),
            next_loop: 0,
            cleanups: Vec::new(),
            deferred_scopes: Vec::new(),
            cleanup_context: None,
        };
        let result = (|| {
            let mut parameters = Vec::new();
            for parameter in &signature.parameters {
                parameters.push(resolver.declare(parameter.name, parameter.ty)?);
            }
            let body = resolver.block(&procedure.body, false)?;
            if procedure.return_type != ReturnType::Void && body.flow != Flow::Terminates {
                return Err(Diagnostic::new(
                    procedure.span,
                    "value-returning procedure may reach its end",
                ));
            }
            Ok(Procedure {
                id: signature.id,
                signature: signature.ty,
                parameters,
                locals: resolver.locals,
                cleanups: resolver.cleanups,
                body,
            })
        })()
        .map_err(|error| located(graph, file, error))?;
        procedures.push(result);
    }
    let types = types.freeze().map_err(|error| {
        located(
            graph,
            root_file,
            Diagnostic::new(Span::default(), error.to_string()),
        )
    })?;
    let procedure_ids = declarations
        .signatures
        .iter()
        .map(|(declaration, signature)| (*declaration, signature.id))
        .collect();
    Ok(Library {
        procedures,
        globals,
        types,
        procedure_ids,
    })
}
/// Resolve an executable whose main declaration belongs to the application module.
pub fn resolve_graph(graph: &ModuleGraph) -> Result<Program, LocatedDiagnostic> {
    let library = resolve_library(graph)?;
    let root_file = graph.module(graph.root()).unwrap().entry();
    let main = graph
        .symbols()
        .find("main")
        .and_then(|name| declaration_id(graph, root_file, &path(name), Span::default()).ok());
    let main = main
        .filter(|id| {
            graph
                .file(graph.declaration(*id).unwrap().file())
                .unwrap()
                .module()
                == graph.root()
        })
        .and_then(|id| {
            library
                .procedure(id)
                .map(|procedure| (graph.declaration(id).unwrap(), procedure))
        })
        .ok_or_else(|| {
            located(
                graph,
                root_file,
                Diagnostic::new(Span::default(), "no application main procedure"),
            )
        })?;
    let FileDeclarationKind::Procedure(syntax) = &main.0.syntax().kind else {
        unreachable!()
    };
    if !main.1.parameters.is_empty() {
        return Err(located(
            graph,
            main.0.file(),
            Diagnostic::new(syntax.span, "main cannot take parameters"),
        ));
    }
    let entry = match syntax.return_type {
        ReturnType::Void => EntryPoint::Void(main.1.id),
        ReturnType::Value(ScalarType::Int(IntegerType::S64)) => EntryPoint::Int(main.1.id),
        _ => {
            return Err(located(
                graph,
                main.0.file(),
                Diagnostic::new(syntax.span, "main must return int or void"),
            ));
        }
    };
    Ok(Program {
        procedures: library.procedures,
        globals: library.globals,
        entry,
        types: library.types,
    })
}
