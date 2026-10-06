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

#[derive(Default)]
pub struct Records {
    list: Vec<Record>,
}

impl Records {
    /// Reserve an id now and fill the record later (records may refer to themselves).
    pub fn reserve(&mut self, tag: &'static str) -> i64 {
        self.list.push(Record::new(tag));
        self.list.len() as i64
    }

    pub fn add(&mut self, record: Record) -> i64 {
        self.list.push(record);
        self.list.len() as i64
    }

    pub fn get(&self, id: i64) -> Option<&Record> {
        if id <= 0 {
            return None;
        }
        self.list.get(id as usize - 1)
    }

    pub fn get_mut(&mut self, id: i64) -> Option<&mut Record> {
        if id <= 0 {
            return None;
        }
        self.list.get_mut(id as usize - 1)
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
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
