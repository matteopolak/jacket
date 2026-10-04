//! Source constructor authority is separate from pointer shape and Type descriptors.
use crate::{ParameterId, ProcedureId, SourceProcedureIdentity};
use jai_source::DeclarationId;
use jai_types::{ByteOrder, LayoutEngine, LayoutPolicy, TypeId, TypeKind, TypeView};
use std::{
    collections::HashMap,
    fmt,
    hash::{Hash, Hasher},
    sync::Arc,
};

/// These are original checked source owners, never replacement runtime identities.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TypedConstructorOwner {
    Procedure {
        declaration: DeclarationId,
        procedure: ProcedureId,
        signature: TypeId,
    },
    /// Exact `MacroId::Module` declaration; canonical roles cannot be lexical macros.
    ModuleMacro { declaration: DeclarationId },
}
#[derive(Clone, Debug)]
pub struct TypedConstructorSource {
    owner: TypedConstructorOwner,
    identity: SourceProcedureIdentity,
}
impl TypedConstructorSource {
    /// Trusted semantic binding must already match the selected immutable catalog.
    pub fn from_checked_source(
        owner: TypedConstructorOwner,
        identity: SourceProcedureIdentity,
    ) -> Self {
        Self {
            owner,
            identity,
        }
    }
    pub fn same_source(&self, other: &Self) -> bool {
        self.owner == other.owner && self.identity.matches_identity(&other.identity)
    }
    /// Fresh type/procedure arenas retain the same original definition and bytes.
    /// Emitted IDs may be explicitly rebound; another load cannot replace source.
    pub fn same_definition(&self, other: &Self) -> bool {
        let same_role = matches!(
            (self.owner(), other.owner()),
            (
                TypedConstructorOwner::Procedure { .. },
                TypedConstructorOwner::Procedure { .. }
            ) | (
                TypedConstructorOwner::ModuleMacro { .. },
                TypedConstructorOwner::ModuleMacro { .. }
            )
        );
        same_role
            && self.declaration() == other.declaration()
            && same_identity(self.identity(), other.identity())
    }
    pub fn owner(&self) -> TypedConstructorOwner {
        self.owner
    }
    pub fn identity(&self) -> &SourceProcedureIdentity {
        &self.identity
    }
    pub fn declaration(&self) -> DeclarationId {
        match self.owner {
            TypedConstructorOwner::Procedure {
                declaration, ..
            }
            | TypedConstructorOwner::ModuleMacro {
                declaration,
            } => declaration,
        }
    }
}

