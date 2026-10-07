//! Native debug information (DWARF; CodeView-ready) for lowered programs.
//!
//! The IR carries what a debugger needs: `Inst::Loc` line markers (with a lexical scope),
//! an optional `ir::FuncDebug` side table per function (named variables and scopes) and
//! `Program::debug_types` (Jai type layouts). This module turns them into LLVM debug
//! metadata: one compile unit per module (so split codegen units each describe their own
//! functions), a `DISubprogram` per defined function, `DILexicalBlock`s for scopes,
//! `#dbg_declare` records for variables and a debug location on every instruction.
//!
//! inkwell's debug-info wrappers cannot build forward-declared (replaceable) struct types
//! and return dangling handles for LLVM 19+ debug records, so this module talks to the
//! LLVM-C `DIBuilder` API directly; the unsafe calls are confined to the `raw` helpers.
//! See `docs/native/debug-info.md`.
#![allow(unsafe_code, clippy::too_many_arguments)]

use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::{AsContextRef, Context};
use inkwell::llvm_sys::debuginfo as di;
use inkwell::llvm_sys::prelude::{LLVMDIBuilderRef, LLVMMetadataRef};
use inkwell::module::{FlagBehavior, Module};
use inkwell::values::{AsValueRef, BasicValueEnum, FunctionValue, GlobalValue, PointerValue};
use jaic::fxhash::HashMap;
use jaic::ir::{DebugGlobal, DebugTypeKind, Func, Inst, Program, Term, Val};
use std::cell::RefCell;
use std::path::Path;

type Md = LLVMMetadataRef;

/// Which debug format the target's tools read.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DebugFormat {
    /// DWARF version 4 (Apple) or 5.
    Dwarf(u32),
    /// Windows PDB-style CodeView (`-windows-msvc` targets).
    CodeView,
}

impl DebugFormat {
    pub fn for_triple(triple: &str) -> Self {
        if triple.contains("windows-msvc") {
            DebugFormat::CodeView
        } else if triple.contains("apple") {
            DebugFormat::Dwarf(4)
        } else {
            DebugFormat::Dwarf(5)
        }
    }
}

// DWARF constants.
const DW_TAG_STRUCTURE_TYPE: u32 = 0x13;

const DW_TAG_UNION_TYPE: u32 = 0x17;
const DW_ATE_BOOLEAN: u32 = 0x02;
const DW_ATE_FLOAT: u32 = 0x04;
const DW_ATE_SIGNED: u32 = 0x05;
const DW_ATE_UNSIGNED: u32 = 0x07;
const DW_ATE_UNSIGNED_CHAR: u32 = 0x08;
const DW_OP_DEREF: u64 = 0x06;

/// Thin wrappers over the LLVM-C DIBuilder functions this module uses.
mod raw {
    use super::*;
    use std::ffi::c_char;

    fn s(text: &str) -> (*const c_char, usize) {
        (text.as_ptr() as *const c_char, text.len())
    }

    pub fn file(b: LLVMDIBuilderRef, name: &str, dir: &str) -> Md {
        let (n, nl) = s(name);
        let (d, dl) = s(dir);
        unsafe { di::LLVMDIBuilderCreateFile(b, n, nl, d, dl) }
    }

    pub fn compile_unit(b: LLVMDIBuilderRef, file: Md, optimized: bool) -> Md {
        let (p, pl) = s("jaic");
        let empty = s("");
        unsafe {
            di::LLVMDIBuilderCreateCompileUnit(
                b,
                di::LLVMDWARFSourceLanguage::LLVMDWARFSourceLanguageC99,
                file,
                p,
                pl,
                optimized as i32,
                empty.0,
                0,
                0,
                empty.0,
                0,
                di::LLVMDWARFEmissionKind::LLVMDWARFEmissionKindFull,
                0,
                0,
                0,
                empty.0,
                0,
                empty.0,
                0,
            )
        }
    }

    pub fn function(
        b: LLVMDIBuilderRef,
        file: Md,
        name: &str,
        linkage: &str,
        line: u32,
        ty: Md,
        local: bool,
        optimized: bool,
    ) -> Md {
        let (n, nl) = s(name);
        let (l, ll) = s(linkage);
        unsafe {
            di::LLVMDIBuilderCreateFunction(
                b,
                file,
                n,
                nl,
                l,
                ll,
                file,
                line,
                ty,
                local as i32,
                1,
                line,
                di::LLVMDIFlagPrototyped,
                optimized as i32,
            )
        }
    }

