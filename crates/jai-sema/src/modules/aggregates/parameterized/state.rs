//! Canonical template instances are reserved before any recursive field is visited.
use crate::local_declarations::{FieldMetadata, RecordMetadata};
use crate::polymorphism::{BakedValue, Substitution};
use jai_modules::FileInstanceId;
use jai_source::{DeclarationId, Symbol};
use jai_types::{Integer, IntegerType, RecordKind, TypeId, TypeRegistry};
use std::collections::HashMap;
use std::sync::Arc;

/// The successful selected body is the authority for construction chronology.
#[derive(Clone)]
pub(crate) struct SelectedRecordSource {
    pub(crate) owner: TypeId,
    pub(crate) file: FileInstanceId,
    pub(crate) location: jai_source::SourceSpan,
    pub(crate) parameters: Substitution,
    pub(crate) members: Arc<[jai_syntax::RecordMember]>,
}

type FieldNotes = Box<[Box<[u8]>]>;

/// A template has its declaration's nominal identity, never its printed name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct RecordTemplateId(pub(crate) DeclarationId);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct RecordSpecializationKey {
    pub(crate) template: RecordTemplateId,
    /// Values are normalized and ordered by the template's parameter list.
    pub(crate) arguments: Box<[BakedValue]>,
}
/// Anonymous nominal identities follow their source site and enclosing bindings.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct InlineTypeKey {
    pub(crate) file: FileInstanceId,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) owner: Option<TypeId>,
    pub(crate) substitution: Substitution,
}

#[derive(Clone)]
pub(crate) struct SpecializedRecord {
    pub(crate) origin: Option<RecordTemplateId>,
    pub(crate) file: FileInstanceId,
    pub(crate) shape: RecordMetadata,
    pub(crate) substitution: Substitution,
    pub(crate) nested: bool,
    pub(crate) defaults: HashMap<Symbol, BakedValue>,
}
#[derive(Clone)]
pub(crate) struct MemberEnum {
    pub(crate) name: Option<Symbol>,
    pub(crate) representation: IntegerType,
    pub(crate) flags: bool,
    pub(crate) values: Vec<(Symbol, Integer)>,
}
#[derive(Clone)]
pub(crate) struct RecordMethodEnvironment {
    pub(crate) file: FileInstanceId,
    pub(crate) name: Option<Symbol>,
    pub(crate) substitution: Substitution,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Resolving,
    Ready,
    Retry,
}

struct Entry {
    ty: TypeId,
    state: State,
}

#[derive(Default)]
pub(crate) struct RecordSpecializations {
    source_locations: HashMap<TypeId, jai_source::SourceSpan>,
    selected_sources: HashMap<TypeId, Arc<SelectedRecordSource>>,
    entries: HashMap<RecordSpecializationKey, Entry>,
    keys: HashMap<TypeId, RecordSpecializationKey>,
    inline: HashMap<InlineTypeKey, Entry>,
    owners: Vec<TypeId>,
    records: HashMap<TypeId, SpecializedRecord>,
    nested: HashMap<(TypeId, usize), TypeId>,
    namespaces: HashMap<TypeId, Substitution>,
    source_members: HashMap<TypeId, Vec<Symbol>>,
    enums: HashMap<TypeId, MemberEnum>,
    methods: HashMap<TypeId, Vec<super::RecordMethod>>,
    default_overrides: HashMap<jai_types::FieldId, Vec<super::RecordDefaultOverride>>,
    default_override_roots: HashMap<TypeId, Vec<jai_types::FieldId>>,
    method_environments: HashMap<TypeId, RecordMethodEnvironment>,
    reflection: HashMap<TypeId, jai_types::ReflectedRecordMetadata>,
    field_notes: HashMap<jai_types::FieldId, FieldNotes>,
    field_owners: HashMap<jai_types::FieldId, (TypeId, usize)>,
    depth: usize,
    pub(super) modifiers: super::modifier_intents::RecordModifierTable,
}

pub(crate) enum Reservation {
    Existing(TypeId),
    Resolve(TypeId),
}

impl RecordSpecializations {
    pub(crate) fn selected_source(&self, ty: TypeId) -> Option<&Arc<SelectedRecordSource>> {
        self.selected_sources.get(&ty)
    }
    pub(super) fn remember_selected_source(&mut self, source: SelectedRecordSource) {
        self.selected_sources
            .entry(source.owner)
            .or_insert_with(|| Arc::new(source));
    }
    pub(crate) fn modifier_readiness(
        &self,
        id: super::RecordModifierId,
    ) -> super::RecordModifierReadiness {
        self.modifiers.readiness(id)
    }

