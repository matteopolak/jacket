//! Bounded semantic values from the exact image selected for a default write.
use crate::ide::lifetimes::{LifetimeDefaultLeaf, LifetimeDefaultValue};
use crate::ir::{Reloc, RelocTarget};
use crate::sema::{Compiler, TypeId, lower::FnCtx};
use crate::types::{ArrayKind, TypeKind};

const MAX_LEAVES: usize = 128;
const MAX_WORK: usize = 512;
const MAX_DEPTH: usize = 16;

impl Compiler {
    /// Record the exact zero operation or immutable global selected for this write.
    pub(in crate::sema) fn record_lifetime_default_image(
        &self,
        f: &mut FnCtx,
        ty: TypeId,
        place: crate::ir::Val,
        bytes: u64,
        source: Option<crate::ir::GlobalId>,
    ) {
        if f.type_only || f.lifetimes.is_none() {
            return;
        }
        let values = if bounded_size(self, ty, 0) == Some(bytes) {
            match source {
                None => self.lifetime_default_values(ty, None),
                Some(source) => self
                    .out
                    .program
                    .globals
                    .get(source.0 as usize)
                    .filter(|global| {
                        global.read_only
                            && global.size == bytes
                            && u64::try_from(global.init.len()).ok() == Some(bytes)
                    })
                    .map_or_else(
                        || (Vec::new(), false),
                        |global| {
                            self.lifetime_default_values(ty, Some((&global.init, &global.relocs)))
                        },
                    ),
            }
        } else {
            (Vec::new(), false)
        };
        f.lifetimes.as_mut().unwrap().push(
            f.b.current,
            crate::ide::lifetimes::LifetimeEvent::DefaultImage {
                place,
                ty,
                bytes,
                zeroed: source.is_none(),
                leaves: values.0,
                complete: values.1,
            },
        );
    }

    /// Capture selected write values now; later default-image cache changes must
    /// not alter this event. For example, a proc leaf retains its selected `ProcId`.
    pub(crate) fn lifetime_default_values(
        &self,
        ty: TypeId,
        image: Option<(&[u8], &[Reloc])>,
    ) -> (Vec<LifetimeDefaultLeaf>, bool) {
        let Some(size) = bounded_size(self, ty, 0) else {
            return (Vec::new(), false);
        };
        if image.is_some_and(|(_, relocs)| relocs.len() > MAX_WORK) {
            return (Vec::new(), false);
        }
        let mut state = Capture {
            compiler: self,
            image,
            size,
            leaves: Vec::new(),
            complete: true,
            work: 0,
        };
        state.visit(ty, 0, 0);
        (state.leaves, state.complete)
    }
}

struct Capture<'a> {
    compiler: &'a Compiler,
    image: Option<(&'a [u8], &'a [Reloc])>,
    size: u64,
    leaves: Vec<LifetimeDefaultLeaf>,
    complete: bool,
    work: usize,
}