    pub fn subroutine_type(b: LLVMDIBuilderRef, file: Md) -> Md {
        let mut params: [Md; 1] = [std::ptr::null_mut()];
        unsafe { di::LLVMDIBuilderCreateSubroutineType(b, file, params.as_mut_ptr(), 1, 0) }
    }

    pub fn lexical_block(b: LLVMDIBuilderRef, scope: Md, file: Md, line: u32, col: u32) -> Md {
        unsafe { di::LLVMDIBuilderCreateLexicalBlock(b, scope, file, line, col) }
    }

    pub fn location(ctx: &Context, line: u32, col: u32, scope: Md) -> Md {
        unsafe {
            di::LLVMDIBuilderCreateDebugLocation(
                ctx.as_ctx_ref(),
                line,
                col,
                scope,
                std::ptr::null_mut(),
            )
        }
    }

    pub fn set_location(builder: &Builder, loc: Md) {
        unsafe { inkwell::llvm_sys::core::LLVMSetCurrentDebugLocation2(builder.as_mut_ptr(), loc) }
    }

    pub fn set_subprogram(function: FunctionValue, sp: Md) {
        unsafe { di::LLVMSetSubprogram(function.as_value_ref(), sp) }
    }

    pub fn basic(b: LLVMDIBuilderRef, name: &str, bits: u64, encoding: u32) -> Md {
        let (n, nl) = s(name);
        unsafe { di::LLVMDIBuilderCreateBasicType(b, n, nl, bits, encoding, 0) }
    }

    pub fn pointer(b: LLVMDIBuilderRef, pointee: Md, name: &str) -> Md {
        let (n, nl) = s(name);
        unsafe { di::LLVMDIBuilderCreatePointerType(b, pointee, 64, 0, 0, n, nl) }
    }

    pub fn typedef(b: LLVMDIBuilderRef, ty: Md, name: &str, file: Md, scope: Md) -> Md {
        let (n, nl) = s(name);
        unsafe { di::LLVMDIBuilderCreateTypedef(b, ty, n, nl, file, 0, scope, 0) }
    }

    pub fn forward_struct(
        b: LLVMDIBuilderRef,
        tag: u32,
        name: &str,
        scope: Md,
        file: Md,
        bits: u64,
        align_bits: u32,
    ) -> Md {
        let (n, nl) = s(name);
        let empty = s("");
        unsafe {
            di::LLVMDIBuilderCreateReplaceableCompositeType(
                b, tag, n, nl, scope, file, 0, 0, bits, align_bits, 0, empty.0, 0,
            )
        }
    }

    pub fn member(
        b: LLVMDIBuilderRef,
        scope: Md,
        name: &str,
        file: Md,
        bits: u64,
        align_bits: u32,
        offset_bits: u64,
        ty: Md,
    ) -> Md {
        let (n, nl) = s(name);
        unsafe {
            di::LLVMDIBuilderCreateMemberType(
                b,
                scope,
                n,
                nl,
                file,
                0,
                bits,
                align_bits,
                offset_bits,
                0,
                ty,
            )
        }
    }

    pub fn composite(
        b: LLVMDIBuilderRef,
        union: bool,
        scope: Md,
        name: &str,
        file: Md,
        bits: u64,
        align_bits: u32,
        members: &mut [Md],
    ) -> Md {
        let (n, nl) = s(name);
        let empty = s("");
        let count = members.len() as u32;
        unsafe {
            if union {
                di::LLVMDIBuilderCreateUnionType(
                    b,
                    scope,
                    n,
                    nl,
                    file,
                    0,
                    bits,
                    align_bits,
                    0,
                    members.as_mut_ptr(),
                    count,
                    0,
                    empty.0,
                    0,
                )
            } else {
                di::LLVMDIBuilderCreateStructType(
                    b,
                    scope,
                    n,
                    nl,
                    file,
                    0,
                    bits,
                    align_bits,
                    0,
                    std::ptr::null_mut(),
                    members.as_mut_ptr(),
                    count,
                    0,
                    std::ptr::null_mut(),
                    empty.0,
                    0,
                )
            }
        }
    }

