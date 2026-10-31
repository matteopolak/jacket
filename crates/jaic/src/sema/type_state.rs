//! Types and what sema keeps per type: struct declarations and layout inputs, `Type_Info`
//! globals, default values and initializers.

use super::body_probe::{Probed, W};
use std::sync::Arc;

use crate::ast;
use crate::fxhash::{HashMap, HashSet};
use crate::intern::Sym;
use crate::ir;
use crate::types::{StructId, TypeId, Types};

use super::{EntityId, ScopeId, structs, value};

pub struct TypeState {
    pub types: Probed<Types, { W::TyTypes as usize }>,
    /// Source and lexical context of nominal enums and distinct types.
    pub nominal_origins: Probed<
        HashMap<TypeId, (crate::source::Span, super::ScopeId)>,
        { W::TyNominalOrigins as usize },
    >,
    pub poly_structs: Probed<Vec<structs::PolyStruct>, { W::TyPolyStructs as usize }>,
    /// Struct declaration AST per struct (for layout).
    pub struct_asts: Probed<HashMap<StructId, structs::StructSource>, { W::TyStructAsts as usize }>,
    /// Field type queries do not require storage layout or declaration reordering.
    pub field_types: Probed<HashMap<ScopeId, HashMap<Sym, TypeId>>, { W::TyFieldTypes as usize }>,
    pub(in crate::sema) field_declarations:
        Probed<HashMap<ScopeId, HashMap<Sym, Arc<ast::Decl>>>, { W::TyFieldDeclarations as usize }>,
    pub(in crate::sema) field_type_pending:
        Probed<HashSet<(ScopeId, ast::AstId)>, { W::TyFieldTypePending as usize }>,
    pub(in crate::sema) field_filter_cache: Probed<
        HashMap<(ScopeId, ast::AstId), ast::UsingFilter>,
        { W::TyFieldFilterCache as usize },
    >,
    pub(in crate::sema) field_filter_pending:
        Probed<HashSet<(ScopeId, ast::AstId)>, { W::TyFieldFilterPending as usize }>,
    pub(in crate::sema) field_using_order:
        Probed<HashMap<ScopeId, Vec<structs::FieldUsing>>, { W::TyFieldUsingOrder as usize }>,
    pub context_type: Option<TypeId>,
    /// `Type_Info` globals per type.
    pub infos: Probed<HashMap<TypeId, ir::GlobalId>, { W::TyInfos as usize }>,
    /// `Type_Info` globals whose descriptor could not be built yet (`type_info_global`).
    pub(in crate::sema) failed_infos:
        Probed<HashMap<TypeId, ir::GlobalId>, { W::TyFailedInfos as usize }>,
    /// While `defer_infos` is set, a descriptor that cannot be built yet keeps its global and
    /// is queued in `deferred_infos`. `strict_sizes` makes a descriptor fail rather than
    /// record an unknown size.
    pub(in crate::sema) defer_infos: bool,
    pub(in crate::sema) strict_sizes: bool,
    pub(in crate::sema) deferred_infos: Probed<Vec<TypeId>, { W::TyDeferredInfos as usize }>,
    /// The types in `types: ..Type` of each instance of a polymorphic struct.
    pub(in crate::sema) variadic_cases:
        Probed<HashMap<TypeId, Vec<TypeId>>, { W::TyVariadicCases as usize }>,
    pub(in crate::sema) cases_building: Probed<Vec<TypeId>, { W::TyCasesBuilding as usize }>,
    /// Descriptors being built right now: they are empty until the build ends.
    pub(in crate::sema) building_infos:
        Probed<Vec<(TypeId, ir::GlobalId)>, { W::TyBuildingInfos as usize }>,
    /// Descriptors only a compile-time `get_type_table()` made (kept out of the runtime table
    /// until real code asks for them), the table's state when it was last built, and the
    /// public type-table primitives once the program references them.
    pub(in crate::sema) snapshot_infos: Probed<HashSet<TypeId>, { W::TySnapshotInfos as usize }>,
    pub(in crate::sema) in_snapshot: bool,
    pub(in crate::sema) snapshot_key: (usize, usize, usize),
    pub(in crate::sema) table_funcs: Probed<HashSet<ir::FuncId>, { W::TyTableFuncs as usize }>,
    /// `compiler_set_type_info_flags` bits per struct (`Type_Info_Flags`).
    pub info_flags: Probed<HashMap<TypeId, u32>, { W::TyInfoFlags as usize }>,
    pub anonymous: Probed<HashMap<(ast::AstId, ScopeId), TypeId>, { W::TyAnonymous as usize }>,
    /// The shapes `#type (value: *$T)` aliases name: patterns that match types, not values.
    pub(in crate::sema) poly_aliases: Probed<HashSet<TypeId>, { W::TyPolyAliases as usize }>,
    /// Set while matching an array or slice element pattern, where `$T/Struct` does not
    /// require the argument to derive from the struct.
    pub(in crate::sema) in_element_pattern: bool,
    pub(in crate::sema) enum_member_scopes:
        Probed<HashMap<TypeId, ScopeId>, { W::TyEnumMemberScopes as usize }>,
    /// Where an enum writes its backing type (`enum u8 { ... }`), for the fix that widens it.
    pub(in crate::sema) enum_base_spans:
        Probed<HashMap<TypeId, crate::source::Span>, { W::TyEnumBaseSpans as usize }>,
    /// Names an enum reaches through `using Other;` in its body, with their values. They are
    /// not members of the enum.
    pub(crate) enum_borrowed:
        Probed<HashMap<TypeId, Vec<(Sym, EntityId)>>, { W::TyEnumBorrowed as usize }>,
    /// The entity of each declared member of an enum, in declaration order. A name another
    /// enum borrows with `using` refers to one of these.
    pub(crate) enum_member_entities: Probed<
        HashMap<crate::types::EnumId, Vec<(Sym, EntityId)>>,
        { W::TyEnumMemberEntities as usize },
    >,
    /// `using,only(...) field.path;` aliases declared in struct bodies.
    pub member_aliases:
        Probed<HashMap<StructId, Vec<structs::MemberAlias>>, { W::TyMemberAliases as usize }>,
    /// `using,only(..)`/`using,except(..)` on struct members: field index and filter.
    pub(in crate::sema) using_filters:
        Probed<HashMap<StructId, structs::UsingFilters>, { W::TyUsingFilters as usize }>,
    pub default_images:
        Probed<HashMap<TypeId, Option<Arc<value::Aggregate>>>, { W::TyDefaultImages as usize }>,
    pub default_globals: Probed<HashMap<TypeId, ir::GlobalId>, { W::TyDefaultGlobals as usize }>,
    pub initializers: Probed<HashMap<TypeId, ir::FuncId>, { W::TyInitializers as usize }>,
    /// The stand-in `Type_Info_Struct` of each polymorphic struct, by its literal.
    pub generic_struct_infos:
        Probed<HashMap<ast::AstId, ir::GlobalId>, { W::TyGenericStructInfos as usize }>,
}

