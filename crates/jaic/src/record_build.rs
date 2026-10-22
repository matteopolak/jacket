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
use crate::interp::{Interp, Trap};
use crate::records::{FieldRef, Item, List, Name, Records};
use std::rc::Rc;

/// Internal member plan kinds used by public API layout binding.
pub const PLAN_OTHER: u64 = 0;

pub const PLAN_INT: u64 = 1;
pub const PLAN_STRING: u64 = 2;
pub const PLAN_POINTER: u64 = 3;

/// A struct held in place.
pub const PLAN_STRUCT: u64 = 4;

/// A view or resizable array.
pub const PLAN_ARRAY: u64 = 5;

pub const PLAN_FIXED_ARRAY: u64 = 6;

/// Size of `Record_Plan_Entry`: offset, kind, size, name (16 bytes), type, `element_kind`,
/// `element_size`, target, resizable.
const ENTRY_SIZE: u64 = 80;

/// Structs are carved out of chunks this big; bigger ones get their own block.
const CHUNK: usize = 256 * 1024;

struct Entry {
    offset: u64,
    kind: u64,
    size: usize,
    name: Name,
    /// An array's elements: their kind and size.
    element_kind: u64,
    element_size: u64,
    /// The type a pointer (or a pointer element) points to, or the struct held in place.
    target: u64,
    resizable: bool,
}

struct Plan {
    size: u64,
    entries: Vec<Entry>,
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
    chunks: Vec<Box<[u128]>>,
    next: u64,
    left: usize,
    /// Null record references may acquire targets in later compiler messages.
    pending_refs: HashMap<i64, Vec<PendingRef>>,
    change_cursor: usize,
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

/// Freshly allocated builder memory being filled: the bytes for `len` bytes at `base`, written
/// to the program in one transfer once complete. The memory is zeroed and unread until then.
struct Image<'a> {
    base: u64,
    bytes: &'a mut [u8],
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

    fn put_pair(&mut self, p: u64, count: u64, data: u64) -> Result<(), Trap> {
        let mut bytes = [0; 16];
        bytes[..8].copy_from_slice(&count.to_le_bytes());
        bytes[8..].copy_from_slice(&data.to_le_bytes());
        self.put(p, &bytes)
    }

    /// The part of this image for `len` bytes at `p`.
    fn part(&mut self, p: u64, len: usize) -> Result<Image<'_>, Trap> {
        let at = p.checked_sub(self.base).map(|at| at as usize);
        let range = at.and_then(|at| Some(at..at.checked_add(len)?));
        match range.and_then(|range| self.bytes.get_mut(range)) {
            Some(bytes) => Ok(Image {
                base: p,
                bytes,
            }),
            None => Err(Trap::unwritable(p, len)),
        }
    }
}

/// The Jai `string` at `p`, or "" when it cannot be read.
fn read_string(interp: &Interp, p: u64) -> String {
    match interp.read_pair(p) {
        Some((count, data)) if count > 0 => interp
            .read(data, count as usize)
            .map(|b| String::from_utf8_lossy(&b).into_owned())
            .unwrap_or_default(),
        _ => String::new(),
    }
}

/// An integer or a record id.
fn number(item: &Item) -> Option<i64> {
    match item {
        Item::Int(v) | Item::Ref(v) => Some(*v),
        Item::Str(_) => None,
    }
}

/// What a build is writing into: strings handed out and the records read.
/// How one plan entry is stored: its plan kind, its size in bytes and, for a pointer or nested
/// struct, the record type it refers to.
struct Slot {
    kind: u64,
    size: usize,
    type_target: u64,
}