    pub fn replace(temp: Md, real: Md) {
        unsafe { di::LLVMMetadataReplaceAllUsesWith(temp, real) }
    }

    pub fn array(b: LLVMDIBuilderRef, elem: Md, count: u64, bits: u64, align_bits: u32) -> Md {
        unsafe {
            let mut sub = [di::LLVMDIBuilderGetOrCreateSubrange(b, 0, count as i64)];
            di::LLVMDIBuilderCreateArrayType(b, bits, align_bits, elem, sub.as_mut_ptr(), 1)
        }
    }

    pub fn enumeration(
        b: LLVMDIBuilderRef,
        scope: Md,
        name: &str,
        file: Md,
        bits: u64,
        align_bits: u32,
        members: &[(String, i64)],
        base: Md,
        unsigned: bool,
    ) -> Md {
        let mut elements: Vec<Md> = members
            .iter()
            .map(|(m, v)| {
                let (n, nl) = s(m);
                unsafe { di::LLVMDIBuilderCreateEnumerator(b, n, nl, *v, unsigned as i32) }
            })
            .collect();
        let (n, nl) = s(name);
        unsafe {
            di::LLVMDIBuilderCreateEnumerationType(
                b,
                scope,
                n,
                nl,
                file,
                0,
                bits,
                align_bits,
                elements.as_mut_ptr(),
                elements.len() as u32,
                base,
            )
        }
    }

    pub fn expression(b: LLVMDIBuilderRef, ops: &mut [u64]) -> Md {
        unsafe { di::LLVMDIBuilderCreateExpression(b, ops.as_mut_ptr(), ops.len()) }
    }

    pub fn variable(
        b: LLVMDIBuilderRef,
        scope: Md,
        name: &str,
        arg: u32,
        file: Md,
        line: u32,
        ty: Md,
    ) -> Md {
        let (n, nl) = s(name);
        unsafe {
            if arg > 0 {
                di::LLVMDIBuilderCreateParameterVariable(b, scope, n, nl, arg, file, line, ty, 1, 0)
            } else {
                di::LLVMDIBuilderCreateAutoVariable(b, scope, n, nl, file, line, ty, 1, 0, 0)
            }
        }
    }

    pub fn declare(
        b: LLVMDIBuilderRef,
        storage: PointerValue,
        var: Md,
        expr: Md,
        loc: Md,
        block: BasicBlock,
    ) {
        unsafe {
            di::LLVMDIBuilderInsertDeclareRecordAtEnd(
                b,
                storage.as_value_ref(),
                var,
                expr,
                loc,
                block.as_mut_ptr(),
            );
        }
    }

    pub fn global_variable(
        b: LLVMDIBuilderRef,
        scope: Md,
        name: &str,
        linkage: &str,
        file: Md,
        line: u32,
        ty: Md,
        local: bool,
        expr: Md,
    ) -> Md {
        let (n, nl) = s(name);
        let (l, ll) = s(linkage);
        unsafe {
            di::LLVMDIBuilderCreateGlobalVariableExpression(
                b,
                scope,
                n,
                nl,
                l,
                ll,
                file,
                line,
                ty,
                local as i32,
                expr,
                std::ptr::null_mut(),
                0,
            )
        }
    }

    pub fn attach_global(global: GlobalValue, kind: u32, gve: Md) {
        unsafe { inkwell::llvm_sys::core::LLVMGlobalSetMetadata(global.as_value_ref(), kind, gve) }
    }

    pub fn create(module: &Module) -> LLVMDIBuilderRef {
        unsafe { di::LLVMCreateDIBuilder(module.as_mut_ptr()) }
    }

    pub fn finalize(b: LLVMDIBuilderRef) {
        unsafe { di::LLVMDIBuilderFinalize(b) }
    }

    pub fn dispose(b: LLVMDIBuilderRef) {
        unsafe { di::LLVMDisposeDIBuilder(b) }
    }
}

/// Module-wide debug info state: the DIBuilder, compile unit, files and types.
pub struct DebugInfo<'ctx, 'p> {
    ctx: &'ctx Context,
    builder: LLVMDIBuilderRef,
    program: &'p Program,
    optimized: bool,
    /// `DIFile` per source file id.
    files: RefCell<HashMap<u32, Md>>,
    /// Debug type per `Program::debug_types` key (`None`: no storage, e.g. `void`).
    types: RefCell<HashMap<u32, Option<Md>>>,
    subroutine_type: Md,
    cu_file: Md,
}

