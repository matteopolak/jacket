//! Type table. Structural types are interned; structs, enums and distinct
//! types are nominal and carry their own info records, filled in lazily by sema.
use std::sync::Arc;

use crate::ast::AstId;
use crate::fxhash::HashMap;
use crate::intern::Sym;
use crate::layout::{BuiltinLayouts, DataLayout};
use crate::source::Span;

/// Identifies an interned type in `Types`; equal types have equal ids.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct TypeId(pub u32);

/// How an array's length is held.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ArrayKind {
    /// Element count.
    Fixed(u64),
    View,
    Resizable,
}

/// The signature of a procedure type, as far as type identity goes.
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
    /// `#cpp_return_type_is_non_pod`: a struct result is returned through a hidden
    /// result pointer even when the C ABI would return it in registers.
    pub non_pod_return: bool,
    /// `#cpp_method`: a C++ member function, whose first parameter is the object. The
    /// Microsoft C++ ABI returns its struct results through a pointer passed after it.
    pub cpp_method: bool,
}

/// The shape of a type, which `Types` interns to a `TypeId`.
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
    /// C `long double` where it is wider than `float64` (a jaic extension, reached only through
    /// `#jaic_type long_double`, see `docs/language/long-double.md`). Its values live in
    /// 16 bytes of memory, like a small struct.
    WideFloat(crate::wide_float::WideFloat),
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
    Proc(Arc<ProcType>),
    Struct(StructId),
    Enum(EnumId),
    Distinct(DistinctId),
    /// The type of `null` before it meets a pointer/proc type.
    Null,
    /// Overload sets, modules and other compile-time-only entities.
    CompileTimeOnly,
    /// A polymorphic parameter or result inside the type of a polymorphic procedure
    /// (`type_of(proc)`); it prints as `$`.
    PolyParam,
    /// A polymorphic struct before it is given arguments (`type_of(Table)` is `Type`, and
    /// the template itself is a `Type` value naming it).
    PolyStruct {
        id: crate::sema::value::PolyStructId,
        name: Sym,
    },
}

/// Index into `Types::structs`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct StructId(pub u32);

/// Index into `Types::enums`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct EnumId(pub u32);

/// Index into `Types::distincts`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct DistinctId(pub u32);

/// A member of a struct, laid out.
#[derive(Clone, Debug)]
pub struct Field {
    pub name: Option<Sym>,
    pub ty: TypeId,
    /// Byte offset in the struct.
    pub offset: u64,
    /// Alignment the layout used: the type's, or the member's `#align`.
    pub align: u64,
    pub using: bool,
    pub as_: bool,
    /// Declared after `#overlay(f)`: shares another field's storage.
    pub overlay: bool,
    /// The member's notes with where each is written (a note's place lets an editor compile
    /// watch which compile-time code reads it through type info).
    pub notes: Vec<crate::ast::Note>,
    /// Where the member is declared.
    pub span: Span,
}

/// How far laying out a struct has got.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutState {
    Pending,
    InProgress,
    Done,
}

/// A struct or union, its members and its layout.
#[derive(Clone, Debug)]
pub struct StructInfo {
    pub name: Sym,
    pub ast: Option<AstId>,
    pub is_union: bool,
    /// Shared, so a caller can hold the fields while it changes other types.
    pub fields: std::sync::Arc<[Field]>,
    pub size: u64,
    pub align: u64,
    pub layout: LayoutState,
    /// Polymorphic struct this was instantiated from, with its argument values (rendered).
    pub poly_parent: Option<StructId>,
    pub poly_args: Vec<crate::sema::value::Value>,
    pub span: Span,
}

/// An enum, its members and how it converts.
#[derive(Clone, Debug)]
pub struct EnumInfo {
    pub name: Sym,
    pub base: TypeId,
    pub members: Vec<(Sym, i128)>,
    pub is_flags: bool,
    /// `#specified` / `#complete` on the declaration; with `is_flags` they make up
    /// `Type_Info_Enum.enum_type_flags`.
    pub specified: bool,
    pub complete: bool,
    /// For `E.loose`: the strict enum type it mirrors. Loose enums convert implicitly to and
    /// from integers.
    pub loose_of: Option<TypeId>,
    pub span: Span,
}