impl Capture<'_> {
    fn visit(&mut self, ty: TypeId, offset: u64, depth: usize) {
        self.work += 1;
        let Some(repr) = representation(self.compiler, ty) else {
            self.complete = false;
            return;
        };
        let Some(width) = bounded_size(self.compiler, repr, depth) else {
            self.complete = false;
            return;
        };
        if self.work > MAX_WORK
            || depth > MAX_DEPTH
            || self.leaves.len() >= MAX_LEAVES
            || offset.checked_add(width).is_none_or(|end| end > self.size)
        {
            self.complete = false;
            return;
        }
        if width == 0 {
            return;
        }
        match self.compiler.ty.types.kind(repr) {
            TypeKind::Struct(id) => {
                let info = self.compiler.ty.types.struct_info(*id);
                if info.fields.len() > MAX_WORK
                    || info.is_union
                    || info.fields.iter().any(|field| field.overlay)
                {
                    self.complete = false;
                    return;
                }
                for field in info.fields.iter() {
                    if self.work >= MAX_WORK || self.leaves.len() >= MAX_LEAVES {
                        self.complete = false;
                        break;
                    }
                    let Some(at) = offset.checked_add(field.offset) else {
                        self.complete = false;
                        break;
                    };
                    self.visit(field.ty, at, depth + 1);
                }
            }
            TypeKind::Array {
                elem,
                kind: ArrayKind::Fixed(count),
            } => {
                let Some(stride) = bounded_size(self.compiler, *elem, depth + 1) else {
                    self.complete = false;
                    return;
                };
                if stride == 0 {
                    return;
                }
                for index in 0..*count {
                    if self.work >= MAX_WORK || self.leaves.len() >= MAX_LEAVES {
                        self.complete = false;
                        break;
                    }
                    let Some(at) = index
                        .checked_mul(stride)
                        .and_then(|delta| offset.checked_add(delta))
                    else {
                        self.complete = false;
                        break;
                    };
                    self.visit(*elem, at, depth + 1);
                }
            }
            TypeKind::Bool
            | TypeKind::Int {
                ..
            }
            | TypeKind::Float {
                ..
            }
            | TypeKind::Pointer(_)
            | TypeKind::Proc(_) => self.scalar(ty, offset, width),
            _ => self.complete = false,
        }
    }

    fn scalar(&mut self, ty: TypeId, offset: u64, width: u64) {
        let value = self.value(ty, offset, width);
        self.complete &= value != LifetimeDefaultValue::Unknown;
        self.leaves.push(LifetimeDefaultLeaf {
            offset,
            width,
            ty,
            value,
        });
    }

    fn value(&self, ty: TypeId, offset: u64, width: u64) -> LifetimeDefaultValue {
        if width == 0 || width > 8 {
            return LifetimeDefaultValue::Unknown;
        }
        let Some(end) = offset.checked_add(width) else {
            return LifetimeDefaultValue::Unknown;
        };
        let Some((image, relocs)) = self.image else {
            return LifetimeDefaultValue::Scalar(0);
        };
        let Ok(start) = usize::try_from(offset) else {
            return LifetimeDefaultValue::Unknown;
        };
        let Ok(end) = usize::try_from(end) else {
            return LifetimeDefaultValue::Unknown;
        };
        let Some(bytes) = image.get(start..end) else {
            return LifetimeDefaultValue::Unknown;
        };
        let mut relocations = relocs.iter().filter(|reloc| overlaps(reloc, offset, width));
        if let Some(reloc) = relocations.next() {
            if relocations.next().is_some()
                || width != 8
                || reloc.offset != offset
                || reloc.addend != 0
                || !matches!(
                    self.compiler
                        .ty
                        .types
                        .kind(representation(self.compiler, ty).unwrap_or(TypeId::VOID)),
                    TypeKind::Proc(_)
                )
            {
                return LifetimeDefaultValue::Unknown;
            }
            let RelocTarget::Func(function) = reloc.target else {
                return LifetimeDefaultValue::Unknown;
            };
            if self
                .compiler
                .pr
                .func_targets
                .get(&function)
                .is_none_or(|targets| targets.len() > MAX_WORK)
            {
                return LifetimeDefaultValue::Unknown;
            }
            let mut procedures = self.compiler.procs_of_func(function);
            let Some(procedure) = procedures.next() else {
                return LifetimeDefaultValue::Unknown;
            };
            return if procedures.next().is_none() {
                LifetimeDefaultValue::Procedure {
                    procedure,
                    function,
                }
            } else {
                LifetimeDefaultValue::Unknown
            };
        }
        // Aggregate::write_value defines this image representation as little endian.
        let mut raw = [0; 8];
        raw[..bytes.len()].copy_from_slice(bytes);
        LifetimeDefaultValue::Scalar(u64::from_le_bytes(raw))
    }
}

fn bounded_size(c: &Compiler, ty: TypeId, depth: usize) -> Option<u64> {
    if depth > MAX_DEPTH {
        return None;
    }
    let ty = representation(c, ty)?;
    match c.ty.types.kind(ty) {
        TypeKind::Array {
            elem,
            kind: ArrayKind::Fixed(count),
        } => bounded_size(c, *elem, depth + 1)?.checked_mul(*count),
        _ => Some(c.ty.types.size_of(ty)),
    }
}

/// Strip nominal wrappers with a separate finite traversal bound.
fn representation(c: &Compiler, mut ty: TypeId) -> Option<TypeId> {
    for _ in 0..MAX_DEPTH {
        ty = match c.ty.types.kind(ty) {
            TypeKind::Distinct(id) => c.ty.types.distincts[id.0 as usize].base,
            TypeKind::Enum(id) => c.ty.types.enum_info(*id).base,
            _ => return Some(ty),
        };
    }
    None
}

