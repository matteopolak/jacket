//! Building a metaprogram's structs from compiler records, natively.
//!
//! Public operation binding resolves declaration layouts into member plans. Building
//! a record and everything it reaches uses a work list rather than interpreted
//! per-member calls; large TYPECHECKED graphs contain hundreds of thousands of nodes.
//!
//! The structs and their arrays live in memory the registry owns, like the strings handed
//! to Jai code, for the rest of the compilation. Each record is built once per program, so
//! pointers to the same record stay the same pointer.
use crate::fxhash::HashMap;
use crate::interp::note_watch::NotePlace;
use crate::interp::{Interp, Trap};
use crate::lang_items;
use crate::layout::DataLayout;
use crate::records::{FieldRef, Item, List, Name, Record, RecordTag, Records};
use std::rc::Rc;
use std::sync::Arc;

/// How a plan entry (or an array's element) is stored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// Nothing a record fills (an empty type, say).
    Other,
    /// An integer, enum or float of up to 8 bytes.
    Int,
    String,
    Pointer,
    /// A struct held in place.
    Struct,
    /// A view or resizable array.
    Array,
    FixedArray,
}

const NO_ENTRY: u16 = u16::MAX;

impl Plan {
    fn new(size: u64, entries: Vec<PlanEntry>, constant_storage: Option<u64>) -> Self {
        let mut by_name = Vec::new();

        for (i, entry) in entries.iter().enumerate() {
            let at = entry.name.index();

            if by_name.len() <= at {
                by_name.resize(at + 1, NO_ENTRY);
            }

            // The first entry of a name is the one a lookup by name would find.
            if by_name[at] == NO_ENTRY {
                by_name[at] = i as u16;
            }
        }

        Self {
            size,
            entries,
            by_name,
            constant_storage,
        }
    }
}

/// Structs are carved out of chunks this big; bigger ones get their own block.
const CHUNK: usize = 256 * 1024;

/// One member a record fills: its place in the struct and how it is stored.
pub struct PlanEntry {
    pub offset: u64,
    pub shape: Shape,
    pub size: usize,
    pub name: Name,
    /// An array's elements: their shape and size.
    pub element: Shape,
    pub element_size: u64,
    /// The type a pointer (or a pointer element) points to, or the struct held in place.
    pub target: u64,
    pub resizable: bool,
}

impl PlanEntry {
    /// An entry that is not an array.
    pub fn single(offset: u64, shape: Shape, size: usize, name: Name, target: u64) -> Self {
        Self {
            offset,
            shape,
            size,
            name,
            element: Shape::Other,
            element_size: 0,
            target,
            resizable: false,
        }
    }
}

struct Plan {
    size: u64,
    entries: Vec<PlanEntry>,
    /// The entry for each field name by `Name::index` (`NO_ENTRY` for names it has none for;
    /// names newer than the plan have no entry either).
    by_name: Vec<u16>,
    /// Where a `Type_Info_Struct`'s `constant_storage` is: its polymorph parameters that are
    /// types point at other descriptors (`__constant_pointers`).
    constant_storage: Option<u64>,
}

/// One program's structs built from compiler-owned records.
#[derive(Default)]
pub struct Builder {
    plans: HashMap<u64, Rc<Plan>>,
    /// The struct type a record tag stands for; zero means the name is ambiguous.
    tags: HashMap<Box<str>, u64>,
    /// The program's descriptors of primitive types, by name (`__builtin` records).
    builtins: HashMap<Box<str>, u64>,
    /// Built structs by record id (0 = not built).
    built: Vec<u64>,
    /// Record ids by built struct, the first record wins (`record_of`).
    ids: HashMap<u64, i64>,
    chunks: Vec<Chunk>,
    next: u64,
    left: usize,
    /// Null record references may acquire targets in later compiler messages.
    pending_refs: HashMap<i64, Vec<PendingRef>>,
    change_cursor: usize,
    /// The names of the fields that mark a descriptor or a builtin, once interned.
    marks: Option<(Name, Name)>,
}