impl Drop for DebugInfo<'_, '_> {
    fn drop(&mut self) {
        raw::dispose(self.builder);
    }
}

impl<'ctx, 'p> DebugInfo<'ctx, 'p> {
    pub fn new(
        ctx: &'ctx Context,
        module: &Module<'ctx>,
        program: &'p Program,
        format: DebugFormat,
        optimized: bool,
    ) -> Self {
        let i32t = ctx.i32_type();
        module.add_basic_value_flag(
            "Debug Info Version",
            FlagBehavior::Warning,
            i32t.const_int(inkwell::debug_info::debug_metadata_version() as u64, false),
        );
        match format {
            DebugFormat::Dwarf(version) => module.add_basic_value_flag(
                "Dwarf Version",
                FlagBehavior::Warning,
                i32t.const_int(version as u64, false),
            ),
            DebugFormat::CodeView => module.add_basic_value_flag(
                "CodeView",
                FlagBehavior::Warning,
                i32t.const_int(1, false),
            ),
        }
        let builder = raw::create(module);
        let mut this = DebugInfo {
            ctx,
            builder,
            program,
            optimized,
            files: RefCell::new(HashMap::default()),
            types: RefCell::new(HashMap::default()),
            subroutine_type: std::ptr::null_mut(),
            cu_file: std::ptr::null_mut(),
        };
        let main_file = program
            .funcs
            .iter()
            .flatten()
            .find(|f| f.name == "main")
            .map_or(0, |f| f.source_file);
        this.cu_file = this.file(main_file);
        raw::compile_unit(builder, this.cu_file, optimized);
        this.subroutine_type = raw::subroutine_type(builder, this.cu_file);
        this
    }

    /// Resolve forward references; must run before the module is verified or emitted.
    pub fn finalize(&self) {
        raw::finalize(self.builder);
    }

    fn file(&self, id: u32) -> Md {
        if let Some(&f) = self.files.borrow().get(&id) {
            return f;
        }
        let path = self
            .program
            .file_paths
            .get(id as usize)
            .map_or("<unknown>.jai", |p| p.as_str());
        let absolute = std::path::absolute(path).unwrap_or_else(|_| Path::new(path).to_path_buf());
        let name = absolute
            .file_name()
            .map_or_else(|| path.to_string(), |n| n.to_string_lossy().into_owned());
        let dir = absolute
            .parent()
            .map_or_else(String::new, |d| d.to_string_lossy().into_owned());
        let f = raw::file(self.builder, &name, &dir);
        self.files.borrow_mut().insert(id, f);
        f
    }