/// The actual defining type/default scope, independent of the factory caller scope.
#[derive(Clone, Debug)]
pub struct TypedConstructorDefaultScope {
    pub declaration: Option<DeclarationId>,
    pub identity: SourceProcedureIdentity,
}
/// Inputs to a checked initializer binding, kept together when rebinding the
/// same source declaration into another type/procedure arena.
#[derive(Clone, Debug)]
pub struct TypedConstructorBinding {
    pub constructor: TypedConstructorSource,
    pub initializer: TypedConstructorSource,
    pub scope: TypedConstructorDefaultScope,
    pub storage: TypeId,
    pub pointer: TypeId,
    pub parameter: Option<ParameterId>,
}
#[derive(Clone, Debug)]
pub enum TypedConstructorInitialization {
    Uninitialized,
    Default {
        initializer: TypedConstructorSource,
        scope: TypedConstructorDefaultScope,
        step: CheckedTypedConstructorInitializationStep,
    },
}
/// Checked initializer step. Issuance precedes execution; it grants no completed
/// initialization or typed memory authority by itself.
#[derive(Clone, Debug)]
pub struct CheckedTypedConstructorInitializationStep(Arc<InitializationStep>);
#[derive(Debug)]
struct InitializationStep {
    source_issuance: Arc<()>,
    constructor: TypedConstructorSource,
    initializer: TypedConstructorSource,
    scope: TypedConstructorDefaultScope,
    storage: TypeId,
    pointer: TypeId,
    parameter: Option<ParameterId>,
}
impl CheckedTypedConstructorInitializationStep {
    pub fn from_checked_binding(
        binding: TypedConstructorBinding,
        types: &dyn TypeView,
    ) -> Result<Self, TypedConstructorError> {
        let TypedConstructorBinding {
            constructor,
            initializer,
            scope,
            storage,
            pointer,
            parameter,
        } = binding;
        let result = Self(Arc::new(InitializationStep {
            source_issuance: Arc::new(()),
            constructor,
            initializer,
            scope,
            storage,
            pointer,
            parameter,
        }));
        result.validate(types)?;
        Ok(result)
    }
    pub fn constructor(&self) -> &TypedConstructorSource {
        &self.0.constructor
    }
    pub fn initializer(&self) -> &TypedConstructorSource {
        &self.0.initializer
    }
    pub fn default_scope(&self) -> &TypedConstructorDefaultScope {
        &self.0.scope
    }
    pub fn storage(&self) -> TypeId {
        self.0.storage
    }
    pub fn pointer_type(&self) -> TypeId {
        self.0.pointer
    }
    /// Real emitted pointer formal for a procedure; inline macros have no ABI ID.
    pub fn initializer_parameter(&self) -> Option<ParameterId> {
        self.0.parameter
    }
    pub fn same_issuance(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
    /// Captured graph ownership retains this issuance across checked arena remaps.
    /// This equality is not a native initializer-completion witness.
    pub fn same_source_issuance(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0.source_issuance, &other.0.source_issuance)
    }
    pub fn rebind_checked_binding(
        &self,
        binding: TypedConstructorBinding,
        types: &dyn TypeView,
    ) -> Result<Self, TypedConstructorError> {
        if !self.constructor().same_definition(&binding.constructor)
            || !self.initializer().same_definition(&binding.initializer)
            || !same_default_scope(self.default_scope(), &binding.scope)
            || self.initializer_parameter() != binding.parameter
        {
            return Err(TypedConstructorError::ChangedSource);
        }
        let mut rebound = Self::from_checked_binding(binding, types)?;
        Arc::get_mut(&mut rebound.0)
            .expect("new constructor initialization issuance")
            .source_issuance = self.0.source_issuance.clone();
        Ok(rebound)
    }
    /// Conservative O(1) byte charge, including shared immutable full sources.
    pub fn retained_byte_upper(&self) -> Result<usize, TypedConstructorError> {
        let bytes = std::mem::size_of::<InitializationStep>()
            .checked_add(2 * arc_header())
            .ok_or(TypedConstructorError::RetainedBytes)?;
        source_bytes(
            bytes,
            [
                self.constructor().identity(),
                self.initializer().identity(),
                &self.default_scope().identity,
            ],
        )
    }
    pub fn validate(&self, types: &dyn TypeView) -> Result<(), TypedConstructorError> {
        if !matches!(types.kind(self.pointer_type())?, TypeKind::Pointer(element) if *element == self.storage())
        {
            return Err(TypedConstructorError::PointerType);
        }
        if let TypedConstructorOwner::Procedure {
            signature, ..
        } = self.constructor().owner()
            && types.procedure_definition(signature)?.results.as_ref() != [self.pointer_type()]
        {
            return Err(TypedConstructorError::ConstructorSignature);
        }
        match self.initializer().owner() {
            TypedConstructorOwner::Procedure {
                signature, ..
            } => {
                let definition = types.procedure_definition(signature)?;
                if definition.parameters.as_ref() != [self.pointer_type()]
                    || !definition.results.is_empty()
                    || self
                        .initializer_parameter()
                        .is_none_or(|parameter| parameter.index() != 0)
                {
                    return Err(TypedConstructorError::InitializerSignature);
                }
            }
            TypedConstructorOwner::ModuleMacro {
                ..
            } if self.initializer_parameter().is_none() => {}
            _ => return Err(TypedConstructorError::InitializerSignature),
        }
        Ok(())
    }
    pub fn validate_signatures(
        &self,
        types: &dyn TypeView,
        signatures: &HashMap<ProcedureId, TypeId>,
    ) -> Result<(), TypedConstructorError> {
        self.validate(types)?;
        for source in [self.constructor(), self.initializer()] {
            if let TypedConstructorOwner::Procedure {
                procedure,
                signature,
                ..
            } = source.owner()
                && signatures.get(&procedure) != Some(&signature)
            {
                return Err(TypedConstructorError::ChangedProcedure);
            }
        }
        Ok(())
    }
}
impl PartialEq for CheckedTypedConstructorInitializationStep {
    fn eq(&self, other: &Self) -> bool {
        self.same_issuance(other)
    }
}
impl Eq for CheckedTypedConstructorInitializationStep {
}
impl Hash for CheckedTypedConstructorInitializationStep {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}

#[derive(Debug)]
struct Receipt {
    source_issuance: Arc<()>,
    constructor: TypedConstructorSource,
    storage: TypeId,
    pointer: TypeId,
    initialization: TypedConstructorInitialization,
    layout: LayoutPolicy,
    byte_order: ByteOrder,
    extent: u64,
    alignment: u32,
}

