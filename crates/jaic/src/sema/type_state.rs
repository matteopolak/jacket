//! Types and what sema keeps per type: struct declarations and layout inputs, `Type_Info`
//! globals, default values and initializers.

use std::rc::Rc;

use crate::ast;
use crate::fxhash::{HashMap, HashSet};
use crate::intern::Sym;
use crate::ir;
use crate::types::{StructId, TypeId, Types};

use super::{ScopeId, structs, value};

pub struct TypeState {
    pub types: Types,
    /// Source and lexical context of nominal enums and distinct types.
    pub nominal_origins: HashMap<TypeId, (crate::source::Span, super::ScopeId)>,
    pub poly_structs: Vec<structs::PolyStruct>,
    /// Struct declaration AST per struct (for layout).
    pub struct_asts: HashMap<StructId, structs::StructSource>,
    /// Field type queries do not require storage layout or declaration reordering.
    pub field_types: HashMap<ScopeId, HashMap<Sym, TypeId>>,
    pub(in crate::sema) field_declarations: HashMap<ScopeId, HashMap<Sym, Rc<ast::Decl>>>,
    pub(in crate::sema) field_type_pending: HashSet<(ScopeId, ast::AstId)>,
    pub(in crate::sema) field_filter_cache: HashMap<(ScopeId, ast::AstId), ast::UsingFilter>,
    pub(in crate::sema) field_filter_pending: HashSet<(ScopeId, ast::AstId)>,
    pub(in crate::sema) field_using_order: HashMap<ScopeId, Vec<structs::FieldUsing>>,
    pub context_type: Option<TypeId>,
    /// `Type_Info` globals per type.
    pub infos: HashMap<TypeId, ir::GlobalId>,
    /// `Type_Info` globals whose descriptor could not be built yet (`type_info_global`).
    pub(in crate::sema) failed_infos: HashMap<TypeId, ir::GlobalId>,
    /// Descriptors only a compile-time `get_type_table()` made (kept out of the runtime table
    /// until real code asks for them), the table's state when it was last built, and the
    /// public type-table primitives once the program references them.
    pub(in crate::sema) snapshot_infos: HashSet<TypeId>,
    pub(in crate::sema) in_snapshot: bool,
    pub(in crate::sema) snapshot_key: (usize, usize, usize),
    pub(in crate::sema) table_funcs: HashSet<ir::FuncId>,
    /// `compiler_set_type_info_flags` bits per struct (`Type_Info_Flags`).
    pub info_flags: HashMap<TypeId, u32>,
    pub anonymous: HashMap<(ast::AstId, ScopeId), TypeId>,
    /// The shapes `#type (value: *$T)` aliases name: patterns that match types, not values.
    pub(in crate::sema) poly_aliases: HashSet<TypeId>,
    pub(in crate::sema) enum_member_scopes: HashMap<TypeId, ScopeId>,
    /// Where an enum writes its backing type (`enum u8 { ... }`), for the fix that widens it.
    pub(in crate::sema) enum_base_spans: HashMap<TypeId, crate::source::Span>,
    /// Names an enum reaches through `using Other;` in its body, with their values. They are
    /// not members of the enum.
    pub(crate) enum_borrowed: HashMap<TypeId, Vec<(Sym, i128)>>,
    /// `using,only(...) field.path;` aliases declared in struct bodies.
    pub member_aliases: HashMap<StructId, Vec<structs::MemberAlias>>,
    /// `using,only(..)`/`using,except(..)` on struct members: field index and filter.
    pub(in crate::sema) using_filters: HashMap<StructId, structs::UsingFilters>,
    pub default_images: HashMap<TypeId, Option<Rc<value::Aggregate>>>,
    pub default_globals: HashMap<TypeId, ir::GlobalId>,
    pub initializers: HashMap<TypeId, ir::FuncId>,
    /// The stand-in `Type_Info_Struct` of each polymorphic struct, by its literal.
    pub generic_struct_infos: HashMap<ast::AstId, ir::GlobalId>,
}

impl Default for TypeState {
    fn default() -> Self {
        Self {
            types: Types::new(),
            nominal_origins: HashMap::default(),
            poly_structs: Vec::new(),
            struct_asts: HashMap::default(),
            field_types: HashMap::default(),
            field_declarations: HashMap::default(),
            field_type_pending: HashSet::default(),
            field_filter_cache: HashMap::default(),
            field_filter_pending: HashSet::default(),
            field_using_order: HashMap::default(),
            context_type: None,
            infos: HashMap::default(),
            failed_infos: HashMap::default(),
            snapshot_infos: HashSet::default(),
            in_snapshot: false,
            snapshot_key: (usize::MAX, 0, 0),
            table_funcs: HashSet::default(),
            info_flags: HashMap::default(),
            anonymous: HashMap::default(),
            poly_aliases: HashSet::default(),
            enum_member_scopes: HashMap::default(),
            enum_base_spans: HashMap::default(),
            enum_borrowed: HashMap::default(),
            member_aliases: HashMap::default(),
            using_filters: HashMap::default(),
            default_images: HashMap::default(),
            default_globals: HashMap::default(),
            initializers: HashMap::default(),
            generic_struct_infos: HashMap::default(),
        }
    }
}