/// Zeroed program memory the builder keeps, at program address `start`.
struct Chunk {
    start: u64,
    len: usize,
    /// Owns the memory.
    _block: crate::interp::ZeroedBlock,
}

#[derive(Clone)]
struct PendingRef {
    name: Name,
    address: u64,
    expected: u64,
}

fn store(interp: &mut Interp, p: u64, bytes: &[u8]) -> Result<(), Trap> {
    interp
        .write(p, bytes)
        .then_some(())
        .ok_or_else(|| Trap::unwritable(p, bytes.len()))
}

/// Store the pointer `value` at `p` in program memory.
fn store_ptr(interp: &mut Interp, p: u64, value: u64) -> Result<(), Trap> {
    let layout = *interp.data_layout();
    let mut bytes = [0; 8];

    layout.write_ptr(&mut bytes, value);
    store(interp, p, &bytes[..layout.pointer_size as usize])
}

/// Freshly allocated builder memory being filled: the bytes for `len` bytes at `base`, written
/// to the program in one transfer once complete. The memory is zeroed and unread until then.
struct Image<'a> {
    base: u64,
    bytes: &'a mut [u8],
    layout: DataLayout,
}

impl Image<'_> {
    fn put(&mut self, p: u64, value: &[u8]) -> Result<(), Trap> {
        let at = p.checked_sub(self.base).map(|at| at as usize);
        let range = at.and_then(|at| Some(at..at.checked_add(value.len())?));

        match range.and_then(|range| self.bytes.get_mut(range)) {
            Some(slot) => {
                slot.copy_from_slice(value);
                Ok(())
            }
            None => Err(Trap::unwritable(p, value.len())),
        }
    }

    /// A `string` or view image: the count, then the data pointer.
    fn put_view(&mut self, p: u64, count: u64, data: u64) -> Result<(), Trap> {
        let mut bytes = vec![0; self.layout.builtins().view.size as usize];

        self.layout.write_view(&mut bytes, count, data);
        self.put(p, &bytes)
    }

    /// The low `size` bytes (at most 8) of `value`, in the target's byte order.
    fn put_int(&mut self, p: u64, value: u64, size: usize) -> Result<(), Trap> {
        let mut bytes = [0; 8];

        self.layout.write_uint(&mut bytes[..size], value);
        self.put(p, &bytes[..size])
    }

    /// The count and data pointer of the view written at `p`.
    fn view_at(&self, p: u64) -> Result<(u64, u64), Trap> {
        let size = self.layout.builtins().view.size as usize;
        let at = p.checked_sub(self.base).map(|at| at as usize);
        let range = at.and_then(|at| Some(at..at.checked_add(size)?));

        match range.and_then(|range| self.bytes.get(range)) {
            Some(bytes) => Ok(self.layout.read_view(bytes)),
            None => Err(Trap::unwritable(p, size)),
        }
    }

    fn put_ptr(&mut self, p: u64, value: u64) -> Result<(), Trap> {
        self.put_int(p, value, self.layout.pointer_size as usize)
    }

    /// The part of this image for `len` bytes at `p`.
    fn part(&mut self, p: u64, len: usize) -> Result<Image<'_>, Trap> {
        let at = p.checked_sub(self.base).map(|at| at as usize);
        let range = at.and_then(|at| Some(at..at.checked_add(len)?));

        match range.and_then(|range| self.bytes.get_mut(range)) {
            Some(bytes) => Ok(Image {
                base: p,
                bytes,
                layout: self.layout,
            }),
            None => Err(Trap::unwritable(p, len)),
        }
    }
}

/// Where the `Code_Note` `record` is written, from its location fields and the file record
/// they point to; `None` for a note with no location (one made from `#code` at run time).
fn note_place(records: &Records, record: &Record) -> Option<NotePlace> {
    let int = |name: &str| match record.field(name)? {
        FieldRef::Int(v) => u32::try_from(v).ok(),
        _ => None,
    };
    let Some(FieldRef::Ref(file)) = record.field("enclosing_load") else {
        return None;
    };
    let Some(FieldRef::Str(path)) = records.field(file, "fully_pathed_filename") else {
        return None;
    };

    Some(NotePlace {
        path: String::from_utf8_lossy(path).into(),
        start: (int("l0")?, int("c0")?),
        end: (int("l1")?, int("c1")?),
    })
}

