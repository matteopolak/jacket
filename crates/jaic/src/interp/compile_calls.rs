//! Retained compile-time calls. Only the owning compiler may service a dependency.
use super::*;

pub(crate) struct CompileContinuation {
    frames: Vec<threads_inline::SuspFrame>,
    stack_top: u64,
    depth: usize,
    calls: usize,
    loc: Option<(u32, u32, u32)>,
    trace_loc: Option<Option<(u32, u32, u32)>>,
}

pub(crate) enum CompileCall {
    Complete(Res<Vec<u64>>),
    NeedsBody {
        function: FuncId,
        callable: bool,
        optional: bool,
        continuation: CompileContinuation,
    },
}

impl Interp {
    /// Before handing code to a native callback/worker, its reachable code must
    /// be ready: such frames cannot call back into the owning compiler service.
    pub(super) fn require_callable(&mut self, program: &Program, function: FuncId) -> Res<()> {
        if self.compile_time
            && self.defer_missing_bodies
            && !self.ready_callables.contains(&function)
        {
            self.callable_request = Some((function, false));
            self.no_body(program, function)
        } else {
            Ok(())
        }
    }

    pub(crate) fn mark_callable_ready(&mut self, function: FuncId) {
        self.ready_callables.insert(function);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn inspect_callable(&mut self, program: &Program, function: FuncId) -> Res<()> {
        if self.compile_time
            && self.defer_missing_bodies
            && !self.ready_callables.contains(&function)
            && self.inspected_callables.insert(function)
        {
            self.callable_request = Some((function, true));
            self.no_body(program, function)
        } else {
            Ok(())
        }
    }

    pub(crate) fn invalidate_callable_inspections(&mut self) {
        self.inspected_callables.clear();
    }

    pub(crate) fn call_for_compiler(
        &mut self,
        program: &Program,
        function: FuncId,
        args: &[u64],
    ) -> CompileCall {
        let outer = std::mem::replace(&mut self.defer_missing_bodies, true);
        let result = self.call(program, function, args);
        self.defer_missing_bodies = outer;
        self.compiler_outcome(result)
    }

    fn compiler_outcome(&mut self, result: Res<Vec<u64>>) -> CompileCall {
        if let Err(trap) = &result
            && let Some(TrapKind::CompileDependency(function)) = trap.kind
        {
            let frames = std::mem::take(&mut self.captured);
            debug_assert!(!frames.is_empty(), "compiler entry thunks have bodies");
            let request = self.callable_request.take();
            CompileCall::NeedsBody {
                function,
                callable: request.is_some_and(|(id, _)| id == function),
                optional: request.is_some_and(|(_, optional)| optional),
                continuation: CompileContinuation {
                    frames,
                    stack_top: self.sp,
                    depth: self.depth,
                    calls: self.calls.len(),
                    loc: self.loc,
                    trace_loc: self.trace_loc,
                },
            }
        } else {
            self.callable_request = None;
            CompileCall::Complete(result)
        }
    }

    pub(crate) fn resume_for_compiler(
        &mut self,
        program: &Program,
        continuation: CompileContinuation,
    ) -> CompileCall {
        // Compiler service thunks use the same owning stack above the parked
        // frames, and must unwind back to this boundary before resumption.
        self.restore_compile_boundary(&continuation);
        let outer = std::mem::replace(&mut self.defer_missing_bodies, true);
        self.call_nesting += 1;
        let result = self
            .resume_frames(program, continuation.frames)
            .map(Rets::into_vec);
        self.call_nesting -= 1;
        self.defer_missing_bodies = outer;
        self.compiler_outcome(result)
    }

    pub(crate) fn cancel_compiler_call(
        &mut self,
        program: &Program,
        continuation: CompileContinuation,
    ) {
        self.restore_compile_boundary(&continuation);
        self.unwind_compile_frames(program, continuation.frames);
    }

    fn restore_compile_boundary(&mut self, continuation: &CompileContinuation) {
        debug_assert_eq!(self.sp, continuation.stack_top);
        debug_assert_eq!(self.depth, continuation.depth);
        debug_assert_eq!(self.calls.len(), continuation.calls);
        self.loc = continuation.loc;
        self.trace_loc = continuation.trace_loc;
    }
}
