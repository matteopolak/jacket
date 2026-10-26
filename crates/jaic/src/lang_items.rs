//! What the compiler expects the standard library to define.
//!
//! The prelude and a few modules are written in Jai, but the compiler reaches into them by
//! name: the types it builds descriptors from, the modules it loads itself, and the record
//! fields a metaprogram reads. Every such name is written here once, so the contract has one
//! place to read and to check. `tests/lang_items.rs` resolves each [`PreludeType`] against the
//! real prelude and checks the sizes `crate::layout` assumes; the unit test below checks that
//! the prelude's sources declare every name.

/// Modules the compiler loads or consults by name.
pub mod modules {
    /// The folder that holds the prelude's `module.jai`.
    pub const PRELUDE: &str = "prelude";

    /// What `#import "Preload"` and the prelude's own scope are called.
    pub const PRELOAD: &str = "Preload";

    /// The module that defines program entry, assertions and crash handling.
    pub const RUNTIME_SUPPORT: &str = "Runtime_Support";

    /// The `Runtime_Support` type compile-time code keeps its temporary storage in.
    pub const TEMPORARY_STORAGE: &str = "Temporary_Storage";

    /// Modules the compiler loads itself; they are never offered as imports.
    pub const INTERNAL: &[&str] = &[PRELUDE, PRELOAD, RUNTIME_SUPPORT];
}

/// Fields of the records a metaprogram reads from the `Compiler` module.
pub mod record_fields {
    /// The address of this compiler's own descriptor for a type.
    pub const ADDRESS: &str = "__address";

    /// The name of a primitive type whose descriptor the metaprogram supplies.
    pub const BUILTIN: &str = "__builtin";
}

/// The types of the bootstrap prelude that the compiler builds values of or checks against,
/// named by the prelude's own spelling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreludeType {
    Allocator,
    ForFlags,
    Reg,
    SourceCodeLocation,
    StackTraceNode,
    StackTraceProcedureInfo,
    TypeInfo,
    TypeInfoArray,
    TypeInfoEnum,
    TypeInfoFloat,
    TypeInfoInteger,
    TypeInfoPointer,
    TypeInfoProcedure,
    TypeInfoString,
    TypeInfoStruct,
    TypeInfoStructMember,
    TypeInfoTaggedUnionBinding,
    TypeInfoVariant,
}

impl PreludeType {
    pub const ALL: &'static [PreludeType] = &[
        PreludeType::Allocator,
        PreludeType::ForFlags,
        PreludeType::Reg,
        PreludeType::SourceCodeLocation,
        PreludeType::StackTraceNode,
        PreludeType::StackTraceProcedureInfo,
        PreludeType::TypeInfo,
        PreludeType::TypeInfoArray,
        PreludeType::TypeInfoEnum,
        PreludeType::TypeInfoFloat,
        PreludeType::TypeInfoInteger,
        PreludeType::TypeInfoPointer,
        PreludeType::TypeInfoProcedure,
        PreludeType::TypeInfoString,
        PreludeType::TypeInfoStruct,
        PreludeType::TypeInfoStructMember,
        PreludeType::TypeInfoTaggedUnionBinding,
        PreludeType::TypeInfoVariant,
    ];

    /// The name the prelude declares the type under.
    pub const fn name(self) -> &'static str {
        match self {
            PreludeType::Allocator => "Allocator",
            PreludeType::ForFlags => "For_Flags",
            PreludeType::Reg => "__reg",
            PreludeType::SourceCodeLocation => "Source_Code_Location",
            PreludeType::StackTraceNode => "Stack_Trace_Node",
            PreludeType::StackTraceProcedureInfo => "Stack_Trace_Procedure_Info",
            PreludeType::TypeInfo => "Type_Info",
            PreludeType::TypeInfoArray => "Type_Info_Array",
            PreludeType::TypeInfoEnum => "Type_Info_Enum",
            PreludeType::TypeInfoFloat => "Type_Info_Float",
            PreludeType::TypeInfoInteger => "Type_Info_Integer",
            PreludeType::TypeInfoPointer => "Type_Info_Pointer",
            PreludeType::TypeInfoProcedure => "Type_Info_Procedure",
            PreludeType::TypeInfoString => "Type_Info_String",
            PreludeType::TypeInfoStruct => "Type_Info_Struct",
            PreludeType::TypeInfoStructMember => "Type_Info_Struct_Member",
            PreludeType::TypeInfoTaggedUnionBinding => "Type_Info_Tagged_Union_Binding",
            PreludeType::TypeInfoVariant => "Type_Info_Variant",
        }
    }
}

#[cfg(test)]
mod prelude_type_tests {
    use super::PreludeType;

    #[test]
    fn every_prelude_type_is_declared_in_the_prelude() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib/prelude");
        let mut text = String::new();
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|e| e == "jai") {
                text.push_str(&std::fs::read_to_string(path).unwrap());
                text.push('\n');
            }
        }
        for ty in PreludeType::ALL {
            let declared = text.lines().any(|line| {
                line.trim_start()
                    .strip_prefix(ty.name())
                    .is_some_and(|rest| rest.trim_start().starts_with("::"))
            });
            assert!(declared, "{} is not declared in stdlib/prelude", ty.name());
        }
    }
}