/// An integer or a record id.
fn number(item: &Item) -> Option<i64> {
    match item {
        Item::Int(v) | Item::Ref(v) => Some(*v),
        Item::Str(_) => None,
    }
}

/// How one plan entry is stored: its shape, its size in bytes and, for a pointer or nested
/// struct, the record type it refers to.
struct Slot {
    shape: Shape,
    size: usize,
    type_target: u64,
}

/// What a build is writing into: strings handed out and the records read.
struct Fill<'a> {
    records: &'a Records,
    kept: &'a mut Vec<Arc<[u8]>>,
    /// Records built and not filled yet: (id, memory, type).
    queue: Vec<(i64, u64, u64)>,
    created: Vec<i64>,
    linked: Vec<(i64, PendingRef)>,
}

impl Builder {
    pub fn add_type(&mut self, name: String, info: u64) {
        self.tags
            .entry(name.into())
            .and_modify(|existing| {
                if *existing != info {
                    *existing = 0;
                }
            })
            .or_insert(info);
    }

    pub fn add_builtin(&mut self, name: String, info: u64) {
        self.builtins.insert(name.into(), info);
    }

    /// The member plan of the struct type whose descriptor is `info`. A `Type_Info_Struct`
    /// (`type_info_struct`) also gets its polymorph parameters that are types patched.
    pub fn add_plan(
        &mut self,
        info: u64,
        size: u64,
        entries: Vec<PlanEntry>,
        type_info_struct: bool,
    ) {
        let constant_storage = type_info_struct
            .then(|| {
                entries
                    .iter()
                    .find(|e| e.name.as_str() == "constant_storage")
            })
            .flatten()
            .map(|e| e.offset);

        self.plans
            .insert(info, Rc::new(Plan::new(size, entries, constant_storage)));
    }

    /// The record a built struct came from, 0 if none.
    pub fn record_of(&self, pointer: u64) -> i64 {
        self.ids.get(&pointer).copied().unwrap_or(0)
    }

    /// The struct for `record`, building it and
    /// everything it reaches that is not built yet. `expected` is the type the caller wants
    /// (a pointer member's target); the record's tag may name a subtype of it.
    pub fn build(
        &mut self,
        records: &Records,
        kept: &mut Vec<Arc<[u8]>>,
        interp: &mut Interp,
        id: i64,
        expected: u64,
    ) -> Result<u64, Trap> {
        let mut fill = Fill {
            records,
            kept,
            queue: Vec::new(),
            created: Vec::new(),
            linked: Vec::new(),
        };
        let old_cursor = self.change_cursor;
        let result = (|| {
            self.link_ready_references(&mut fill, interp)?;
            let root = self.obtain(&mut fill, interp, id, expected)?;

            while let Some((id, memory, info)) = fill.queue.pop() {
                let plan = self.plans[&info].clone();
                let mut image = Image {
                    base: memory,
                    bytes: self.own_bytes(interp, memory, plan.size as usize)?,
                    layout: *interp.data_layout(),
                };

                self.fill_struct(&mut fill, interp, &mut image, memory, &plan, id)?;

                if let Some(at) = plan.constant_storage {
                    self.patch_constant_pointers(&mut fill, interp, memory + at, id)?;
                }
            }

            Ok(root)
        })();

        if result.is_err() {
            for id in fill.created {
                self.pending_refs.remove(&id);
                let pointer = std::mem::take(&mut self.built[id as usize]);

                if self.ids.get(&pointer) == Some(&id) {
                    self.ids.remove(&pointer);
                }
            }

            for (id, entry) in fill.linked {
                store_ptr(interp, entry.address, 0)?;

                if self.built.get(id as usize).is_some_and(|&p| p != 0) {
                    self.pending_refs.entry(id).or_default().push(entry);
                }
            }

            self.change_cursor = old_cursor;
        }

        result
    }