impl EnumInfo {
    /// `Enum_Type_Flags` bits: FLAGS 0x1, COMPLETE 0x2, SPECIFIED 0x4.
    pub fn type_flags(&self) -> i64 {
        (self.is_flags as i64) | ((self.complete as i64) << 1) | ((self.specified as i64) << 2)
    }
}

/// A `#type,distinct` type over `base`.
#[derive(Clone, Debug)]
pub struct DistinctInfo {
    pub name: Sym,
    pub base: TypeId,
    /// `#type,isa` converts implicitly to its base.
    pub isa: bool,
}

/// The interned types of one compilation and the structs, enums and distinct types they refer to.
pub struct Types {
    kinds: Vec<TypeKind>,
    intern: HashMap<TypeKind, TypeId>,
    pub structs: Vec<StructInfo>,
    pub enums: Vec<EnumInfo>,
    pub distincts: Vec<DistinctInfo>,
    /// How the target lays out data; every size and alignment here follows it.
    layout: DataLayout,
    builtins: BuiltinLayouts,
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
    POLY_PARAM = 20 => TypeKind::PolyParam,
}

/// Largest size in bytes of any type: a 48-bit address space, which is what 64-bit hosts
/// actually map. Bigger arrays and structs are compile errors, so size arithmetic cannot overflow.
pub const MAX_SIZE: u64 = 1 << 48;