fn overlaps(reloc: &Reloc, offset: u64, width: u64) -> bool {
    let Some(end) = reloc.offset.checked_add(8) else {
        return true;
    };
    offset
        .checked_add(width)
        .is_none_or(|leaf_end| reloc.offset < leaf_end && offset < end)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::GlobalId;
    use crate::sema::value::Aggregate;
    use crate::sema::{NativeFs, Options};
    use std::rc::Rc;

    #[test]
    fn default_leaf_capture_rejects_partial_overlapping_relocations_and_truncation() {
        let c = Compiler::new(Options::host(), Rc::new(NativeFs));
        for relocs in [
            vec![Reloc {
                offset: 4,
                target: RelocTarget::Global(GlobalId(0)),
                addend: 0,
            }],
            vec![
                Reloc {
                    offset: 0,
                    target: RelocTarget::Global(GlobalId(0)),
                    addend: 0,
                },
                Reloc {
                    offset: 0,
                    target: RelocTarget::Global(GlobalId(1)),
                    addend: 0,
                },
            ],
        ] {
            let image = Aggregate {
                bytes: vec![0; 8],
                relocs,
            };
            let (leaves, complete) =
                c.lifetime_default_values(TypeId::U64, Some((&image.bytes, &image.relocs)));
            assert!(!complete);
            assert_eq!(leaves[0].value, LifetimeDefaultValue::Unknown);
        }
        let short = Aggregate {
            bytes: vec![1; 4],
            relocs: Vec::new(),
        };
        let (leaves, complete) =
            c.lifetime_default_values(TypeId::U64, Some((&short.bytes, &short.relocs)));
        assert!(!complete);
        assert_eq!(leaves[0].value, LifetimeDefaultValue::Unknown);
    }

    #[test]
    fn default_leaf_capture_uses_the_cached_global_selected_for_the_actual_copy() {
        use crate::ide::lifetimes::{LifetimeEvent, LifetimeRecording};
        use crate::ir::{Conv, Global, Sig};
        use crate::sema::{Arc, FileId, Span, lower::FnCtx};
        let mut c = Compiler::new(Options::host(), Rc::new(NativeFs));
        // Seed a divergence directly: this tests event selection, not whether a
        // source program can cause the compiler's existing caches to diverge.
        let old = c.out.program.add_global(Global {
            name: "prior-default".into(),
            size: 8,
            align: 8,
            init: 11_u64.to_le_bytes().to_vec(),
            relocs: Vec::new(),
            read_only: true,
            export: None,
        });
        c.ty.default_globals.insert(TypeId::U64, old);
        c.ty.default_images.insert(
            TypeId::U64,
            Some(Arc::new(Aggregate {
                bytes: 77_u64.to_le_bytes().to_vec(),
                relocs: Vec::new(),
            })),
        );
        let mut f = FnCtx::new(
            "selected-default-test".into(),
            Sig {
                params: Vec::new(),
                returns: Vec::new(),
                conv: Conv::Jai,
                c_varargs: false,
                c_fixed: 0,
                c_abi: None,
            },
            FileId(0),
        );
        f.lifetimes = Some(Box::new(LifetimeRecording::default()));
        let slot = f.b.slot(8, 8);
        let address = f.b.slot_addr(slot);
        c.init_default(&mut f, TypeId::U64, address, Span::new(FileId(0), 0, 0))
            .unwrap();
        let values: Vec<_> = f
            .lifetimes
            .as_ref()
            .unwrap()
            .blocks
            .iter()
            .flat_map(|b| &b.events)
            .filter_map(|event| match event {
                LifetimeEvent::DefaultImage {
                    leaves, ..
                } => Some(leaves),
                _ => None,
            })
            .flatten()
            .collect();
        assert_eq!(values.len(), 1);
        assert_eq!(
            values[0].value,
            LifetimeDefaultValue::Scalar(11),
            "retained values must describe the cached global actually copied, not the newly computed image"
        );
        for (size, init_bytes, read_only) in [(4, 8, true), (8, 4, true), (8, 8, false)] {
            let global = &mut c.out.program.globals[old.0 as usize];
            global.size = size;
            global.init = vec![0; init_bytes];
            global.read_only = read_only;
            f.lifetimes = Some(Box::new(LifetimeRecording::default()));
            c.init_default(&mut f, TypeId::U64, address, Span::new(FileId(0), 0, 0))
                .unwrap();
            assert!(f.lifetimes.as_ref().unwrap().blocks.iter().flat_map(|b|&b.events).any(|event|matches!(event,LifetimeEvent::DefaultImage{bytes:8,zeroed:false,leaves,complete:false,..} if leaves.is_empty())),"invalid cached source extent or mutability must stay unknown");
        }
        f.lifetimes = Some(Box::new(LifetimeRecording::default()));
        c.record_lifetime_default_image(&mut f, TypeId::U64, address, 4, None);
        assert!(f.lifetimes.as_ref().unwrap().blocks.iter().flat_map(|b|&b.events).any(|event|matches!(event,LifetimeEvent::DefaultImage{bytes:4,zeroed:true,leaves,complete:false,..} if leaves.is_empty())),"a selected write cannot acquire the larger current type extent");
    }

    #[test]
    fn default_leaf_capture_bounds_relocation_and_type_size_scans() {
        let mut c = Compiler::new(Options::host(), Rc::new(NativeFs));
        let image = Aggregate {
            bytes: vec![0; 8],
            relocs: vec![
                Reloc {
                    offset: 0,
                    target: RelocTarget::Global(GlobalId(0)),
                    addend: 0
                };
                MAX_WORK + 1
            ],
        };
        let (leaves, complete) =
            c.lifetime_default_values(TypeId::U64, Some((&image.bytes, &image.relocs)));
        assert!(!complete && leaves.is_empty());
        let mut ty = TypeId::U8;
        for _ in 0..=MAX_DEPTH {
            ty = c.ty.types.intern(TypeKind::Array {
                elem: ty,
                kind: ArrayKind::Fixed(1),
            });
        }
        let (leaves, complete) = c.lifetime_default_values(ty, None);
        assert!(!complete && leaves.is_empty());
    }

    #[test]
    fn default_leaf_capture_bounds_fixed_array_traversal_without_implicit_zero_tail() {
        let mut c = Compiler::new(Options::host(), Rc::new(NativeFs));
        let ty = c.ty.types.intern(TypeKind::Array {
            elem: TypeId::U8,
            kind: ArrayKind::Fixed(1_000_000),
        });
        let (leaves, complete) = c.lifetime_default_values(ty, None);
        assert!(!complete);
        assert_eq!(leaves.len(), MAX_LEAVES);
        assert_eq!(leaves.last().unwrap().offset, 127);
        assert!(
            leaves
                .iter()
                .all(|leaf| leaf.value == LifetimeDefaultValue::Scalar(0))
        );
    }
}

