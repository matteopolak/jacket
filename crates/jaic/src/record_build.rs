//! Building a metaprogram's structs from compiler records, natively.
//!
//! `stdlib/Compiler/records.jai` describes each struct type it may build once, as a plan of
//! members read from its `Type_Info_Struct` (`__jaic_rec_plan`), and names the types record
//! tags stand for (`__jaic_rec_type`). From then on `__jaic_rec_build` turns a record and
//! everything it reaches into structs in one call: a TYPECHECKED message of a large program
//! is hundreds of thousands of records, and filling them member by member in interpreted
//! code took most of a metaprogram's time.
//!
//! The structs and their arrays live in memory the registry owns, like the strings handed
//! to Jai code, for the rest of the compilation. Each record is built once per program, so
//! pointers to the same record stay the same pointer.
use crate::fxhash::HashMap;
use crate::interp::{Interp, Trap};
use crate::records::{FieldRef, Item, List, Name, Records};
use std::rc::Rc;

/// Plan entry kinds shared with `Record_Plan_Entry` in `stdlib/Compiler/records.jai`.
pub const PLAN_OTHER: u64 = 0;

pub const PLAN_INT: u64 = 1;
pub const PLAN_STRING: u64 = 2;
pub const PLAN_POINTER: u64 = 3;

/// A struct held in place.
pub const PLAN_STRUCT: u64 = 4;

/// A view or resizable array.
pub const PLAN_ARRAY: u64 = 5;

/// Size of `Record_Plan_Entry`: offset, kind, size, name (16 bytes), type, element_kind,
/// element_size, target, resizable.
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

/// One program's structs built from records (one per `__jaic_rec_builder`).
#[derive(Default)]
pub struct Builder {
    plans: HashMap<u64, Rc<Plan>>,
    /// The struct type a record tag stands for.
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
}

fn unwritable(p: u64) -> Trap {
    Trap::new(format!("invalid memory access: {p:#x} cannot be written"))
}

fn unreadable(p: u64) -> Trap {
    Trap::new(format!("invalid memory access: {p:#x} cannot be read"))
}

fn store(interp: &mut Interp, p: u64, bytes: &[u8]) -> Result<(), Trap> {
    interp
        .write(p, bytes)
        .then_some(())
        .ok_or_else(|| unwritable(p))
}

fn store_pair(interp: &mut Interp, p: u64, count: u64, data: u64) -> Result<(), Trap> {
    let mut bytes = [0; 16];
    bytes[..8].copy_from_slice(&count.to_le_bytes());
    bytes[8..].copy_from_slice(&data.to_le_bytes());
    store(interp, p, &bytes)
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
struct Fill<'a> {
    records: &'a Records,
    kept: &'a mut Vec<Rc<[u8]>>,
    /// Records built and not filled yet: (id, memory, type).
    queue: Vec<(i64, u64, u64)>,
}

