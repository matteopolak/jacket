//! Parallel machine-code generation for optimized builds.
//!
//! The optimizer needs the whole program in one module (to inline across it), but after it
//! has run, instruction selection and register allocation work one function at a time. So
//! the optimized module is written to bitcode once, each worker thread reads it into its own
//! LLVM context, keeps the bodies of its share of the functions (and, in unit 0, the global
//! data), turns everything else into declarations, and writes its own object file.
//!
//! Local symbols must be visible across those objects, so before splitting every internal or
//! private definition becomes a hidden external one (unnamed ones get a name). The linked
//! executable exports the same symbols as before; only the objects differ.
#![allow(unsafe_code)]

use inkwell::context::Context;
use inkwell::llvm_sys::LLVMLinkage;
use inkwell::llvm_sys::LLVMVisibility;
use inkwell::llvm_sys::core as llvm;
use inkwell::llvm_sys::prelude::{LLVMModuleRef, LLVMValueRef};
use inkwell::memory_buffer::MemoryBuffer;
use inkwell::module::Module;
use inkwell::targets::{FileType, TargetMachine};
use std::collections::HashMap;
use std::ffi::CStr;
use std::path::{Path, PathBuf};

/// LLVM instructions per codegen unit below which another thread does not pay for itself
/// (each unit re-reads the whole module's bitcode).
const INSTS_PER_UNIT: usize = 10_000;

/// Each unit after the first holds its own copy of the module's declarations and debug
/// information, so more units cost memory for little gain once the optimizer (which stays
/// serial) dominates.
const MAX_UNITS: usize = 4;

