//! Declaring functions, foreign symbols and globals, and initializing globals.
use super::error::Error;
use super::{Backend, R};
use inkwell::DLLStorageClass;
use inkwell::GlobalVisibility;
use inkwell::attributes::{Attribute, AttributeLoc};
use inkwell::module::Linkage;
use inkwell::types::{BasicType, BasicTypeEnum};
use inkwell::values::{BasicValueEnum, FunctionValue, GlobalValue, PointerValue};
use jaic::ir::{Foreign, Func, Global, Linkage as IrLinkage, RelocTarget};

impl<'ctx, 'p> Backend<'ctx, 'p> {
    pub(super) fn owns_func(&self, i: usize) -> bool {
        self.shard.is_none_or(|s| s.plan.func_owner[i] == s.index)
    }

    pub(super) fn owns_global(&self, i: usize) -> bool {
        self.shard.is_none_or(|s| s.plan.global_owner[i] == s.index)
    }

    /// Whether a `#program_export` is one of Wasi_Runtime's C library definitions (`malloc`,
    /// `fopen`, `fma`...) rather than an entry point. Those stay external symbols, which the
    /// program's references bind to, but are not exports of the wasm module: wasm-ld then drops
    /// the ones nothing calls, as it would members of a C library. `_start` is still exported.
    fn is_wasi_library(&self, func: &jaic::ir::Func, name: &str) -> bool {
        name != "_start"
            && self
                .program
                .file_paths
                .get(func.source_file as usize)
                .and_then(|p| std::path::Path::new(p).parent()?.file_name())
                .is_some_and(|dir| dir == "Wasi_Runtime")
    }

    /// Linkage of an internal symbol: hidden and external when other modules refer to it
    /// (`shared`) or when it is only declared here, internal when this module alone uses it.
    fn internal_linkage(&self, gv: GlobalValue<'ctx>, owned: bool, shared: bool) {
        if self.shard.is_some() && (shared || !owned) {
            gv.set_linkage(Linkage::External);
            // COFF has no hidden visibility, and MinGW would list the symbol in an unquoted
            // `-exclude-symbols:` linker directive, which names with spaces break.
            if !self.arch.is_windows() {
                gv.set_visibility(GlobalVisibility::Hidden);
            }
        } else {
            gv.set_linkage(Linkage::Internal);
        }
    }

    /// Declare the program's function or foreign of this name, if it has one, before code that
    /// looks a symbol up by name adds its own: the module would otherwise rename the later one.
    pub(super) fn declare_named(&self, name: &str) {
        if let Some(&i) = self.exports.get(name) {
            self.func(i);
        }
        if let Some(&id) = self.foreign_ids.get(name) {
            self.foreign(id);
        }
    }

