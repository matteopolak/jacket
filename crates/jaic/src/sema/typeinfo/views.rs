//! Writing fields, pointers and views of the type information tables.
use crate::sema::value::Aggregate;
use crate::sema::{Compiler, Rc, Result, Span, Sym, TypeId, Value, err, ir, structs};

/// The elements a `[] T` view field points at: how many, their bytes and the alignment of the
/// global that holds them.
pub(super) struct View {
    count: usize,
    data: Aggregate,
    align: u64,
}

impl View {
    /// A view whose data is aligned to eight bytes.
    pub(super) fn of(count: usize, data: Aggregate) -> Self {
        Self::new(count, data, 8)
    }

    pub(super) fn new(count: usize, data: Aggregate, align: u64) -> Self {
        Self {
            count,
            data,
            align,
        }
    }
}

impl Compiler {
    pub(super) fn set_field(
        &mut self,
        agg: &mut Aggregate,
        desc: TypeId,
        name: &str,
        value: Value,
        span: Span,
    ) -> Result<()> {
        let Some((path, fty)) = self.find_member(desc, Sym::intern(name), span)? else {
            return err(
                span,
                format!("{} has no field `{name}`", self.ty.types.name(desc)),
            );
        };
        let offset = path
            .iter()
            .map(|s| {
                if let structs::PathStep::Offset(o) = s {
                    *o
                } else {
                    0
                }
            })
            .sum();
        self.write_value(agg, offset, &value, fty, span)
    }

    pub(super) fn set_info_ptr(
        &mut self,
        agg: &mut Aggregate,
        desc: TypeId,
        name: &str,
        target: TypeId,
        span: Span,
    ) -> Result<()> {
        let Some((path, _)) = self.find_member(desc, Sym::intern(name), span)? else {
            return err(
                span,
                format!("{} has no field `{name}`", self.ty.types.name(desc)),
            );
        };
        let offset = path
            .iter()
            .map(|s| {
                if let structs::PathStep::Offset(o) = s {
                    *o
                } else {
                    0
                }
            })
            .sum();
        let g = self.type_info_global(target, span)?;
        agg.relocs.push(ir::Reloc {
            offset,
            target: ir::RelocTarget::Global(g),
            addend: 0,
        });
        Ok(())
    }

    /// Write the `notes` view field of `desc` from the note strings.
    pub(super) fn set_notes_view(
        &mut self,
        agg: &mut Aggregate,
        desc: TypeId,
        notes: &[Rc<[u8]>],
        span: Span,
    ) -> Result<()> {
        let data = self.string_view(notes, span)?;
        self.set_view(agg, desc, "notes", View::of(notes.len(), data), span)
    }

    /// Write a `[] T` view field pointing at a new read-only global.
    pub(super) fn set_view(
        &mut self,
        agg: &mut Aggregate,
        desc: TypeId,
        name: &str,
        view: View,
        span: Span,
    ) -> Result<()> {
        let View {
            count,
            data,
            align,
        } = view;
        let Some((path, _)) = self.find_member(desc, Sym::intern(name), span)? else {
            return err(
                span,
                format!("{} has no field `{name}`", self.ty.types.name(desc)),
            );
        };
        let offset: u64 = path
            .iter()
            .map(|s| {
                if let structs::PathStep::Offset(o) = s {
                    *o
                } else {
                    0
                }
            })
            .sum();
        agg.bytes[offset as usize..offset as usize + 8]
            .copy_from_slice(&(count as u64).to_le_bytes());
        if count > 0 {
            let g = self.out.program.add_global(ir::Global {
                name: format!("type_info.{name}.{}", self.out.program.globals.len()),
                size: data.bytes.len() as u64,
                align,
                init: data.bytes,
                relocs: data.relocs,
                read_only: true,
                export: None,
            });
            agg.relocs.push(ir::Reloc {
                offset: offset + 8,
                target: ir::RelocTarget::Global(g),
                addend: 0,
            });
        }
        Ok(())
    }

    pub(super) fn info_view(&mut self, types: &[TypeId], span: Span) -> Result<Aggregate> {
        let mut data = Aggregate {
            bytes: vec![0; types.len() * 8],
            relocs: Vec::new(),
        };
        for (i, &t) in types.iter().enumerate() {
            let g = self.type_info_global(t, span)?;
            data.relocs.push(ir::Reloc {
                offset: i as u64 * 8,
                target: ir::RelocTarget::Global(g),
                addend: 0,
            });
        }
        Ok(data)
    }

    pub(super) fn string_view(&mut self, strings: &[Rc<[u8]>], span: Span) -> Result<Aggregate> {
        let mut data = Aggregate {
            bytes: vec![0; strings.len() * 16],
            relocs: Vec::new(),
        };
        for (i, s) in strings.iter().enumerate() {
            self.write_value(
                &mut data,
                i as u64 * 16,
                &Value::String(s.clone()),
                TypeId::STRING,
                span,
            )?;
        }
        Ok(data)
    }
}
