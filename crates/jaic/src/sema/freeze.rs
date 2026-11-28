//! Reading a `#run` result out of interpreter memory: pointers inside it become relocations and
//! the memory they reach is copied into program globals.
use super::value::Aggregate;
use super::{Arc, Compiler, Result, Span, TypeId, Value, err, ir};
use crate::diagnostics::Code;
use crate::types::{ArrayKind, TypeKind};

/// The `count` elements of `elem` (`bytes` in all) that a pointer freezes to, read at `p` in the
/// interpreter's memory.
struct Run {
    p: u64,
    elem: TypeId,
    bytes: u64,
    count: u64,
}

/// Whether the union word `p`, which points into no global, is an address of memory only the
/// interpreter holds, `readable` saying whether the interpreter can read it. A host with
/// 64-bit addresses keeps them far from the numbers programs store, so a readable word is an
/// address. On a 32-bit host (the browser engine) every integer below the memory size is
/// readable, so a word cannot be told from data and stays as written.
fn holds_host_address(p: u64, host_bits: u32, readable: impl FnOnce(u64) -> bool) -> bool {
    host_bits >= 64 && readable(p)
}

impl Compiler {
    pub(super) fn type_at(&self, p: u64, span: Span) -> Result<TypeId> {
        if let Some((g, 0)) = self.ct.interp.global_at(p)
            && let Some(t) = self.ty.type_from_info_global(g)
        {
            return Ok(t);
        }

        err(
            Code::CompileTimeValueUnreadable,
            span,
            "compile-time value of type Type does not refer to a type descriptor",
        )
    }

    pub(super) fn read_aggregate(&mut self, addr: u64, ty: TypeId, span: Span) -> Result<Value> {
        let size = self.size_of(ty, span)?;
        let mut agg = Aggregate {
            bytes: self.ct.program_bytes(addr, size as usize, span)?,
            relocs: Vec::new(),
        };

        self.ct.frozen.clear();
        let result = self.freeze(&mut agg, 0, addr, ty, span);

        self.ct.frozen.clear();
        result?;
        Ok(Value::Bytes(Arc::new(agg)))
    }

    /// The `s64` a frozen value holds at `offset`, which `agg` copied from `addr` a moment ago.
    fn frozen_u64(&self, agg: &Aggregate, offset: u64, addr: u64, span: Span) -> Result<u64> {
        let at = offset as usize;

        match agg.bytes.get(at..at + 8) {
            Some(bytes) => Ok(self.options.layout.read_u64(bytes)),
            None => self.ct.program_u64(addr, span),
        }
    }

    /// The pointer a frozen value holds at `offset`, which `agg` copied from `addr` a moment ago.
    fn frozen_ptr(&self, agg: &Aggregate, offset: u64, addr: u64, span: Span) -> Result<u64> {
        let at = offset as usize;
        let size = self.options.layout.pointer_size as usize;

        match agg.bytes.get(at..at + size) {
            Some(bytes) => Ok(self.options.layout.read_ptr(bytes)),
            None => self.ct.program_ptr(addr, span),
        }
    }

    /// Store the pointer `p` in a frozen value at `offset`.
    fn put_frozen_ptr(&self, agg: &mut Aggregate, offset: u64, p: u64) {
        self.options
            .layout
            .write_ptr(&mut agg.bytes[offset as usize..], p);
    }

    /// Zero the pointer in a frozen value at `offset`.
    fn clear_frozen_ptr(&self, agg: &mut Aggregate, offset: u64) {
        let at = offset as usize;

        agg.bytes[at..at + self.options.layout.pointer_size as usize].fill(0);
    }

