//! Neutral data records handed from the compiler to metaprograms.
//!
//! The compiler cannot write Jai structs into the metaprogram's memory while
//! the metaprogram runs (the metaprogram's interpreter borrows its own
//! compiler), and the target workspace's types are not the metaprogram's
//! types anyway. So syntax trees, type descriptors and messages are exported
//! as records: a tag naming the Jai struct to build (`"Code_Ident"`,
//! `"Type_Info_Struct"`, `"Message_Import"`...) and named fields. Jai code in
//! `stdlib/Compiler/records.jai` turns a record into the struct by reflection,
//! matching fields to members by name.
//!
//! Record ids are global to a top-level compilation, start at 1, and 0 means
//! null. Records live until the compilation ends, so a record id (and the
//! struct the metaprogram built from it) keeps its identity across messages.
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::builtin_keys::SyntaxKey;
use crate::fxhash::HashMap;

#[derive(Clone, Debug)]
pub enum Item {
    /// Integers, enums, bools; floats as their raw bits.
    Int(i64),
    Str(Rc<[u8]>),
    /// Another record (0 = null).
    Ref(i64),
}

/// A field's value, as given to [`Record::set`].
#[derive(Clone, Debug)]
pub enum Field {
    Item(Item),
    List(Vec<Item>),
}

/// A field's value as read back: strings and lists are borrowed from the record.
#[derive(Clone, Copy, Debug)]
pub enum FieldRef<'a> {
    Int(i64),
    Str(&'a Rc<[u8]>),
    Ref(i64),
    List(List<'a>),
}

/// A list field as read back.
#[derive(Clone, Copy, Debug)]
pub enum List<'a> {
    Items(&'a [Item]),
    /// A list of record ids, kept as plain ids: most lists are.
    Refs(&'a [i64]),
}

impl List<'_> {
    pub fn len(self) -> usize {
        match self {
            List::Items(items) => items.len(),
            List::Refs(ids) => ids.len(),
        }
    }

    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    pub fn get(self, index: usize) -> Option<Item> {
        match self {
            List::Items(items) => items.get(index).cloned(),
            List::Refs(ids) => ids.get(index).map(|&id| Item::Ref(id)),
        }
    }

    pub fn iter(self) -> impl Iterator<Item = Item> {
        (0..self.len()).filter_map(move |i| self.get(i))
    }
}

impl FieldRef<'_> {
    /// The value as an [`Item`]; `None` for a list.
    pub fn item(self) -> Option<Item> {
        match self {
            FieldRef::Int(v) => Some(Item::Int(v)),
            FieldRef::Ref(v) => Some(Item::Ref(v)),
            FieldRef::Str(s) => Some(Item::Str(s.clone())),
            FieldRef::List(_) => None,
        }
    }
}

/// A field name, interned. Records hold `Rc`s, so they never leave the thread that made
/// them, and a table per thread is enough.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Name(u16);

#[derive(Default)]
struct Names {
    list: Vec<&'static str>,
    by_text: HashMap<&'static str, Name>,
    /// Literals by address: interning one is a pointer hash, not a string hash.
    by_address: HashMap<(usize, usize), Name>,
}

thread_local! {
    static NAMES: RefCell<Names> = RefCell::default();
    static SYNTAX_NAMES: std::cell::OnceCell<Vec<Name>> = const { std::cell::OnceCell::new() };
}

impl Name {
    /// The name `name`, interned on first use.
    pub fn of(name: &'static str) -> Name {
        NAMES.with_borrow_mut(|names| {
            let key = (name.as_ptr() as usize, name.len());
            if let Some(&id) = names.by_address.get(&key) {
                return id;
            }
            let id = names.add(name);
            names.by_address.insert(key, id);
            id
        })
    }

    /// The name `name` if any field was ever given it.
    pub fn find(name: &str) -> Option<Name> {
        NAMES.with_borrow(|names| names.by_text.get(name).copied())
    }

    /// The name `name`, interned on first use: for names that are not literals, from a
    /// small set (a leaked copy is kept).
    pub fn intern(name: &str) -> Name {
        Name::find(name).unwrap_or_else(|| {
            let name: &'static str = Box::leak(name.into());
            NAMES.with_borrow_mut(|names| names.add(name))
        })
    }

    pub fn as_str(self) -> &'static str {
        NAMES.with_borrow(|names| names.list[self.0 as usize])
    }

    /// The field a syntax attribute is stored in (`__syntax_` and the key's name).
    pub fn syntax(key: SyntaxKey) -> Name {
        SYNTAX_NAMES.with(|cache| {
            cache.get_or_init(|| {
                SyntaxKey::ALL
                    .iter()
                    .map(|key| {
                        let member = format!("{key:?}");
                        let mut field = String::from("__syntax_");
                        for (at, c) in member.chars().enumerate() {
                            if c.is_ascii_uppercase() && at > 0 {
                                field.push('_');
                            }
                            field.push(c.to_ascii_lowercase());
                        }
                        Name::intern(&field)
                    })
                    .collect()
            })[key as usize]
        })
    }
}

/// What names a record field: a literal, or a typed syntax attribute.
pub trait FieldKey: Copy {
    fn name(self) -> Name;
}

