//! Builtin declarations and overloadability.
use super::expand::{decl_is_proc, is_alias_decl};
use super::model::{Builtin, BuiltinProc, EntityId, EntityKind, Resolved};
use crate::sema::{Compiler, Span, Sym, Value};
use crate::types::TypeId;

impl Compiler {
    pub(in crate::sema) fn declare_builtins(&mut self) {
        let root = self.ns.root_scope;
        let register = self.ty.types.new_distinct(crate::types::DistinctInfo {
            name: Sym::intern("__reg"),
            base: TypeId::U16,
            isa: false,
        });
        let types: &[(&str, TypeId)] = &[
            ("void", TypeId::VOID),
            ("bool", TypeId::BOOL),
            ("s8", TypeId::S8),
            ("s16", TypeId::S16),
            ("s32", TypeId::S32),
            ("s64", TypeId::S64),
            ("int", TypeId::S64),
            ("u8", TypeId::U8),
            ("u16", TypeId::U16),
            ("u32", TypeId::U32),
            ("u64", TypeId::U64),
            ("float", TypeId::F32),
            ("float32", TypeId::F32),
            ("float64", TypeId::F64),
            ("string", TypeId::STRING),
            ("Type", TypeId::TYPE),
            ("Any", TypeId::ANY),
            ("Code", TypeId::CODE),
            // Source-level identity; operand binding is handled independently of Code.
            ("__reg", register),
        ];
        for &(name, ty) in types {
            self.add_const(
                root,
                Sym::intern(name),
                Span::NONE,
                Value::Type(ty),
                TypeId::TYPE,
            );
        }
        let procs: &[(&str, BuiltinProc)] = &[
            ("size_of", BuiltinProc::SizeOf),
            ("type_of", BuiltinProc::TypeOf),
            ("code_of", BuiltinProc::CodeOf),
            ("type_info", BuiltinProc::TypeInfo),
            ("initializer_of", BuiltinProc::InitializerOf),
            ("is_constant", BuiltinProc::IsConstant),
            ("align_of", BuiltinProc::AlignOf),
            ("offset_of", BuiltinProc::OffsetOf),
            ("is_value_type", BuiltinProc::IsValueType),
        ];
        for &(name, p) in procs {
            let id = self.add_entity(
                root,
                Sym::intern(name),
                Span::NONE,
                EntityKind::Builtin(Builtin::Proc(p)),
                true,
            );
            let _ = id;
        }
        for name in [
            "OS",
            "CPU",
            "BUILD_OS",
            "BUILD_CPU",
            "IS_CROSS_COMPILING",
            "MACHINE_OPTIONS_SIZE",
            "TEMPORARY_STORAGE_SIZE",
            "LANGUAGE_VERSION_MAJOR",
            "LANGUAGE_VERSION_MINOR",
        ] {
            self.add_entity(
                root,
                Sym::intern(name),
                Span::NONE,
                EntityKind::Builtin(Builtin::TargetConstant(Sym::intern(name))),
                true,
            );
        }
    }

    pub fn entity_is_overloadable(&self, id: EntityId) -> bool {
        match &self.entity(id).kind {
            EntityKind::Decl {
                decl, ..
            } => decl_is_proc(decl),
            EntityKind::Const {
                value: Value::Proc(_),
                ..
            } => true,
            _ => false,
        }
    }

    /// Like `entity_is_overloadable`, but `name :: other_proc;` aliases count too
    /// (resolved on demand; an alias that cannot be resolved yet shadows).
    pub(super) fn overloadable_or_alias(&mut self, id: EntityId) -> bool {
        if self.entity_is_overloadable(id) {
            return true;
        }
        let EntityKind::Decl {
            decl, ..
        } = &self.entity(id).kind
        else {
            return false;
        };
        is_alias_decl(decl)
            && matches!(
                self.resolve_entity(id),
                Ok(Resolved::Proc(_)
                    | Resolved::ProcSet(_)
                    | Resolved::Const {
                        value: Value::Proc(_),
                        ..
                    })
            )
    }
}
