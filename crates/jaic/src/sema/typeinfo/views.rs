//! Writing fields, pointers and views of the type information tables.
use crate::diagnostics::Code;
use crate::sema::value::Aggregate;
use crate::sema::{Arc, Compiler, Result, Span, Sym, TypeId, Value, err, ir, structs};

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
                Code::TypeInfoNoField,
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
        self.set_info_ref(agg, desc, name, target, false, span)
    }

    /// Like `set_info_ptr`, for a pointer to a type that may still be laid out.
    pub(super) fn set_info_ptr_later(
        &mut self,
        agg: &mut Aggregate,
        desc: TypeId,
        name: &str,
        target: TypeId,
        span: Span,
    ) -> Result<()> {
        self.set_info_ref(agg, desc, name, target, true, span)
    }

    fn set_info_ref(
        &mut self,
        agg: &mut Aggregate,
        desc: TypeId,
        name: &str,
        target: TypeId,
        later: bool,
        span: Span,
    ) -> Result<()> {
        let Some((path, _)) = self.find_member(desc, Sym::intern(name), span)? else {
            return err(
                Code::TypeInfoNoField,
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
        let g = if later {
            self.type_info_global_later(target, span)?
        } else {
            self.type_info_global(target, span)?
        };
        agg.relocs.push(ir::Reloc {
            offset,
            target: ir::RelocTarget::Global(g),
            addend: 0,
        });
        Ok(())
    }

    /// Write the `notes` view field of `desc` from `notes`. An editor compile gives each note
    /// its own bytes with its place, so the note watch sees which compile-time code reads it
    /// ([note readers](../../../../../docs/compiler/note-readers.md)).
    pub(super) fn set_notes_view(
        &mut self,
        agg: &mut Aggregate,
        desc: TypeId,
        notes: &[crate::ast::Note],
        span: Span,
    ) -> Result<()> {
        let data = if self.ed.is_live() {
            self.note_view(notes)
        } else {
            let texts: Vec<Arc<[u8]>> =
                notes.iter().map(|n| Arc::from(n.text.as_bytes())).collect();
            self.string_view(&texts, span)?
        };
        self.set_view(agg, desc, "notes", View::of(notes.len(), data), span)
    }

    /// The `[] string` data of `notes`, each pointing at a note global of its own.
    fn note_view(&mut self, notes: &[crate::ast::Note]) -> Aggregate {
        let stride = self.ty.types.size_of(TypeId::STRING);
        let view = self.ty.types.builtins().view;
        let mut data = Aggregate {
            bytes: vec![0; notes.len() * stride as usize],
            relocs: Vec::new(),
        };
        for (i, note) in notes.iter().enumerate() {
            let at = i as u64 * stride;
            let text = note.text.as_bytes();
            self.options.layout.write_u64(
                &mut data.bytes[(at + view.count) as usize..],
                text.len() as u64,
            );
            if text.is_empty() {
                continue;
            }
            let g = match self.note_place(note.span) {
                Some(place) => self.out.note_global(text, place),
                None => self.out.string_global(&Arc::from(text)),
            };
            data.relocs.push(ir::Reloc {
                offset: at + view.data,
                target: ir::RelocTarget::Global(g),
                addend: 0,
            });
        }
        data
    }

    /// Where the note at `span` is written, as the note watch names places.
    fn note_place(&self, span: Span) -> Option<crate::interp::note_watch::NotePlace> {
        let source = self.src.sources.try_get(span.file)?;
        source.path.as_file()?;
        Some(crate::interp::note_watch::NotePlace {
            path: crate::paths::program_path(&source.path.to_string()).into(),
            start: source.line_col(span.start),
            end: source.line_col(span.end),
        })
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
                Code::TypeInfoNoField,
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
        let view = self.ty.types.builtins().view;
        let count_at = (offset + view.count) as usize;
        self.options
            .layout
            .write_u64(&mut agg.bytes[count_at..], count as u64);
        if count > 0 {
            let name = format!("type_info.{name}.{}", self.out.program.globals.len());
            let g = self.out.program.add_global(ir::Global {
                name,
                size: data.bytes.len() as u64,
                align,
                init: data.bytes,
                relocs: data.relocs,
                read_only: true,
                export: None,
                section: None,
            });
            agg.relocs.push(ir::Reloc {
                offset: offset + view.data,
                target: ir::RelocTarget::Global(g),
                addend: 0,
            });
        }
        Ok(())
    }

    pub(super) fn info_view(&mut self, types: &[TypeId], span: Span) -> Result<Aggregate> {
        let slot = u64::from(self.ty.types.data_layout().pointer_size);
        let mut data = Aggregate {
            bytes: vec![0; types.len() * slot as usize],
            relocs: Vec::new(),
        };
        for (i, &t) in types.iter().enumerate() {
            let g = self.type_info_global(t, span)?;
            data.relocs.push(ir::Reloc {
                offset: i as u64 * slot,
                target: ir::RelocTarget::Global(g),
                addend: 0,
            });
        }
        Ok(data)
    }

    pub(super) fn string_view(&mut self, strings: &[Arc<[u8]>], span: Span) -> Result<Aggregate> {
        let stride = self.ty.types.size_of(TypeId::STRING);
        let mut data = Aggregate {
            bytes: vec![0; strings.len() * stride as usize],
            relocs: Vec::new(),
        };
        for (i, s) in strings.iter().enumerate() {
            self.write_value(
                &mut data,
                i as u64 * stride,
                &Value::String(s.clone()),
                TypeId::STRING,
                span,
            )?;
        }
        Ok(data)
    }
}
