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
use std::cell::Cell;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum Item {
    /// Integers, enums, bools; floats as their raw bits.
    Int(i64),
    Str(Rc<[u8]>),
    /// Another record (0 = null).
    Ref(i64),
}

#[derive(Clone, Debug)]
pub enum Field {
    Item(Item),
    List(Vec<Item>),
}

#[derive(Clone, Debug)]
pub struct Record {
    pub tag: &'static str,
    pub fields: Vec<(&'static str, Field)>,
}

impl Record {
    pub fn new(tag: &'static str) -> Self {
        Record {
            tag,
            fields: Vec::new(),
        }
    }

    pub fn int(&mut self, name: &'static str, v: i64) -> &mut Self {
        self.fields.push((name, Field::Item(Item::Int(v))));
        self
    }

    pub fn str(&mut self, name: &'static str, s: &[u8]) -> &mut Self {
        self.fields.push((name, Field::Item(Item::Str(s.into()))));
        self
    }

    /// A pointer field; `0` (null) is left out.
    pub fn ptr(&mut self, name: &'static str, id: i64) -> &mut Self {
        if id != 0 {
            self.fields.push((name, Field::Item(Item::Ref(id))));
        }
        self
    }

    pub fn list(&mut self, name: &'static str, items: Vec<Item>) -> &mut Self {
        self.fields.push((name, Field::List(items)));
        self
    }

    pub fn refs(&mut self, name: &'static str, ids: impl IntoIterator<Item = i64>) -> &mut Self {
        self.list(name, ids.into_iter().map(Item::Ref).collect())
    }

    pub fn field(&self, name: &str) -> Option<&Field> {
        self.fields.iter().find(|(n, _)| *n == name).map(|(_, f)| f)
    }

    pub fn set(&mut self, name: &'static str, field: Field) {
        match self.fields.iter_mut().find(|(n, _)| *n == name) {
            Some((_, f)) => *f = field,
            None => self.fields.push((name, field)),
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
        self.list.get_mut(at)?.as_mut()
    }

    /// Take the records out of `slot`, leaving an empty part in their place that numbers
    /// new records after every id given out so far (on either side).
    pub fn lend(slot: &mut Records) -> Records {
        let part = Records {
            list: Vec::new(),
            first: slot.last.get(),
            last: slot.last.clone(),
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

    pub fn field(&self, id: i64, name: &str) -> Option<&Field> {
        self.get(id)?.field(name)
    }

    pub fn kind(&self, id: i64, name: &str) -> i64 {
        match self.field(id, name) {
            None => KIND_NONE,
            Some(Field::Item(Item::Int(_))) => KIND_INT,
            Some(Field::Item(Item::Str(_))) => KIND_STRING,
            Some(Field::Item(Item::Ref(_))) => KIND_REF,
            Some(Field::List(_)) => KIND_LIST,
        }
    }

    /// The item named `name`, or element `index` of the list named `name`.
    pub fn item(&self, id: i64, name: &str, index: Option<usize>) -> Option<&Item> {
        match (self.field(id, name)?, index) {
            (Field::Item(item), None) => Some(item),
            (Field::List(items), Some(i)) => items.get(i),
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
