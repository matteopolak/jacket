//! Module-level symbols: the program's functions, foreign symbols, globals and the C library
//! helpers lowering calls, declared before any function is compiled.
use cranelift_codegen::ir;
use cranelift_codegen::isa::CallConv;
use cranelift_module::{DataDescription, DataId, FuncId, Linkage, Module, ModuleRelocTarget};
use cranelift_object::ObjectModule;
use jaic::abi::Arch;
use jaic::fxhash::HashMap;
use jaic::ir::{Linkage as IrLinkage, Program, RelocTarget};
use jaic::target::Target;

use super::abi::{Lowered, lower_sig};
use super::{R, unsupported};

/// What a foreign symbol is to the code that uses it.
pub(crate) enum ForeignSym {
    Func(FuncId),
    Data(DataId),
    /// `#intrinsic "llvm.<name>"`: lowered as the operation it names (`intrinsics::llvm`).
    Llvm(String),
}

/// A C library function lowering calls (`memcmp` for `Memcmp`, `write` for `CompilerWrite`...).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum Helper {
    Memcmp,
    Write,
    Sin,
    Sinf,
    Cos,
    Cosf,
    Round,
    Roundf,
    /// Reports a failed null check and traps (`checks::null_fail_function`).
    NullFail,
    /// Reads the cycle counter: `rdtsc` on x86-64, the virtual counter on `AArch64` (the cycle
    /// counter itself is not readable from user code there).
    CycleCounter,
    /// A spin-wait hint: `pause`, or `yield`.
    Pause,
    /// x86-64: calls the C-variadic function whose address is in the pinned register `r15`
    /// after setting `%al`, which System V has a variadic call pass as an upper bound on the
    /// vector registers its arguments use, and which Cranelift does not set.
    VariadicCall,
}

impl Helper {
    fn name(self) -> &'static str {
        match self {
            Helper::Memcmp => "memcmp",
            Helper::Write => "write",
            Helper::Sin => "sin",
            Helper::Sinf => "sinf",
            Helper::Cos => "cos",
            Helper::Cosf => "cosf",
            Helper::Round => "round",
            Helper::Roundf => "roundf",
            Helper::NullFail => "jaic.null_fail",
            Helper::CycleCounter => "jaic.cycle_counter",
            Helper::Pause => "jaic.pause",
            Helper::VariadicCall => "jaic.variadic_call",
        }
    }

    /// The machine code of a helper defined in the program as bytes: instructions Cranelift
    /// has no operation for.
    pub(crate) fn machine_code(self, arch: Arch) -> Option<&'static [u8]> {
        const AARCH64_CYCLES: [u8; 8] = [
            0x40, 0xe0, 0x3b, 0xd5, // mrs x0, cntvct_el0
            0xc0, 0x03, 0x5f, 0xd6, // ret
        ];

        const AARCH64_YIELD: [u8; 8] = [
            0x3f, 0x20, 0x03, 0xd5, // yield
            0xc0, 0x03, 0x5f, 0xd6, // ret
        ];

        const X64_CYCLES: [u8; 10] = [
            0x0f, 0x31, // rdtsc
            0x48, 0xc1, 0xe2, 0x20, // shl rdx, 32
            0x48, 0x09, 0xd0, // or rax, rdx
            0xc3, // ret
        ];

        const X64_PAUSE: [u8; 3] = [0xf3, 0x90, 0xc3]; // pause; ret

        const X64_VARIADIC_CALL: [u8; 8] = [
            0xb8, 8, 0, 0, 0, // mov eax, 8
            0x41, 0xff, 0xe7, // jmp r15
        ];

        match (self, arch.is_aarch64()) {
            (Helper::CycleCounter, true) => Some(&AARCH64_CYCLES),
            (Helper::CycleCounter, false) => Some(&X64_CYCLES),
            (Helper::Pause, true) => Some(&AARCH64_YIELD),
            (Helper::Pause, false) => Some(&X64_PAUSE),
            (Helper::VariadicCall, false) => Some(&X64_VARIADIC_CALL),
            _ => None,
        }
    }

    /// Defined in this object (by Cranelift or as machine code) rather than imported.
    pub(crate) fn is_local(self) -> bool {
        matches!(
            self,
            Helper::NullFail | Helper::CycleCounter | Helper::Pause | Helper::VariadicCall
        )
    }

    /// The C signature, as Cranelift types.
    pub(crate) fn signature(self, call_conv: CallConv) -> ir::Signature {
        use ir::types::{F32, F64, I32, I64};
        let (params, returns): (&[ir::Type], &[ir::Type]) = match self {
            Helper::Memcmp => (&[I64, I64, I64], &[I32]),
            Helper::Write => (&[I32, I64, I64], &[I64]),
            Helper::Sin | Helper::Cos | Helper::Round => (&[F64], &[F64]),
            Helper::Sinf | Helper::Cosf | Helper::Roundf => (&[F32], &[F32]),
            Helper::NullFail => (&[I64, I64, I64], &[]),
            Helper::CycleCounter => (&[], &[I64]),
            // Declared bare: each call site gives the signature of the function it calls.
            Helper::Pause | Helper::VariadicCall => (&[], &[]),
        };
        let mut sig = ir::Signature::new(call_conv);

        sig.params
            .extend(params.iter().map(|&t| ir::AbiParam::new(t)));
        sig.returns
            .extend(returns.iter().map(|&t| ir::AbiParam::new(t)));
        sig
    }
}

