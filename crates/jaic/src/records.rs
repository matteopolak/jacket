//! Neutral data records handed from the compiler to metaprograms.
//!
//! The compiler cannot write Jai structs into the metaprogram's memory while
//! the metaprogram runs (the metaprogram's interpreter borrows its own
//! compiler), and the target workspace's types are not the metaprogram's
//! types anyway. So syntax trees, type descriptors and messages are exported
//! as records: a tag naming the Jai struct to build (`Code_Ident`,
//! `Type_Info_Struct`, `Message_Import`...) and named fields. Jai code in
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
use std::hash::BuildHasherDefault;
use std::sync::{OnceLock, PoisonError, RwLock};

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

/// A field name, interned in one table for the whole process. Records hold `Rc`s and stay on
/// the thread that made them, but a record can be made on any thread a compile-time run
/// started, so the names must mean the same everywhere.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Name(u16);

struct Names {
    list: Vec<&'static str>,
    by_text: HashMap<&'static str, Name>,
}

static NAMES: RwLock<Names> = RwLock::new(Names {
    list: Vec::new(),
    by_text: HashMap::with_hasher(BuildHasherDefault::new()),
});

/// What this thread has seen of `NAMES`, so that after warm-up no lookup takes the lock:
/// literals by address (a pointer hash, not a string hash), names by text, and a copy of the
/// spellings for `as_str`.
#[derive(Default)]
struct Seen {
    by_address: HashMap<(usize, usize), Name>,
    by_text: HashMap<&'static str, Name>,
    list: Vec<&'static str>,
}

thread_local! {
    static SEEN: RefCell<Seen> = RefCell::default();
}

fn names() -> std::sync::RwLockReadGuard<'static, Names> {
    NAMES.read().unwrap_or_else(PoisonError::into_inner)
}

impl Name {
    /// The name `name`, interned on first use.
    pub fn of(name: &'static str) -> Name {
        let key = (name.as_ptr() as usize, name.len());
        if let Some(id) = SEEN.with_borrow(|seen| seen.by_address.get(&key).copied()) {
            return id;
        }
        let id = Name::find(name).unwrap_or_else(|| Name::add(name));
        SEEN.with_borrow_mut(|seen| seen.by_address.insert(key, id));
        id
    }

    /// The name's position in the table: small, dense, the same on every thread.
    pub fn index(self) -> usize {
        self.0 as usize
    }

    /// The name `name` if any field was ever given it.
    pub fn find(name: &str) -> Option<Name> {
        if let Some(id) = SEEN.with_borrow(|seen| seen.by_text.get(name).copied()) {
            return Some(id);
        }
        let (&text, &id) = names().by_text.get_key_value(name)?;
        SEEN.with_borrow_mut(|seen| seen.by_text.insert(text, id));
        Some(id)
    }

    /// The name `name`, interned on first use: for names that are not literals, from a
    /// small set (a leaked copy is kept).
    pub fn intern(name: &str) -> Name {
        Name::find(name).unwrap_or_else(|| Name::add(Box::leak(name.into())))
    }

    fn add(name: &'static str) -> Name {
        let mut names = NAMES.write().unwrap_or_else(PoisonError::into_inner);
        if let Some(&id) = names.by_text.get(name) {
            return id;
        }
        let id = Name(u16::try_from(names.list.len()).expect("too many record field names"));
        names.list.push(name);
        names.by_text.insert(name, id);
        id
    }

