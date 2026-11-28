//! Experimental Cranelift backend for unoptimized jaic builds (`-backend cranelift`).
//!
//! [`emit_objects`] lowers an `ir::Program` to Cranelift IR, compiles its functions in parallel on
//! jaic's thread pool and writes them as one object file, which the caller links exactly as it
//! links LLVM's objects. It covers what an `-O0` build of a native program needs; anything else
//! (optimized builds, sanitizers, other targets, `#asm`-only instructions...) is an
//! [`Error::Unsupported`] naming the construct, and the driver builds with LLVM instead
//! (`docs/native/cranelift-backend.md`).
mod abi;
mod checks;
mod data;
mod function;
mod inst;
mod intrinsics;
mod isa;

use std::path::{Path, PathBuf};

use cranelift_codegen::control::ControlPlane;
use cranelift_codegen::ir;
use cranelift_module::{FuncId, Module, ModuleReloc};
use cranelift_object::{ObjectBuilder, ObjectModule};
use jaic::ir::Program;
use jaic_native::{OptLevel, Options};

pub(crate) use self::data::Symbols;

/// Why this backend wrote no object.
#[derive(Debug)]
pub enum Error {
    /// The build uses something this backend does not compile; LLVM can build it instead.
    Unsupported {
        /// What, as in "does not support {what}".
        what: String,
        /// Where, as in "in 'main'" or "calling 'printf'".
        place: Option<String>,
    },
    /// Anything else: an I/O error, or a fault in this backend.
    Failed(String),
}

impl Error {
    /// The error said to happen at `place` ("in 'main'"), unless it already names a place
    /// further in.
    pub(crate) fn at(self, place: impl FnOnce() -> String) -> Error {
        match self {
            Error::Unsupported {
                what,
                place: None,
            } => Error::Unsupported {
                what,
                place: Some(place()),
            },
            Error::Failed(message) => Error::Failed(format!("{}: {message}", place())),
            unsupported @ Error::Unsupported {
                ..
            } => unsupported,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Unsupported {
                what,
                place: None,
            } => {
                write!(f, "Cranelift does not support {what}")
            }
            Error::Unsupported {
                what,
                place: Some(place),
            } => write!(f, "Cranelift does not support {what}, {place}"),
            Error::Failed(message) => f.write_str(message),
        }
    }
}

impl From<String> for Error {
    fn from(message: String) -> Error {
        Error::Failed(message)
    }
}

impl From<&str> for Error {
    fn from(message: &str) -> Error {
        Error::Failed(message.to_string())
    }
}

pub(crate) type R<T> = Result<T, Error>;

/// An error naming something this backend does not compile.
pub(crate) fn unsupported(what: impl std::fmt::Display) -> Error {
    Error::Unsupported {
        what: what.to_string(),
        place: None,
    }
}

/// The Cranelift release this backend uses, for the build's `Backend` line.
pub const VERSION: &str = cranelift_object::VERSION;

