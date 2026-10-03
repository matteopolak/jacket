//! Compiler quotation receipts use checked plan slots, never a native owner.
use super::*;
use jai_types::TypeId;
use jai_vm::{CompilerCodePlan, CompilerCodePlanId, CompilerReturnSiteId, CompilerSlotId};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
pub(crate) struct CompilerQuoteSource {
    pub(crate) file: FileInstanceId,
    pub(crate) source_file: FileInstanceId,
    pub(crate) location: SourceSpan,
    pub(crate) checks: crate::safety_checks::ActiveChecks,
    pub(crate) debug: jai_types::DebugPolicy,
    pub(crate) origins: Vec<SourceSpan>,
}

#[derive(Clone)]
pub(crate) enum CompilerQuoteBinding {
    Static(Binding),
    /// An actual enclosing source place. Compiler execution never reads it;
    /// procedural quotation insertion rebinds the original source capture.
    Lexical(crate::Storage),
    Native {
        slot: CompilerSlotId,
        ty: TypeId,
    },
}

pub(crate) struct CompilerQuoteTemplate {
    plan: CompilerCodePlanId,
    site: CompilerReturnSiteId,
    body: syntax::CodeBody,
    source: CompilerQuoteSource,
    frames: Box<[Box<[(Symbol, CompilerQuoteBinding)]>]>,
    captures: Box<[CompilerSlotId]>,
    source_retention: Rc<RefCell<super::CompilerQuoteBudget>>,
}

impl CompilerQuoteTemplate {
    #[cfg(test)]
    pub(crate) fn checked(
        plan: &CompilerCodePlan,
        site: CompilerReturnSiteId,
        body: &syntax::CodeBody,
        source: CompilerQuoteSource,
        frames: Vec<Vec<(Symbol, CompilerQuoteBinding)>>,
    ) -> Result<Self, Diagnostic> {
        Self::checked_in_session(plan, site, body, source, frames, Default::default())
    }

    pub(crate) fn checked_in_session(
        plan: &CompilerCodePlan,
        site: CompilerReturnSiteId,
        body: &syntax::CodeBody,
        source: CompilerQuoteSource,
        frames: Vec<Vec<(Symbol, CompilerQuoteBinding)>>,
        source_retention: Rc<RefCell<super::CompilerQuoteBudget>>,
    ) -> Result<Self, Diagnostic> {
        let fail = |message| Diagnostic::at_source(source.location, message);
        let captures = plan
            .return_site_captures(site)
            .ok_or_else(|| fail("compiler quotation return site belongs to another plan"))?;
        if source.location.span.start > source.location.span.end {
            return Err(fail("compiler quotation source range is invalid"));
        }
        if frames.len() > 128 || source.origins.len() > 128 {
            return Err(fail("compiler quotation capture exceeds its scope limit"));
        }
        let allowed: HashSet<_> = captures.iter().copied().collect();
        let mut used = HashSet::new();
        let mut retained = source.origins.len();
        for frame in &frames {
            lexical_keys::charge(
                &mut retained,
                frame.len(),
                lexical_keys::MAX_LEXICAL_NODES,
                source.location.span,
            )?;
            let mut names = HashSet::new();
            for (name, binding) in frame {
                if !names.insert(*name) {
                    return Err(fail(
                        "compiler quotation has duplicate names in one lexical frame",
                    ));
                }
                match binding {
                    CompilerQuoteBinding::Native { slot, ty } => {
                        if !allowed.contains(slot)
                            || plan
                                .slot_schema(*slot)
                                .is_none_or(|schema| schema.ty != *ty)
                        {
                            return Err(fail(
                                "compiler quotation native capture has a foreign slot or type",
                            ));
                        }
                        used.insert(*slot);
                    }
                    CompilerQuoteBinding::Static(
                        Binding::Storage(_)
                        | Binding::Discarded(_)
                        | Binding::LambdaPreview(_)
                        | Binding::CompilerInput { .. },
                    ) => {
                        return Err(fail(
                            "compiler quotation static facts cannot contain runtime or preview storage",
                        ));
                    }
                    CompilerQuoteBinding::Static(_) => {}
                    CompilerQuoteBinding::Lexical(_) => {}
                }
            }
        }
        if used != allowed {
            return Err(fail(
                "compiler quotation does not retain the selected return site's exact native captures",
            ));
        }
        source_retention
            .borrow_mut()
            .admit(body, source.location.span)?;
        Ok(Self {
            plan: plan.id(),
            site,
            body: body.clone(),
            source,
            frames: frames.into_iter().map(Vec::into_boxed_slice).collect(),
            captures: captures.into(),
            source_retention,
        })
    }