#[cfg(test)]
mod identity_tests {
    use super::*;
    use crate::sema::value::Aggregate;
    use crate::sema::{NativeFs, Options, ProgramSource, procs::ProcTarget};
    use std::rc::Rc;

    #[test]
    fn default_procedure_relocation_requires_one_selected_identity() {
        let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
        let mut options = Options::host();
        options.import_paths = vec![stdlib.clone()];
        options.set_stdlib(&NativeFs, &stdlib);
        let mut c = Compiler::new(options, Rc::new(NativeFs));
        c.begin_sources(&[ProgramSource::String(
            "choose::()->s64{return 1;} other::()->s64{return 2;} main::(){choose();other();}"
                .into(),
        )])
        .unwrap();
        c.finish_program().unwrap();
        let first =
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == "choose")
                .unwrap();
        let other =
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == "other")
                .unwrap();
        let ProcTarget::Func(function) = c.pr.procs[first].target.unwrap() else {
            panic!("checked local function")
        };
        let ty = c.pr.procs[first].sig.as_ref().unwrap().ty;
        let image = Aggregate {
            bytes: vec![0; 8],
            relocs: vec![Reloc {
                offset: 0,
                target: RelocTarget::Func(function),
                addend: 0,
            }],
        };
        let (leaves, complete) = c.lifetime_default_values(ty, Some((&image.bytes, &image.relocs)));
        assert!(complete);
        assert_eq!(
            leaves[0].value,
            LifetimeDefaultValue::Procedure {
                procedure: crate::sema::ProcId(first as u32),
                function
            }
        );
        c.pr.procs[other].target = Some(ProcTarget::Func(function));
        c.pr.func_targets
            .get_mut(&function)
            .unwrap()
            .push(crate::sema::ProcId(other as u32));
        let (leaves, complete) = c.lifetime_default_values(ty, Some((&image.bytes, &image.relocs)));
        assert!(!complete);
        assert_eq!(leaves[0].value, LifetimeDefaultValue::Unknown);
    }
}