    pub(crate) fn completed_modifier_count(&self) -> usize {
        self.modifiers.completed_count()
    }
    pub(crate) fn queued_modifier_count(&self) -> usize {
        self.modifiers.queued_count()
    }
    pub(crate) fn remember_source(&mut self, ty: TypeId, location: jai_source::SourceSpan) {
        self.source_locations.entry(ty).or_insert(location);
    }
    pub(crate) fn source_location(&self, ty: TypeId) -> Option<jai_source::SourceSpan> {
        self.source_locations.get(&ty).copied()
    }
    pub(crate) fn push_owner(&mut self, owner: TypeId) {
        self.owners.push(owner);
    }
    pub(crate) fn pop_owner(&mut self, owner: TypeId) {
        assert_eq!(
            self.owners.pop(),
            Some(owner),
            "record owner scopes are nested"
        );
    }
    pub(crate) fn active_owner(&self) -> Option<TypeId> {
        self.owners.last().copied()
    }
    pub(crate) fn is_materializing(&self, owner: TypeId) -> bool {
        self.owners.contains(&owner)
    }
    pub(crate) fn reserve_inline_record(
        &mut self,
        key: InlineTypeKey,
        kind: RecordKind,
        types: &mut TypeRegistry,
    ) -> Reservation {
        self.reserve_inline(key, |types| types.reserve_record(kind), types)
    }
    pub(crate) fn reserve_inline_enum(
        &mut self,
        key: InlineTypeKey,
        representation: IntegerType,
        types: &mut TypeRegistry,
    ) -> Reservation {
        self.reserve_inline(key, |types| types.reserve_enum(representation), types)
    }
    fn reserve_inline(
        &mut self,
        key: InlineTypeKey,
        reserve: impl FnOnce(&mut TypeRegistry) -> TypeId,
        types: &mut TypeRegistry,
    ) -> Reservation {
        if let Some(entry) = self.inline.get_mut(&key) {
            if entry.state != State::Retry {
                return Reservation::Existing(entry.ty);
            }
            entry.state = State::Resolving;
            return Reservation::Resolve(entry.ty);
        }
        let ty = reserve(types);
        self.inline.insert(
            key,
            Entry {
                ty,
                state: State::Resolving,
            },
        );
        Reservation::Resolve(ty)
    }
    pub(crate) fn complete_inline(&mut self, key: &InlineTypeKey) {
        self.inline
            .get_mut(key)
            .expect("anonymous type reserved")
            .state = State::Ready;
    }
    pub(crate) fn retry_inline(&mut self, key: &InlineTypeKey) {
        self.inline
            .get_mut(key)
            .expect("anonymous type reserved")
            .state = State::Retry;
    }
    pub(crate) fn reserve(
        &mut self,
        key: RecordSpecializationKey,
        kind: RecordKind,
        types: &mut TypeRegistry,
    ) -> Reservation {
        if let Some(entry) = self.entries.get_mut(&key) {
            if entry.state != State::Retry {
                return Reservation::Existing(entry.ty);
            }
            entry.state = State::Resolving;
            return Reservation::Resolve(entry.ty);
        }
        let ty = types.reserve_record(kind);
        self.keys.insert(ty, key.clone());
        self.entries.insert(
            key,
            Entry {
                ty,
                state: State::Resolving,
            },
        );
        Reservation::Resolve(ty)
    }
    pub(crate) fn complete(&mut self, key: &RecordSpecializationKey, record: SpecializedRecord) {
        let entry = self
            .entries
            .get_mut(key)
            .expect("specialization was reserved");
        entry.state = State::Ready;
        let ty = entry.ty;
        self.complete_nested(ty, record);
    }
    pub(crate) fn retry(&mut self, key: &RecordSpecializationKey) {
        self.entries
            .get_mut(key)
            .expect("specialization was reserved")
            .state = State::Retry;
    }
    pub(crate) fn record(&self, ty: TypeId) -> Option<&SpecializedRecord> {
        self.records.get(&ty)
    }
    pub(crate) fn key_for_type(&self, ty: TypeId) -> Option<&RecordSpecializationKey> {
        self.keys.get(&ty)
    }
    pub(crate) fn reserve_nested(
        &mut self,
        owner: TypeId,
        member: usize,
        kind: RecordKind,
        types: &mut TypeRegistry,
    ) -> TypeId {
        *self
            .nested
            .entry((owner, member))
            .or_insert_with(|| types.reserve_record(kind))
    }
    pub(crate) fn reserve_nested_enum(
        &mut self,
        owner: TypeId,
        member: usize,
        representation: IntegerType,
        types: &mut TypeRegistry,
    ) -> TypeId {
        *self
            .nested
            .entry((owner, member))
            .or_insert_with(|| types.reserve_enum(representation))
    }
    pub(crate) fn reserve_namespace(&mut self, ty: TypeId, substitution: Substitution) {
        self.namespaces.insert(ty, substitution);
    }
    pub(crate) fn member_bindings(&self, ty: TypeId) -> Option<&Substitution> {
        self.record(ty)
            .map(|record| &record.substitution)
            .or_else(|| self.namespaces.get(&ty))
    }
    pub(crate) fn define_source_members(&mut self, owner: TypeId, names: Vec<Symbol>) {
        self.source_members.insert(owner, names);
    }
    pub(crate) fn source_member_names(&self, owner: TypeId) -> Option<&[Symbol]> {
        self.source_members.get(&owner).map(Vec::as_slice)
    }
    pub(crate) fn source_namespaces(&self) -> impl Iterator<Item = (TypeId, &[Symbol])> {
        self.source_members
            .iter()
            .map(|(&owner, names)| (owner, names.as_slice()))
    }
    pub(crate) fn publish_method_constants(
        &mut self,
        owner: TypeId,
        methods: impl IntoIterator<Item = (Symbol, jai_ir::ConstantValue)>,
    ) {
        let scope = self
            .records
            .get_mut(&owner)
            .map(|record| &mut record.substitution)
            .unwrap_or_else(|| self.namespaces.entry(owner).or_default());
        for (name, value) in methods {
            scope.constants.retain(|binding| binding.name != name);
            scope.types.retain(|binding| binding.name != name);
            scope.bind_constant(name, BakedValue::Value(value));
        }
    }
    pub(crate) fn define_member_enum(&mut self, ty: TypeId, enumeration: MemberEnum) {
        self.enums.insert(ty, enumeration);
    }
    pub(crate) fn member_enum(&self, ty: TypeId) -> Option<&MemberEnum> {
        self.enums.get(&ty)
    }
    pub(crate) fn member_enums(&self) -> impl Iterator<Item = (TypeId, &MemberEnum)> {
        self.enums
            .iter()
            .map(|(&ty, enumeration)| (ty, enumeration))
    }
    pub(crate) fn define_reflection(
        &mut self,
        ty: TypeId,
        metadata: jai_types::ReflectedRecordMetadata,
        fields: Vec<(jai_types::FieldId, FieldNotes)>,
    ) {
        self.reflection.insert(ty, metadata);
        self.field_notes.extend(fields);
    }
    pub(crate) fn reflected_record(
        &self,
        ty: TypeId,
    ) -> Option<&jai_types::ReflectedRecordMetadata> {
        self.reflection.get(&ty)
    }
    pub(crate) fn reflected_field_notes(&self, id: jai_types::FieldId) -> Option<&[Box<[u8]>]> {
        self.field_notes.get(&id).map(AsRef::as_ref)
    }
    pub(crate) fn define_methods(&mut self, ty: TypeId, methods: Vec<super::RecordMethod>) {
        self.methods.insert(ty, methods);
    }
    pub(crate) fn define_default_overrides(
        &mut self,
        owner: TypeId,
        overrides: Vec<super::RecordDefaultOverride>,
    ) {
        if let Some(previous) = self.default_override_roots.remove(&owner) {
            for field in previous {
                self.default_overrides.remove(&field);
            }
        }
        let mut roots = Vec::new();
        for override_ in overrides {
            let root = *override_
                .path
                .first()
                .expect("default override paths have a root field");
            if !roots.contains(&root) {
                roots.push(root);
            }
            self.default_overrides
                .entry(root)
                .or_default()
                .push(override_);
        }
        self.default_override_roots.insert(owner, roots);
    }
    pub(crate) fn default_overrides(
        &self,
        field: jai_types::FieldId,
    ) -> &[super::RecordDefaultOverride] {
        self.default_overrides
            .get(&field)
            .map_or(&[], Vec::as_slice)
    }
    pub(crate) fn reserve_method_environment(
        &mut self,
        owner: TypeId,
        file: FileInstanceId,
        name: Option<Symbol>,
        substitution: Substitution,
    ) {
        self.method_environments.insert(
            owner,
            RecordMethodEnvironment {
                file,
                name,
                substitution,
            },
        );
    }
    pub(crate) fn method_environment(&self, owner: TypeId) -> Option<RecordMethodEnvironment> {
        self.record(owner)
            .map(|record| RecordMethodEnvironment {
                file: record.file,
                name: record.shape.name,
                substitution: record.substitution.clone(),
            })
            .or_else(|| {
                self.method_environments
                    .get(&owner)
                    .map(|environment| RecordMethodEnvironment {
                        substitution: self
                            .namespaces
                            .get(&owner)
                            .cloned()
                            .unwrap_or_else(|| environment.substitution.clone()),
                        ..environment.clone()
                    })
            })
    }
    pub(crate) fn method_owners(&self) -> impl Iterator<Item = TypeId> + '_ {
        self.methods
            .iter()
            .filter_map(|(&owner, methods)| (!methods.is_empty()).then_some(owner))
    }
    pub(crate) fn methods(&self, ty: TypeId) -> Option<&[super::RecordMethod]> {
        self.methods.get(&ty).map(Vec::as_slice)
    }
    pub(crate) fn complete_nested(&mut self, ty: TypeId, record: SpecializedRecord) {
        self.field_owners.extend(
            record
                .shape
                .fields
                .iter()
                .enumerate()
                .map(|(index, field)| (field.id, (ty, index))),
        );
        self.records.insert(ty, record);
    }
    /// Completed nested shapes are immutable. Late ancestor bindings only
    /// extend their default/method scope; they do not rebuild descendant trees.
    pub(crate) fn inherit_nested_bindings(
        &mut self,
        owner: TypeId,
        outer: &Substitution,
    ) -> Result<(), ()> {
        let mut pending = vec![(owner, outer.clone())];
        let mut visited = std::collections::HashSet::new();
        while let Some((owner, outer)) = pending.pop() {
            if !visited.insert(owner) {
                continue;
            }
            if visited.len() > 65_536 {
                return Err(());
            }
            let Some(record) = self.records.get_mut(&owner) else {
                continue;
            };
            for binding in &outer.types {
                if record.substitution.ty(binding.name).is_none()
                    && record.substitution.constant(binding.name).is_none()
                {
                    record.substitution.bind_type(binding.name, binding.ty);
                }
            }
            for binding in &outer.constants {
                if record.substitution.ty(binding.name).is_none()
                    && record.substitution.constant(binding.name).is_none()
                {
                    record
                        .substitution
                        .bind_constant(binding.name, binding.value.clone());
                }
            }
            let scope = record.substitution.clone();
            self.namespaces.insert(owner, scope.clone());
            for (&(parent, _), &child) in &self.nested {
                if parent == owner && self.records.contains_key(&child) {
                    pending.push((child, scope.clone()));
                }
            }
        }
        Ok(())
    }
    pub(crate) fn records(&self) -> impl Iterator<Item = (TypeId, &SpecializedRecord)> {
        self.records.iter().map(|(&ty, record)| (ty, record))
    }
    pub(crate) fn field(
        &self,
        id: jai_types::FieldId,
    ) -> Option<(&SpecializedRecord, &FieldMetadata)> {
        let &(ty, index) = self.field_owners.get(&id)?;
        let record = self.records.get(&ty)?;
        Some((record, &record.shape.fields[index]))
    }
    pub(crate) fn enter(&mut self) -> bool {
        if self.depth >= crate::constant_limits::MAX_CONSTANT_DEPTH {
            return false;
        }
        self.depth += 1;
        true
    }
    pub(crate) fn leave(&mut self) {
        self.depth -= 1;
    }
}