    pub(crate) fn plan(&self) -> CompilerCodePlanId {
        self.plan
    }
    pub(crate) fn site(&self) -> CompilerReturnSiteId {
        self.site
    }
    pub(crate) fn body(&self) -> &syntax::CodeBody {
        &self.body
    }
    pub(crate) fn source(&self) -> &CompilerQuoteSource {
        &self.source
    }
    pub(crate) fn frames(&self) -> &[Box<[(Symbol, CompilerQuoteBinding)]>] {
        &self.frames
    }
    pub(crate) fn captures(&self) -> &[CompilerSlotId] {
        &self.captures
    }

    pub(crate) fn admit_publication_body(&self) -> Result<(), Diagnostic> {
        self.source_retention
            .borrow_mut()
            .admit(&self.body, self.source.location.span)
    }

    pub(crate) fn admit_publication_scope(
        &self,
        lexical: &crate::local_declarations::LocalScopes,
        substitution: Option<&crate::polymorphism::Substitution>,
    ) -> Result<(), Diagnostic> {
        let mut budget = self.source_retention.borrow_mut();
        let span = self.source.location.span;
        lexical.admit_compiler_clone(&mut budget, span)?;
        budget.retain_metadata(self.source.origins.len().saturating_mul(2), 0, span)?;
        if let Some(substitution) = substitution {
            // Scope and canonical capture key each own their substitution.
            budget.admit_substitution(substitution, span)?;
            budget.admit_substitution(substitution, span)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
    use jai_types::{Integer, IntegerType, ScalarType, TypeRegistry};
    use jai_vm::{CompilerCodePlanBuilder, CompilerRuntimeLeaf};
    use std::path::Path;

    fn quotation() -> (ModuleGraph, CompilerQuoteSource, syntax::CodeBody) {
        let path = Path::new("/own-compiler-quotation/main.jai");
        let mut sources = SourceOverlay::new();
        sources
            .insert(
                path,
                b"quoted :: #code { Answer :: captured; } main :: () -> int { return 42; }"
                    .to_vec(),
            )
            .unwrap();
        let graph =
            ModuleGraph::load_with_provider(path, GraphOptions::default(), &sources).unwrap();
        let declaration = &graph.declarations()[0];
        let syntax::FileDeclarationKind::Constant(constant) = &declaration.syntax().kind else {
            panic!("source quotation constant")
        };
        let syntax::ExpressionKind::Code(body) = &constant.initializer.kind else {
            panic!("original quotation")
        };
        let source = CompilerQuoteSource {
            file: declaration.file(),
            source_file: declaration.file(),
            location: SourceSpan {
                source: declaration.location().source,
                span: constant.initializer.span,
            },
            checks: crate::safety_checks::ActiveChecks {
                array_bounds: jai_ir::CheckMode::Disabled,
                ..Default::default()
            },
            debug: jai_types::DebugPolicy::Suppress,
            origins: vec![declaration.location()],
        };
        let body = body.clone();
        (graph, source, body)
    }

    fn plan(types: &TypeRegistry) -> (CompilerCodePlan, CompilerReturnSiteId, CompilerSlotId) {
        let ty = types.scalar(ScalarType::Int(IntegerType::S64));
        let mut builder = CompilerCodePlanBuilder::new(Default::default()).unwrap();
        let slot = builder.slot(ty).unwrap();
        let initialized = builder
            .assign(
                slot,
                CompilerRuntimeLeaf::expression(jai_ir::ValueExpr::Int(jai_ir::IntExpr::constant(
                    Integer::checked(IntegerType::S64, 42).unwrap(),
                ))),
            )
            .unwrap();
        let site = builder
            .reserve_return_site_with_captures(vec![slot])
            .unwrap();
        let returned = builder.return_code(site).unwrap();
        let root = builder.block(vec![initialized, returned]).unwrap();
        (builder.finish(root).unwrap(), site, slot)
    }

    #[test]
    fn checked_template_retains_original_source_and_exact_native_slot_schema() {
        let types = TypeRegistry::new();
        let ty = types.scalar(ScalarType::Int(IntegerType::S64));
        let (plan, site, slot) = plan(&types);
        let (graph, source, body) = quotation();
        let name = graph.symbols().find("captured").unwrap();
        let alias = graph.symbols().find("Answer").unwrap();
        let expected_location = source.location;
        let expected_origin = source.origins[0];
        let template = CompilerQuoteTemplate::checked(
            &plan,
            site,
            &body,
            source,
            vec![vec![
                (name, CompilerQuoteBinding::Native { slot, ty }),
                (alias, CompilerQuoteBinding::Native { slot, ty }),
            ]],
        )
        .unwrap();
        assert_eq!(template.plan(), plan.id());
        assert_eq!(template.site(), site);
        assert_eq!(template.captures(), &[slot]);
        assert_eq!(template.frames()[0].len(), 2);
        assert_eq!(template.source().location, expected_location);
        assert_eq!(template.source().origins, [expected_origin]);
        assert!(!template.source().checks.array_bounds.enabled());
        assert_eq!(template.source().debug, jai_types::DebugPolicy::Suppress);
        let syntax::CodeBody::Block(body) = template.body() else {
            panic!("original block")
        };
        let syntax::StatementKind::Constant(constant) = &body[0].kind else {
            panic!("original declaration")
        };
        assert_eq!(graph.symbols().name(constant.name), "Answer");
        assert!(constant.span.start >= expected_location.span.start);
        assert!(constant.span.end <= expected_location.span.end);
    }

    #[test]
    fn foreign_plan_return_site_and_native_slots_cannot_mint_a_template() {
        let types = TypeRegistry::new();
        let ty = types.scalar(ScalarType::Int(IntegerType::S64));
        let (plan, site, slot) = plan(&types);
        let (_, foreign_site, foreign_slot) = self::plan(&types);
        let (graph, source, body) = quotation();
        let name = graph.symbols().find("captured").unwrap();
        assert!(
            CompilerQuoteTemplate::checked(
                &plan,
                foreign_site,
                &body,
                source.clone(),
                vec![vec![(name, CompilerQuoteBinding::Native { slot, ty })]],
            )
            .is_err()
        );
        for binding in [
            CompilerQuoteBinding::Native {
                slot: foreign_slot,
                ty,
            },
            CompilerQuoteBinding::Native {
                slot,
                ty: types.scalar(ScalarType::Int(IntegerType::U8)),
            },
            CompilerQuoteBinding::Static(Binding::Type(ty)),
        ] {
            assert!(
                CompilerQuoteTemplate::checked(
                    &plan,
                    site,
                    &body,
                    source.clone(),
                    vec![vec![(name, binding)]],
                )
                .is_err()
            );
        }
    }

    #[test]
    fn unreadable_static_facts_and_ambiguous_frame_names_are_rejected() {
        let types = TypeRegistry::new();
        let ty = types.scalar(ScalarType::Int(IntegerType::S64));
        let (plan, site, slot) = plan(&types);
        let (graph, source, body) = quotation();
        let name = graph.symbols().find("captured").unwrap();
        let alias = graph.symbols().find("Answer").unwrap();
        let native = CompilerQuoteBinding::Native { slot, ty };
        for frame in [
            vec![(name, native.clone()), (name, native.clone())],
            vec![
                (name, native.clone()),
                (alias, CompilerQuoteBinding::Static(Binding::Discarded(ty))),
            ],
        ] {
            assert!(
                CompilerQuoteTemplate::checked(&plan, site, &body, source.clone(), vec![frame],)
                    .is_err()
            );
        }
        let mut source = source;
        source.origins = vec![source.location; 129];
        assert!(
            CompilerQuoteTemplate::checked(&plan, site, &body, source, vec![vec![(name, native)]],)
                .is_err()
        );
    }
}