impl Types {
    /// An empty table holding only the builtin types, laid out for `layout`.
    pub fn new(layout: DataLayout) -> Self {
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
            builtins: layout.builtins(),
            layout,
        }
    }

    /// The data layout of the target the types are laid out for.
    pub fn data_layout(&self) -> &DataLayout {
        &self.layout
    }

    /// Offsets and sizes of the builtin aggregates.
    pub fn builtins(&self) -> &BuiltinLayouts {
        &self.builtins
    }

    /// The id of `kind`, adding it when no equal type is interned yet.
    pub fn intern(&mut self, kind: TypeKind) -> TypeId {
        if let Some(&id) = self.intern.get(&kind) {
            return id;
        }

        let id = TypeId(self.kinds.len() as u32);

        self.kinds.push(kind.clone());
        self.intern.insert(kind, id);
        id
    }

    /// The id of an interned type, if there is one.
    pub fn find(&self, kind: &TypeKind) -> Option<TypeId> {
        self.intern.get(kind).copied()
    }

    /// How many types are interned.
    pub fn type_count(&self) -> usize {
        self.kinds.len()
    }

    /// The shape of `ty`.
    pub fn kind(&self, ty: TypeId) -> &TypeKind {
        &self.kinds[ty.0 as usize]
    }

    /// The type of a pointer to `to`.
    pub fn pointer(&mut self, to: TypeId) -> TypeId {
        self.intern(TypeKind::Pointer(to))
    }

    /// The type of an array of `elem` of the given `kind`.
    pub fn array(&mut self, elem: TypeId, kind: ArrayKind) -> TypeId {
        self.intern(TypeKind::Array {
            elem,
            kind,
        })
    }

    /// The builtin integer type of `bits` bits; any width but 8, 16 or 32 gives the 64-bit type.
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

    /// Adds a struct and returns its type.
    pub fn new_struct(&mut self, info: StructInfo) -> TypeId {
        let id = StructId(self.structs.len() as u32);

        self.structs.push(info);
        self.intern(TypeKind::Struct(id))
    }

    /// Adds an enum and returns its type.
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

    /// Whether `ty` is an `E.loose` enum.
    pub fn is_loose_enum(&self, ty: TypeId) -> bool {
        matches!(self.kind(ty), TypeKind::Enum(e) if self.enums[e.0 as usize].loose_of.is_some())
    }

    /// Adds a distinct type and returns it.
    pub fn new_distinct(&mut self, info: DistinctInfo) -> TypeId {
        let id = DistinctId(self.distincts.len() as u32);

        self.distincts.push(info);
        self.intern(TypeKind::Distinct(id))
    }

    /// The struct `id`.
    pub fn struct_info(&self, id: StructId) -> &StructInfo {
        &self.structs[id.0 as usize]
    }

    /// The struct `id`, for modification.
    pub fn struct_info_mut(&mut self, id: StructId) -> &mut StructInfo {
        &mut self.structs[id.0 as usize]
    }

    /// The enum `id`.
    pub fn enum_info(&self, id: EnumId) -> &EnumInfo {
        &self.enums[id.0 as usize]
    }

    /// The struct `ty` is, or `None` when it is another kind of type.
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

    /// Whether `ty` is an integer, through enums and distinct wrappers.
    pub fn is_integer(&self, ty: TypeId) -> bool {
        matches!(self.kind(self.repr(ty)), TypeKind::Int { .. })
    }

    /// Whether `ty` is `f32` or `f64`, through enums and distinct wrappers.
    pub fn is_float(&self, ty: TypeId) -> bool {
        matches!(self.kind(self.repr(ty)), TypeKind::Float { .. })
    }

    /// The format of a wide `long double` type (through distinct wrappers).
    pub fn wide_float(&self, ty: TypeId) -> Option<crate::wide_float::WideFloat> {
        match self.kind(self.repr(ty)) {
            TypeKind::WideFloat(fmt) => Some(*fmt),
            _ => None,
        }
    }

    /// Whether `ty` is a pointer, through distinct wrappers.
    pub fn is_pointer(&self, ty: TypeId) -> bool {
        matches!(self.kind(self.repr(ty)), TypeKind::Pointer(_))
    }

    /// The bit width and signedness of integer type `ty` (through enums and distinct wrappers), or
    /// `None`.
    pub fn int_info(&self, ty: TypeId) -> Option<(u8, bool)> {
        match self.kind(self.repr(ty)) {
            TypeKind::Int {
                bits,
                signed,
            } => Some((*bits, *signed)),
            _ => None,
        }
    }

    /// The type a pointer `ty` points to (through distinct wrappers), or `None`.
    pub fn pointee(&self, ty: TypeId) -> Option<TypeId> {
        match self.kind(self.repr(ty)) {
            TypeKind::Pointer(t) => Some(*t),
            _ => None,
        }
    }

    /// Size in bytes. Structs must already be laid out. Saturates instead of overflowing; sema
    /// rejects types larger than `MAX_SIZE` when it lays them out.
    pub fn size_of(&self, ty: TypeId) -> u64 {
        match self.kind(ty) {
            TypeKind::Void
            | TypeKind::CompileTimeOnly
            | TypeKind::PolyParam
            | TypeKind::PolyStruct {
                ..
            } => 0,
            TypeKind::Bool => 1,
            TypeKind::Int {
                bits, ..
            }
            | TypeKind::Float {
                bits,
            } => *bits as u64 / 8,
            TypeKind::WideFloat(crate::wide_float::WideFloat::X87) => self.layout.f80_size as u64,
            TypeKind::WideFloat(crate::wide_float::WideFloat::Binary128) => 16,
            TypeKind::String => self.builtins.view.size,
            TypeKind::Type | TypeKind::Code => self.layout.ptr_bytes(),
            TypeKind::Any => self.builtins.any.size,
            TypeKind::Pointer(_) | TypeKind::Proc(_) | TypeKind::Null => self.layout.ptr_bytes(),
            TypeKind::Array {
                elem,
                kind,
            } => match kind {
                ArrayKind::Fixed(n) => self.size_of(*elem).saturating_mul(*n),
                ArrayKind::View => self.builtins.view.size,
                ArrayKind::Resizable => self.builtins.resizable.size,
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

    /// Alignment in bytes. Structs must already be laid out.
    pub fn align_of(&self, ty: TypeId) -> u64 {
        match self.kind(ty) {
            TypeKind::Void
            | TypeKind::CompileTimeOnly
            | TypeKind::PolyParam
            | TypeKind::PolyStruct {
                ..
            } => 1,
            TypeKind::Bool => 1,
            TypeKind::Int {
                bits, ..
            } => self.layout.int_align_of(*bits as u64 / 8),
            TypeKind::Float {
                bits,
            } => self.layout.float_align_of(*bits as u64 / 8),
            TypeKind::WideFloat(crate::wide_float::WideFloat::X87) => self.layout.f80_align as u64,
            TypeKind::WideFloat(crate::wide_float::WideFloat::Binary128) => 16,
            TypeKind::String => self.builtins.view.align,
            TypeKind::Any => self.builtins.any.align,
            TypeKind::Type | TypeKind::Code => self.layout.pointer_align as u64,
            TypeKind::Pointer(_) | TypeKind::Proc(_) | TypeKind::Null => {
                self.layout.pointer_align as u64
            }
            TypeKind::Array {
                elem,
                kind,
            } => match kind {
                ArrayKind::Fixed(_) => self.align_of(*elem),
                ArrayKind::View => self.builtins.view.align,
                ArrayKind::Resizable => self.builtins.resizable.align,
            },
            TypeKind::Struct(s) => self.structs[s.0 as usize].align.max(1),
            TypeKind::Enum(e) => self.align_of(self.enums[e.0 as usize].base),
            TypeKind::Distinct(d) => self.align_of(self.distincts[d.0 as usize].base),
        }
    }

    /// The spelling for diagnostics and the editor: like [`Self::full_name`], but a long string
    /// argument of a polymorphic struct (a typed query's row, say) is cut short, so the name stays
    /// on one readable line.
    pub fn name(&self, ty: TypeId) -> String {
        self.spell(ty, Spelling::Short)
    }

    /// The spelling for a label with no room for detail (an inlay hint): like [`Self::name`],
    /// but a polymorph instance whose arguments would be cut short is named by its polymorphic
    /// struct alone, so `Row("…")` reads `Row`.
    pub fn label(&self, ty: TypeId) -> String {
        self.spell(ty, Spelling::Label)
    }

    /// Human-readable spelling, as Jai prints types, with every polymorph argument in full.
    /// Anything that names a type for the program or other tools (symbols, debug info, printed
    /// output) uses this one.
    pub fn full_name(&self, ty: TypeId) -> String {
        self.spell(ty, Spelling::Full)
    }

    fn spell(&self, ty: TypeId, mode: Spelling) -> String {
        let short = mode != Spelling::Full;

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
            TypeKind::WideFloat(_) => "Long_Double".into(),
            TypeKind::String => "string".into(),
            TypeKind::Type => "Type".into(),
            TypeKind::Any => "Any".into(),
            TypeKind::Code => "Code".into(),
            TypeKind::Null => "null".into(),
            TypeKind::CompileTimeOnly => "(compile-time entity)".into(),
            TypeKind::PolyParam => "$".into(),
            TypeKind::PolyStruct {
                name, ..
            } => name.to_string(),
            TypeKind::Pointer(t) => format!("*{}", self.spell(*t, mode)),
            TypeKind::Array {
                elem,
                kind,
            } => match kind {
                ArrayKind::Fixed(n) => format!("[{n}] {}", self.spell(*elem, mode)),
                ArrayKind::View => format!("[] {}", self.spell(*elem, mode)),
                ArrayKind::Resizable => format!("[..] {}", self.spell(*elem, mode)),
            },
            TypeKind::Proc(p) => {
                let params: Vec<String> = p.params.iter().map(|t| self.spell(*t, mode)).collect();
                let mut s = format!("({})", params.join(", "));

                if !p.returns.is_empty() {
                    let r: Vec<String> = p.returns.iter().map(|t| self.spell(*t, mode)).collect();

                    s.push_str(" -> ");
                    s.push_str(&r.join(", "));
                }

                if p.cpp_method {
                    s.push_str(" #cpp_method");
                } else if p.c_call {
                    s.push_str(" #c_call");
                }

                s
            }
            TypeKind::Struct(s) => {
                let info = &self.structs[s.0 as usize];

                if info.poly_args.is_empty() {
                    info.name.to_string()
                } else {
                    let args: Vec<String> = info
                        .poly_args
                        .iter()
                        .map(|v| v.render(self, short))
                        .collect();
                    let cut = || {
                        info.poly_args
                            .iter()
                            .zip(&args)
                            .any(|(v, a)| *a != v.render(self, false))
                    };

                    if mode == Spelling::Label && cut() {
                        return info.name.to_string();
                    }

                    format!("{}({})", info.name, args.join(", "))
                }
            }
            TypeKind::Enum(e) => self.enums[e.0 as usize].name.to_string(),
            TypeKind::Distinct(d) => self.distincts[d.0 as usize].name.to_string(),
        }
    }

    /// How many types are interned; the same as `type_count`.
    pub fn count(&self) -> usize {
        self.kinds.len()
    }
}

/// How much of a type's name [`Types::spell`] writes out.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Spelling {
    /// Every polymorph argument in full.
    Full,
    /// Long string arguments cut short.
    Short,
    /// A polymorph instance with a cut argument named by its polymorphic struct alone.
    Label,
}