impl crate::overloads::NominalView for RecordSpecializations {
    fn specialization(&self, ty: TypeId) -> Option<(DeclarationId, &Substitution)> {
        let record = self.record(ty)?;
        if record.nested {
            return None;
        }
        Some((record.origin?.0, &record.substitution))
    }
    fn default_argument(&self, ty: TypeId, parameter: Symbol) -> Option<&BakedValue> {
        self.record(ty)?.defaults.get(&parameter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::Identities;
    #[test]
    fn recursion_and_retry_reuse_the_reserved_identity() {
        let mut types = TypeRegistry::new();
        let mut instances = RecordSpecializations::default();
        let key = RecordSpecializationKey {
            template: RecordTemplateId(Identities::default().declaration()),
            arguments: Box::new([]),
        };
        let Reservation::Resolve(first) =
            instances.reserve(key.clone(), RecordKind::Struct, &mut types)
        else {
            panic!("first visit must reserve");
        };
        let Reservation::Existing(recursive) =
            instances.reserve(key.clone(), RecordKind::Struct, &mut types)
        else {
            panic!("recursive visit must reuse");
        };
        assert_eq!(first, recursive);
        instances.retry(&key);
        let Reservation::Resolve(retry) = instances.reserve(key, RecordKind::Struct, &mut types)
        else {
            panic!("failed body can retry");
        };
        assert_eq!(first, retry);
    }
    #[test]
    fn equal_printed_names_do_not_merge_distinct_templates() {
        let mut ids = Identities::default();
        let first = RecordTemplateId(ids.declaration());
        let second = RecordTemplateId(ids.declaration());
        assert_ne!(first, second);
    }
}