    /// Turn pointers inside a value read from interpreter memory into relocations,
    /// copying pointed-to data that does not live in a global.
    fn freeze(
        &mut self,
        agg: &mut Aggregate,
        offset: u64,
        addr: u64,
        ty: TypeId,
        span: Span,
    ) -> Result<()> {
        let layout = *self.ty.types.builtins();

        match self.ty.types.kind(ty).clone() {
            TypeKind::Struct(s) => {
                self.layout_struct(s, span)?;
                let info = self.ty.types.struct_info(s).clone();

                if info.is_union {
                    let size = self.size_of(ty, span)?;
                    return self.freeze_union_words(agg, offset, size);
                }

                for field in info.fields.iter() {
                    self.freeze(
                        agg,
                        offset + field.offset,
                        addr + field.offset,
                        field.ty,
                        span,
                    )?;
                }
            }
            TypeKind::Array {
                elem,
                kind: ArrayKind::Fixed(n),
            } => {
                let size = self.size_of(elem, span)?;

                for i in 0..n {
                    self.freeze(agg, offset + i * size, addr + i * size, elem, span)?;
                }
            }
            TypeKind::Array {
                elem,
                kind,
            } => {
                let count = self.frozen_u64(
                    agg,
                    offset + layout.view.count,
                    addr + layout.view.count,
                    span,
                )?;
                let data = self.frozen_ptr(
                    agg,
                    offset + layout.view.data,
                    addr + layout.view.data,
                    span,
                )?;
                let esize = self.size_of(elem, span)?;

                // Only the elements in use: the unused capacity of a resizable array is memory the
                // program never wrote, so copying it would put whatever the interpreter's heap held
                // before into the output, differently from one run to the next.
                self.freeze_pointer(
                    agg,
                    offset + layout.view.data,
                    Run {
                        p: data,
                        elem,
                        bytes: esize * count,
                        count,
                    },
                    span,
                )?;

                if kind == ArrayKind::Resizable {
                    // The frozen copy is not owned by any allocator.
                    let allocated = (offset + layout.resizable.allocated) as usize;
                    let end = (offset + layout.resizable.size) as usize;

                    agg.bytes[allocated..end].fill(0);
                    self.options
                        .layout
                        .write_u64(&mut agg.bytes[allocated..], count);
                }
            }
            TypeKind::String => {
                let count = self.frozen_u64(
                    agg,
                    offset + layout.view.count,
                    addr + layout.view.count,
                    span,
                )?;
                let data = self.frozen_ptr(
                    agg,
                    offset + layout.view.data,
                    addr + layout.view.data,
                    span,
                )?;

                self.freeze_pointer(
                    agg,
                    offset + layout.view.data,
                    Run {
                        p: data,
                        elem: TypeId::U8,
                        bytes: count,
                        count,
                    },
                    span,
                )?;
            }
            TypeKind::Pointer(to) => {
                let p = self.frozen_ptr(agg, offset, addr, span)?;
                let size = if to == TypeId::VOID {
                    0
                } else {
                    self.size_of(to, span)?
                };

                self.freeze_pointer(
                    agg,
                    offset,
                    Run {
                        p,
                        elem: to,
                        bytes: size,
                        count: 1,
                    },
                    span,
                )?;
            }
            TypeKind::Type => {
                let p = self.frozen_ptr(agg, offset, addr, span)?;

                if p != 0 {
                    let t = self.type_at(p, span)?;
                    let g = self.type_info_global(t, span)?;

                    agg.relocs.push(ir::Reloc {
                        offset,
                        target: ir::RelocTarget::Global(g),
                        addend: 0,
                    });
                    self.clear_frozen_ptr(agg, offset);
                }
            }
            TypeKind::Proc(_) => {
                let p = self.frozen_ptr(agg, offset, addr, span)?;

                self.clear_frozen_ptr(agg, offset);

                if let Some(func) = self.ct.interp.func_of(p) {
                    agg.relocs.push(ir::Reloc {
                        offset,
                        target: ir::RelocTarget::Func(func),
                        addend: 0,
                    });
                } else if self.ct.interp.is_integer_address(p) {
                    // Sentinels cast to a procedure type (`SIG_IGN`, `SIG_ERR`) stay integers.
                    self.put_frozen_ptr(agg, offset, p);
                } else {
                    return err(
                        Code::CompileTimeValueUnreadable,
                        span,
                        "a compile-time value holds a foreign procedure address",
                    );
                }
            }
            TypeKind::Any => {
                let t = self.frozen_ptr(
                    agg,
                    offset + layout.any.type_,
                    addr + layout.any.type_,
                    span,
                )?;
                let v = self.frozen_ptr(
                    agg,
                    offset + layout.any.value_pointer,
                    addr + layout.any.value_pointer,
                    span,
                )?;

                agg.bytes[offset as usize..(offset + layout.any.size) as usize].fill(0);

                if t != 0 {
                    let vt = self.type_at(t, span)?;
                    let g = self.type_info_global(vt, span)?;

                    agg.relocs.push(ir::Reloc {
                        offset,
                        target: ir::RelocTarget::Global(g),
                        addend: 0,
                    });
                    let size = self.size_of(vt, span)?;

                    self.freeze_pointer(
                        agg,
                        offset + layout.any.value_pointer,
                        Run {
                            p: v,
                            elem: vt,
                            bytes: size,
                            count: 1,
                        },
                        span,
                    )?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// A union has no single type for its bytes, so each aligned word that is an address into
    /// compile-time memory is handled as an opaque pointer is: relocated when it points into a
    /// global, null when it points at memory only the interpreter holds. Other words are data.
    /// See `holds_host_address` for what counts as an address.
    fn freeze_union_words(&mut self, agg: &mut Aggregate, offset: u64, size: u64) -> Result<()> {
        let word = self.options.layout.ptr_bytes();

        for at in (offset..offset + size / word * word).step_by(word as usize) {
            let Some(bytes) = agg.bytes.get(at as usize..(at + word) as usize) else {
                break;
            };
            let p = self.options.layout.read_ptr(bytes);

            if p == 0 || self.ct.interp.is_integer_address(p) {
                continue;
            }

            if let Some((g, off)) = self.ct.interp.global_at(p)
                && self.ct.temporary_buffer != Some(g)
            {
                self.clear_frozen_ptr(agg, at);
                agg.relocs.push(ir::Reloc {
                    offset: at,
                    target: ir::RelocTarget::Global(g),
                    addend: off as i64,
                });
            } else if holds_host_address(p, usize::BITS, |p| self.ct.interp.read(p, 1).is_some()) {
                self.clear_frozen_ptr(agg, at);
            }
        }

        Ok(())
    }

    /// Replace the pointer at `offset` with a relocation to (a frozen copy of) `count` elements at `p`.
    fn freeze_pointer(
        &mut self,
        agg: &mut Aggregate,
        offset: u64,
        run: Run,
        span: Span,
    ) -> Result<()> {
        let Run {
            p,
            elem,
            bytes,
            count,
        } = run;

        self.clear_frozen_ptr(agg, offset);

        if p == 0 {
            return Ok(());
        }

        if let Some((g, off)) = self.ct.interp.global_at(p)
            && self.ct.temporary_buffer != Some(g)
        {
            agg.relocs.push(ir::Reloc {
                offset,
                target: ir::RelocTarget::Global(g),
                addend: off as i64,
            });
            return Ok(());
        }

        if self.ct.interp.is_integer_address(p) {
            // Integers cast to pointers (handle-like constants such as `cast(*void) 32512` or
            // `HKEY_CLASSES_ROOT`) have no memory to freeze.
            self.put_frozen_ptr(agg, offset, p);
            return Ok(());
        }

        if bytes == 0 {
            // Opaque pointers have no pointee extent, so there is no data to capture. Globals
            // above still relocate. Memory only the interpreter holds has no address in the
            // program (copying it would make the output depend on where the host put it), so
            // the pointer is null; anything else is a number and stays one.
            if self.ct.interp.read(p, 1).is_none() {
                self.put_frozen_ptr(agg, offset, p);
            }

            return Ok(());
        }

        // Nothing to copy where nothing can be read: an integer cast to a pointer that the
        // range above does not catch (`cast(*T) 80000000`). It stays a number, like those.
        let Some(contents) = self.ct.interp.read(p, bytes as usize) else {
            self.put_frozen_ptr(agg, offset, p);
            return Ok(());
        };

        if let Some(&g) = self.ct.frozen.get(&(p, bytes)) {
            agg.relocs.push(ir::Reloc {
                offset,
                target: ir::RelocTarget::Global(g),
                addend: 0,
            });
            return Ok(());
        }

        let align = if elem == TypeId::VOID {
            self.options.layout.pointer_align as u64
        } else {
            self.align_of(elem, span)?
        };
        // Registered before its contents are frozen, so a pointer back to it ends the walk.
        let g = self.out.program.add_global(ir::Global {
            name: "frozen".into(),
            size: bytes + 1,
            align,
            init: Vec::new(),
            relocs: Vec::new(),
            read_only: false,
            export: None,
            section: None,
        });

        self.ct.frozen.insert((p, bytes), g);
        let mut inner = Aggregate {
            bytes: contents,
            relocs: Vec::new(),
        };

        if elem != TypeId::U8 && elem != TypeId::VOID {
            let esize = self.size_of(elem, span)?;

            for i in 0..count {
                self.freeze(&mut inner, i * esize, p + i * esize, elem, span)?;
            }
        }

        let mut init = inner.bytes;

        init.push(0); // NUL after frozen strings
        let global = &mut self.out.program.globals[g.0 as usize];

        global.init = init;
        global.relocs = inner.relocs;
        agg.relocs.push(ir::Reloc {
            offset,
            target: ir::RelocTarget::Global(g),
            addend: 0,
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::holds_host_address;

    #[test]
    fn a_readable_word_is_an_address_only_on_a_64_bit_host() {
        assert!(holds_host_address(0x1234_5678, 64, |_| true));
        assert!(!holds_host_address(0x1234_5678, 64, |_| false));
        assert!(!holds_host_address(0x1234_5678, 32, |_| true));
    }
}