/// Every symbol of the program, by IR id.
pub(crate) struct Symbols {
    pub(crate) call_conv: CallConv,
    pub(crate) funcs: Vec<Option<FuncId>>,
    /// The signature lowering of each function with a body (`None`: never lowered).
    pub(crate) func_sigs: Vec<Option<Lowered>>,
    pub(crate) foreigns: Vec<ForeignSym>,
    /// The signature lowering of each foreign function (`None` for data, intrinsics and
    /// C-variadic functions, whose call sites lower their own).
    pub(crate) foreign_sigs: Vec<Option<Lowered>>,
    pub(crate) globals: Vec<DataId>,
    pub(crate) helpers: HashMap<Helper, FuncId>,
    /// The path of each source file code is located in, as failed checks print it: a C string.
    pub(crate) check_files: Vec<Option<DataId>>,
    /// Whether the function or data id is defined in this module (a direct, PC-relative
    /// reference) rather than imported.
    local_funcs: Vec<bool>,
}

impl Symbols {
    pub(crate) fn declare(
        module: &mut ObjectModule,
        program: &Program,
        target: Target,
    ) -> R<Symbols> {
        let call_conv = module.isa().default_call_conv();
        let arch = super::isa::arch(target)?;
        let layout = target.data_layout();
        let apple = target.is_apple();
        let lower = |sig: &jaic::ir::Sig| lower_sig(sig, arch, apple, call_conv, &layout);
        let err = |e: cranelift_module::ModuleError| e.to_string();
        let mut by_name: HashMap<String, FuncId> = HashMap::default();
        let mut funcs = Vec::with_capacity(program.funcs.len());
        let mut func_sigs = Vec::with_capacity(program.funcs.len());

        for (i, func) in program.funcs.iter().enumerate() {
            let Some(func) = func else {
                funcs.push(None);
                func_sigs.push(None);

                continue;
            };

            if func.section.is_some() {
                return Err(unsupported(format!(
                    "placing procedure '{}' in a section (`@Section`)",
                    func.name
                )));
            }

            // `#target_features` only permits instructions; without vector operations this
            // backend has none to use, so the procedure is compiled for the baseline.
            let lowered = lower(&func.sig).map_err(|e| e.at(|| format!("in '{}'", func.name)))?;
            let (name, linkage) = match &func.linkage {
                IrLinkage::Export(name) => (name.clone(), Linkage::Export),
                IrLinkage::Internal => (format!("{}.{i}", func.name), Linkage::Local),
            };
            let id = if let Some(&id) = by_name.get(&name) {
                id
            } else {
                let id = module
                    .declare_function(&name, linkage, &lowered.sig)
                    .map_err(err)?;

                by_name.insert(name, id);
                id
            };

            funcs.push(Some(id));
            func_sigs.push(Some(lowered));
        }

        let mut data_by_name: HashMap<String, DataId> = HashMap::default();
        let mut foreigns = Vec::with_capacity(program.foreigns.len());
        let mut foreign_sigs = Vec::with_capacity(program.foreigns.len());

        for foreign in &program.foreigns {
            let symbol = foreign.symbol.as_str();

            if let Some(name) = symbol.strip_prefix("llvm.") {
                foreigns.push(ForeignSym::Llvm(name.to_string()));
                foreign_sigs.push(None);

                continue;
            }

            if foreign.is_data {
                let id = if let Some(&id) = data_by_name.get(symbol) {
                    id
                } else {
                    let id = module
                        .declare_data(symbol, Linkage::Import, true, false)
                        .map_err(err)?;

                    data_by_name.insert(symbol.to_string(), id);
                    id
                };

                foreigns.push(ForeignSym::Data(id));
                foreign_sigs.push(None);

                continue;
            }

            // A runtime may export a function under another name (`Toolchain::renames`).
            let name = target.toolchain().rename(symbol).unwrap_or(symbol);
            let lowered = if foreign.sig.c_varargs {
                None
            } else {
                Some(lower(&foreign.sig).map_err(|e| e.at(|| format!("calling '{symbol}'")))?)
            };
            // A `#program_export` of the same name is what the linker would bind the reference
            // to: the declaration merges into it.
            let id = if let Some(&id) = by_name.get(name) {
                id
            } else {
                let sig = match &lowered {
                    Some(l) => l.sig.clone(),
                    None => ir::Signature::new(call_conv),
                };
                let id = module
                    .declare_function(name, Linkage::Import, &sig)
                    .map_err(err)?;

                by_name.insert(name.to_string(), id);
                id
            };

            foreigns.push(ForeignSym::Func(id));
            foreign_sigs.push(lowered);
        }

        let mut helpers = HashMap::default();

        for helper in helpers_used(program, arch) {
            let name = helper.name();
            let id = if let Some(&id) = by_name.get(name) {
                id
            } else {
                let linkage = if helper.is_local() {
                    Linkage::Local
                } else {
                    Linkage::Import
                };
                let id = module
                    .declare_function(name, linkage, &helper.signature(call_conv))
                    .map_err(err)?;

                by_name.insert(name.to_string(), id);
                id
            };

            helpers.insert(helper, id);
        }

        let mut globals = Vec::with_capacity(program.globals.len());

        for (i, g) in program.globals.iter().enumerate() {
            let (name, linkage) = match &g.export {
                Some(name) => (name.clone(), Linkage::Export),
                // A variable in a named section is the linker script's to place or drop: it
                // stays visible.
                None if g.section.is_some() => (format!("{}.g{i}", g.name), Linkage::Export),
                None => (format!("{}.g{i}", g.name), Linkage::Local),
            };
            let id = module
                .declare_data(&name, linkage, !g.read_only, false)
                .map_err(err)?;

            globals.push(id);
        }

        let mut check_files = vec![None; program.file_paths.len()];

        for func in program.funcs.iter().flatten() {
            let locs = func
                .blocks
                .iter()
                .flat_map(|b| &b.insts)
                .filter_map(|i| match i {
                    jaic::ir::Inst::Loc {
                        file, ..
                    } => Some(*file),
                    _ => None,
                });

            for file in std::iter::once(func.source_file).chain(locs) {
                if let Some(slot @ None) = check_files.get_mut(file as usize) {
                    *slot = Some(module.declare_anonymous_data(false, false).map_err(err)?);
                }
            }
        }

        let decls = module.declarations();
        let local_funcs = decls
            .get_functions()
            .map(|(_, d)| d.linkage.is_final())
            .collect();

        Ok(Symbols {
            call_conv,
            funcs,
            func_sigs,
            foreigns,
            foreign_sigs,
            globals,
            helpers,
            check_files,
            local_funcs,
        })
    }

