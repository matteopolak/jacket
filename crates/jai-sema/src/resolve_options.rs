//! Explicit semantic target policy and bounded compile-time execution settings.

#[derive(Clone, Debug, Default)]
pub struct ResolveOptions {
    /// Target-dependent queries remain pending when no target was selected.
    pub target: Option<jai_types::BuildTarget>,
    pub layout: Option<jai_types::LayoutPolicy>,
    pub compile_time_limits: jai_vm::Limits,
    pub compiler: Option<crate::CompilerBindingContext>,
    pub file_abi: Option<crate::FileAbiBindingContext>,
    pub process_abi: Option<crate::ProcessAbiBindingContext>,
}

impl ResolveOptions {
    pub fn effective_layout(&self) -> Option<jai_types::LayoutPolicy> {
        self.target
            .as_ref()
            .map(|target| target.layout)
            .or(self.layout)
    }
}