impl Default for TypeState {
    fn default() -> Self {
        Self {
            types: Probed::new(Types::new()),
            nominal_origins: Probed::default(),
            poly_structs: Probed::default(),
            struct_asts: Probed::default(),
            field_types: Probed::default(),
            field_declarations: Probed::default(),
            field_type_pending: Probed::default(),
            field_filter_cache: Probed::default(),
            field_filter_pending: Probed::default(),
            field_using_order: Probed::default(),
            context_type: None,
            infos: Probed::default(),
            failed_infos: Probed::default(),
            defer_infos: false,
            strict_sizes: false,
            deferred_infos: Probed::default(),
            variadic_cases: Probed::default(),
            cases_building: Probed::default(),
            building_infos: Probed::default(),
            snapshot_infos: Probed::default(),
            in_snapshot: false,
            snapshot_key: (usize::MAX, 0, 0),
            table_funcs: Probed::default(),
            info_flags: Probed::default(),
            anonymous: Probed::default(),
            poly_aliases: Probed::default(),
            in_element_pattern: false,
            enum_member_scopes: Probed::default(),
            enum_base_spans: Probed::default(),
            enum_borrowed: Probed::default(),
            enum_member_entities: Probed::default(),
            member_aliases: Probed::default(),
            using_filters: Probed::default(),
            default_images: Probed::default(),
            default_globals: Probed::default(),
            initializers: Probed::default(),
            generic_struct_infos: Probed::default(),
        }
    }
}