    /// Whether function `id` is defined in this object.
    pub(crate) fn is_local(&self, id: FuncId) -> bool {
        self.local_funcs[id.as_u32() as usize]
    }

    /// Give every program global and source file name its contents.
    pub(crate) fn define_data(
        &self,
        module: &mut ObjectModule,
        program: &Program,
        macho: bool,
    ) -> R<()> {
        for (i, g) in program.globals.iter().enumerate() {
            let mut d = DataDescription::new();
            let size = g.size.max(g.init.len() as u64).max(1) as usize;

            if g.relocs.is_empty() && g.init.iter().all(|&b| b == 0) {
                d.define_zeroinit(size);
            } else {
                let mut bytes = g.init.clone();

                bytes.resize(size, 0);
                d.define(bytes.into_boxed_slice());
            }

            for reloc in &g.relocs {
                let offset = u32::try_from(reloc.offset)
                    .map_err(|_| format!("global '{}' is too large", g.name))?;
                let func = |d: &mut DataDescription, id: FuncId| {
                    if reloc.addend != 0 {
                        return Err(unsupported(
                            "an offset from a procedure's address in a global",
                        ));
                    }

                    let f = d.import_function(ModuleRelocTarget::user(0, id.as_u32()));

                    d.write_function_addr(offset, f);
                    Ok(())
                };
                let data = |d: &mut DataDescription, id: DataId| {
                    let gv = d.import_global_value(ModuleRelocTarget::user(1, id.as_u32()));

                    d.write_data_addr(offset, gv, reloc.addend);
                };

                match reloc.target {
                    RelocTarget::Global(id) => data(&mut d, self.globals[id.0 as usize]),
                    RelocTarget::Func(id) => match self.funcs[id.0 as usize] {
                        Some(f) => func(&mut d, f)?,
                        None => {
                            return Err(format!(
                                "global '{}' refers to '{}', which was never lowered",
                                g.name, program.func_names[id.0 as usize]
                            )
                            .into());
                        }
                    },
                    RelocTarget::Foreign(id) => match &self.foreigns[id.0 as usize] {
                        ForeignSym::Func(f) => func(&mut d, *f)?,
                        ForeignSym::Data(id) => data(&mut d, *id),
                        ForeignSym::Llvm(name) => {
                            return Err(unsupported(format!(
                                "the address of the intrinsic `llvm.{name}`"
                            )));
                        }
                    },
                }
            }

            d.set_align(g.align.max(1));

            if let Some(section) = &g.section {
                // Mach-O names `segment,section`; ELF only the section.
                let (segment, name) = if macho {
                    section.split_once(',').unwrap_or(("__DATA", section))
                } else {
                    ("", section.as_str())
                };

                d.set_segment_section(segment, name, 0);
            }

            module
                .define_data(self.globals[i], &d)
                .map_err(|e| format!("global '{}': {e}", g.name))?;
        }

        for (file, id) in self.check_files.iter().enumerate() {
            let Some(id) = *id else {
                continue;
            };
            let path = program
                .file_paths
                .get(file)
                .map(|p| jaic::display_path(std::path::Path::new(p)))
                .unwrap_or_default();
            let mut bytes = path.into_bytes();

            bytes.push(0);
            let mut d = DataDescription::new();

            d.define(bytes.into_boxed_slice());
            module.define_data(id, &d).map_err(|e| e.to_string())?;
        }

        Ok(())
    }
}