    fn link_ready_references(&mut self, fill: &mut Fill, interp: &mut Interp) -> Result<(), Trap> {
        let (cursor, changes) = fill.records.changes_since(self.change_cursor);
        let changed: crate::fxhash::HashSet<_> = changes.into_iter().collect();

        for id in changed {
            let Some(mut pending) = self.pending_refs.remove(&id) else {
                continue;
            };
            let mut waiting = Vec::new();

            while let Some(entry) = pending.pop() {
                let result = (|| {
                    let current = interp
                        .read_ptr(entry.address)
                        .ok_or_else(|| Trap::unreadable(entry.address, 8))?;

                    if current != 0 {
                        return Ok(true);
                    } // Preserve a metaprogram-supplied link.

                    let item = fill
                        .records
                        .get(id)
                        .and_then(|r| r.field_named(entry.name))
                        .and_then(super::records::FieldRef::item);

                    match item {
                        None | Some(Item::Ref(0) | Item::Int(0)) => Ok(false),
                        Some(Item::Ref(target) | Item::Int(target)) => {
                            let pointer = self.obtain(fill, interp, target, entry.expected)?;

                            store_ptr(interp, entry.address, pointer)?;
                            fill.linked.push((id, entry.clone()));
                            Ok(true)
                        }
                        _ => Err(Trap::new(
                            "records: late pointer field requires a record reference",
                        )),
                    }
                })();

                match result {
                    Ok(true) => {}
                    Ok(false) => waiting.push(entry),
                    Err(error) => {
                        waiting.push(entry);
                        waiting.extend(pending);
                        self.pending_refs.insert(id, waiting);

                        return Err(error);
                    }
                }
            }

            if !waiting.is_empty() {
                self.pending_refs.insert(id, waiting);
            }
        }

        self.change_cursor = cursor;
        Ok(())
    }

    /// The struct for record `id`: built already, or allocated now and queued to be filled.
    fn obtain(
        &mut self,
        fill: &mut Fill,
        interp: &Interp,
        id: i64,
        expected: u64,
    ) -> Result<u64, Trap> {
        if id == 0 {
            return Ok(0);
        }

        if let Some(&p) = self.built.get(id as usize)
            && p != 0
        {
            return Ok(p);
        }

        let record = fill
            .records
            .get(id)
            .ok_or_else(|| Trap::new(format!("records: no record with id {id}")))?;
        let record = Some(record);
        let (address_name, builtin_name) = *self.marks.get_or_insert_with(|| {
            (
                Name::intern(lang_items::record_fields::ADDRESS),
                Name::intern(lang_items::record_fields::BUILTIN),
            )
        });

        if let Some(record) = record {
            // A descriptor of the program reading the records: the real one.
            if let Some(FieldRef::Int(address)) = record.field_named(address_name) {
                self.remember(id, address as u64);
                fill.created.push(id);

                return Ok(address as u64);
            }

            if let Some(FieldRef::Str(name)) = record.field_named(builtin_name)
                && let Some(&info) = std::str::from_utf8(name)
                    .ok()
                    .and_then(|name| self.builtins.get(name))
            {
                self.remember(id, info);
                fill.created.push(id);

                return Ok(info);
            }
        }

        // Nested records may share an unqualified name. Their declared field type
        // chooses the layout; only an unambiguous tag can select a subtype.
        let tagged = record
            .and_then(|r| self.tags.get(r.tag.name()))
            .copied()
            .filter(|&info| info != 0);
        let Some(info) = tagged.or_else(|| self.plans.contains_key(&expected).then_some(expected))
        else {
            return Err(Trap::new(format!("records: no type plan for record {id}")));
        };
        let plan = self.plans.get(&info).ok_or_else(|| {
            Trap::new(format!(
                "records: type {info:#x} was registered without a plan"
            ))
        })?;
        let size = plan.size as usize;
        let memory = self.alloc(interp, size)?;

        // Before filling: records may refer to themselves.
        self.remember(id, memory);
        fill.created.push(id);

        if record.is_some() {
            fill.queue.push((id, memory, info));
        }

        Ok(memory)
    }