    /// The debug type for a `Program::debug_types` key.
    fn ty(&self, key: u32) -> Option<Md> {
        if let Some(&t) = self.types.borrow().get(&key) {
            return t;
        }
        let b = self.builder;
        let Some(t) = self.program.debug_types.get(&key) else {
            self.types.borrow_mut().insert(key, None);
            return None;
        };
        let bits = t.size * 8;
        let align_bits = (t.align * 8) as u32;
        let scope = self.cu_file;
        let md = match &t.kind {
            DebugTypeKind::Void => None,
            DebugTypeKind::Bool => Some(raw::basic(b, &t.name, bits, DW_ATE_BOOLEAN)),
            DebugTypeKind::Int {
                signed,
            } => {
                let encoding = if *signed {
                    DW_ATE_SIGNED
                } else {
                    DW_ATE_UNSIGNED
                };
                Some(self.named_basic(&t.name, bits, encoding))
            }
            DebugTypeKind::Char => Some(raw::basic(b, &t.name, 8, DW_ATE_UNSIGNED_CHAR)),
            DebugTypeKind::Float => Some(self.named_basic(&t.name, bits, DW_ATE_FLOAT)),
            DebugTypeKind::Pointer(p) => {
                // A pointer breaks no cycle by itself; structs insert themselves first.
                let pointee = self.ty(*p).unwrap_or(std::ptr::null_mut());
                Some(raw::pointer(b, pointee, ""))
            }
            DebugTypeKind::Typedef(base) => {
                let base = self.ty(*base).unwrap_or(std::ptr::null_mut());
                Some(raw::typedef(b, base, &t.name, self.cu_file, scope))
            }
            DebugTypeKind::Array {
                elem,
                count,
            } => self
                .ty(*elem)
                .map(|e| raw::array(b, e, *count, bits, align_bits)),
            DebugTypeKind::Enum {
                base,
                members,
            } => {
                let unsigned = matches!(
                    self.program.debug_types.get(base).map(|t| &t.kind),
                    Some(DebugTypeKind::Int {
                        signed: false
                    })
                );
                let base = self.ty(*base).unwrap_or(std::ptr::null_mut());
                Some(raw::enumeration(
                    b,
                    scope,
                    &t.name,
                    self.cu_file,
                    bits,
                    align_bits,
                    members,
                    base,
                    unsigned,
                ))
            }
            DebugTypeKind::Struct {
                fields,
                union,
            } => {
                let tag = if *union {
                    DW_TAG_UNION_TYPE
                } else {
                    DW_TAG_STRUCTURE_TYPE
                };
                // Insert a forward declaration first so self-referential fields resolve.
                let temp =
                    raw::forward_struct(b, tag, &t.name, scope, self.cu_file, bits, align_bits);
                self.types.borrow_mut().insert(key, Some(temp));
                let mut members = Vec::with_capacity(fields.len());
                for f in fields {
                    let Some(fty) = self.ty(f.ty) else {
                        continue;
                    };
                    let (size, align) = self
                        .program
                        .debug_types
                        .get(&f.ty)
                        .map_or((0, 1), |t| (t.size, t.align));
                    members.push(raw::member(
                        b,
                        temp,
                        &f.name,
                        self.cu_file,
                        size * 8,
                        (align * 8) as u32,
                        f.offset * 8,
                        fty,
                    ));
                }
                let real = raw::composite(
                    b,
                    *union,
                    scope,
                    &t.name,
                    self.cu_file,
                    bits,
                    align_bits,
                    &mut members,
                );
                raw::replace(temp, real);
                Some(real)
            }
        };
        self.types.borrow_mut().insert(key, md);
        md
    }

    /// A basic type that debuggers show by its Jai name. LLDB maps DWARF base types to C
    /// types by encoding and size (`s64` would print as `long`), but keeps typedef names.
    fn named_basic(&self, name: &str, bits: u64, encoding: u32) -> Md {
        let base = raw::basic(self.builder, name, bits, encoding);
        raw::typedef(self.builder, base, name, self.cu_file, self.cu_file)
    }

    /// Describe a program global (call where the global is defined).
    pub fn global(&self, gv: GlobalValue<'ctx>, linkage_name: &str, local: bool, g: &DebugGlobal) {
        let Some(ty) = self.ty(g.ty) else {
            return;
        };
        let file = self.file(g.file);
        let expr = raw::expression(self.builder, &mut []);
        let gve = raw::global_variable(
            self.builder,
            file,
            &g.name,
            linkage_name,
            file,
            g.line,
            ty,
            local,
            expr,
        );
        raw::attach_global(gv, self.ctx.get_kind_id("dbg"), gve);
    }

    /// Attach a subprogram to `function` and position both builders' debug locations at
    /// its first line.
    pub fn begin_function(
        &self,
        func: &Func,
        function: FunctionValue<'ctx>,
        local: bool,
        builder: &Builder<'ctx>,
        allocas: &Builder<'ctx>,
    ) -> FnDebug<'ctx> {
        let debug = func.debug.as_deref();
        let file = self.file(func.source_file);
        let line = debug.map_or_else(|| first_line(func).unwrap_or(0), |d| d.line);
        let name = debug.map_or(func.name.as_str(), |d| d.name.as_str());
        let name = if name.is_empty() {
            func.name.as_str()
        } else {
            name
        };
        // No linkage name: debuggers would show the internal symbol (`name.index`) instead.
        let sp = raw::function(
            self.builder,
            file,
            name,
            "",
            line,
            self.subroutine_type,
            local,
            self.optimized,
        );
        raw::set_subprogram(function, sp);
        // The entry block (allocas, parameter binding; `declare_vars` also stores scalar
        // parameters to their slots there) is where LLVM puts the end of the prologue, so it
        // gets the first statement's line: a breakpoint on the procedure stops there with
        // the parameters in place. Code before the first statement in the body (parameter
        // spills, stack trace bookkeeping) is line 0, which stepping skips.
        // Allocas have no location: -O0 instruction selection materializes their addresses
        // where they are used and would attribute that code to the entry line.
        let first = first_line(func).unwrap_or(line);
        let entry = raw::location(self.ctx, first, 0, sp);
        raw::set_location(builder, entry);
        raw::set_location(allocas, std::ptr::null_mut());
        let scope_count = debug.map_or(1, |d| d.scopes.len().max(1));
        let mut scopes = vec![None; scope_count];
        scopes[0] = Some(sp);
        FnDebug {
            file,
            entry,
            start: (0, 0, 0),
            current: (0, 0, 0),
            scopes,
            block_entry: vec![None; func.blocks.len()],
            watch: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }
}