/// The helpers some function's lowering calls, so that only those are declared: an undefined
/// symbol nothing refers to would still have to be found by the linker (`sin` is in `libm`).
fn helpers_used(program: &Program, arch: Arch) -> Vec<Helper> {
    use jaic::ir::{Inst, Intrinsic, Ty};
    let mut used = Vec::new();
    let mut add = |h: Helper| {
        if !used.contains(&h) {
            used.push(h);
        }
    };

    for func in program.funcs.iter().flatten() {
        if func.null_checks {
            add(Helper::NullFail);
        }

        for inst in func.blocks.iter().flat_map(|b| &b.insts) {
            if let Inst::Call(call) = inst
                && arch == Arch::X86_64
                && callee_is_variadic(program, &call.callee)
            {
                add(Helper::VariadicCall);
            }

            let Inst::Intrinsic(call) = inst else {
                continue;
            };
            let f32 = call
                .args
                .first()
                .is_some_and(|a| func.vals[a.0 as usize] == Ty::F32);

            match call.op {
                Intrinsic::Memcmp => add(Helper::Memcmp),
                Intrinsic::CompilerWrite => add(Helper::Write),
                Intrinsic::Sin => add(if f32 {
                    Helper::Sinf
                } else {
                    Helper::Sin
                }),
                Intrinsic::Cos => add(if f32 {
                    Helper::Cosf
                } else {
                    Helper::Cos
                }),
                Intrinsic::Round => add(if f32 {
                    Helper::Roundf
                } else {
                    Helper::Round
                }),
                Intrinsic::CycleCounter => add(Helper::CycleCounter),
                Intrinsic::Pause => add(Helper::Pause),
                _ => {}
            }
        }
    }

    used
}

/// Whether a call to `callee` is a C-variadic one.
pub(crate) fn callee_is_variadic(program: &Program, callee: &jaic::ir::Callee) -> bool {
    match callee {
        jaic::ir::Callee::Foreign(id) => program.foreigns[id.0 as usize].sig.c_varargs,
        jaic::ir::Callee::Indirect(_, sig) => sig.c_varargs,
        jaic::ir::Callee::Func(_) => false,
    }
}
