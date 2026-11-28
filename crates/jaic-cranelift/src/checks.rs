//! Runtime checks: null pointer checks around memory accesses, the checks that trap (division by
//! zero, bounds) and the report a failed check makes through `Runtime_Support`.
use cranelift_codegen::ir::{self, InstBuilder, TrapCode, Value, condcodes::IntCC, types};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use jaic::ir::{CheckReason, Ty, Val};

use super::R;
use super::Shared;
use super::abi::ParamPlan;
use super::data::Helper;
use super::function::Cx;

/// The trap every failed check and `Term::Unreachable` ends in.
pub(crate) const TRAP: TrapCode = TrapCode::unwrap_user(1);

/// Addresses below this are never mapped: an access there is a null pointer access.
const FIRST_PAGE: i64 = 4096;

/// `__jaic_check_failed(reason, a, b, fatal, line, filename)`, which a failed check calls.
fn check_handler<'a>(
    sh: &Shared<'a>,
) -> R<Option<(cranelift_module::FuncId, &'a super::abi::Lowered)>> {
    let Some(handler) = sh.program.check_failed else {
        return Ok(None);
    };
    let sig = sh
        .program
        .func(handler)
        .map(|f| &f.sig)
        .ok_or("private native check reporter has no body")?;

    if sig.params.len() != 6 || sig.params[5] != Ty::Ptr {
        return Err("private native check reporter must take (reason: s64, a: s64, b: s64, fatal: bool, line: s64, filename: *u8)".into());
    }

    let lowered = sh.symbols.func_sigs[handler.0 as usize]
        .as_ref()
        .ok_or("private native check reporter has no signature")?;

    if !lowered
        .params
        .iter()
        .all(|p| matches!(p, ParamPlan::Scalar))
    {
        return Err("Runtime_Support's check handler takes only scalars".into());
    }

    let id = sh.symbols.funcs[handler.0 as usize].ok_or("check reporter was never declared")?;
    Ok(Some((id, lowered)))
}

impl Cx<'_, '_> {
    /// The failure block and the continuation of a check. The failure block is cold, so the
    /// passing path falls through.
    fn check_blocks(&mut self) -> (ir::Block, ir::Block) {
        let fail = self.b.create_block();

        self.b.set_cold_block(fail);
        (fail, self.b.create_block())
    }

    /// The name of the current source file, a C string.
    fn check_file(&mut self) -> R<Value> {
        let file = self.loc.0 as usize;
        let id = self
            .sh
            .symbols
            .check_files
            .get(file)
            .copied()
            .flatten()
            .ok_or("a check in a source file with no name")?;

        Ok(self.data_addr(id, true))
    }

    /// Stop with a null pointer error when `addr` lies in the never-mapped first page.
    pub(crate) fn null_check(&mut self, addr: Val, kind: u64) -> R<()> {
        if !self.checks.check(addr) {
            return Ok(());
        }

        let value = self.get_i64(addr)?;
        let low = self
            .b
            .ins()
            .icmp_imm_s(IntCC::UnsignedLessThan, value, FIRST_PAGE);
        let (fail, cont) = self.check_blocks();

        self.b.ins().brif(low, fail, &[], cont, &[]);
        self.b.switch_to_block(fail);
        let desc = kind | (u64::from(self.loc.1) << 8);
        let desc = self.b.ins().iconst(types::I64, desc as i64);
        let file = self.check_file()?;
        let f = self.helper(Helper::NullFail);

        self.b.ins().call(f, &[desc, value, file]);
        self.b.ins().trap(TRAP);
        self.b.switch_to_block(cont);
        Ok(())
    }

    /// Branch to a failure block when `cond` holds: it reports `reason` (with `a`, `b`) and
    /// traps.
    pub(crate) fn trap_if(
        &mut self,
        cond: Value,
        reason: CheckReason,
        a: Value,
        b: Value,
    ) -> R<()> {
        let (fail, cont) = self.check_blocks();

        self.b.ins().brif(cond, fail, &[], cont, &[]);
        self.b.switch_to_block(fail);
        let fatal = self.b.ins().iconst(types::I8, 1);

        self.report_check(reason as u64, a, b, fatal)?;
        self.b.ins().trap(TRAP);
        self.b.switch_to_block(cont);
        Ok(())
    }

    /// Report a failed check through `Runtime_Support` (`Program::check_failed`) with the
    /// current source location; nothing when the program has no reporting procedure.
    pub(crate) fn report_check(&mut self, reason: u64, a: Value, b: Value, fatal: Value) -> R<()> {
        let Some((id, lowered)) = check_handler(self.sh)? else {
            return Ok(());
        };
        let reason = self.b.ins().iconst(types::I64, reason as i64);
        let line = self.b.ins().iconst(types::I64, i64::from(self.loc.1));
        let file = self.check_file()?;
        let r = self.func_ref(id, &lowered.sig);
        let args = [reason, a, b, fatal, line, file];

        self.call(super::function::Target::Direct(r), lowered, &args)?;
        Ok(())
    }
}

/// `jaic.null_fail(desc, address, file)`: reports a null pointer access through
/// `Runtime_Support` and traps. `desc` is the access kind in its low byte and the source line
/// above it. One call per check, where the check itself would need the handler's six arguments.
pub(crate) fn null_fail_function(sh: &Shared, fb: &mut FunctionBuilderContext) -> R<ir::Function> {
    let call_conv = sh.symbols.call_conv;
    let mut f = ir::Function::with_name_signature(
        ir::UserFuncName::default(),
        Helper::NullFail.signature(call_conv),
    );
    let mut b = FunctionBuilder::new(&mut f, fb);
    let entry = b.create_block();

    b.append_block_params_for_function_params(entry);
    b.switch_to_block(entry);
    let params = b.block_params(entry).to_vec();

    if let Some((id, lowered)) = check_handler(sh)? {
        let (desc, address, file) = (params[0], params[1], params[2]);
        let kind = b.ins().band_imm_u(desc, 0xff);
        let line = b.ins().ushr_imm_u(desc, 8);
        let reason = b.ins().iconst(types::I64, CheckReason::NullPointer as i64);
        let fatal = b.ins().iconst(types::I64, 1);
        let mut args = Vec::with_capacity(6);

        for (v, p) in [reason, kind, address, fatal, line, file]
            .into_iter()
            .zip(&lowered.sig.params)
        {
            let want = p.value_type;
            let v = if want.bits() < 64 {
                b.ins().ireduce(want, v)
            } else {
                v
            };

            args.push(v);
        }

        let signature = b.import_signature(lowered.sig.clone());
        let name = b.func.declare_imported_user_function(ir::UserExternalName {
            namespace: 0,
            index: id.as_u32(),
        });
        let callee = b.import_function(ir::ExtFuncData {
            name: ir::ExternalName::user(name),
            signature,
            colocated: sh.symbols.is_local(id),
            patchable: false,
        });

        b.ins().call(callee, &args);
    }

    // Disabling runtime support removes diagnostics, not memory checks.
    b.ins().trap(TRAP);
    b.seal_all_blocks();
    b.finalize(sh.isa.frontend_config());
    Ok(f)
}