/// Opaque source issuance. Runtime Values, casts and byte images cannot mint it.
/// It permits a typed heap window only; descriptor/header and AST event authority
/// remain separate checked-source capabilities.
#[derive(Clone, Debug)]
pub struct CheckedTypedConstructorReceipt(Arc<Receipt>);
impl CheckedTypedConstructorReceipt {
    /// Called only after original constructor/specialization, T and initializer
    /// ownership are checked against the configured source catalog and body scope.
    pub fn from_checked_specialization(
        constructor: TypedConstructorSource,
        storage: TypeId,
        pointer: TypeId,
        initialization: TypedConstructorInitialization,
        layout: LayoutPolicy,
        byte_order: ByteOrder,
        types: &dyn TypeView,
    ) -> Result<Self, TypedConstructorError> {
        if !matches!(types.kind(pointer)?, TypeKind::Pointer(element) if *element == storage) {
            return Err(TypedConstructorError::PointerType);
        }
        let mut engine = LayoutEngine::new(types, layout);
        let storage_layout = engine
            .layout(storage)
            .map_err(TypedConstructorError::Layout)?;
        let result = Self(Arc::new(Receipt {
            source_issuance: Arc::new(()),
            constructor,
            storage,
            pointer,
            initialization,
            layout,
            byte_order,
            extent: storage_layout.size,
            alignment: storage_layout.alignment,
        }));
        result.validate(types)?;
        Ok(result)
    }
    pub fn constructor(&self) -> &TypedConstructorSource {
        &self.0.constructor
    }
    pub fn storage(&self) -> TypeId {
        self.0.storage
    }
    pub fn pointer_type(&self) -> TypeId {
        self.0.pointer
    }
    pub fn initialization(&self) -> &TypedConstructorInitialization {
        &self.0.initialization
    }
    pub fn layout_policy(&self) -> LayoutPolicy {
        self.0.layout
    }
    pub fn byte_order(&self) -> ByteOrder {
        self.0.byte_order
    }
    pub fn extent(&self) -> u64 {
        self.0.extent
    }
    pub fn alignment(&self) -> u32 {
        self.0.alignment
    }
    pub fn same_issuance(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
    pub fn same_source_issuance(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0.source_issuance, &other.0.source_issuance)
    }
    /// Source dictionary rebinds only actual checked TypeIds/emitted owners. The
    /// original source issuance/default event and target policy remain immutable.
    /// This creates no native completion witness and grants no descriptor role.
    pub fn rebind_checked_specialization(
        &self,
        constructor: TypedConstructorSource,
        storage: TypeId,
        pointer: TypeId,
        initialization: TypedConstructorInitialization,
        types: &dyn TypeView,
    ) -> Result<Self, TypedConstructorError> {
        if !self.constructor().same_definition(&constructor) {
            return Err(TypedConstructorError::ChangedSource);
        }
        match (self.initialization(), &initialization) {
            (
                TypedConstructorInitialization::Uninitialized,
                TypedConstructorInitialization::Uninitialized,
            ) => {}
            (
                TypedConstructorInitialization::Default {
                    initializer: old_initializer,
                    scope: old_scope,
                    step: old_step,
                },
                TypedConstructorInitialization::Default {
                    initializer,
                    scope,
                    step,
                },
            ) if old_initializer.same_definition(initializer)
                && same_default_scope(old_scope, scope)
                && old_step.same_source_issuance(step) => {}
            _ => return Err(TypedConstructorError::ChangedSource),
        }
        let mut rebound = Self::from_checked_specialization(
            constructor,
            storage,
            pointer,
            initialization,
            self.layout_policy(),
            self.byte_order(),
            types,
        )?;
        if rebound.extent() != self.extent() || rebound.alignment() != self.alignment() {
            return Err(TypedConstructorError::ChangedLayout);
        }
        Arc::get_mut(&mut rebound.0)
            .expect("new constructor receipt issuance")
            .source_issuance = self.0.source_issuance.clone();
        Ok(rebound)
    }
    /// One retained Rust byte is one ValueCell. This deliberately overcounts
    /// shared source arcs, avoiding an uncharged cold traversal or dedup cache.
    pub fn retained_byte_upper(&self) -> Result<usize, TypedConstructorError> {
        let mut bytes = std::mem::size_of::<Receipt>()
            .checked_add(2 * arc_header())
            .ok_or(TypedConstructorError::RetainedBytes)?;
        bytes = source_bytes(bytes, [self.constructor().identity()])?;
        if let TypedConstructorInitialization::Default {
            initializer,
            scope,
            step,
        } = self.initialization()
        {
            bytes = source_bytes(bytes, [initializer.identity(), &scope.identity])?;
            bytes = bytes
                .checked_add(step.retained_byte_upper()?)
                .ok_or(TypedConstructorError::RetainedBytes)?;
        }
        Ok(bytes)
    }
    pub fn validate(&self, types: &dyn TypeView) -> Result<(), TypedConstructorError> {
        if !matches!(types.kind(self.pointer_type())?,TypeKind::Pointer(element) if *element==self.storage())
        {
            return Err(TypedConstructorError::PointerType);
        }
        let mut engine = LayoutEngine::new(types, self.layout_policy());
        let layout = engine
            .layout(self.storage())
            .map_err(TypedConstructorError::Layout)?;
        if layout.size != self.extent() || layout.alignment != self.alignment() {
            return Err(TypedConstructorError::ChangedLayout);
        }
        if let TypedConstructorOwner::Procedure {
            signature, ..
        } = self.constructor().owner()
            && types.procedure_definition(signature)?.results.as_ref() != [self.pointer_type()]
        {
            return Err(TypedConstructorError::ConstructorSignature);
        }
        if let TypedConstructorInitialization::Default {
            initializer,
            scope,
            step,
        } = self.initialization()
        {
            step.validate(types)?;
            if !step.constructor().same_source(self.constructor())
                || !step.initializer().same_source(initializer)
                || step.storage() != self.storage()
                || step.pointer_type() != self.pointer_type()
                || !same_default_scope(step.default_scope(), scope)
            {
                return Err(TypedConstructorError::InitializerSignature);
            }
            if let TypedConstructorOwner::Procedure {
                signature, ..
            } = initializer.owner()
            {
                let definition = types.procedure_definition(signature)?;
                if definition.parameters.as_ref() != [self.pointer_type()]
                    || !definition.results.is_empty()
                {
                    return Err(TypedConstructorError::InitializerSignature);
                }
            }
        }
        Ok(())
    }
    pub fn validate_signatures(
        &self,
        types: &dyn TypeView,
        signatures: &HashMap<ProcedureId, TypeId>,
    ) -> Result<(), TypedConstructorError> {
        self.validate(types)?;
        for source in std::iter::once(self.constructor()).chain(match self.initialization() {
            TypedConstructorInitialization::Default {
                initializer, ..
            } => Some(initializer),
            TypedConstructorInitialization::Uninitialized => None,
        }) {
            if let TypedConstructorOwner::Procedure {
                procedure,
                signature,
                ..
            } = source.owner()
                && signatures.get(&procedure) != Some(&signature)
            {
                return Err(TypedConstructorError::ChangedProcedure);
            }
        }
        Ok(())
    }
}
impl PartialEq for CheckedTypedConstructorReceipt {
    fn eq(&self, other: &Self) -> bool {
        self.same_issuance(other)
    }
}
impl Eq for CheckedTypedConstructorReceipt {
}
impl Hash for CheckedTypedConstructorReceipt {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}
#[derive(Debug)]
pub enum TypedConstructorError {
    Type(jai_types::TypeError),
    Layout(jai_types::LayoutError),
    PointerType,
    ConstructorSignature,
    InitializerSignature,
    ChangedLayout,
    ChangedProcedure,
    ChangedSource,
    RetainedBytes,
}
impl From<jai_types::TypeError> for TypedConstructorError {
    fn from(error: jai_types::TypeError) -> Self {
        Self::Type(error)
    }
}
impl fmt::Display for TypedConstructorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Type(error) => error.fmt(f),
            Self::Layout(error) => error.fmt(f),
            Self::PointerType => {
                f.write_str("typed constructor result does not point to its original T")
            }
            Self::ConstructorSignature => {
                f.write_str("typed constructor differs from its checked source result")
            }
            Self::InitializerSignature => {
                f.write_str("typed constructor initializer differs from its original T signature")
            }
            Self::ChangedLayout => f.write_str("typed constructor layout changed"),
            Self::ChangedSource => {
                f.write_str("typed constructor original source issuance changed")
            }
            Self::RetainedBytes => f.write_str("typed constructor retained byte charge overflowed"),
            Self::ChangedProcedure => f.write_str(
                "typed constructor original procedure signature is unavailable or changed",
            ),
        }
    }
}
impl std::error::Error for TypedConstructorError {
}

fn same_identity(left: &SourceProcedureIdentity, right: &SourceProcedureIdentity) -> bool {
    left.matches_identity(right)
}
fn same_default_scope(
    left: &TypedConstructorDefaultScope,
    right: &TypedConstructorDefaultScope,
) -> bool {
    left.declaration == right.declaration && same_identity(&left.identity, &right.identity)
}
fn arc_header() -> usize {
    4 * std::mem::size_of::<usize>()
}
fn source_bytes<const N: usize>(
    mut bytes: usize,
    sources: [&SourceProcedureIdentity; N],
) -> Result<usize, TypedConstructorError> {
    for source in sources {
        bytes = bytes
            .checked_add(std::mem::size_of::<SourceProcedureIdentity>())
            .and_then(|bytes| bytes.checked_add(2 * arc_header()))
            .and_then(|bytes| bytes.checked_add(source.source_text().len()))
            .and_then(|bytes| bytes.checked_add(source.path().as_os_str().len()))
            .ok_or(TypedConstructorError::RetainedBytes)?;
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests;