    /// The declaration of program function `i`, made on first use (`None`: never lowered).
    pub(super) fn func(&self, i: usize) -> Option<FunctionValue<'ctx>> {
        let func = self.program.funcs.get(i)?.as_ref()?;
        Some(*self.funcs[i].get_or_init(|| self.declare_function(i, func)))
    }

    fn declare_function(&self, i: usize, func: &Func) -> FunctionValue<'ctx> {
        let lowered = self.lower_sig(&func.sig);
        let name = match &func.linkage {
            IrLinkage::Export(name) => name.clone(),
            IrLinkage::Internal => format!("{}.{i}", func.name),
        };
        let f = self
            .module
            .add_function(&name, lowered.fn_ty, Some(Linkage::External));
        if func.linkage == IrLinkage::Internal {
            let shared = self.shard.is_some_and(|s| s.plan.func_shared[i]);
            self.internal_linkage(f.as_global_value(), self.owns_func(i), shared);
        } else if self.arch.is_windows() && self.owns_func(i) {
            // `#program_export`: in a DLL's export table (harmless in an executable).
            f.as_global_value()
                .set_dll_storage_class(DLLStorageClass::Export);
        } else if self.arch.is_wasm() && self.owns_func(i) && !self.is_wasi_library(func, &name) {
            // `#program_export`: an export of the wasm module, which is how the host calls in.
            f.add_attribute(
                AttributeLoc::Function,
                self.ctx.create_string_attribute("wasm-export-name", &name),
            );
        }
        if self.imported.get(i).copied().unwrap_or(false) {
            f.as_global_value()
                .set_linkage(Linkage::AvailableExternally);
        }
        self.apply_attrs(&lowered, |loc, attr| f.add_attribute(loc, attr));
        let inline = match func.inline {
            jaic::ir::InlineHint::Always => Some("alwaysinline"),
            jaic::ir::InlineHint::Never => Some("noinline"),
            jaic::ir::InlineHint::Default if self.no_inline.get(i).copied().unwrap_or(false) => {
                Some("noinline")
            }
            jaic::ir::InlineHint::Default => None,
        };
        if let Some(name) = inline {
            f.add_attribute(
                AttributeLoc::Function,
                self.ctx
                    .create_enum_attribute(Attribute::get_named_enum_kind_id(name), 0),
            );
        }
        // Keep a frame record in every function that calls another, as clang does by default
        // on Apple and AArch64 targets. Without one, frame-pointer stack walks (macOS libc
        // `backtrace`, which Debug uses, and sampling profilers) stop at the first jaic frame.
        f.add_attribute(
            AttributeLoc::Function,
            self.ctx
                .create_string_attribute("frame-pointer", "non-leaf"),
        );
        if self.arch.is_wasm() {
            // Freestanding, like `-fno-builtin`: on wasm the C library is Jai code in the
            // same module (Wasi_Runtime), so a loop LLVM turned into a `strlen` or `memcmp`
            // call could end up calling itself.
            f.add_attribute(
                AttributeLoc::Function,
                self.ctx.create_string_attribute("no-builtins", ""),
            );
        }
        f
    }

    /// The address of foreign function or variable `id`, declared on first use.
    pub(super) fn foreign(&self, id: usize) -> PointerValue<'ctx> {
        *self.foreigns[id].get_or_init(|| self.declare_foreign(&self.program.foreigns[id]))
    }

    fn declare_foreign(&self, foreign: &'p Foreign) -> PointerValue<'ctx> {
        let Foreign {
            symbol,
            sig,
            is_data,
            ..
        } = foreign;
        if let Some(&p) = self.foreign_symbols.borrow().get(symbol.as_str()) {
            return p;
        }
        // A `#program_export` of the same name (a runtime written in Jai that supplies
        // `malloc`, say) is what the linker would bind the reference to. Use it directly:
        // a second LLVM function of that name would be renamed and stay undefined.
        let defined = if *is_data {
            None
        } else {
            self.exports
                .get(symbol.as_str())
                .and_then(|&i| self.func(i))
        };
        let ptr = if let Some(f) = defined {
            f.as_global_value().as_pointer_value()
        } else if *is_data {
            let g = self.module.add_global(self.ctx.i8_type(), None, symbol);
            g.set_linkage(Linkage::External);
            // A Windows DLL exports a variable only as `__imp_<name>`, a pointer to it:
            // import libraries define no `<name>` for data. System libraries are DLLs there,
            // except the C runtime, which MSVC builds link statically.
            let dll = foreign
                .library
                .and_then(|l| self.program.libraries.get(l))
                .is_some_and(|l| l.system && !crate::is_windows_c_runtime(&l.name));
            if self.arch.is_windows() && dll {
                g.set_dll_storage_class(DLLStorageClass::Import);
            }
            g.as_pointer_value()
        } else {
            let lowered = self.lower_sig(sig);
            let f = self
                .module
                .add_function(symbol, lowered.fn_ty, Some(Linkage::External));
            if self.arch.is_wasm() && !symbol.starts_with("llvm.") {
                self.wasm_import(f, symbol, foreign.library);
            }
            f.as_global_value().as_pointer_value()
        };
        self.foreign_symbols
            .borrow_mut()
            .insert(symbol.as_str(), ptr);
        ptr
    }

    /// Make an undefined function a wasm import. The module is the `#library` it was declared
    /// with (`wasi_snapshot_preview1`), or `env` for the C library and for `#foreign` without
    /// one, which is where wasm-ld and most hosts look by default.
    pub(super) fn wasm_import(&self, f: FunctionValue<'ctx>, symbol: &str, library: Option<usize>) {
        let module = library
            .and_then(|i| self.program.libraries.get(i))
            .map(|lib| wasm_import_module(&lib.name))
            .unwrap_or("env");
        for (key, value) in [("wasm-import-module", module), ("wasm-import-name", symbol)] {
            f.add_attribute(
                AttributeLoc::Function,
                self.ctx.create_string_attribute(key, value),
            );
        }
    }

    /// Initial-value layout of a global: runs of plain bytes interleaved with
    /// pointer relocations.
    fn segments(g: &Global) -> Vec<Segment> {
        let size = g.size.max(g.init.len() as u64);
        let mut relocs: Vec<usize> = (0..g.relocs.len()).collect();
        relocs.sort_by_key(|&i| g.relocs[i].offset);
        let mut segs = Vec::new();
        let mut pos = 0u64;
        for i in relocs {
            let off = g.relocs[i].offset;
            if off < pos || off + 8 > size {
                continue;
            }
            if off > pos {
                segs.push(Segment::Bytes(pos, off));
            }
            segs.push(Segment::Reloc(i));
            pos = off + 8;
        }
        if size > pos || segs.is_empty() {
            segs.push(Segment::Bytes(pos, size.max(1)));
        }
        segs
    }

    /// The program global `i`, declared on first use.
    pub(super) fn global(&self, i: usize) -> GlobalValue<'ctx> {
        *self.globals[i].get_or_init(|| {
            let g = &self.program.globals[i];
            let fields: Vec<BasicTypeEnum> = Self::segments(g)
                .iter()
                .map(|s| match s {
                    Segment::Bytes(a, b) => self.bytes_ty(b - a),
                    Segment::Reloc(_) => self.ptr_ty().into(),
                })
                .collect();
            let ty = self.ctx.struct_type(&fields, true);
            let name = g
                .export
                .clone()
                .unwrap_or_else(|| format!("{}.{i}", g.name));
            let gv = self.module.add_global(ty, None, &name);
            gv.set_linkage(Linkage::External);
            if self.replicated(g) {
                gv.set_linkage(Linkage::Internal);
            } else if g.export.is_none() {
                let shared = self.shard.is_some_and(|s| s.plan.global_shared[i]);
                self.internal_linkage(gv, self.owns_global(i), shared);
            }
            if self.imported_globals.get(i).copied().unwrap_or(false) {
                gv.set_linkage(Linkage::AvailableExternally);
            }
            gv.set_alignment(g.align.max(1) as u32);
            gv.set_constant(g.read_only);
            gv
        })
    }

    pub(super) fn init_globals(&self) -> R<()> {
        for (i, g) in self.program.globals.iter().enumerate() {
            if self.replicated(g) || !self.owns_global(i) && !self.imported_globals[i] {
                continue;
            }
            self.global(i).set_initializer(&self.initializer(g)?);
        }
        Ok(())
    }

    /// Whether every module that refers to the global gets a private copy of it
    /// (`partition::replicated`).
    fn replicated(&self, g: &Global) -> bool {
        self.shard.is_some() && crate::partition::replicated(g)
    }

    /// Give the private copies this module declared their contents.
    pub(super) fn init_replicated(&self) -> R<()> {
        for (i, g) in self.program.globals.iter().enumerate() {
            if self.replicated(g) && self.globals[i].get().is_some() {
                self.global(i).set_initializer(&self.initializer(g)?);
            }
        }
        Ok(())
    }

    fn initializer(&self, g: &Global) -> R<inkwell::values::StructValue<'ctx>> {
        let mut fields: Vec<BasicValueEnum> = Vec::new();
        for seg in Self::segments(g) {
            match seg {
                Segment::Bytes(a, b) => {
                    // `init` may be shorter than the global: the rest is zero. Bytes go in as
                    // one ConstantDataArray (or an all-zero aggregate) instead of one
                    // ConstantInt per byte.
                    let end = (b as usize).min(g.init.len());
                    let start = (a as usize).min(end);
                    let data = &g.init[start..end];
                    let ty = self.ctx.i8_type().array_type((b - a) as u32);
                    if data.iter().all(|&x| x == 0) {
                        fields.push(ty.const_zero().into());
                    } else if data.len() as u64 == b - a {
                        fields.push(self.ctx.const_string(data, false).into());
                    } else {
                        let mut padded = data.to_vec();
                        padded.resize((b - a) as usize, 0);
                        fields.push(self.ctx.const_string(&padded, false).into());
                    }
                }
                Segment::Reloc(r) => {
                    let reloc = &g.relocs[r];
                    let base = match reloc.target {
                        RelocTarget::Global(id) => self.global(id.0 as usize).as_pointer_value(),
                        RelocTarget::Func(id) => self.func_ptr(id)?,
                        RelocTarget::Foreign(id) => self.foreign(id.0 as usize),
                    };
                    let ptr = if reloc.addend == 0 {
                        base
                    } else {
                        let i64t = self.ctx.i64_type();
                        let sum = base
                            .const_to_int(i64t)
                            .const_add(i64t.const_int(reloc.addend as u64, true));
                        sum.const_to_pointer(self.ptr_ty())
                    };
                    fields.push(ptr.into());
                }
            }
        }
        Ok(self.ctx.const_struct(&fields, true))
    }

    pub(super) fn func_ptr(&self, id: jaic::ir::FuncId) -> R<PointerValue<'ctx>> {
        match self.func(id.0 as usize) {
            Some(f) => Ok(f.as_global_value().as_pointer_value()),
            None => Err(Error(format!(
                "function '{}' is referenced but was never lowered",
                self.program
                    .func_names
                    .get(id.0 as usize)
                    .map_or("?", |s| s)
            ))),
        }
    }
}

enum Segment {
    /// Plain initial bytes `[start, end)`.
    Bytes(u64, u64),
    /// Index into `Global::relocs`.
    Reloc(usize),
}

/// The wasm import module for a `#library`: its file name, except that the C and math
/// libraries (whose functions a wasm runtime supplies) map to `env`.
fn wasm_import_module(library: &str) -> &str {
    let name = library.rsplit(['/', '\\']).next().unwrap_or(library);
    match name {
        "" | "c" | "libc" | "crt" | "msvcrt" | "m" | "libm" => "env",
        other => other,
    }
}
