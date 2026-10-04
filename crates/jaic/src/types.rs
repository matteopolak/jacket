//! Type table. Structural types are interned; structs, enums and distinct
//! types are nominal and carry their own info records, filled in lazily by sema.
use crate::ast::AstId;
use crate::intern::Sym;
use crate::source::Span;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct TypeId(pub u32);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ArrayKind {
    Fixed(u64),
    View,
    Resizable,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProcType {
    pub params: Vec<TypeId>,
    pub returns: Vec<TypeId>,
    /// Last parameter is a Jai variadic (`..T`, received as `[] T`).
    pub variadic: bool,
    /// C-style varargs (`..` with `#c_call`/`#foreign`).
    pub c_varargs: bool,
    pub c_call: bool,
    pub no_context: bool,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TypeKind {
    Void,
    Bool,
    Int {
        bits: u8,
        signed: bool,
    },
    Float {
        bits: u8,
    },
    String,
    /// The type of types (`Type`).
    Type,
    Any,
    Code,
    Pointer(TypeId),
    Array {
        elem: TypeId,
        kind: ArrayKind,
    },
    Proc(Rc<ProcType>),
    Struct(StructId),
    Enum(EnumId),
    Distinct(DistinctId),
    /// The type of `null` before it meets a pointer/proc type.
    Null,
    /// Overload sets, modules and other compile-time-only entities.
    CompileTimeOnly,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct StructId(pub u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct EnumId(pub u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct DistinctId(pub u32);

#[derive(Clone, Debug)]
pub struct Field {
    pub name: Option<Sym>,
    pub ty: TypeId,
    pub offset: u64,
    pub using: bool,
    pub as_: bool,
    pub notes: Vec<Rc<str>>,
    /// Constant initializer bytes (or none = zero / ---).
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutState {
    Pending,
    InProgress,
    Done,
}

#[derive(Clone, Debug)]
pub struct StructInfo {
    pub name: Sym,
    pub ast: Option<AstId>,
    pub is_union: bool,
    pub fields: Vec<Field>,
    pub size: u64,
    pub align: u64,
    pub layout: LayoutState,
    /// Polymorphic struct this was instantiated from, with its argument values (rendered).
    pub poly_parent: Option<StructId>,
    pub poly_args: Vec<crate::sema::value::Value>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct EnumInfo {
    pub name: Sym,
    pub base: TypeId,
    pub members: Vec<(Sym, i128)>,
    pub is_flags: bool,
    /// For `E.loose`: the strict enum type it mirrors. Loose enums convert implicitly to and
    /// from integers.
    pub loose_of: Option<TypeId>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct DistinctInfo {
    pub name: Sym,
    pub base: TypeId,
    /// `#type,isa` converts implicitly to its base.
    pub isa: bool,
}

pub struct Types {
    kinds: Vec<TypeKind>,
    intern: HashMap<TypeKind, TypeId>,
    pub structs: Vec<StructInfo>,
    pub enums: Vec<EnumInfo>,
    pub distincts: Vec<DistinctInfo>,
    /// Pointer size in bytes (8 for every supported target layout).
    pub pointer_size: u64,
}

macro_rules! builtin {
    ($($name:ident = $idx:expr => $kind:expr),* $(,)?) => {
        impl TypeId { $(pub const $name: TypeId = TypeId($idx);)* }
        fn builtin_kinds() -> Vec<TypeKind> { vec![$($kind),*] }
    };
}
builtin! {
    VOID = 0 => TypeKind::Void,
    BOOL = 1 => TypeKind::Bool,
    S8 = 2 => TypeKind::Int { bits: 8, signed: true },
    S16 = 3 => TypeKind::Int { bits: 16, signed: true },
    S32 = 4 => TypeKind::Int { bits: 32, signed: true },
    S64 = 5 => TypeKind::Int { bits: 64, signed: true },
    U8 = 6 => TypeKind::Int { bits: 8, signed: false },
    U16 = 7 => TypeKind::Int { bits: 16, signed: false },
    U32 = 8 => TypeKind::Int { bits: 32, signed: false },
    U64 = 9 => TypeKind::Int { bits: 64, signed: false },
    F32 = 10 => TypeKind::Float { bits: 32 },
    F64 = 11 => TypeKind::Float { bits: 64 },
    STRING = 12 => TypeKind::String,
    TYPE = 13 => TypeKind::Type,
    ANY = 14 => TypeKind::Any,
    CODE = 15 => TypeKind::Code,
    NULL = 16 => TypeKind::Null,
    COMPILE_TIME = 17 => TypeKind::CompileTimeOnly,
    VOID_PTR = 18 => TypeKind::Pointer(TypeId(0)),
    U8_PTR = 19 => TypeKind::Pointer(TypeId(6)),
}

impl Default for Types {
    fn default() -> Self {
        Self::new()
    }
}

impl Types {
    pub fn new() -> Self {
        let kinds = builtin_kinds();
        let intern = kinds
            .iter()
            .enumerate()
            .map(|(i, k)| (k.clone(), TypeId(i as u32)))
            .collect();
        Self {
            kinds,
            intern,
            structs: Vec::new(),
            enums: Vec::new(),
            distincts: Vec::new(),
            pointer_size: 8,
        }
    }
    pub fn intern(&mut self, kind: TypeKind) -> TypeId {
        if let Some(&id) = self.intern.get(&kind) {
            return id;
        }
        let id = TypeId(self.kinds.len() as u32);
        self.kinds.push(kind.clone());
        self.intern.insert(kind, id);
        id
    }
    pub fn kind(&self, ty: TypeId) -> &TypeKind {
        &self.kinds[ty.0 as usize]
    }
    pub fn pointer(&mut self, to: TypeId) -> TypeId {
        self.intern(TypeKind::Pointer(to))
    }
    pub fn array(&mut self, elem: TypeId, kind: ArrayKind) -> TypeId {
        self.intern(TypeKind::Array {
            elem,
            kind,
        })
    }
    pub fn int(bits: u8, signed: bool) -> TypeId {
        match (bits, signed) {
            (8, true) => TypeId::S8,
            (16, true) => TypeId::S16,
            (32, true) => TypeId::S32,
            (64, true) => TypeId::S64,
            (8, false) => TypeId::U8,
            (16, false) => TypeId::U16,
            (32, false) => TypeId::U32,
            _ => TypeId::U64,
        }
    }
    pub fn new_struct(&mut self, info: StructInfo) -> TypeId {
        let id = StructId(self.structs.len() as u32);
        self.structs.push(info);
        self.intern(TypeKind::Struct(id))
    }
    pub fn new_enum(&mut self, info: EnumInfo) -> TypeId {
        let id = EnumId(self.enums.len() as u32);
        self.enums.push(info);
        self.intern(TypeKind::Enum(id))
    }
    /// The `E.loose` variant of enum type `ty` (created on first use).
    pub fn loose_enum(&mut self, ty: TypeId) -> TypeId {
        let TypeKind::Enum(e) = *self.kind(ty) else {
            return ty;
        };
        if self.enums[e.0 as usize].loose_of.is_some() {
            return ty;
        }
        if let Some(i) = self.enums.iter().position(|x| x.loose_of == Some(ty)) {
            return self.intern(TypeKind::Enum(EnumId(i as u32)));
        }
        let mut info = self.enums[e.0 as usize].clone();
        info.name = Sym::intern(&format!("{}.loose", info.name));
        info.loose_of = Some(ty);
        self.new_enum(info)
    }
    pub fn is_loose_enum(&self, ty: TypeId) -> bool {
        matches!(self.kind(ty), TypeKind::Enum(e) if self.enums[e.0 as usize].loose_of.is_some())
    }
    pub fn new_distinct(&mut self, info: DistinctInfo) -> TypeId {
        let id = DistinctId(self.distincts.len() as u32);
        self.distincts.push(info);
        self.intern(TypeKind::Distinct(id))
    }
    pub fn struct_info(&self, id: StructId) -> &StructInfo {
        &self.structs[id.0 as usize]
    }
    pub fn struct_info_mut(&mut self, id: StructId) -> &mut StructInfo {
        &mut self.structs[id.0 as usize]
    }
    pub fn enum_info(&self, id: EnumId) -> &EnumInfo {
        &self.enums[id.0 as usize]
    }
    pub fn as_struct(&self, ty: TypeId) -> Option<StructId> {
        match self.kind(ty) {
            TypeKind::Struct(s) => Some(*s),
            _ => None,
        }
    }

    /// Strip distinct wrappers and enums to the underlying representation type.
    pub fn repr(&self, ty: TypeId) -> TypeId {
        match self.kind(ty) {
            TypeKind::Enum(e) => self.enums[e.0 as usize].base,
            TypeKind::Distinct(d) => self.repr(self.distincts[d.0 as usize].base),
            _ => ty,
        }
    }
    /// Strip distinct wrappers only (enums stay nominal).
    pub fn repr_struct(&self, ty: TypeId) -> TypeId {
        match self.kind(ty) {
            TypeKind::Distinct(d) => self.repr_struct(self.distincts[d.0 as usize].base),
            _ => ty,
        }
    }
    pub fn is_integer(&self, ty: TypeId) -> bool {
        matches!(self.kind(self.repr(ty)), TypeKind::Int { .. })
    }
    pub fn is_float(&self, ty: TypeId) -> bool {
        matches!(self.kind(self.repr(ty)), TypeKind::Float { .. })
    }
    pub fn is_pointer(&self, ty: TypeId) -> bool {
        matches!(self.kind(self.repr(ty)), TypeKind::Pointer(_))
    }
    pub fn int_info(&self, ty: TypeId) -> Option<(u8, bool)> {
        match self.kind(self.repr(ty)) {
            TypeKind::Int {
                bits,
                signed,
            } => Some((*bits, *signed)),
            _ => None,
        }
    }
    pub fn pointee(&self, ty: TypeId) -> Option<TypeId> {
        match self.kind(self.repr(ty)) {
            TypeKind::Pointer(t) => Some(*t),
            _ => None,
        }
    }

    /// Size in bytes. Structs must already be laid out.
    pub fn size_of(&self, ty: TypeId) -> u64 {
        let p = self.pointer_size;
        match self.kind(ty) {
            TypeKind::Void | TypeKind::CompileTimeOnly => 0,
            TypeKind::Bool => 1,
            TypeKind::Int {
                bits, ..
            }
            | TypeKind::Float {
                bits,
            } => *bits as u64 / 8,
            TypeKind::String => 16,
            TypeKind::Type => 8,
            TypeKind::Any => 16,
            TypeKind::Code => 8,
            TypeKind::Pointer(_) | TypeKind::Proc(_) | TypeKind::Null => p,
            TypeKind::Array {
                elem,
                kind,
            } => match kind {
                ArrayKind::Fixed(n) => self.size_of(*elem) * n,
                ArrayKind::View => 16,
                ArrayKind::Resizable => 40,
            },
            TypeKind::Struct(s) => {
                debug_assert!(
                    self.structs[s.0 as usize].layout == LayoutState::Done,
                    "struct not laid out"
                );
                self.structs[s.0 as usize].size
            }
            TypeKind::Enum(e) => self.size_of(self.enums[e.0 as usize].base),
            TypeKind::Distinct(d) => self.size_of(self.distincts[d.0 as usize].base),
        }
    }
    pub fn align_of(&self, ty: TypeId) -> u64 {
        match self.kind(ty) {
            TypeKind::Void | TypeKind::CompileTimeOnly => 1,
            TypeKind::Bool => 1,
            TypeKind::Int {
                bits, ..
            }
            | TypeKind::Float {
                bits,
            } => *bits as u64 / 8,
            TypeKind::String | TypeKind::Any => 8,
            TypeKind::Type | TypeKind::Code => 8,
            TypeKind::Pointer(_) | TypeKind::Proc(_) | TypeKind::Null => self.pointer_size,
            TypeKind::Array {
                elem,
                kind,
            } => match kind {
                ArrayKind::Fixed(_) => self.align_of(*elem),
                _ => 8,
            },
            TypeKind::Struct(s) => self.structs[s.0 as usize].align.max(1),
            TypeKind::Enum(e) => self.align_of(self.enums[e.0 as usize].base),
            TypeKind::Distinct(d) => self.align_of(self.distincts[d.0 as usize].base),
        }
    }

    /// Human-readable spelling, as Jai prints types.
    pub fn name(&self, ty: TypeId) -> String {
        match self.kind(ty) {
            TypeKind::Void => "void".into(),
            TypeKind::Bool => "bool".into(),
            TypeKind::Int {
                bits,
                signed,
            } => format!(
                "{}{}",
                if *signed {
                    "s"
                } else {
                    "u"
                },
                bits
            ),
            TypeKind::Float {
                bits,
            } => format!("float{bits}"),
            TypeKind::String => "string".into(),
            TypeKind::Type => "Type".into(),
            TypeKind::Any => "Any".into(),
            TypeKind::Code => "Code".into(),
            TypeKind::Null => "null".into(),
            TypeKind::CompileTimeOnly => "(compile-time entity)".into(),
            TypeKind::Pointer(t) => format!("*{}", self.name(*t)),
            TypeKind::Array {
                elem,
                kind,
            } => match kind {
                ArrayKind::Fixed(n) => format!("[{n}] {}", self.name(*elem)),
                ArrayKind::View => format!("[] {}", self.name(*elem)),
                ArrayKind::Resizable => format!("[..] {}", self.name(*elem)),
            },
            TypeKind::Proc(p) => {
                let params: Vec<String> = p.params.iter().map(|t| self.name(*t)).collect();
                let mut s = format!("({})", params.join(", "));
                if !p.returns.is_empty() {
                    let r: Vec<String> = p.returns.iter().map(|t| self.name(*t)).collect();
                    s.push_str(" -> ");
                    s.push_str(&r.join(", "));
                }
                if p.c_call {
                    s.push_str(" #c_call");
                }
                s
            }
            TypeKind::Struct(s) => {
                let info = &self.structs[s.0 as usize];
                if info.poly_args.is_empty() {
                    info.name.to_string()
                } else {
                    let args: Vec<String> = info.poly_args.iter().map(|v| v.render(self)).collect();
                    format!("{}({})", info.name, args.join(", "))
                }
            }
            TypeKind::Enum(e) => self.enums[e.0 as usize].name.to_string(),
            TypeKind::Distinct(d) => self.distincts[d.0 as usize].name.to_string(),
        }
    }
    pub fn count(&self) -> usize {
        self.kinds.len()
    }
}