struct Fill<'a> {
    records: &'a Records,
    kept: &'a mut Vec<Rc<[u8]>>,
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

    /// `__jaic_rec_plan(builder, info, size, entries, count, is_type_info_struct)`.
    pub fn add_plan(
        &mut self,
        interp: &Interp,
        info: u64,
        size: u64,
        at: u64,
        count: u64,
        type_info_struct: bool,
    ) -> Result<(), Trap> {
        let mut entries = Vec::with_capacity(count as usize);
        for k in 0..count {
            let entry = at + k * ENTRY_SIZE;
            let read = |off: u64| {
                interp
                    .read_u64(entry + off)
                    .ok_or_else(|| Trap::unreadable(entry + off, 8))
            };
            entries.push(Entry {
                offset: read(0)?,
                kind: read(8)?,
                size: read(16)? as usize,
                name: Name::intern(&read_string(interp, entry + 24)),
                element_kind: read(48)?,
                element_size: read(56)?,
                target: read(64)?,
                resizable: read(72)? != 0,
            });
        }
        let constant_storage = type_info_struct
            .then(|| {
                entries
                    .iter()
                    .find(|e| e.name.as_str() == "constant_storage")
            })
            .flatten()
            .map(|e| e.offset);
        self.plans.insert(
            info,
            Rc::new(Plan {
                size,
                entries,
                constant_storage,
            }),
        );
        Ok(())
    }

    /// The record a built struct came from, 0 if none.
    pub fn record_of(&self, pointer: u64) -> i64 {
        self.ids.get(&pointer).copied().unwrap_or(0)
    }

    /// `__jaic_rec_build(builder, record, expected)`: the struct for `record`, building it and
    /// everything it reaches that is not built yet. `expected` is the type the caller wants
    /// (a pointer member's target); the record's tag may name a subtype of it.
    pub fn build(
        &mut self,
        records: &Records,
        kept: &mut Vec<Rc<[u8]>>,
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
            let root = self.obtain(&mut fill, id, expected)?;
            while let Some((id, memory, info)) = fill.queue.pop() {
                let plan = self.plans[&info].clone();
                let mut bytes = vec![0; plan.size as usize];
                let mut image = Image {
                    base: memory,
                    bytes: &mut bytes,
                };
                self.fill_struct(&mut fill, interp, &mut image, memory, &plan, id)?;
                self.place(interp, memory, &bytes)?;
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
                store(interp, entry.address, &0u64.to_le_bytes())?;
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
                        .read_u64(entry.address)
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
                            let pointer = self.obtain(fill, target, entry.expected)?;
                            store(interp, entry.address, &pointer.to_le_bytes())?;
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
    fn obtain(&mut self, fill: &mut Fill, id: i64, expected: u64) -> Result<u64, Trap> {
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
        if let Some(record) = record {
            // A descriptor of the program reading the records: the real one.
            if let Some(FieldRef::Int(address)) = record.field("__address") {
                self.remember(id, address as u64);
                fill.created.push(id);
                return Ok(address as u64);
            }
            if let Some(FieldRef::Str(name)) = record.field("__builtin")
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
        let memory = self.alloc(plan.size as usize);
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

    /// Copy `bytes` to `p`, memory from `alloc` that the program has not been given yet. It
    /// is the builder's own, so it is copied in place rather than through the interpreter's
    /// checked transfers (one kernel call each on some hosts).
    fn place(&mut self, interp: &mut Interp, p: u64, bytes: &[u8]) -> Result<(), Trap> {
        let words = bytes.len().div_ceil(16);
        let chunk = self.chunks.iter_mut().rev().find_map(|chunk| {
            let start = chunk.as_ptr() as u64;
            let at = p.checked_sub(start)?;
            let index = (at % 16 == 0).then_some(at as usize / 16)?;
            chunk.get_mut(index..index.checked_add(words)?)
        });
        let Some(slots) = chunk else {
            return store(interp, p, bytes);
        };
        for (slot, word) in slots.iter_mut().zip(bytes.chunks(16)) {
            let mut value = [0; 16];
            value[..word.len()].copy_from_slice(word);
            *slot = u128::from_ne_bytes(value);
        }
        Ok(())
    }

    /// Zeroed, 16-byte aligned memory that lives as long as the builder.
    fn alloc(&mut self, size: usize) -> u64 {
        let size = (size.max(1) + 15) & !15;
        if size > CHUNK / 4 {
            let mut block = vec![0u128; size / 16].into_boxed_slice();
            let at = block.as_mut_ptr() as u64;
            self.chunks.push(block);
            return at;
        }
        if size > self.left {
            let mut chunk = vec![0u128; CHUNK / 16].into_boxed_slice();
            self.next = chunk.as_mut_ptr() as u64;
            self.left = CHUNK;
            self.chunks.push(chunk);
        }
        let at = self.next;
        self.next += size as u64;
        self.left -= size;
        at
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
        for entry in &plan.entries {
            if entry.kind == PLAN_POINTER
                && matches!(
                    record.field_named(entry.name),
                    None | Some(FieldRef::Ref(0) | FieldRef::Int(0))
                )
            {
                self.pending_refs.entry(id).or_default().push(PendingRef {
                    name: entry.name,
                    address: memory + entry.offset,
                    expected: entry.target,
                });
            }
            let Some(field) = record.field_named(entry.name) else {
                continue;
            };
            let target = memory + entry.offset;
            match field {
                FieldRef::List(items)
                    if entry.kind == PLAN_ARRAY || entry.kind == PLAN_FIXED_ARRAY =>
                {
                    if items.is_empty() {
                        continue;
                    }
                    let size = entry.element_size;
                    let bytes = (items.len() as u64)
                        .checked_mul(size)
                        .filter(|&n| n <= crate::sema::value::MAX_IMAGE)
                        .ok_or_else(|| Trap::new("records: array size overflow"))?;
                    if entry.kind == PLAN_FIXED_ARRAY && bytes > entry.size as u64 {
                        return Err(Trap::new("records: fixed array has too many elements"));
                    }
                    let data = if entry.kind == PLAN_FIXED_ARRAY {
                        target
                    } else {
                        self.alloc(bytes as usize)
                    };
                    let mut elements = vec![0; bytes as usize];
                    let mut array = Image {
                        base: data,
                        bytes: &mut elements,
                    };
                    for (k, item) in items.iter().enumerate() {
                        let item = &item;
                        self.fill_item(
                            fill,
                            interp,
                            &mut array,
                            Slot {
                                kind: entry.element_kind,
                                size: size as usize,
                                type_target: entry.target,
                            },
                            item,
                            data + k as u64 * size,
                        )?;
                    }
                    if entry.kind == PLAN_FIXED_ARRAY {
                        image.put(target, &elements)?;
                    } else {
                        self.place(interp, data, &elements)?;
                        image.put_pair(target, items.len() as u64, data)?;
                    }
                    if entry.resizable {
                        image.put(target + 16, &(items.len() as u64).to_le_bytes())?;
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
                            kind: entry.kind,
                            size: entry.size,
                            type_target: entry.target,
                        },
                        &item,
                        target,
                    )?;
                }
            }
        }
        Ok(())
    }

    /// Write one value of plan kind `kind` and `size` bytes at `target`.
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
            kind,
            size,
            type_target,
        } = slot;
        match (kind, item) {
            (PLAN_INT, Item::Int(v) | Item::Ref(v)) => {
                image.put(target, &v.to_le_bytes()[..size.min(8)])
            }
            (PLAN_STRING, Item::Str(s)) => {
                fill.kept.push(s.clone());
                image.put_pair(target, s.len() as u64, s.as_ptr() as u64)
            }
            (PLAN_POINTER, item) => {
                let id = number(item).ok_or_else(|| {
                    Trap::new("records: pointer field requires a record reference")
                })?;
                let p = self.obtain(fill, id, type_target)?;
                image.put(target, &p.to_le_bytes())
            }
            (PLAN_STRUCT, item) => {
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
                "records: value shape does not match plan kind {kind}"
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
            .read_u64(storage + 8)
            .ok_or_else(|| Trap::unreadable(storage + 8, 8))?;
        for [offset, record] in items.as_chunks::<2>().0 {
            let (Some(offset), Some(record)) = (number(offset), number(record)) else {
                continue;
            };
            let pointer = self.obtain(fill, record, 0)?;
            store(interp, data + offset as u64, &pointer.to_le_bytes())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::records::{Record, RecordTag};

    #[test]
    fn late_record_references_update_existing_identity_without_replaying_user_edits() {
        let mut b = Builder::default();
        b.add_type("Code_Node".into(), 1);
        b.plans.insert(
            1,
            Rc::new(Plan {
                size: 16,
                entries: vec![
                    Entry {
                        offset: 0,
                        kind: PLAN_INT,
                        size: 8,
                        name: Name::intern("value"),
                        element_kind: 0,
                        element_size: 0,
                        target: 0,
                        resizable: false,
                    },
                    Entry {
                        offset: 8,
                        kind: PLAN_POINTER,
                        size: 8,
                        name: Name::intern("next"),
                        element_kind: 0,
                        element_size: 0,
                        target: 1,
                        resizable: false,
                    },
                ],
                constant_storage: None,
            }),
        );
        let mut records = Records::default();
        let mut root = Record::new(RecordTag::CodeNode);
        root.int("value", 1);
        let root = records.add(root);
        let mut kept = Vec::new();
        let mut interp = Interp::default();
        let address = b.build(&records, &mut kept, &mut interp, root, 1).unwrap();
        assert_eq!(interp.read_u64(address + 8), Some(0));
        assert!(interp.write(address, &77u64.to_le_bytes()));
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
        assert!(interp.write(user_address + 8, &address.to_le_bytes()));
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
        let mut interp = Interp::default();
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
            b.build(&records, &mut Vec::new(), &mut Interp::default(), id, 0)
                .is_err()
        );
        // A failed fill must not leave a cached zero/partially filled success.
        assert!(
            b.build(&records, &mut Vec::new(), &mut Interp::default(), id, 0)
                .is_err()
        );
    }

    #[test]
    fn malformed_field_shape_is_an_error() {
        let mut b = Builder::default();
        b.add_type("Code_Node".into(), 1);
        b.plans.insert(
            1,
            Rc::new(Plan {
                size: 8,
                entries: vec![Entry {
                    offset: 0,
                    kind: PLAN_INT,
                    size: 8,
                    name: Name::intern("value"),
                    element_kind: 0,
                    element_size: 0,
                    target: 0,
                    resizable: false,
                }],
                constant_storage: None,
            }),
        );
        let mut records = Records::default();
        let mut record = Record::new(RecordTag::CodeNode);
        record.str("value", b"not an integer");
        let id = records.add(record);
        assert!(
            b.build(&records, &mut Vec::new(), &mut Interp::default(), id, 0)
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
                Rc::new(Plan {
                    size: 16,
                    entries: vec![Entry {
                        offset,
                        kind: PLAN_INT,
                        size: 8,
                        name: Name::intern("value"),
                        element_kind: 0,
                        element_size: 0,
                        target: 0,
                        resizable: false,
                    }],
                    constant_storage: None,
                }),
            );
        }
        let mut records = Records::default();
        let mut interp = Interp::default();
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
            Rc::new(Plan {
                size: 24,
                entries: vec![Entry {
                    offset: 8,
                    kind: PLAN_FIXED_ARRAY,
                    size: 16,
                    name: Name::intern("values"),
                    element_kind: PLAN_INT,
                    element_size: 8,
                    target: 0,
                    resizable: false,
                }],
                constant_storage: None,
            }),
        );
        let mut records = Records::default();
        let mut record = Record::new(RecordTag::Record);
        record.refs("values", [13, 29]);
        let id = records.add(record);
        let mut interp = Interp::default();
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