impl Builder {
    pub fn add_type(&mut self, name: String, info: u64) {
        self.tags.insert(name.into(), info);
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
                    .ok_or_else(|| unreadable(entry))
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
        };
        let root = self.obtain(&mut fill, id, expected);
        while let Some((id, memory, info)) = fill.queue.pop() {
            let plan = self.plans[&info].clone();
            self.fill_struct(&mut fill, interp, memory, &plan, id)?;
            if let Some(at) = plan.constant_storage {
                self.patch_constant_pointers(&mut fill, interp, memory + at, id)?;
            }
        }
        Ok(root)
    }

    /// The struct for record `id`: built already, or allocated now and queued to be filled.
    fn obtain(&mut self, fill: &mut Fill, id: i64, expected: u64) -> u64 {
        if id <= 0 {
            return 0;
        }
        if let Some(&p) = self.built.get(id as usize)
            && p != 0
        {
            return p;
        }
        let record = fill.records.get(id);
        if let Some(record) = record {
            // A descriptor of the program reading the records: the real one.
            if let Some(FieldRef::Int(address)) = record.field("__address") {
                self.remember(id, address as u64);
                return address as u64;
            }
            if let Some(FieldRef::Str(name)) = record.field("__builtin")
                && let Some(&info) = std::str::from_utf8(name)
                    .ok()
                    .and_then(|name| self.builtins.get(name))
            {
                self.remember(id, info);
                return info;
            }
        }
        let tagged = record.and_then(|r| self.tags.get(r.tag)).copied();
        let Some(info) = tagged.or_else(|| self.plans.contains_key(&expected).then_some(expected))
        else {
            return 0;
        };
        let memory = self.alloc(self.plans[&info].size as usize);
        // Before filling: records may refer to themselves.
        self.remember(id, memory);
        if record.is_some() {
            fill.queue.push((id, memory, info));
        }
        memory
    }

    fn remember(&mut self, id: i64, pointer: u64) {
        let at = id as usize;
        if self.built.len() <= at {
            self.built.resize(at + 1, 0);
        }
        self.built[at] = pointer;
        self.ids.entry(pointer).or_insert(id);
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
        memory: u64,
        plan: &Plan,
        id: i64,
    ) -> Result<(), Trap> {
        let Some(record) = fill.records.get(id) else {
            return Ok(());
        };
        for entry in &plan.entries {
            let Some(field) = record.field_named(entry.name) else {
                continue;
            };
            let target = memory + entry.offset;
            match field {
                FieldRef::List(items) if entry.kind == PLAN_ARRAY && !items.is_empty() => {
                    let size = entry.element_size;
                    let data = self.alloc((items.len() as u64 * size) as usize);
                    for (k, item) in items.iter().enumerate() {
                        let item = &item;
                        self.fill_item(
                            fill,
                            interp,
                            entry.element_kind,
                            size as usize,
                            entry.target,
                            item,
                            data + k as u64 * size,
                        )?;
                    }
                    store_pair(interp, target, items.len() as u64, data)?;
                    if entry.resizable {
                        store(interp, target + 16, &(items.len() as u64).to_le_bytes())?;
                    }
                }
                // A list read as a single value is zero, and the memory is zeroed already.
                FieldRef::List(_) => {}
                single => {
                    let Some(item) = single.item() else {
                        continue;
                    };
                    self.fill_item(
                        fill,
                        interp,
                        entry.kind,
                        entry.size,
                        entry.target,
                        &item,
                        target,
                    )?;
                }
            }
        }
        Ok(())
    }

    /// Write one value of plan kind `kind` and `size` bytes at `target`.
    #[allow(clippy::too_many_arguments)]
    fn fill_item(
        &mut self,
        fill: &mut Fill,
        interp: &mut Interp,
        kind: u64,
        size: usize,
        type_target: u64,
        item: &Item,
        target: u64,
    ) -> Result<(), Trap> {
        match (kind, item) {
            (PLAN_INT, Item::Int(v) | Item::Ref(v)) => {
                store(interp, target, &v.to_le_bytes()[..size.min(8)])
            }
            (PLAN_STRING, Item::Str(s)) => {
                fill.kept.push(s.clone());
                store_pair(interp, target, s.len() as u64, s.as_ptr() as u64)
            }
            (PLAN_POINTER, item) => match number(item).map(|r| self.obtain(fill, r, type_target)) {
                Some(p) if p != 0 => store(interp, target, &p.to_le_bytes()),
                _ => Ok(()),
            },
            (PLAN_STRUCT, item) => match (number(item), self.plans.get(&type_target).cloned()) {
                (Some(r), Some(plan)) if r != 0 => self.fill_struct(fill, interp, target, &plan, r),
                _ => Ok(()),
            },
            // Mismatched shapes read as zero, and the memory is zeroed already.
            _ => Ok(()),
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
            .ok_or_else(|| unreadable(storage))?;
        for [offset, record] in items.as_chunks::<2>().0 {
            let (Some(offset), Some(record)) = (number(offset), number(record)) else {
                continue;
            };
            let pointer = self.obtain(fill, record, 0);
            store(interp, data + offset as u64, &pointer.to_le_bytes())?;
        }
        Ok(())
    }
}