/// How many units to split the optimized `module` into (1: don't).
pub(crate) fn units_for(module: &Module) -> usize {
    // Tests force a split of programs far below the threshold.
    if let Some(n) = std::env::var("JAIC_SPLIT_UNITS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
    {
        return n.max(1);
    }
    let insts: usize = functions(module.as_mut_ptr()).into_iter().map(weight).sum();
    let cores = std::thread::available_parallelism().map_or(1, |n| n.get());
    (insts / INSTS_PER_UNIT).clamp(1, cores.min(MAX_UNITS))
}

/// Write the optimized `module` as `units` object files, `path`, then `path.1.o`..., in
/// parallel. `machine` makes a target machine for each unit.
pub(crate) fn emit(
    module: &Module,
    units: usize,
    path: &Path,
    machine: &(dyn Fn() -> Result<TargetMachine, String> + Sync),
) -> Result<Vec<PathBuf>, String> {
    let m = module.as_mut_ptr();
    expose_locals(m);
    // Largest functions first, each to the lightest unit. Unit 0 also holds the data.
    let mut defs: Vec<(String, usize)> = functions(m)
        .into_iter()
        .filter(|&f| is_definition(f))
        .map(|f| (name(f), weight(f)))
        .collect();
    defs.sort_by_key(|d| std::cmp::Reverse(d.1));
    let mut load = vec![0usize; units];
    load[0] = globals(m).len() / 4;
    let mut owner: HashMap<String, usize> = HashMap::default();
    for (name, w) in defs {
        let unit = (0..units).min_by_key(|&u| load[u]).unwrap_or(0);
        load[unit] += w;
        owner.insert(name, unit);
    }
    let bitcode = module.write_bitcode_to_memory().as_slice().to_vec();
    let paths: Vec<PathBuf> = (0..units)
        .map(|u| {
            if u == 0 {
                return path.to_path_buf();
            }
            let mut name = path.to_path_buf().into_os_string();
            name.push(format!(".{u}.o"));
            PathBuf::from(name)
        })
        .collect();
    let reading = std::sync::Mutex::new(());
    std::thread::scope(|scope| {
        let handles: Vec<_> = (1..units)
            .map(|u| {
                let (bitcode, owner, reading, path) = (&bitcode, &owner, &reading, &paths[u]);
                scope.spawn(move || {
                    let context = Context::create();
                    // One unit at a time holds a whole copy of the module; each drops what
                    // it does not own before the next reads, so memory grows by about one
                    // module rather than one per unit.
                    let module = {
                        let _turn = reading.lock().unwrap_or_else(|e| e.into_inner());
                        let buffer = MemoryBuffer::create_from_memory_range_copy(bitcode, "jai");
                        let module = Module::parse_bitcode_from_buffer(&buffer, &context)
                            .map_err(|e| e.to_string())?;
                        keep_unit(&module, owner, u);
                        module
                    };
                    write(&module, machine, path)
                })
            })
            .collect();
        // Unit 0 is the original module, cut down in place.
        keep_unit(module, &owner, 0);
        let first = write(module, machine, path);
        handles
            .into_iter()
            .map(|h| {
                h.join()
                    .unwrap_or_else(|_| Err("codegen thread panicked".into()))
            })
            .chain([first])
            .collect::<Result<Vec<()>, String>>()
    })?;
    Ok(paths)
}

fn write(
    module: &Module,
    machine: &(dyn Fn() -> Result<TargetMachine, String> + Sync),
    path: &Path,
) -> Result<(), String> {
    machine()?
        .write_to_file(module, FileType::Object, path)
        .map_err(|e| e.to_string())
}

/// Reduce `module` to unit `unit`: other units' functions become declarations, and outside
/// unit 0 so does the data.
fn keep_unit(module: &Module, owner: &HashMap<String, usize>, unit: usize) {
    let m = module.as_mut_ptr();
    for f in functions(m) {
        if is_definition(f) && owner.get(&name(f)).copied().unwrap_or(0) != unit {
            strip_body(f);
        }
    }
    if unit == 0 {
        return;
    }
    for g in globals(m) {
        unsafe {
            if llvm::LLVMGetLinkage(g) == LLVMLinkage::LLVMAppendingLinkage {
                // `llvm.used`, `llvm.global_ctors`...: unit 0 has them.
                llvm::LLVMDeleteGlobal(g);
            } else if !llvm::LLVMGetInitializer(g).is_null() {
                llvm::LLVMSetInitializer(g, std::ptr::null_mut());
                llvm::LLVMSetLinkage(g, LLVMLinkage::LLVMExternalLinkage);
                erase_debug_attachment(g);
            }
        }
    }
}

fn functions(m: LLVMModuleRef) -> Vec<LLVMValueRef> {
    let mut out = Vec::new();
    unsafe {
        let mut f = llvm::LLVMGetFirstFunction(m);
        while !f.is_null() {
            out.push(f);
            f = llvm::LLVMGetNextFunction(f);
        }
    }
    out
}

fn globals(m: LLVMModuleRef) -> Vec<LLVMValueRef> {
    let mut out = Vec::new();
    unsafe {
        let mut g = llvm::LLVMGetFirstGlobal(m);
        while !g.is_null() {
            out.push(g);
            g = llvm::LLVMGetNextGlobal(g);
        }
    }
    out
}

fn is_definition(f: LLVMValueRef) -> bool {
    unsafe { llvm::LLVMIsDeclaration(f) == 0 }
}

fn name(v: LLVMValueRef) -> String {
    let mut len = 0usize;
    unsafe {
        let p = llvm::LLVMGetValueName2(v, &mut len);
        if p.is_null() {
            return String::new();
        }
        String::from_utf8_lossy(std::slice::from_raw_parts(p.cast::<u8>(), len)).into_owned()
    }
}

/// Instructions in a function body.
fn weight(f: LLVMValueRef) -> usize {
    let mut n = 0;
    unsafe {
        let mut b = llvm::LLVMGetFirstBasicBlock(f);
        while !b.is_null() {
            let mut i = llvm::LLVMGetFirstInstruction(b);
            while !i.is_null() {
                n += 1;
                i = llvm::LLVMGetNextInstruction(i);
            }
            b = llvm::LLVMGetNextBasicBlock(b);
        }
    }
    n
}

/// Make every local definition a hidden external one, so other units can refer to it.
fn expose_locals(m: LLVMModuleRef) {
    let mut unnamed = 0usize;
    let defs = functions(m)
        .into_iter()
        .filter(|&f| is_definition(f))
        .chain(
            globals(m)
                .into_iter()
                .filter(|&g| unsafe { !llvm::LLVMGetInitializer(g).is_null() }),
        );
    for v in defs.collect::<Vec<_>>() {
        unsafe {
            let linkage = llvm::LLVMGetLinkage(v);
            if !matches!(
                linkage,
                LLVMLinkage::LLVMInternalLinkage | LLVMLinkage::LLVMPrivateLinkage
            ) {
                continue;
            }
            if name(v).is_empty() {
                let fresh = format!("jaic.local.{unnamed}");
                unnamed += 1;
                llvm::LLVMSetValueName2(v, fresh.as_ptr().cast(), fresh.len());
            }
            llvm::LLVMSetLinkage(v, LLVMLinkage::LLVMExternalLinkage);
            llvm::LLVMSetVisibility(v, LLVMVisibility::LLVMHiddenVisibility);
        }
    }
}

/// Turn a function definition into a declaration.
fn strip_body(f: LLVMValueRef) {
    unsafe {
        let mut blocks = Vec::new();
        let mut b = llvm::LLVMGetFirstBasicBlock(f);
        while !b.is_null() {
            blocks.push(b);
            b = llvm::LLVMGetNextBasicBlock(b);
        }
        // Cut every use of an instruction, and point every branch at its own block, so that
        // nothing outside a block refers into it; then delete whole blocks. Erasing the
        // instructions one by one instead moves their debug records onto the next
        // instruction, and from the last one into the context's table of trailing records,
        // keyed by the block's address. Deleting the block leaves that entry behind, and a
        // block that codegen later allocates at the same address picked up another
        // function's variables, which crashed the DWARF writer
        // (`DwarfDebug::finalizeModuleInfo`) at random.
        for &b in &blocks {
            let mut i = llvm::LLVMGetFirstInstruction(b);
            while !i.is_null() {
                let ty = llvm::LLVMTypeOf(i);
                if llvm::LLVMGetTypeKind(ty) != inkwell::llvm_sys::LLVMTypeKind::LLVMVoidTypeKind {
                    llvm::LLVMReplaceAllUsesWith(i, llvm::LLVMGetPoison(ty));
                }
                i = llvm::LLVMGetNextInstruction(i);
            }
            let term = llvm::LLVMGetBasicBlockTerminator(b);
            if !term.is_null() {
                for k in 0..llvm::LLVMGetNumSuccessors(term) {
                    llvm::LLVMSetSuccessor(term, k, b);
                }
            }
        }
        for b in blocks {
            llvm::LLVMDeleteBasicBlock(b);
        }
        llvm::LLVMSetLinkage(f, LLVMLinkage::LLVMExternalLinkage);
        if llvm::LLVMHasPersonalityFn(f) != 0 {
            llvm::LLVMSetPersonalityFn(f, std::ptr::null_mut());
        }
        erase_debug_attachment(f);
    }
}

/// Drop a global's `!dbg` attachment (a declaration may not carry a definition's
/// subprogram).
fn erase_debug_attachment(v: LLVMValueRef) {
    let kind: &CStr = c"dbg";
    unsafe {
        let context = llvm::LLVMGetModuleContext(llvm::LLVMGetGlobalParent(v));
        let id = llvm::LLVMGetMDKindIDInContext(context, kind.as_ptr(), 3);
        llvm::LLVMGlobalEraseMetadata(v, id);
    }
}