    pub fn as_str(self) -> &'static str {
        SEEN.with_borrow_mut(|seen| {
            let at = self.0 as usize;
            if at >= seen.list.len() {
                let names = names();
                let known = seen.list.len();
                seen.list.extend_from_slice(&names.list[known..]);
            }
            seen.list[at]
        })
    }

    /// The field a syntax attribute is stored in (`__syntax_` and the key's name).
    pub fn syntax(key: SyntaxKey) -> Name {
        static SYNTAX_NAMES: OnceLock<Vec<Name>> = OnceLock::new();
        SYNTAX_NAMES.get_or_init(|| {
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

/// Declares [`RecordTag`] from `Variant = "Jai_Struct_Name"` pairs.
macro_rules! record_tags {
    ($($variant:ident = $name:literal,)*) => {
        /// The Jai struct a [`Record`] becomes, named by the struct's own name.
        ///
        /// `stdlib/Compiler/contract-records.jai` (and the prelude's reflection types) define
        /// the structs; the `every_tag_names_a_stdlib_struct` test keeps the two in step.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum RecordTag {
            $($variant,)*
        }

        impl RecordTag {
            pub const ALL: &'static [RecordTag] = &[$(RecordTag::$variant,)*];

            /// The Jai struct's name.
            pub const fn name(self) -> &'static str {
                match self {
                    $(RecordTag::$variant => $name,)*
                }
            }
        }
    };
}

record_tags! {
    BytecodeReport = "Bytecode_Report",
    CodeArgument = "Code_Argument",
    CodeArrayLiteralInfo = "Code_Array_Literal_Info",
    CodeAsm = "Code_Asm",
    CodeBinaryOperator = "Code_Binary_Operator",
    CodeBlock = "Code_Block",
    CodeCase = "Code_Case",
    CodeCast = "Code_Cast",
    CodeCommaSeparatedArgument = "Code_Comma_Separated_Argument",
    CodeCommaSeparatedArguments = "Code_Comma_Separated_Arguments",
    CodeCompoundDeclaration = "Code_Compound_Declaration",
    CodeContext = "Code_Context",
    CodeDeclaration = "Code_Declaration",
    CodeDefer = "Code_Defer",
    CodeDirectiveAddContext = "Code_Directive_Add_Context",
    CodeDirectiveBake = "Code_Directive_Bake",
    CodeDirectiveBytes = "Code_Directive_Bytes",
    CodeDirectiveCode = "Code_Directive_Code",
    CodeDirectiveExists = "Code_Directive_Exists",
    CodeDirectiveImport = "Code_Directive_Import",
    CodeDirectiveInsert = "Code_Directive_Insert",
    CodeDirectiveLoad = "Code_Directive_Load",
    CodeDirectiveLocation = "Code_Directive_Location",
    CodeDirectiveModify = "Code_Directive_Modify",
    CodeDirectiveModuleParameters = "Code_Directive_Module_Parameters",
    CodeDirectiveOverlay = "Code_Directive_Overlay",
    CodeDirectiveProcedureName = "Code_Directive_Procedure_Name",
    CodeDirectiveRun = "Code_Directive_Run",
    CodeDirectiveScope = "Code_Directive_Scope",
    CodeDirectiveThrough = "Code_Directive_Through",
    CodeEnum = "Code_Enum",
    CodeFor = "Code_For",
    CodeIdent = "Code_Ident",
    CodeIf = "Code_If",
    CodeLiteral = "Code_Literal",
    CodeLoopControl = "Code_Loop_Control",
    CodeNode = "Code_Node",
    CodeNodes = "Code_Nodes",
    CodeNote = "Code_Note",
    CodePointerLiteralInfo = "Code_Pointer_Literal_Info",
    CodeProcedureBody = "Code_Procedure_Body",
    CodeProcedureCall = "Code_Procedure_Call",
    CodeProcedureHeader = "Code_Procedure_Header",
    CodePushContext = "Code_Push_Context",
    CodeReturn = "Code_Return",
    CodeStruct = "Code_Struct",
    CodeStructLiteralInfo = "Code_Struct_Literal_Info",
    CodeTypeInstantiation = "Code_Type_Instantiation",
    CodeUnaryOperator = "Code_Unary_Operator",
    CodeUsing = "Code_Using",
    CodeWhile = "Code_While",
    DistinctPolymorph = "Distinct_Polymorph",
    MessageFailedImport = "Message_Failed_Import",
    MessageFile = "Message_File",
    MessageImport = "Message_Import",
    MessagePerformanceReport = "Message_Performance_Report",
    MessageTypechecked = "Message_Typechecked",
    PolymorphReport = "Polymorph_Report",
    Record = "Record",
    RunDirectiveReport = "Run_Directive_Report",
    SourceCodeLocation = "Source_Code_Location",
    TimeReport = "Time_Report",
    TypeInfo = "Type_Info",
    TypeInfoArray = "Type_Info_Array",
    TypeInfoEnum = "Type_Info_Enum",
    TypeInfoFloat = "Type_Info_Float",
    TypeInfoInteger = "Type_Info_Integer",
    TypeInfoPointer = "Type_Info_Pointer",
    TypeInfoProcedure = "Type_Info_Procedure",
    TypeInfoString = "Type_Info_String",
    TypeInfoStruct = "Type_Info_Struct",
    TypeInfoStructMember = "Type_Info_Struct_Member",
    TypeInfoVariant = "Type_Info_Variant",
    Typechecked = "Typechecked",
}

impl std::fmt::Display for RecordTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

#[derive(Clone, Debug)]
pub struct Record {
    pub tag: RecordTag,
    slots: Vec<Slot>,
    extra: Vec<Extra>,
}

impl Record {
    pub fn new(tag: RecordTag) -> Self {
        Record {
            tag,
            slots: Vec::new(),
            extra: Vec::new(),
        }
    }

    /// A record with room for `fields` fields, for one whose size is known up front.
    pub fn with_capacity(tag: RecordTag, fields: usize) -> Self {
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

    /// How many fields the record has set (a field set twice counts twice).
    pub fn slot_count(&self) -> usize {
        self.slots.len()
    }

    /// The name of the `i`th field in the order they were added.
    pub fn slot_name(&self, i: usize) -> Name {
        self.slots[i].name
    }

    /// The value of the `i`th field in the order they were added.
    pub fn slot_value(&self, i: usize) -> FieldRef<'_> {
        self.value(&self.slots[i])
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

/// How a field slot's value is stored.
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
    pub fn reserve(&mut self, tag: RecordTag) -> i64 {
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
        let before = registry.add(Record::new(RecordTag::CodeIf));
        let mut lent = Records::lend(&mut registry);
        let outer = lent.reserve(RecordTag::CodeWhile);
        let nested = registry.add(Record::new(RecordTag::CodeFor));
        let mut deeper = Records::lend(&mut registry);
        let deepest = registry.add(Record::new(RecordTag::CodeBlock));
        let after_nested = deeper.add(Record::new(RecordTag::CodeCase));
        Records::give_back(&mut registry, deeper);
        let last = lent.add(Record::new(RecordTag::CodeCast));
        Records::give_back(&mut registry, lent);
        let ids = [before, outer, nested, deepest, after_nested, last];
        assert_eq!(ids, [1, 2, 3, 4, 5, 6]);
        for (id, tag) in ids.into_iter().zip([
            RecordTag::CodeIf,
            RecordTag::CodeWhile,
            RecordTag::CodeFor,
            RecordTag::CodeBlock,
            RecordTag::CodeCase,
            RecordTag::CodeCast,
        ]) {
            assert_eq!(registry.get(id).map(|r| r.tag), Some(tag), "record {id}");
        }
        assert!(registry.get(0).is_none() && registry.get(7).is_none());
    }

    /// Struct names defined in the Jai sources the tags draw from, nested ones included.
    fn stdlib_struct_names(files: &[&str]) -> std::collections::BTreeSet<String> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
        let mut names = std::collections::BTreeSet::new();
        for file in files {
            let text = std::fs::read_to_string(root.join(file)).unwrap();
            for line in text.lines() {
                if let Some((name, rest)) = line.trim_start().split_once("::")
                    && rest.trim_start().starts_with("struct")
                {
                    names.insert(name.trim().to_string());
                }
            }
        }
        names
    }

    const TAG_SOURCES: &[&str] = &[
        "Compiler/contract-records.jai",
        "prelude/reflection.jai",
        "prelude/diagnostics.jai",
    ];

    #[test]
    fn every_tag_names_a_stdlib_struct() {
        let structs = stdlib_struct_names(TAG_SOURCES);
        for tag in RecordTag::ALL {
            if *tag == RecordTag::CodeNodes {
                continue;
            } // Internal multi-result envelope.
            assert!(
                structs.contains(tag.name()),
                "{tag} is not a struct in the stdlib"
            );
        }
    }

    /// The syntax-node structs the compiler has no export for yet.
    const UNEXPORTED_NODES: &[&str] = &[
        "Code_Directive_Context_Type",
        "Code_Directive_Library",
        "Code_Directive_Poke_Name",
        "Code_Directive_Wildcard",
        "Code_Expression_Query",
        "Code_Extract",
        "Code_Make_Varargs",
        "Code_Placeholder",
        "Code_Resolved_Overload",
        "Code_Scope_Entry",
        "Code_Type_Definition",
        "Code_Type_Query",
    ];

    #[test]
    fn every_stdlib_syntax_node_has_a_tag_or_is_listed_unexported() {
        let tags: std::collections::BTreeSet<_> = RecordTag::ALL.iter().map(|t| t.name()).collect();
        for name in stdlib_struct_names(&["Compiler/contract-records.jai"]) {
            if name.starts_with("Code_") {
                assert!(
                    tags.contains(name.as_str()) ^ UNEXPORTED_NODES.contains(&name.as_str()),
                    "{name} must be exactly one of a RecordTag and an unexported node"
                );
            }
        }
    }
}