/// Translate `program` to one native object file at `path`, as the caller of `jaic_llvm`'s
/// `emit_objects` would get. Returns the objects written (always just `path`).
///
/// # Errors
/// [`Error::Unsupported`] before anything is written when the build needs what this backend
/// does not compile.
pub fn emit_objects(program: &Program, options: &Options, path: &Path) -> R<Vec<PathBuf>> {
    check_options(options)?;
    check_program(program)?;
    let target = options.target.unwrap_or_else(jaic::target::Target::host);
    let isa = isa::target_isa(target, options)?;
    let mut builder = ObjectBuilder::new(
        isa.clone(),
        "jai",
        cranelift_module::default_libcall_names(),
    )
    .map_err(|e| e.to_string())?;

    builder.per_function_section(options.codegen.function_sections == Some(true));
    let mut module = ObjectModule::new(builder);
    let symbols = Symbols::declare(&mut module, program, target)?;
    let shared = Shared {
        program,
        isa: &*isa,
        target,
        arch: isa::arch(target)?,
        layout: target.data_layout(),
        symbols: &symbols,
    };
    let mut compiled = compile_all(&shared)?;

    if let Some(&id) = symbols.helpers.get(&data::Helper::NullFail) {
        let mut ctx = cranelift_codegen::Context::new();
        let mut fb = cranelift_frontend::FunctionBuilderContext::new();

        ctx.func = checks::null_fail_function(&shared, &mut fb)?;
        compiled.push(finish(&shared, id, "jaic.null_fail".into(), &mut ctx)?);
    }

    for (&helper, &id) in &symbols.helpers {
        if let Some(code) = helper.machine_code(shared.arch) {
            compiled.push(Compiled {
                id,
                name: format!("{helper:?}"),
                alignment: 16,
                code: code.to_vec(),
                relocs: Vec::new(),
            });
        }
    }

    for unit in compiled {
        module
            .define_function_bytes(unit.id, unit.alignment, &unit.code, &unit.relocs)
            .map_err(|e| format!("in '{}': {e}", unit.name))?;
    }

    symbols.define_data(&mut module, program, target.is_apple())?;
    let product = module.finish();
    let bytes = product
        .emit()
        .map_err(|e| format!("could not write `{}`: {e}", path.display()))?;

    std::fs::write(path, bytes).map_err(|e| {
        format!(
            "could not write `{}`: {}",
            path.display(),
            jaic::io_reason(&e)
        )
    })?;
    Ok(vec![path.to_path_buf()])
}

/// Build options this backend cannot honour.
fn check_options(options: &Options) -> R<()> {
    if options.opt_level != OptLevel::O0 {
        return Err(unsupported(
            "optimized builds (`-o`, `-od`, `-O1` and above)",
        ));
    }

    let s = options.sanitize;

    if s.address || s.undefined || s.fuzzer {
        return Err(unsupported("sanitizers (`-sanitize`)"));
    }

    if s.coverage {
        return Err(unsupported("coverage (`-coverage`)"));
    }

    let c = &options.codegen;

    if options.emit_ir.is_some()
        || c.ir_after.is_some()
        || c.bitcode_after.is_some()
        || c.bitcode_before.is_some()
    {
        return Err(unsupported(
            "writing LLVM IR or bitcode (`-emit_ir`, `output_llvm_ir`, `output_bitcode`)",
        ));
    }

    if c.o0_passes.is_some() {
        return Err(unsupported("LLVM passes (`-dev_o0_passes`)"));
    }

    if options.cpu.is_some() || options.features.is_some() {
        return Err(unsupported(
            "a target CPU or features (`llvm_options.target_system_cpu`/`_features`)",
        ));
    }

    if c.frame_pointers == Some(false) {
        return Err(unsupported(
            "omitting frame pointers (`enable_frame_pointers = false`)",
        ));
    }

    if c.no_red_zone {
        return Err(unsupported("`disable_redzone`"));
    }

    Ok(())
}

/// Program-wide constructs this backend cannot compile.
fn check_program(program: &Program) -> R<()> {
    if !program.module_asm.is_empty() {
        return Err(unsupported("module assembly (`@Module_Asm`)"));
    }

    Ok(())
}

/// What every function's lowering reads; shared by the worker threads.
pub(crate) struct Shared<'a> {
    pub(crate) program: &'a Program,
    pub(crate) isa: &'a dyn cranelift_codegen::isa::TargetIsa,
    pub(crate) target: jaic::target::Target,
    pub(crate) arch: jaic::abi::Arch,
    pub(crate) layout: jaic::layout::DataLayout,
    pub(crate) symbols: &'a Symbols,
}

/// One function's machine code, ready to be placed in the object.
struct Compiled {
    id: FuncId,
    name: String,
    alignment: u64,
    code: Vec<u8>,
    relocs: Vec<ModuleReloc>,
}

/// IR instructions per compile task: enough to keep the threads' queues short, few enough to
/// balance.
const INSTS_PER_TASK: usize = 2_000;