/// Line of the first statement of a function (blocks are not in source order, and the
/// stack trace instrumentation moves the entry block's code to the end).
fn first_line(func: &Func) -> Option<u32> {
    func.blocks
        .iter()
        .flat_map(|b| &b.insts)
        .filter_map(|i| match i {
            Inst::Loc {
                line, ..
            } if *line > 0 => Some(*line),
            _ => None,
        })
        .min()
}

/// Per-function debug state while its body is lowered.
pub struct FnDebug<'ctx> {
    file: Md,
    /// Location of the entry block's branch into the body (the end of the prologue).
    entry: Md,
    /// (line, col, scope) at the function's start (line 0: before the first statement).
    start: (u32, u32, u32),
    current: (u32, u32, u32),
    /// `DIScope` per `FuncDebug::scopes` entry, created on first use.
    scopes: Vec<Option<Md>>,
    /// The location in effect where each IR block starts (from its first lowered predecessor).
    block_entry: Vec<Option<(u32, u32, u32)>>,
    /// Variables whose address is an instruction result: the slot it is copied to.
    pub watch: Vec<(Val, PointerValue<'ctx>)>,
    _marker: std::marker::PhantomData<&'ctx ()>,
}

impl<'ctx> FnDebug<'ctx> {
    fn scope(&mut self, di: &DebugInfo<'ctx, '_>, func: &Func, index: u32) -> Md {
        let index = index as usize;
        if let Some(Some(s)) = self.scopes.get(index) {
            return *s;
        }
        let Some(info) = func
            .debug
            .as_ref()
            .and_then(|d| d.scopes.get(index).copied())
        else {
            return self.scopes[0].expect("subprogram");
        };
        let parent = if info.parent as usize == index {
            self.scopes[0].expect("subprogram")
        } else {
            self.scope(di, func, info.parent)
        };
        let block = raw::lexical_block(di.builder, parent, self.file, info.line, info.col);
        self.scopes[index] = Some(block);
        block
    }

    fn apply(&mut self, di: &DebugInfo<'ctx, '_>, func: &Func, builder: &Builder<'ctx>) {
        let (line, col, scope) = self.current;
        let scope = self.scope(di, func, scope);
        raw::set_location(builder, raw::location(di.ctx, line, col, scope));
    }

    /// An `Inst::Loc` marker.
    pub fn loc(
        &mut self,
        di: &DebugInfo<'ctx, '_>,
        func: &Func,
        builder: &Builder<'ctx>,
        line: u32,
        col: u32,
        scope: u32,
    ) {
        if self.current != (line, col, scope) {
            self.current = (line, col, scope);
            self.apply(di, func, builder);
        }
    }

    /// Before lowering IR block `b`: continue from the location its predecessor ended at.
    pub fn enter_block(
        &mut self,
        di: &DebugInfo<'ctx, '_>,
        func: &Func,
        builder: &Builder<'ctx>,
        b: usize,
    ) {
        self.current = self.block_entry[b].unwrap_or(self.start);
        self.apply(di, func, builder);
    }

