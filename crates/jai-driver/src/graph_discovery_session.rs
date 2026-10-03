//! Borrow the immutable discovery snapshot while real source execution is parked.
use crate::{DiscoveryEffectPolicy, SemanticDiscoveryOptions};
use jai_modules::GraphDiscovery;
use jai_sema::{DiscoveryReadiness, PreparedDiscoveryRequests, PreparedDiscoverySession};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiscoveryQuery {
    Insertions,
    Conditions,
    Cases,
    Using,
}

/// Holds the original source requests and semantic arenas until a drive completes.
/// Its borrow prevents graph advancement while a VM checkpoint remains active.
pub struct PreparedGraphDiscoverySession<'graph> {
    semantic: PreparedDiscoverySession<'graph>,
    policy: DiscoveryEffectPolicy,
}

impl<'graph> PreparedGraphDiscoverySession<'graph> {
    pub fn new(
        discovery: &'graph GraphDiscovery<'_>,
        options: &SemanticDiscoveryOptions,
        query: DiscoveryQuery,
    ) -> Result<Self, jai_source::LocatedDiagnostic> {
        let graph = discovery.graph();
        let requests = match query {
            DiscoveryQuery::Insertions => PreparedDiscoveryRequests::Insertions(
                discovery.pending_insertion_requests().cloned().collect(),
            ),
            DiscoveryQuery::Conditions => PreparedDiscoveryRequests::Conditions(
                discovery.pending_conditions().cloned().collect(),
            ),
            DiscoveryQuery::Cases => {
                PreparedDiscoveryRequests::Cases(discovery.pending_cases().cloned().collect())
            }
            DiscoveryQuery::Using => {
                PreparedDiscoveryRequests::Using(discovery.pending_using_requests())
            }
        };
        let resolve = jai_sema::ResolveOptions {
            target: Some(options.target.clone()),
            compile_time_limits: options.limits,
            compiler: Some(jai_sema::CompilerBindingContext::from_graph(
                graph,
                &options.graph.import_dirs,
                options.workspace,
            )),
            file_abi: jai_sema::FileAbiBindingContext::allocator_from_graph(
                graph,
                &options.graph.import_dirs,
                options.target.clone(),
            ),
            ..Default::default()
        };
        Ok(Self {
            semantic: PreparedDiscoverySession::with_insertion_admission(
                graph,
                &resolve,
                requests,
                Box::new(move |request, code| discovery.admit_insertion(request, code)),
            )?,
            policy: options.effect_policy,
        })
    }

    /// Keep the same effects owner between drives; publish source choices only
    /// after Complete and after releasing this immutable graph borrow.
    pub fn drive(&mut self, effects: &mut dyn jai_vm::CompilerEffects) -> DiscoveryReadiness {
        match self.policy {
            DiscoveryEffectPolicy::Disabled => self.semantic.drive(&mut jai_vm::NoEffects),
            DiscoveryEffectPolicy::CompilerSession => self.semantic.drive(effects),
        }
    }

    /// Cancel before replacing the graph snapshot or releasing its effects owner.
    pub fn cancel(
        &mut self,
        effects: &mut dyn jai_vm::CompilerEffects,
    ) -> Result<(), jai_vm::Error> {
        match self.policy {
            DiscoveryEffectPolicy::Disabled => self.semantic.cancel(&mut jai_vm::NoEffects),
            DiscoveryEffectPolicy::CompilerSession => self.semantic.cancel(effects),
        }
    }
}