/// Lower and compile every function with a body, in parallel; results in program order.
fn compile_all(shared: &Shared) -> R<Vec<Compiled>> {
    let size = |f: &jaic::ir::Func| f.blocks.iter().map(|b| b.insts.len() + 1).sum::<usize>();
    // Largest first, in tasks of about `INSTS_PER_TASK` instructions: the pool starts the big
    // functions early and the small ones even out the end.
    let mut funcs: Vec<(usize, usize)> = shared
        .program
        .funcs
        .iter()
        .enumerate()
        .filter_map(|(i, f)| f.as_ref().map(|f| (i, size(f))))
        .collect();

    funcs.sort_by_key(|&(i, n)| (std::cmp::Reverse(n), i));
    let mut tasks: Vec<&[(usize, usize)]> = Vec::new();
    let mut start = 0;
    let mut insts = 0;

    for (k, &(_, n)) in funcs.iter().enumerate() {
        insts += n;

        if insts >= INSTS_PER_TASK {
            tasks.push(&funcs[start..=k]);
            (start, insts) = (k + 1, 0);
        }
    }

    if start < funcs.len() {
        tasks.push(&funcs[start..]);
    }

    let results = jaic::pool::units(
        tasks.len(),
        |task| {
            let mut out = Vec::with_capacity(tasks[task].len());
            let mut ctx = cranelift_codegen::Context::new();
            let mut fb = cranelift_frontend::FunctionBuilderContext::new();

            for &(i, _) in tasks[task] {
                out.push(compile_one(shared, i, &mut ctx, &mut fb)?);
            }

            Ok::<_, Error>(out)
        },
        || {},
    );
    let mut all = Vec::with_capacity(funcs.len());

    for result in results {
        all.extend(result?);
    }

    Ok(all)
}

fn compile_one(
    shared: &Shared,
    index: usize,
    ctx: &mut cranelift_codegen::Context,
    fb: &mut cranelift_frontend::FunctionBuilderContext,
) -> R<Compiled> {
    let func = shared.program.funcs[index].as_ref().expect("has a body");
    let name = if func.name.is_empty() {
        func.origin.thunk_name().to_string()
    } else {
        func.name.clone()
    };
    let id = shared.symbols.funcs[index].expect("declared");

    ctx.clear();
    ctx.func = function::lower(shared, index, fb).map_err(|e| e.at(|| format!("in '{name}'")))?;
    finish(shared, id, name, ctx)
}

/// Compile the function in `ctx` to machine code for function `id`.
fn finish(
    shared: &Shared,
    id: FuncId,
    name: String,
    ctx: &mut cranelift_codegen::Context,
) -> R<Compiled> {
    ctx.compile(shared.isa, &mut ControlPlane::default())
        .map_err(|e| format!("in '{name}': Cranelift failed: {:?}", e.inner))?;
    let compiled = ctx.compiled_code().expect("just compiled");
    let alignment = u64::from(compiled.buffer.alignment);
    let code = compiled.buffer.data().to_vec();
    let relocs = compiled
        .buffer
        .relocs()
        .iter()
        .map(|r| ModuleReloc::from_mach_reloc(r, &ctx.func, id))
        .collect();

    Ok(Compiled {
        id,
        name,
        alignment,
        code,
        relocs,
    })
}

/// The Cranelift type of an IR scalar class.
pub(crate) fn clif_ty(ty: jaic::ir::Ty) -> R<ir::Type> {
    use jaic::ir::Ty;
    Ok(match ty {
        Ty::I8 => ir::types::I8,
        Ty::I16 => ir::types::I16,
        Ty::I32 => ir::types::I32,
        Ty::I64 | Ty::Ptr => ir::types::I64,
        Ty::F32 => ir::types::F32,
        Ty::F64 => ir::types::F64,
        Ty::F80 | Ty::F128 => return Err(unsupported("`long double` values")),
    })
}