impl FieldKey for &'static str {
    fn name(self) -> Name {
        Name::of(self)
    }
}

impl FieldKey for SyntaxKey {
    fn name(self) -> Name {
        Name::syntax(self)
    }
}

impl Names {
    fn add(&mut self, name: &'static str) -> Name {
        if let Some(&id) = self.by_text.get(name) {
            return id;
        }
        let id = Name(u16::try_from(self.list.len()).expect("too many record field names"));
        self.list.push(name);
        self.by_text.insert(name, id);
        id
    }
}

/// One field: an integer or record id inline, a string or list by index into the record's
/// `extra`. 16 bytes, where a name and a [`Field`] took 48: syntax trees exported to a
/// metaprogram are most of its compiler's memory.
#[derive(Clone, Copy, Debug)]
struct Slot {
    name: Name,
    kind: u8,
    value: i64,
}

#[derive(Clone, Debug)]
enum Extra {
    Str(Rc<[u8]>),
    List(Vec<Item>),
    Refs(Box<[i64]>),
}

#[derive(Clone, Debug)]
pub struct Record {
    pub tag: &'static str,
    slots: Vec<Slot>,
    extra: Vec<Extra>,
}

impl Record {
    pub fn new(tag: &'static str) -> Self {
        Record {
            tag,
            slots: Vec::new(),
            extra: Vec::new(),
        }
    }

    /// A record with room for `fields` fields, for one whose size is known up front.
    pub fn with_capacity(tag: &'static str, fields: usize) -> Self {
        Record {
            tag,
            slots: Vec::with_capacity(fields),
            extra: Vec::new(),
        }
    }

    fn push(&mut self, name: Name, kind: i64, value: i64) {
        self.slots.push(Slot {
            name,
            kind: kind as u8,
            value,
        });
    }

    fn push_extra(&mut self, extra: Extra) -> i64 {
        self.extra.push(extra);
        self.extra.len() as i64 - 1
    }

    pub fn int(&mut self, name: impl FieldKey, v: i64) -> &mut Self {
        self.push(name.name(), KIND_INT, v);
        self
    }

    pub fn str(&mut self, name: impl FieldKey, s: &[u8]) -> &mut Self {
        let at = self.push_extra(Extra::Str(s.into()));
        self.push(name.name(), KIND_STRING, at);
        self
    }

    /// A pointer field; `0` (null) is left out.
    pub fn ptr(&mut self, name: impl FieldKey, id: i64) -> &mut Self {
        if id != 0 {
            self.push(name.name(), KIND_REF, id);
        }
        self
    }

    pub fn list(&mut self, name: impl FieldKey, items: Vec<Item>) -> &mut Self {
        let at = self.push_extra(Extra::List(items));
        self.push(name.name(), KIND_LIST, at);
        self
    }

    pub fn refs(&mut self, name: impl FieldKey, ids: impl IntoIterator<Item = i64>) -> &mut Self {
        let at = self.push_extra(Extra::Refs(ids.into_iter().collect()));
        self.push(name.name(), KIND_LIST, at);
        self
    }

    fn value(&self, slot: &Slot) -> FieldRef<'_> {
        match (slot.kind as i64, &self.extra.get(slot.value as usize)) {
            (KIND_STRING, Some(Extra::Str(s))) => FieldRef::Str(s),
            (KIND_LIST, Some(Extra::List(items))) => FieldRef::List(List::Items(items)),
            (KIND_LIST, Some(Extra::Refs(ids))) => FieldRef::List(List::Refs(ids)),
            (KIND_REF, _) => FieldRef::Ref(slot.value),
            _ => FieldRef::Int(slot.value),
        }
    }

    pub fn field(&self, name: &str) -> Option<FieldRef<'_>> {
        self.field_named(Name::find(name)?)
    }

    pub fn field_named(&self, name: Name) -> Option<FieldRef<'_>> {
        let slot = self.slots.iter().find(|s| s.name == name)?;
        Some(self.value(slot))
    }

    pub fn set(&mut self, name: impl FieldKey, field: Field) {
        let (kind, value) = match field {
            Field::Item(Item::Int(v)) => (KIND_INT, v),
            Field::Item(Item::Ref(v)) => (KIND_REF, v),
            Field::Item(Item::Str(s)) => (KIND_STRING, self.push_extra(Extra::Str(s))),
            Field::List(items) => (KIND_LIST, self.push_extra(Extra::List(items))),
        };
        let id = name.name();
        match self.slots.iter_mut().find(|s| s.name == id) {
            Some(slot) => {
                slot.kind = kind as u8;
                slot.value = value;
            }
            None => self.push(id, kind, value),
        }
    }
}

/// Field kinds as reported to Jai (`__jaic_rec_field`).
pub const KIND_NONE: i64 = 0;

pub const KIND_INT: i64 = 1;
pub const KIND_STRING: i64 = 2;
pub const KIND_REF: i64 = 3;
pub const KIND_LIST: i64 = 4;