    fn remember(&mut self, id: i64, pointer: u64) {
        let at = id as usize;

        if self.built.len() <= at {
            self.built.resize(at + 1, 0);
        }

        self.built[at] = pointer;
        self.ids.entry(pointer).or_insert(id);
    }

    /// The `len` bytes at `p`, memory from `alloc` that the program has not been given yet, to
    /// be written in place rather than through the interpreter's checked transfers (one kernel
    /// call each on some hosts).
    #[allow(unsafe_code)]
    fn own_bytes(
        &mut self,
        interp: &Interp,
        p: u64,
        len: usize,
    ) -> Result<&'static mut [u8], Trap> {
        let inside = self.chunks.iter().rev().any(|chunk| {
            p.checked_sub(chunk.start)
                .and_then(|at| usize::try_from(at).ok())
                .and_then(|at| at.checked_add(len))
                .is_some_and(|end| end <= chunk.len)
        });

        if !inside {
            return Err(Trap::unwritable(p, len));
        }

        // SAFETY: `p..p + len` lies inside one chunk, which the builder keeps (never moves or
        // frees) for as long as it lives. The memory is zeroed and unread until the build that
        // fills it returns, and each region is filled by one build step at a time, so nothing
        // else reads or writes these bytes while the slice is in use.
        Ok(unsafe { std::slice::from_raw_parts_mut(interp.host_addr(p) as *mut u8, len) })
    }

    /// Zeroed, 16-byte aligned program memory that lives as long as the builder.
    fn alloc(&mut self, interp: &Interp, size: usize) -> Result<u64, Trap> {
        let size = (size.max(1) + 15) & !15;
        let mut take = |size: usize| {
            let block = interp
                .zeroed_block(size, 16)
                .ok_or_else(|| Trap::new("records: out of memory"))?;
            let start = block.addr();

            self.chunks.push(Chunk {
                start,
                len: size,
                _block: block,
            });
            Ok::<u64, Trap>(start)
        };

        if size > CHUNK / 4 {
            return take(size);
        }

        if size > self.left {
            self.next = take(CHUNK)?;
            self.left = CHUNK;
        }

        let at = self.next;

        self.next += size as u64;
        self.left -= size;
        Ok(at)
    }

    fn fill_struct(
        &mut self,
        fill: &mut Fill,
        interp: &mut Interp,
        image: &mut Image,
        memory: u64,
        plan: &Plan,
        id: i64,
    ) -> Result<(), Trap> {
        let record = fill
            .records
            .get(id)
            .ok_or_else(|| Trap::new(format!("records: no record with id {id}")))?;
        // The record's field for each entry (the first one set under a name), found by one
        // pass over the record instead of a search per entry.
        let mut inline = [NO_ENTRY; 48];
        let mut spilled;
        let slots: &mut [u16] = if plan.entries.len() <= inline.len() {
            &mut inline[..plan.entries.len()]
        } else {
            spilled = vec![NO_ENTRY; plan.entries.len()];
            &mut spilled
        };

        for i in 0..record.slot_count() {
            let Some(&entry) = plan.by_name.get(record.slot_name(i).index()) else {
                continue;
            };

            if entry != NO_ENTRY && slots[entry as usize] == NO_ENTRY {
                slots[entry as usize] = i as u16;
            }
        }

        // A note's text handed to an editor compile's program is watched (`note_watch.rs`).
        let note = if record.tag == RecordTag::CodeNote && interp.note_watch.is_some() {
            note_place(fill.records, record)
        } else {
            None
        };

        for (entry, &slot) in plan.entries.iter().zip(slots.iter()) {
            let field = (slot != NO_ENTRY).then(|| record.slot_value(slot as usize));

            if entry.shape == Shape::Pointer
                && matches!(field, None | Some(FieldRef::Ref(0) | FieldRef::Int(0)))
            {
                self.pending_refs.entry(id).or_default().push(PendingRef {
                    name: entry.name,
                    address: memory + entry.offset,
                    expected: entry.target,
                });
            }

            let Some(field) = field else {
                continue;
            };
            let target = memory + entry.offset;

            match field {
                FieldRef::List(items)
                    if matches!(entry.shape, Shape::Array | Shape::FixedArray) =>
                {
                    if items.is_empty() {
                        continue;
                    }

                    let size = entry.element_size;
                    let bytes = (items.len() as u64)
                        .checked_mul(size)
                        .filter(|&n| n <= crate::sema::value::MAX_IMAGE)
                        .ok_or_else(|| Trap::new("records: array size overflow"))?;

                    if entry.shape == Shape::FixedArray && bytes > entry.size as u64 {
                        return Err(Trap::new("records: fixed array has too many elements"));
                    }

                    let data = if entry.shape == Shape::FixedArray {
                        target
                    } else {
                        self.alloc(interp, bytes as usize)?
                    };
                    let mut inline;
                    let elements: &mut [u8] = if entry.shape == Shape::FixedArray {
                        inline = vec![0; bytes as usize];
                        &mut inline
                    } else {
                        self.own_bytes(interp, data, bytes as usize)?
                    };
                    let mut array = Image {
                        base: data,
                        bytes: &mut *elements,
                        layout: image.layout,
                    };

                    for (k, item) in items.iter().enumerate() {
                        let item = &item;

                        self.fill_item(
                            fill,
                            interp,
                            &mut array,
                            Slot {
                                shape: entry.element,
                                size: size as usize,
                                type_target: entry.target,
                            },
                            item,
                            data + k as u64 * size,
                        )?;
                    }

                    if entry.shape == Shape::FixedArray {
                        image.put(target, elements)?;
                    } else {
                        image.put_view(target, items.len() as u64, data)?;
                    }

                    if entry.resizable {
                        let allocated = image.layout.builtins().resizable.allocated;

                        image.put_int(target + allocated, items.len() as u64, 8)?;
                    }
                }
                FieldRef::List(_) => {
                    return Err(Trap::new(format!(
                        "records: field `{}` is a list but its plan expects a scalar",
                        entry.name.as_str()
                    )));
                }
                single => {
                    let Some(item) = single.item() else {
                        continue;
                    };

                    self.fill_item(
                        fill,
                        interp,
                        image,
                        Slot {
                            shape: entry.shape,
                            size: entry.size,
                            type_target: entry.target,
                        },
                        &item,
                        target,
                    )?;

                    if let Some(place) = &note
                        && entry.shape == Shape::String
                        && entry.name.as_str() == "text"
                    {
                        let (count, data) = image.view_at(target)?;

                        if let Some(watch) = interp.note_watch.as_mut() {
                            watch.add(data, count, place.clone());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Write one value of `slot`'s shape and size at `target`.
    fn fill_item(
        &mut self,
        fill: &mut Fill,
        interp: &mut Interp,
        image: &mut Image,
        slot: Slot,
        item: &Item,
        target: u64,
    ) -> Result<(), Trap> {
        let Slot {
            shape,
            size,
            type_target,
        } = slot;

        match (shape, item) {
            (Shape::Int, Item::Int(v) | Item::Ref(v)) => {
                image.put_int(target, *v as u64, size.min(8))
            }
            (Shape::String, Item::Str(s)) => {
                let data = if let Some(data) = interp.keep_narrow(s) {
                    data
                } else {
                    fill.kept.push(s.clone());
                    s.as_ptr() as u64
                };
                image.put_view(target, s.len() as u64, data)
            }
            (Shape::Pointer, item) => {
                let id = number(item).ok_or_else(|| {
                    Trap::new("records: pointer field requires a record reference")
                })?;
                let p = self.obtain(fill, interp, id, type_target)?;

                image.put_ptr(target, p)
            }
            (Shape::Struct, item) => {
                let id = number(item).ok_or_else(|| {
                    Trap::new("records: struct field requires a record reference")
                })?;

                if id == 0 {
                    return Ok(());
                }

                let plan = self
                    .plans
                    .get(&type_target)
                    .cloned()
                    .ok_or_else(|| Trap::new("records: nested struct has no plan"))?;
                let size = plan.size as usize;

                self.fill_struct(
                    fill,
                    interp,
                    &mut image.part(target, size)?,
                    target,
                    &plan,
                    id,
                )
            }
            _ => Err(Trap::new(format!(
                "records: value shape does not match plan shape {shape:?}"
            ))),
        }
    }

    /// `Type` parameters of a polymorphic struct instance: descriptors in constant storage,
    /// as (offset, record) pairs.
    fn patch_constant_pointers(
        &mut self,
        fill: &mut Fill,
        interp: &mut Interp,
        storage: u64,
        id: i64,
    ) -> Result<(), Trap> {
        let Some(FieldRef::List(List::Items(items))) =
            fill.records.field(id, "__constant_pointers")
        else {
            return Ok(());
        };
        let data = interp
            .read_ptr(storage + interp.data_layout().builtins().view.data)
            .ok_or_else(|| Trap::unreadable(storage, 8))?;

        for [offset, record] in items.as_chunks::<2>().0 {
            let (Some(offset), Some(record)) = (number(offset), number(record)) else {
                continue;
            };
            let pointer = self.obtain(fill, interp, record, 0)?;

            store_ptr(interp, data + offset as u64, pointer)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::records::{Record, RecordTag};

    /// `value` as a program word on the host.
    fn word(value: u64) -> [u8; 8] {
        let mut bytes = [0; 8];

        DataLayout::host().write_u64(&mut bytes, value);
        bytes
    }

    #[test]
    fn late_record_references_update_existing_identity_without_replaying_user_edits() {
        let mut b = Builder::default();

        b.add_type("Code_Node".into(), 1);
        b.plans.insert(
            1,
            Rc::new(Plan::new(
                16,
                vec![
                    PlanEntry::single(0, Shape::Int, 8, Name::intern("value"), 0),
                    PlanEntry::single(8, Shape::Pointer, 8, Name::intern("next"), 1),
                ],
                None,
            )),
        );
        let mut records = Records::default();
        let mut root = Record::new(RecordTag::CodeNode);

        root.int("value", 1);
        let root = records.add(root);
        let mut kept = Vec::new();
        let mut interp = Interp::native(crate::layout::DataLayout::host());
        let address = b.build(&records, &mut kept, &mut interp, root, 1).unwrap();

        assert_eq!(interp.read_u64(address + 8), Some(0));
        assert!(interp.write(address, &word(77)));
        let mut child = Record::new(RecordTag::CodeNode);

        child.list("value", Vec::new()).ptr("next", root);
        let child = records.add(child);

        records.get_mut(root).unwrap().ptr("next", child);
        assert!(b.build(&records, &mut kept, &mut interp, root, 1).is_err());
        assert_eq!(
            interp.read_u64(address + 8),
            Some(0),
            "a failed target fill must roll back its published link"
        );
        records
            .get_mut(child)
            .unwrap()
            .set("value", crate::records::Field::Item(Item::Int(2)));
        assert_eq!(
            b.build(&records, &mut kept, &mut interp, root, 1).unwrap(),
            address
        );
        let next = interp.read_u64(address + 8).unwrap();

        assert_ne!(next, 0);
        assert_eq!(interp.read_u64(next), Some(2));
        assert_eq!(interp.read_u64(next + 8), Some(address));
        assert_eq!(interp.read_u64(address), Some(77));
        let mut user = Record::new(RecordTag::CodeNode);

        user.int("value", 3);
        let user = records.add(user);
        let user_address = b.build(&records, &mut kept, &mut interp, user, 1).unwrap();

        assert!(interp.write(user_address + 8, &word(address)));
        records.get_mut(user).unwrap().ptr("next", child);
        assert_eq!(
            b.build(&records, &mut kept, &mut interp, user, 1).unwrap(),
            user_address
        );
        assert_eq!(
            interp.read_u64(user_address + 8),
            Some(address),
            "compiler late linking must preserve an already supplied metaprogram link"
        );
    }

    #[test]
    fn missing_record_is_an_error_and_null_remains_null() {
        let mut b = Builder::default();
        let records = Records::default();
        let mut kept = Vec::new();
        let mut interp = Interp::native(crate::layout::DataLayout::host());

        assert_eq!(b.build(&records, &mut kept, &mut interp, 0, 0).unwrap(), 0);
        assert!(b.build(&records, &mut kept, &mut interp, 17, 0).is_err());
    }

    #[test]
    fn tagged_type_without_plan_is_an_error_not_a_panic() {
        let mut b = Builder::default();

        b.add_type("Code_Node".into(), 1);
        let mut records = Records::default();
        let id = records.add(Record::new(RecordTag::CodeNode));

        assert!(
            b.build(
                &records,
                &mut Vec::new(),
                &mut Interp::native(crate::layout::DataLayout::host()),
                id,
                0
            )
            .is_err()
        );
        // A failed fill must not leave a cached zero/partially filled success.
        assert!(
            b.build(
                &records,
                &mut Vec::new(),
                &mut Interp::native(crate::layout::DataLayout::host()),
                id,
                0
            )
            .is_err()
        );
    }

    #[test]
    fn malformed_field_shape_is_an_error() {
        let mut b = Builder::default();

        b.add_type("Code_Node".into(), 1);
        b.plans.insert(
            1,
            Rc::new(Plan::new(
                8,
                vec![PlanEntry::single(
                    0,
                    Shape::Int,
                    8,
                    Name::intern("value"),
                    0,
                )],
                None,
            )),
        );
        let mut records = Records::default();
        let mut record = Record::new(RecordTag::CodeNode);

        record.str("value", b"not an integer");
        let id = records.add(record);

        assert!(
            b.build(
                &records,
                &mut Vec::new(),
                &mut Interp::native(crate::layout::DataLayout::host()),
                id,
                0
            )
            .is_err()
        );
    }

    #[test]
    fn ambiguous_nested_type_names_use_the_declared_field_type() {
        let mut builder = Builder::default();

        for (info, offset) in [(1, 0), (2, 8)] {
            builder.add_type("Record".into(), info);
            builder.plans.insert(
                info,
                Rc::new(Plan::new(
                    16,
                    vec![PlanEntry::single(
                        offset,
                        Shape::Int,
                        8,
                        Name::intern("value"),
                        0,
                    )],
                    None,
                )),
            );
        }

        let mut records = Records::default();
        let mut interp = Interp::native(crate::layout::DataLayout::host());

        for (info, offset) in [(1, 0), (2, 8)] {
            let mut record = Record::new(RecordTag::Record);

            record.int("value", 42);
            let id = records.add(record);
            let at = builder
                .build(&records, &mut Vec::new(), &mut interp, id, info)
                .unwrap();

            assert_eq!(interp.read_u64(at + offset), Some(42));
            assert_eq!(interp.read_u64(at + (8 - offset)), Some(0));
        }
    }

    #[test]
    fn fixed_arrays_fill_inline_and_reject_overflow() {
        let mut builder = Builder::default();

        builder.add_type("Record".into(), 1);
        builder.plans.insert(
            1,
            Rc::new(Plan::new(
                24,
                vec![PlanEntry {
                    offset: 8,
                    shape: Shape::FixedArray,
                    size: 16,
                    name: Name::intern("values"),
                    element: Shape::Int,
                    element_size: 8,
                    target: 0,
                    resizable: false,
                }],
                None,
            )),
        );
        let mut records = Records::default();
        let mut record = Record::new(RecordTag::Record);

        record.refs("values", [13, 29]);
        let id = records.add(record);
        let mut interp = Interp::native(crate::layout::DataLayout::host());
        let at = builder
            .build(&records, &mut Vec::new(), &mut interp, id, 1)
            .unwrap();

        assert_eq!(interp.read_u64(at), Some(0));
        assert_eq!(interp.read_u64(at + 8), Some(13));
        assert_eq!(interp.read_u64(at + 16), Some(29));
        let mut overflow = Record::new(RecordTag::Record);

        overflow.refs("values", [1, 2, 3]);
        let id = records.add(overflow);

        assert!(
            builder
                .build(&records, &mut Vec::new(), &mut interp, id, 1)
                .unwrap_err()
                .message
                .contains("fixed array has too many elements")
        );
    }
}
