//! Writing fields, pointers and views of the type information tables.
use crate::sema::value::Aggregate;
use crate::sema::{Compiler, Rc, Result, Span, Sym, TypeId, Value, err, ir, structs};

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
                format!("{} has no field `{name}`", self.types.name(desc)),
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
                format!("{} has no field `{name}`", self.types.name(desc)),
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

    /// Write a `[] T` view field pointing at a new read-only global.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn set_view(
        &mut self,
        agg: &mut Aggregate,
        desc: TypeId,
        name: &str,
        count: usize,
        data: Aggregate,
        align: u64,
        span: Span,
    ) -> Result<()> {
        let Some((path, _)) = self.find_member(desc, Sym::intern(name), span)? else {
            return err(
                span,
                format!("{} has no field `{name}`", self.types.name(desc)),
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
            let g = self.program.add_global(ir::Global {
                name: format!("type_info.{name}.{}", self.program.globals.len()),
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