/// Every record of one compilation, by id (1, 2, ...).
///
/// Ids are dense and never reused: `stdlib/Compiler` builds each record's struct once and
/// keeps it in an array indexed by id for the rest of the compilation. An exporter takes
/// the records out of the registry while it runs ([`Records::lend`]), because resolving
/// names may run compile-time code that reads and makes records of its own; that code gets
/// a part that draws ids from the same counter, merged back by [`Records::give_back`].
#[derive(Default)]
pub struct Records {
    /// Records by `id - 1 - first`. `None`: an id the other side of a lend gave out.
    list: Vec<Option<Record>>,
    /// Ids up to this belong to the records this part was lent from.
    first: usize,
    /// The last id given out, shared by all parts.
    last: Rc<Cell<usize>>,
    /// Mutations of existing record identities, shared across temporary lends.
    changes: Rc<RefCell<Vec<i64>>>,
}

impl Records {
    /// Reserve an id now and fill the record later (records may refer to themselves).
    pub fn reserve(&mut self, tag: &'static str) -> i64 {
        self.add(Record::new(tag))
    }

    pub fn add(&mut self, record: Record) -> i64 {
        let id = self.last.get() + 1;
        self.last.set(id);
        self.put(id, record);
        id as i64
    }

    fn put(&mut self, id: usize, record: Record) {
        let at = id - 1 - self.first;
        if self.list.len() <= at {
            self.list.resize_with(at + 1, || None);
        }
        self.list[at] = Some(record);
    }

    fn index(&self, id: i64) -> Option<usize> {
        (id as usize).checked_sub(1 + self.first).filter(|_| id > 0)
    }

    pub fn get(&self, id: i64) -> Option<&Record> {
        self.list.get(self.index(id)?)?.as_ref()
    }

    pub fn get_mut(&mut self, id: i64) -> Option<&mut Record> {
        let at = self.index(id)?;
        let record = self.list.get_mut(at)?.as_mut()?;
        self.changes.borrow_mut().push(id);
        Some(record)
    }

    pub fn changes_since(&self, cursor: usize) -> (usize, Vec<i64>) {
        let changes = self.changes.borrow();
        (changes.len(), changes[cursor..].to_vec())
    }

    /// Take the records out of `slot`, leaving an empty part in their place that numbers
    /// new records after every id given out so far (on either side).
    pub fn lend(slot: &mut Records) -> Records {
        let part = Records {
            list: Vec::new(),
            first: slot.last.get(),
            last: slot.last.clone(),
            changes: slot.changes.clone(),
        };
        std::mem::replace(slot, part)
    }

    /// Put `records` (from [`Records::lend`]) back in `slot`, with the records added to the
    /// part meanwhile.
    pub fn give_back(slot: &mut Records, records: Records) {
        let part = std::mem::replace(slot, records);
        for (at, record) in part.list.into_iter().enumerate() {
            if let Some(record) = record {
                slot.put(part.first + at + 1, record);
            }
        }
    }

    pub fn field(&self, id: i64, name: &str) -> Option<FieldRef<'_>> {
        self.get(id)?.field(name)
    }

    /// The syntax attribute `key` of record `id`.
    pub fn syntax_field(&self, id: i64, key: SyntaxKey) -> Option<FieldRef<'_>> {
        self.get(id)?.field_named(key.field())
    }

    pub fn kind(&self, id: i64, name: &str) -> i64 {
        match self.field(id, name) {
            None => KIND_NONE,
            Some(FieldRef::Int(_)) => KIND_INT,
            Some(FieldRef::Str(_)) => KIND_STRING,
            Some(FieldRef::Ref(_)) => KIND_REF,
            Some(FieldRef::List(_)) => KIND_LIST,
        }
    }

    /// The item named `name`, or element `index` of the list named `name`.
    pub fn item(&self, id: i64, name: &str, index: Option<usize>) -> Option<Item> {
        match (self.field(id, name)?, index) {
            (FieldRef::List(items), Some(i)) => items.get(i),
            (single, None) => single.item(),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_made_while_lent_keep_unique_ids() {
        // Found by the `lsp_edits` fuzz target: compile-time code run while `compiler_get_nodes`
        // exported a tree numbered its records from 1 again, and `stdlib/Compiler`, which
        // builds each record's struct once by id, handed it structs of other records.
        let mut registry = Records::default();
        let before = registry.add(Record::new("A"));
        let mut lent = Records::lend(&mut registry);
        let outer = lent.reserve("B");
        let nested = registry.add(Record::new("C"));
        let mut deeper = Records::lend(&mut registry);
        let deepest = registry.add(Record::new("D"));
        let after_nested = deeper.add(Record::new("E"));
        Records::give_back(&mut registry, deeper);
        let last = lent.add(Record::new("F"));
        Records::give_back(&mut registry, lent);
        let ids = [before, outer, nested, deepest, after_nested, last];
        assert_eq!(ids, [1, 2, 3, 4, 5, 6]);
        for (id, tag) in ids.into_iter().zip(["A", "B", "C", "D", "E", "F"]) {
            assert_eq!(registry.get(id).map(|r| r.tag), Some(tag), "record {id}");
        }
        assert!(registry.get(0).is_none() && registry.get(7).is_none());
    }
}