    /// After lowering a block: its successors start where it ended.
    pub fn leave_block(&mut self, term: &Term) {
        let mut mark = |b: jaic::ir::BlockId| {
            let entry = &mut self.block_entry[b.0 as usize];
            if entry.is_none() {
                *entry = Some(self.current);
            }
        };
        match term {
            Term::Jump(b) => mark(*b),
            Term::Branch {
                then_block,
                else_block,
                ..
            } => {
                mark(*then_block);
                mark(*else_block);
            }
            Term::Switch {
                cases,
                default,
                ..
            } => {
                for (_, b) in cases {
                    mark(*b);
                }
                mark(*default);
            }
            Term::Ret(_) | Term::Unreachable => {}
        }
    }

    /// Describe the function's variables. `slots` are the slot allocas, `vals` the values
    /// bound so far (the parameters); variables at other addresses are copied to a pointer
    /// slot when their address is computed (`watch`).
    pub fn declare_vars(
        &mut self,
        di: &DebugInfo<'ctx, '_>,
        func: &Func,
        slots: &[PointerValue<'ctx>],
        vals: &[Option<BasicValueEnum<'ctx>>],
        allocas: &Builder<'ctx>,
        alloca_block: BasicBlock<'ctx>,
    ) {
        let Some(debug) = func.debug.as_deref() else {
            return;
        };
        if debug.vars.is_empty() {
            return;
        }
        let mut slot_of: HashMap<u32, u32> = HashMap::default();
        for inst in func.blocks.iter().flat_map(|b| &b.insts) {
            if let Inst::SlotAddr {
                dst,
                slot,
            } = inst
            {
                slot_of.insert(dst.0, slot.0);
            }
        }
        // (slot, value) pairs the body stores as a scalar. An aggregate parameter arrives as
        // a pointer that the body copies from; storing that pointer into the parameter's slot
        // would write the address, not the value, and past the end of a slot under 8 bytes.
        let mut scalar_stores = std::collections::HashSet::new();
        for inst in func.blocks.iter().flat_map(|b| &b.insts) {
            if let Inst::Store {
                addr,
                value,
                ..
            } = inst
                && let Some(&slot) = slot_of.get(&addr.0)
            {
                scalar_stores.insert((slot, value.0));
            }
        }
        // The stores below have no location: they belong to the prologue, so LLVM ends the
        // prologue after them, where a breakpoint on the procedure sees the parameters.
        let deref = raw::expression(di.builder, &mut [DW_OP_DEREF]);
        let direct = raw::expression(di.builder, &mut []);
        let ptr_ty = di.ctx.ptr_type(inkwell::AddressSpace::default());
        let mut seen = std::collections::HashSet::new();
        for var in &debug.vars {
            if !seen.insert((var.name.as_str(), var.scope, var.line, var.col, var.arg)) {
                continue;
            }
            let Some(ty) = di.ty(var.ty) else {
                continue;
            };
            let scope = self.scope(di, func, var.scope);
            let (storage, expr) = if let Some(&slot_id) = slot_of.get(&var.addr.0) {
                let slot = slots[slot_id as usize];
                // A scalar parameter spilled to its slot: store it already in the entry
                // block, where a breakpoint on the procedure stops (the body stores it again).
                if var.arg > 0
                    && scalar_stores.contains(&(slot_id, var.arg - 1))
                    && let Some(value) = vals.get(var.arg as usize - 1).copied().flatten()
                {
                    let _ = allocas.build_store(slot, value);
                }
                (slot, direct)
            } else {
                let Ok(spill) = allocas.build_alloca(ptr_ty, "dbg.addr") else {
                    continue;
                };
                match vals.get(var.addr.0 as usize).copied().flatten() {
                    Some(value) => {
                        if allocas.build_store(spill, value).is_err() {
                            continue;
                        }
                    }
                    None => self.watch.push((var.addr, spill)),
                }
                (spill, deref)
            };
            let dvar = raw::variable(
                di.builder, scope, &var.name, var.arg, self.file, var.line, ty,
            );
            let loc = raw::location(di.ctx, var.line, var.col, scope);
            raw::declare(di.builder, storage, dvar, expr, loc, alloca_block);
        }
    }

    /// Before the entry block's branch into the body is built.
    pub fn finish_entry(&self, allocas: &Builder<'ctx>) {
        raw::set_location(allocas, self.entry);
    }

    /// The pointer slot a watched variable address is copied to.
    pub fn watched(&self, v: Val) -> Option<PointerValue<'ctx>> {
        self.watch.iter().find(|(w, _)| *w == v).map(|&(_, p)| p)
    }
}
